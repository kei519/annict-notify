use std::env;

mod schema;

pub async fn main() -> u8 {
    let Ok(envvar) = env::var("ENV_VAR") else {
        tracing::error!("環境変数 `ENV_VAR` を設定してください");
        return 1;
    };
    println!("{}", envvar);

    0
}
