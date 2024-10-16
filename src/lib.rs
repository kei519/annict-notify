use std::{env, error::Error};

mod schema;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub async fn main() -> Result<()> {
    let Ok(envvar) = env::var("ENV_VAR") else {
        return Err("環境変数 `ENV_VAR` を設定してください".into());
    };
    println!("{}", envvar);

    Ok(())
}
