use std::{future::Future, sync::Arc, time::Duration};

use regex::Regex;
use serenity::{
    all::{
        ChannelId, Command, Context, CreateEmbed, CreateEmbedAuthor, CreateMessage, EventHandler,
        GatewayIntents, GuildId, Http, HttpError, Interaction, Member, Ready, UserId,
    },
    Client,
};
use tokio::time;

use crate::{
    annict::{ActivityItem, RatingState},
    db, get_env, Result,
};

mod annict;
mod notify;

/// Discord の イベントリスナーを開始させ、その [Future] と HTTP クライアント [Http] を返す。
pub async fn start() -> Result<(impl Future<Output = Result<()>>, Arc<Http>)> {
    let mut client = Client::builder(get_env("DISCORD_TOKEN")?, GatewayIntents::default())
        .event_handler(Handler)
        .await?;

    let http = client.http.clone();
    let task = async move { Ok(client.start().await?) };

    Ok((task, http))
}

/// 通知に用いる [Http] クライアントを受け取り、通知タスクを開始する。
pub async fn notify(http: Arc<Http>) -> Result<()> {
    let interval = get_interval()?;
    tracing::info!("更新間隔: {} 秒", interval.as_secs());
    let mut conn = db::connect()?;
    loop {
        tracing::trace!("loop!");

        'chan_loop: for channel in db::get_channels(&mut conn)? {
            for subscriber in db::get_subscribers_by_guild(&mut conn, channel.guild_id as _)? {
                let member = match http
                    .get_member(
                        GuildId::new(subscriber.guild_id as _),
                        UserId::new(subscriber.user_id as _),
                    )
                    .await
                {
                    Ok(mem) => mem,
                    Err(serenity::Error::Http(HttpError::UnsuccessfulRequest(e))) => {
                        match e.error.message.to_ascii_lowercase().as_str() {
                            "unknown member" => {
                                tracing::info!(
                                    "サーバー (ID = {}) にユーザー (ID = {}) が所属していません",
                                    subscriber.guild_id,
                                    subscriber.user_id,
                                );
                                continue;
                            }
                            "unknown user" => {
                                tracing::info!(
                                    "ユーザー (ID = {}) が見つかりませんでした",
                                    subscriber.user_id,
                                );
                                continue;
                            }
                            "unknown guild" => {
                                tracing::info!(
                                    "サーバー (ID = {}) が見つかりませんでした",
                                    subscriber.guild_id,
                                );
                                continue 'chan_loop;
                            }
                            _ => {
                                return Err(serenity::Error::Http(HttpError::UnsuccessfulRequest(
                                    e,
                                ))
                                .into())
                            }
                        }
                    }
                    Err(e) => return Err(e.into()),
                };
                for activity in crate::annict::get_new_activities(&subscriber).await? {
                    notify_activity(
                        &http,
                        ChannelId::new(channel.channel_id as _),
                        &member,
                        &subscriber.annict_name,
                        activity,
                    )
                    .await;
                }
            }
        }

        time::sleep(interval).await;
    }
}

pub struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("Discord に {} として接続", ready.user.name);

        // スラッシュコマンドの設定
        match Command::set_global_commands(&ctx.http, vec![notify::register(), annict::register()])
            .await
        {
            Ok(commands) => {
                for command in commands {
                    tracing::trace!("コマンド {:?} の設定", command)
                }
            }
            Err(e) => tracing::warn!("{}", e),
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        tracing::trace!("interaction {:?} が作成されました", interaction);
        let Interaction::Command(interaction) = interaction else {
            return;
        };

        if let Err(e) = match interaction.data.name.as_str() {
            notify::NAME => notify::handle(&ctx, &interaction).await,
            annict::NAME => annict::handle(&ctx, &interaction).await,
            cmd_name => Err(format!("不明なコマンド `{}` を受信", cmd_name).into()),
        } {
            tracing::warn!("{}", e);
        }
    }
}

fn get_interval() -> Result<Duration> {
    let duration = get_env("NOTIFICATION_INTERVAL")?;
    let regex = Regex::new(r"^\s*(\d+)\s*((?i)s|sec|m|min|h|hour)\s*$")?;
    let (_, [num, unit]) = regex
        .captures(&duration)
        .ok_or_else(|| {
            format!(
                "環境変数 `NOTIFICATION_INTERVAL` (\"{}\") の形式が不正です",
                duration
            )
        })?
        .extract();

    let num: u64 = num.parse()?;
    Ok(match unit.to_ascii_lowercase().as_str() {
        "s" | "sec" => Duration::from_secs(num),
        "m" | "min" => Duration::from_secs(num * 60),
        "h" | "hour" => Duration::from_secs(num * 60 * 60),
        _ => unreachable!("正規表現の不正"),
    })
}

async fn notify_activity(
    http: &Http,
    channel_id: ChannelId,
    member: &Member,
    username: &str,
    activity: ActivityItem,
) {
    let mut author = CreateEmbedAuthor::new(member.display_name())
        .url(format!("https://annict.com/@{}", username));
    if let Some(url) = member.avatar_url() {
        author = author.icon_url(url);
    }
    let author = author;

    let mut embed = CreateEmbed::new().author(author);

    match activity {
        ActivityItem::MultipleRecord(records) => {
            for edge in records.records.edges {
                // NOTE: ここ理解する
                Box::pin(notify_activity(
                    http,
                    channel_id,
                    member,
                    username,
                    ActivityItem::Record(edge.node),
                ))
                .await;
            }
        }
        ActivityItem::Record(record) => {
            // 『**タイトル**』
            let mut desc = format!("『**{}**』", record.work.title);

            // 『**タイトル**』
            // 第n話
            let has_number = if let Some(num_text) = record.episode.number_text {
                desc = format!("{}\n{}", desc, num_text);
                true
            } else if let Some(num) = record.episode.number {
                desc = format!("{}\n第{}話", desc, num);
                true
            } else {
                false
            };
            // 『**タイトル**』
            // 第n話「サブタイトル」
            if let Some(title) = record.episode.title {
                if has_number {
                    desc = format!("{}「{}」", desc, title);
                } else {
                    desc = format!("{}\n「{}」", desc, title);
                }
            }

            if let Some(rating) = record.rating_state {
                embed = embed.colour(rating.to_colour());
            } else {
                embed = embed.color(RatingState::Average.to_colour());
            }

            if let Some(comment) = record.comment {
                if !comment.is_empty() {
                    desc = format!("{}\n{}", desc, comment);
                }
            }

            embed = embed.description(desc);
        }
        ActivityItem::Review(review) => {
            embed = embed.field("タイトル", review.work.title, false);

            if let Some(rating) = review.rating_overall_state {
                embed = embed.field("全体", rating.to_string(), true);
                embed = embed.colour(rating.to_colour());
            } else {
                embed = embed.colour(RatingState::Average.to_colour());
            }
            if let Some(rating) = review.rating_animation_state {
                embed = embed.field("映像", rating.to_string(), true);
            }
            if let Some(rating) = review.rating_character_state {
                embed = embed.field("キャラクター", rating.to_string(), true);
            }
            if let Some(rating) = review.rating_story_state {
                embed = embed.field("ストーリー", rating.to_string(), true);
            }
            if let Some(rating) = review.rating_music_state {
                embed = embed.field("音楽", rating.to_string(), true);
            }

            if !review.body.is_empty() {
                embed = embed.field(
                    "感想",
                    review.body.chars().take(1024).collect::<String>(),
                    false,
                );
            }
        }
        ActivityItem::Status(status) => {
            // 『**タイトル**』
            // 見た/見たい/一時中断/...
            embed = embed.description(format!("『**{}**』\n{}", status.work.title, status.state));
            embed = embed.colour(status.state.to_colour());
        }
    }
    let embed = embed;

    let msg = CreateMessage::new().add_embed(embed);
    channel_id.send_message(http, msg).await.unwrap();
}
