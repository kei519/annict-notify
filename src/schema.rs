// @generated automatically by Diesel CLI.

diesel::table! {
    channels (guild_id) {
        guild_id -> Int8,
        channel_id -> Int8,
    }
}

diesel::table! {
    subscribers (id) {
        id -> Int4,
        user_id -> Int8,
        guild_id -> Int8,
        annict_name -> Text,
        end_cursor -> Nullable<Text>,
        last_activity_date -> Nullable<Timestamptz>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    channels,
    subscribers,
);
