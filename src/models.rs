use diesel::prelude::{Insertable, Queryable};

use super::schema::*;

#[derive(Debug, Queryable, Insertable, PartialEq, Eq)]
#[diesel(table_name = channels)]
pub struct Channel {
    pub guild_id: i64,
    pub channel_id: i64,
}
