use std::process::ExitCode;

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> ExitCode {
    // ロガーの設定
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| env!("CARGO_CRATE_NAME").into()))
        .with(fmt::layer())
        .init();

    // .env の読み込み
    dotenv::dotenv().ok();

    if let Err(e) = annict_notify::main().await {
        tracing::error!("{}", e);
        1.into()
    } else {
        0.into()
    }
}
