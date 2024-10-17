use std::{future::Future, sync::Arc, time::Duration};

use regex::Regex;
use serenity::{
    all::{EventHandler, GatewayIntents, Http},
    Client,
};
use tokio::time;

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
    let interval = get_interval()?;
    tracing::info!("更新間隔: {} 秒", interval.as_secs());
    loop {
        tracing::trace!("loop!");
        time::sleep(interval).await;
    }
}

pub struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {}

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
