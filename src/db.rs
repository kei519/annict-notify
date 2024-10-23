use chrono::{DateTime, Local, TimeZone};
use diesel::{
    query_dsl::methods::{FilterDsl, SelectDsl},
    Connection, ExpressionMethods, PgConnection, QueryResult, RunQueryDsl,
};

use crate::{
    discord::NotifyFlag,
    get_env,
    models::{Channel, NewSubscriber, Subscriber},
    schema::*,
    Result,
};

#[cfg(test)]
mod test;

pub fn connect() -> Result<PgConnection> {
    Ok(PgConnection::establish(&get_env("DATABASE_URL")?)?)
}

pub fn insert_or_update_channel(
    conn: &mut PgConnection,
    guild_id: u64,
    channel_id: u64,
    notify_flag: NotifyFlag,
) -> QueryResult<Channel> {
    let new_chan = Channel {
        channel_id: channel_id as _,
        guild_id: guild_id as _,
        notify_flag,
    };
    diesel::insert_into(channels::table)
        .values(new_chan)
        .on_conflict((channels::guild_id, channels::channel_id))
        .do_update()
        .set(channels::notify_flag.eq(notify_flag.bits()))
        .get_result(conn)
}

pub fn remove_channel(
    conn: &mut PgConnection,
    guild_id: u64,
    channel_id: u64,
) -> QueryResult<bool> {
    let num_deleted = diesel::delete(channels::table)
        .filter(channels::channel_id.eq(channel_id as i64))
        .filter(channels::guild_id.eq(guild_id as i64))
        .execute(conn)?;
    Ok(num_deleted >= 1)
}

pub fn get_channels(conn: &mut PgConnection) -> QueryResult<Vec<Channel>> {
    channels::table.load(conn)
}

pub fn insert_or_update_subscriber(
    conn: &mut PgConnection,
    user_id: u64,
    guild_id: u64,
    annict_name: impl AsRef<str>,
    end_cursor: Option<&str>,
    last_activity_date: Option<DateTime<impl TimeZone>>,
) -> QueryResult<Subscriber> {
    let new_subscriber = NewSubscriber {
        user_id: user_id as _,
        guild_id: guild_id as _,
        annict_name: annict_name.as_ref(),
        end_cursor,
        last_activity_date: last_activity_date.map(|dt| dt.to_utc()),
    };

    diesel::insert_into(subscribers::table)
        .values(&new_subscriber)
        .on_conflict((subscribers::guild_id, subscribers::user_id))
        .do_update()
        .set((
            subscribers::annict_name.eq(new_subscriber.annict_name),
            subscribers::end_cursor.eq(new_subscriber.end_cursor),
            subscribers::last_activity_date.eq(new_subscriber.last_activity_date),
        ))
        .get_result(conn)
}

pub fn update_subscriber_info(
    conn: &mut PgConnection,
    id: i32,
    end_cursor: Option<&str>,
    last_activity_date: Option<DateTime<impl TimeZone>>,
) -> QueryResult<Subscriber> {
    diesel::update(subscribers::table.filter(subscribers::id.eq(id)))
        .set((
            subscribers::end_cursor.eq(end_cursor),
            subscribers::last_activity_date.eq(last_activity_date),
        ))
        .get_result(conn)
}

pub fn get_subscribers_by_guild(
    conn: &mut PgConnection,
    guild_id: u64,
) -> QueryResult<Vec<Subscriber>> {
    subscribers::table
        .filter(subscribers::guild_id.eq(guild_id as i64))
        .load(conn)
}

pub fn get_last_activity_date(
    conn: &mut PgConnection,
    id: i32,
) -> QueryResult<Option<DateTime<Local>>> {
    subscribers::table
        .select(subscribers::last_activity_date)
        .filter(subscribers::id.eq(id))
        .first(conn)
}
