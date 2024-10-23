use chrono::{DateTime, Utc};
use diesel::prelude::{Insertable, Queryable};

use super::schema::*;

#[derive(Debug, Queryable, Insertable, PartialEq, Eq)]
#[diesel(table_name = channels)]
pub struct Channel {
    pub guild_id: i64,
    pub channel_id: i64,
}

#[derive(Debug, Queryable, PartialEq, Eq)]
pub struct Subscriber {
    pub id: i32,
    pub user_id: i64,
    pub guild_id: i64,
    pub annict_name: String,
    pub end_cursor: Option<String>,
    pub last_activity_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = subscribers)]
pub struct NewSubscriber<'a, 'b> {
    pub user_id: i64,
    pub guild_id: i64,
    pub annict_name: &'a str,
    pub end_cursor: Option<&'b str>,
    pub last_activity_date: Option<DateTime<Utc>>,
}
