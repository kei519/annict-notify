-- Your SQL goes here
CREATE TABLE channels (
    guild_id BIGINT PRIMARY KEY,
    channel_id BIGINT NOT NULL
);

CREATE TABLE subscribers (
    id SERIAL PRIMARY KEY,
    -- discord
    user_id BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    -- annict
    annict_name TEXT NOT NULL,
    end_cursor TEXT,
    last_activity_date TIMESTAMP (0) WITH TIME ZONE
);

CREATE UNIQUE INDEX user_and_guild ON subscribers (user_id, guild_id);
