use std::{future::Future, sync::Arc};

use serenity::{
    all::{EventHandler, GatewayIntents, Http},
    Client,
};

use crate::{get_env, Result};

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
    todo!()
}

pub struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {}
