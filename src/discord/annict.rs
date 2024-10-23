use serenity::all::{
    CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage,
};

use crate::{annict, Result};

pub(super) const NAME: &str = "annict";

pub(super) fn register() -> CreateCommand {
    let option = CreateCommandOption::new(
        CommandOptionType::String,
        "ユーザー名",
        "連携する Annict アカウントのユーザー名",
    )
    .required(true);
    CreateCommand::new(NAME)
        .description("Annict アカウントとの連携を行います")
        .add_option(option)
}

pub(super) async fn handle(ctx: &Context, interaction: &CommandInteraction) -> Result<()> {
    let response = CreateInteractionResponseMessage::new();

    let response = if let Some(guild) = &interaction.guild_id {
        // サーバー内の場合
        let username = interaction
            .data
            .options
            .first()
            // 引数は必須になっているから、この unwrap は必ず成功する
            .unwrap()
            .value
            .as_str()
            // 引数の値は文字列であることが決まっているため、この unwrap は必ず成功する
            .unwrap();

        if annict::register_user(username, interaction.user.id.get(), guild.get()).await? {
            // TODO: 既に登録されている場合に変更してもよいか確認する
            response.content(format!(
                // プレビューさせないために < > で囲う
                "ユーザー [{0}](<https://annict.com/@{0}>) と連携しました",
                username,
            ))
        } else {
            response
                .content(format!("ユーザー {} は存在しません", username))
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
