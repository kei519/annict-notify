use diesel::{Connection, ExpressionMethods, PgConnection, QueryResult, RunQueryDsl};

use crate::{get_env, models::Channel, schema::*, Result};

pub fn connect() -> Result<PgConnection> {
    Ok(PgConnection::establish(&get_env("DATABASE_URL")?)?)
}

pub fn insert_or_update_channel(
    conn: &mut PgConnection,
    guild_id: u64,
    channel_id: u64,
) -> QueryResult<Channel> {
    let new_chan = Channel {
        channel_id: channel_id as _,
        guild_id: guild_id as _,
    };
    diesel::insert_into(channels::table)
        .values(&new_chan)
        .on_conflict(channels::guild_id)
        .do_update()
        .set(channels::channel_id.eq(new_chan.channel_id))
        .get_result(conn)
}
