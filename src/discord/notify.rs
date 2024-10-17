use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage, Permissions,
};

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

pub(super) async fn handle(ctx: &Context, interaction: &CommandInteraction) {
    let response = CreateInteractionResponseMessage::new();

    let response = if let Some(_guild) = &interaction.guild_id {
        // サーバー内の場合
        todo!("通知用チャンネル登録処理")
    } else {
        // DM の場合
        response.content("この操作はサーバー内で行ってください")
    };

    if let Err(e) = interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(response))
        .await
    {
        tracing::warn!("{}", e);
    }
}
