create table if not exists notification_channels (
    guild_id text,
    event_name text,
    channel_id text not null,
    primary key (guild_id, event_name)
);
