use std::env;

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() {
    // ロガーの設定
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| env!("CARGO_CRATE_NAME").into()))
        .with(fmt::layer())
        .init();

    // .env の読み込み
    dotenv::dotenv().ok();

    let envvar = env::var("ENV_VAR").expect("環境変数 `ENV_VAR` を設定してください");
    println!("{}", envvar);
}
