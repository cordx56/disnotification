use super::notification::NotificationEvent;
use serenity::{
    builder::{CreateApplicationCommand, CreateApplicationCommandOption},
    model::{
        application::interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        id::{ChannelId, GuildId},
        prelude::{
            command::CommandOptionType, interaction::application_command::CommandDataOptionValue,
        },
    },
    prelude::Context,
};
use sqlx::{PgConnection, Pool, Postgres};
use std::{error, fmt};

enum ConfigSubCommandName {
    Channel,
    VoiceJoinChannel,
}
impl fmt::Display for ConfigSubCommandName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigSubCommandName::Channel => write!(f, "channel"),
            ConfigSubCommandName::VoiceJoinChannel => write!(f, "voice_join_channel"),
        }
    }
}

fn create_channel_sub_option(
    sub_option: &mut CreateApplicationCommandOption,
) -> &mut CreateApplicationCommandOption {
    sub_option
        .name("channel")
        .kind(CommandOptionType::Channel)
        .description("Channel to send notification")
        .required(true)
}
pub fn register_config(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("disnotconfig")
        .description("Update disnot configuration")
        .create_option(|option| {
            option
                .name(ConfigSubCommandName::Channel.to_string())
                .kind(CommandOptionType::SubCommand)
                .description("Update notification channel")
                .create_sub_option(create_channel_sub_option)
        })
        .create_option(|option| {
            option
                .name(ConfigSubCommandName::VoiceJoinChannel.to_string())
                .kind(CommandOptionType::SubCommand)
                .description("Update voice chat joining notification channel")
                .create_sub_option(create_channel_sub_option)
        })
}

async fn set_guild_event_notification_channel_id(
    conn: &mut PgConnection,
    guild_id: &GuildId,
    event_name: NotificationEvent,
    channel_id: &ChannelId,
) -> sqlx::Result<()> {
    sqlx::query(
        "insert into notification_channels (guild_id, event_name, channel_id) values ($1, $2, $3) on conflict (guild_id, event_name) do update set channel_id = $3",
    )
    .bind(guild_id.0.to_string())
    .bind(event_name.to_string())
    .bind(channel_id.0.to_string())
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn create_command_response(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    text: &str,
) -> serenity::Result<()> {
    command
        .create_interaction_response(ctx, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content(text))
        })
        .await
}

#[derive(Debug)]
pub enum ConfigError {
    DbError(String),
    InvalidCommand(String),
}
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::DbError(message) => write!(f, "{}", message),
            ConfigError::InvalidCommand(message) => write!(f, "{}", message),
        }
    }
}
impl error::Error for ConfigError {}

pub async fn run_config(
    pool: &Pool<Postgres>,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) -> Result<(), ConfigError> {
    if let Some(guild_id) = command.guild_id {
        let options = &command.data.options;
        if let Some(option) = options.get(0) {
            if option.name == ConfigSubCommandName::Channel.to_string() {
                if let Some(channel) = option.options.get(0) {
                    if let Some(CommandDataOptionValue::Channel(partial_channel)) =
                        channel.resolved.as_ref()
                    {
                        let channel_id = ChannelId(partial_channel.id.0);

                        let mut transaction = pool.begin().await.or(Err(ConfigError::DbError(
                            "DB transaction beginning error".to_string(),
                        )))?;
                        set_guild_event_notification_channel_id(
                            &mut transaction,
                            &guild_id,
                            NotificationEvent::VoiceJoin,
                            &channel_id,
                        )
                        .await
                        .or(Err(ConfigError::DbError("DB update error".to_string())))?;
                        set_guild_event_notification_channel_id(
                            &mut transaction,
                            &guild_id,
                            NotificationEvent::VoiceLeave,
                            &channel_id,
                        )
                        .await
                        .or(Err(ConfigError::DbError("DB update error".to_string())))?;
                        set_guild_event_notification_channel_id(
                            &mut transaction,
                            &guild_id,
                            NotificationEvent::VoiceMove,
                            &channel_id,
                        )
                        .await
                        .or(Err(ConfigError::DbError("DB update error".to_string())))?;
                        transaction
                            .commit()
                            .await
                            .or(Err(ConfigError::DbError("DB commit error".to_string())))?;

                        create_command_response(ctx, command, "Config updated!")
                            .await
                            .ok();
                        return Ok(());
                    }
                }
            } else if option.name == ConfigSubCommandName::VoiceJoinChannel.to_string() {
                if let Some(channel) = option.options.get(0) {
                    if let Some(CommandDataOptionValue::Channel(partial_channel)) =
                        channel.resolved.as_ref()
                    {
                        let channel_id = ChannelId(partial_channel.id.0);
                        let mut conn = pool
                            .acquire()
                            .await
                            .or(Err(ConfigError::DbError("DB connection error".to_string())))?;
                        set_guild_event_notification_channel_id(
                            &mut conn,
                            &guild_id,
                            NotificationEvent::VoiceJoin,
                            &channel_id,
                        )
                        .await
                        .ok();
                        command
                            .create_interaction_response(ctx, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message.content(
                                            "On voice join notification channel config updated!",
                                        )
                                    })
                            })
                            .await
                            .ok();
                        return Ok(());
                    }
                }
            }
        }
    }
    Err(ConfigError::InvalidCommand("Invalid command".to_string()))
}
