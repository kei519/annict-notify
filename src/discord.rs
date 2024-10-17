use std::{future::Future, sync::Arc, time::Duration};

use regex::Regex;
use serenity::{
    all::{Command, Context, EventHandler, GatewayIntents, Http, Interaction, Ready},
    Client,
};
use tokio::time;

use crate::{get_env, Result};

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
pub async fn notify(_http: Arc<Http>) -> Result<()> {
    let interval = get_interval()?;
    tracing::info!("更新間隔: {} 秒", interval.as_secs());
    loop {
        tracing::trace!("loop!");
        time::sleep(interval).await;
    }
}

pub struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        tracing::info!("Discord に {} として接続", ready.user.name);

        // スラッシュコマンドの設定
        match Command::set_global_commands(&ctx.http, vec![notify::register()]).await {
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

        match interaction.data.name.as_str() {
            notify::NAME => notify::handle(&ctx, &interaction).await,
            cmd_name => tracing::debug!("不明なコマンド {} を受信", cmd_name),
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
