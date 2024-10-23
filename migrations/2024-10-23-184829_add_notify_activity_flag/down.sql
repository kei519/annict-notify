-- This file should undo anything in `up.sql`

-- notify_flag 列を削除
ALTER TABLE channels DROP COLUMN notify_flag;

-- 主キー制約を (guild_id, channel_id) から guild_id に変更
ALTER TABLE channels DROP CONSTRAINT channels_pkey;
ALTER TABLE channels ADD CONSTRAINT channels_pkey PRIMARY KEY(guild_id);
