use serenity::all::{
    ChannelType, CommandInteraction, CommandOptionType, Context, CreateCommand,
    CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage, Mentionable,
    Permissions,
};

use crate::{db, Result};

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
    let response = CreateInteractionResponseMessage::new();

    let response = if let Some(guild) = &interaction.guild_id {
        // サーバー内の場合
        let channel = interaction
            .data
            .options
            .first()
            // このコマンドが引数を受け付けた場合、その値はチャンネルであることが決まっているため、
            // この unwrap は必ず成功する
            .map(|opt| opt.value.as_channel_id().unwrap())
            .unwrap_or_else(|| interaction.channel_id);

        // チャンネルがテキストチャンネルに類するか確認
        // サーバー内のチャンネルであることは分かっているので、unwrap は成功
        let guild_channel = channel.to_channel(&ctx.http).await?.guild().unwrap();
        if matches!(
            guild_channel.kind,
            ChannelType::Text
                | ChannelType::News
                | ChannelType::NewsThread
                | ChannelType::PublicThread
                | ChannelType::PrivateThread
        ) {
            // TODO: チャンネルの変更が伴う場合は、確認を行う
            let mut conn = db::connect()?;
            db::insert_or_update_channel(&mut conn, guild.get(), channel.get())?;

            response.content(format!(
                "通知用チャンネルを {} に設定しました",
                channel.mention()
            ))
        } else {
            response
                .content("通知用チャンネルはテキストチャンネルに設定してください")
                .ephemeral(true)
        }
    } else {
        // DM の場合
        response.content("この操作はサーバー内で行ってください")
    };

    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await?;

    Ok(())
}
