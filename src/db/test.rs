use chrono::{DateTime, Local};
use diesel::{Connection, PgConnection, QueryResult};

use crate::Result;

use super::{
    connect, get_channels, get_subscribers_by_guild, insert_or_update_channel,
    insert_or_update_subscriber,
};

fn test<R>(f: impl FnOnce(&mut PgConnection) -> QueryResult<R>) -> Result<R> {
    dotenv::dotenv().ok();

    let mut conn = connect()?;
    Ok(conn.test_transaction(f))
}

#[test]
fn channels_test() -> Result<()> {
    test(|conn| {
        let channel = insert_or_update_channel(conn, 1, 1)?;
        assert!(channel.guild_id == 1 && channel.channel_id == 1);

        let channels = get_channels(conn)?;
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0], channel);

        let channel = insert_or_update_channel(conn, 1, 32)?;
        assert!(channel.guild_id == 1 && channel.channel_id == 32);

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
        assert!(
            subscriber.user_id == 1
                && subscriber.guild_id == 1
                && subscriber.annict_name == "kei519"
                && subscriber.end_cursor.is_none()
                && subscriber.last_activity_date.is_none()
        );

        let subscribers = get_subscribers_by_guild(conn, subscriber.guild_id as _)?;
        assert_eq!(subscribers.len(), 1);
        assert_eq!(subscribers[0], subscriber);

        let subscriber =
            insert_or_update_subscriber(conn, 1, 1, "hoge", Some("fuga"), Some(Local::now()))?;
        assert!(
            subscriber.user_id == 1
                && subscriber.guild_id == 1
                && subscriber.annict_name == "hoge"
                && subscriber.end_cursor.as_ref().unwrap() == "fuga"
                && subscriber.last_activity_date.is_some()
        );

        let subscribers = get_subscribers_by_guild(conn, subscriber.guild_id as _)?;
        assert_eq!(subscribers.len(), 1);
        assert_eq!(subscribers[0], subscriber);

        Ok(())
    })
}
