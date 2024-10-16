use std::env;

fn main() {
    // .env の読み込み
    dotenv::dotenv().ok();

    let envvar = env::var("ENV_VAR").expect("環境変数 `ENV_VAR` を設定してください");
    println!("{}", envvar);
}
