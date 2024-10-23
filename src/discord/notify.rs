use serenity::all::{
    ChannelType, CommandInteraction, CommandOptionType, ComponentInteractionDataKind, Context,
    CreateCommand, CreateCommandOption, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, CreateSelectMenu,
    CreateSelectMenuKind, CreateSelectMenuOption, Mentionable, Permissions,
};

use crate::{db, Result};

use super::NotifyFlag;

pub(super) const NAME: &str = "notify";

pub(super) fn register() -> CreateCommand {
    let option = CreateCommandOption::new(
        CommandOptionType::Channel,
        "チャンネル",
        "通知を行うチャンネル",
    );
    CreateCommand::new(NAME)
        .description("通知を行うチャンネルを登録します")
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .add_option(option)
}

pub(super) async fn handle(ctx: &Context, interaction: &CommandInteraction) -> Result<()> {
    let Some(guild) = &interaction.guild_id else {
        // DM の場合
        return error_response(ctx, interaction, "この操作はサーバー内で行ってください").await;
    };
    // サーバー内の場合
    let channel = interaction
        .data
        .options
        .first()
        // このコマンドが引数を受け付けた場合、その値はチャンネルであることが決まっているため、
        // この unwrap は必ず成功する
        .map(|opt| opt.value.as_channel_id().unwrap())
        // チャンネルが指定されなかった場合は、現在のチャンネルで設定する
        .unwrap_or_else(|| interaction.channel_id);

    // チャンネルがテキストチャンネルに類するか確認
    // サーバー内のチャンネルであることは分かっているので、unwrap は成功
    let guild_channel = channel.to_channel(&ctx.http).await?.guild().unwrap();

    if !matches!(
        guild_channel.kind,
        ChannelType::Text
            | ChannelType::News
            | ChannelType::NewsThread
            | ChannelType::PublicThread
            | ChannelType::PrivateThread
    ) {
        return error_response(
            ctx,
            interaction,
            "通知用チャンネルはテキストチャンネルに設定してください",
        )
        .await;
    }

    // コマンドへのリアクションが欲しく、それにまた返答したいので、
    // 1回目は ephemeral でやり取りし、2回目を正式なものとして投稿する
    interaction.defer_ephemeral(&ctx.http).await?;

    // 通知するアクティビティの種類を選ばせる
    let options = vec![
        CreateSelectMenuOption::new("感想あり", "with_comment"),
        CreateSelectMenuOption::new("エピソード記録", "record"),
        CreateSelectMenuOption::new("作品記録", "review"),
        CreateSelectMenuOption::new("感想なし", "without_comment"),
        CreateSelectMenuOption::new("ステータス更新", "status"),
    ];
    let num_options = options.len() as _;
    let select_menu = CreateSelectMenu::new("", CreateSelectMenuKind::String { options })
        .placeholder("通知するアクティビティの種類")
        .min_values(1)
        .max_values(num_options);

    let response = CreateInteractionResponseFollowup::new()
        .content("通知するアクティビティの種類を選択してください")
        .select_menu(select_menu);

    let message = interaction.create_followup(&ctx.http, response).await?;

    let Some(component) = message.await_component_interaction(&ctx.shard).await else {
        return Ok(());
    };

    let ComponentInteractionDataKind::StringSelect {
        values: selected_flags,
    } = &component.data.kind
    else {
        // セレクトメニューの作り方的にここには来ない
        unreachable!("unexpected component interaction data");
    };

    let mut notify_flag = NotifyFlag::empty();
    for selected in selected_flags {
        match selected.as_str() {
            "with_comment" => notify_flag |= NotifyFlag::WITH_COMMENT,
            "record" => notify_flag |= NotifyFlag::RECORD,
            "review" => notify_flag |= NotifyFlag::REVIEW,
            "without_comment" => notify_flag |= NotifyFlag::WITHOUT_COMMENT,
            "status" => notify_flag |= NotifyFlag::STATUS,
            s => unreachable!("unknown select menu option {}", s),
        }
    }

    if notify_flag.intersects(NotifyFlag::RECORD | NotifyFlag::REVIEW) {
        // エピソード・作品記録を通知する
        if !notify_flag.intersects(NotifyFlag::WITH_COMMENT | NotifyFlag::WITHOUT_COMMENT) {
            // 感想の有無が指定されていないときは両方通知する
            notify_flag |= NotifyFlag::WITH_COMMENT | NotifyFlag::WITHOUT_COMMENT;
        } else if notify_flag.contains(NotifyFlag::WITH_COMMENT)
            && !notify_flag.contains(NotifyFlag::WITHOUT_COMMENT)
            && notify_flag.contains(NotifyFlag::STATUS)
        {
            // 感想が必要とされているときは、ステータス更新はしないでいいので削る
            notify_flag &= !NotifyFlag::STATUS;
        }
    } else {
        // 感想の有無かステータス更新しか指定されていない
        if notify_flag.contains(NotifyFlag::STATUS) {
            // ステータス更新が指定されているときは感想ありは必要ないので削る
            notify_flag &= !NotifyFlag::WITH_COMMENT;
        } else {
            // 感想の有無しか指定されていない
            if notify_flag.contains(NotifyFlag::WITHOUT_COMMENT) {
                // 感想が無くても良い場合は全て通知する
                notify_flag |= NotifyFlag::RECORD | NotifyFlag::REVIEW | NotifyFlag::STATUS;
            } else {
                // 感想が必要な場合はエピソード・作品記録のみ
                notify_flag |= NotifyFlag::RECORD | NotifyFlag::REVIEW;
            }
        }
    }

    let mut conn = db::connect()?;
    db::insert_or_update_channel(&mut conn, guild.get(), channel.get(), notify_flag)?;

    let flags_text = if notify_flag.is_all() {
        "全て".into()
    } else {
        let about_comment =
            if !notify_flag.contains(NotifyFlag::WITH_COMMENT | NotifyFlag::WITHOUT_COMMENT) {
                // 感想ありかなしかのどちらかのみ
                if notify_flag.contains(NotifyFlag::WITH_COMMENT) {
                    "(感想あり)"
                } else {
                    "(感想なし)"
                }
            } else {
                ""
            };
        let mut flags_strs = vec![];
        if notify_flag.contains(NotifyFlag::RECORD) {
            flags_strs.push(format!("エピソード記録{}", about_comment));
        }
        if notify_flag.contains(NotifyFlag::REVIEW) {
            flags_strs.push(format!("作品記録{}", about_comment));
        }
        if notify_flag.contains(NotifyFlag::STATUS) {
            flags_strs.push("ステータス更新".into());
        }

        flags_strs.join("・")
    };

    // TODO: チャンネルの変更が伴う場合は、確認を行う
    let response = CreateInteractionResponseMessage::new().content(format!(
        "{} で {} のアクティビティを通知します",
        channel.mention(),
        flags_text,
    ));

    component
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await?;

    Ok(())
}

async fn error_response(
    ctx: &Context,
    interaction: &CommandInteraction,
    msg: impl Into<String>,
) -> Result<()> {
    interaction
        .create_response(
            &ctx.http,
            CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .ephemeral(true)
                    .content(msg),
            ),
        )
        .await
        .map_err(|e| e.into())
}
