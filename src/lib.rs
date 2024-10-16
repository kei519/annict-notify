use std::{
    env::{self, VarError},
    error::Error,
};

use diesel::{Connection, PgConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};

mod schema;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

const MIGRATIONS: EmbeddedMigrations = diesel_migrations::embed_migrations!("./migrations");

pub async fn main() -> Result<()> {
    // 反映されていないマイグレーションの実行
    let mut conn = PgConnection::establish(&get_env("DATABASE_URL")?)?;
    conn.run_pending_migrations(MIGRATIONS)?;

    Ok(())
}

/// 環境変数 `key` を取り出す。
/// ただし、存在しなかった場合は分かりやすいエラーメッセージを表示するエラーを返す。
pub fn get_env(key: impl AsRef<str>) -> Result<String> {
    match env::var(key.as_ref()) {
        Err(VarError::NotPresent) => {
            Err(format!("環境変数 `{}` を設定してください", key.as_ref()).into())
        }
        other => Ok(other?),
    }
}
