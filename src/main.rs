use std::{env, process::ExitCode};

use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> ExitCode {
    // .env の読み込み
    dotenv::dotenv().ok();

    // ロガーの設定
    const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::INFO;
    let level_filter: LevelFilter = env::var("LOG_LEVEL")
        .map(|s| s.parse().unwrap_or(DEFAULT_LOG_LEVEL))
        .unwrap_or(DEFAULT_LOG_LEVEL);
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| env!("CARGO_CRATE_NAME").into()))
        .with(fmt::layer())
        .with(level_filter)
        .init();

    if let Err(e) = annict_notify::main().await {
        tracing::error!("{}", e);
        1.into()
    } else {
        0.into()
    }
}
