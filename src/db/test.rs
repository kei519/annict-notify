use std::env;

use chrono::{DateTime, Local};
use diesel::{Connection, PgConnection, QueryResult};

use crate::{db::get_last_activity_date, Result};

use super::{
    connect, get_channels, get_subscribers_by_guild, insert_or_update_channel,
    insert_or_update_subscriber,
};

fn test<R>(f: impl FnOnce(&mut PgConnection) -> QueryResult<R>) -> Result<R> {
    dotenv::dotenv().ok();
    env::set_var(
        "DATABASE_URL",
        env::var("DATABASE_TEST_URL").expect("環境変数 `DATABASE_TEST_URL` を設定してください"),
    );

    let mut conn = connect()?;
    Ok(conn.test_transaction(f))
}

#[test]
fn channels_test() -> Result<()> {
    test(|conn| {
        let channel = insert_or_update_channel(conn, 1, 1)?;
        assert_eq!(channel.guild_id, 1);
        assert_eq!(channel.channel_id, 1);

        let channels = get_channels(conn)?;
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0], channel);

        let channel = insert_or_update_channel(conn, 1, 32)?;
        assert_eq!(channel.guild_id, 1);
        assert_eq!(channel.channel_id, 32);

        let channels = get_channels(conn)?;
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0], channel);

        Ok(())
    })
}

#[test]
fn subscribers_test() -> Result<()> {
    test(|conn| {
        let subscriber =
            insert_or_update_subscriber(conn, 1, 1, "kei519", None, None::<DateTime<Local>>)?;
        assert_eq!(subscriber.user_id, 1);
        assert_eq!(subscriber.guild_id, 1);
        assert_eq!(subscriber.annict_name, "kei519");
        assert!(subscriber.end_cursor.is_none());
        assert!(subscriber.last_activity_date.is_none());

        let subscribers = get_subscribers_by_guild(conn, subscriber.guild_id as _)?;
        assert_eq!(subscribers.len(), 1);
        assert_eq!(subscribers[0], subscriber);

        let datetime = Local::now().format("%Y-%m-%dT%H:%M:%S%:z").to_string();
        let datetime = DateTime::parse_from_rfc3339(&datetime).unwrap();

        let subscriber =
            insert_or_update_subscriber(conn, 1, 1, "hoge", Some("fuga"), Some(datetime))?;

        assert_eq!(subscriber.user_id, 1);
        assert_eq!(subscriber.guild_id, 1);
        assert_eq!(subscriber.annict_name, "hoge");
        assert_eq!(subscriber.end_cursor.as_ref().unwrap(), "fuga");
        assert_eq!(subscriber.last_activity_date.unwrap(), datetime);

        let last_activity_date = get_last_activity_date(conn, subscriber.id)?;
        assert_eq!(last_activity_date.unwrap(), datetime);

        let subscribers = get_subscribers_by_guild(conn, subscriber.guild_id as _)?;
        assert_eq!(subscribers.len(), 1);
        assert_eq!(subscribers[0], subscriber);

        Ok(())
    })
}
