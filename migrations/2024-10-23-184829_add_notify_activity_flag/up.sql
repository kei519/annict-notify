-- Your SQL goes here

-- 主キー制約を guild_id から (guild_id, channel_id) に変更
ALTER TABLE channels DROP CONSTRAINT channels_pkey;
ALTER TABLE channels ADD CONSTRAINT channels_pkey PRIMARY KEY(guild_id, channel_id);

-- notify_flag を追加
ALTER TABLE channels ADD COLUMN notify_flag INTEGER NOT NULL DEFAULT 0x1f;
