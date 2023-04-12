use super::notification::NotificationEvent;
use serenity::{
    builder::CreateApplicationCommand,
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
use sqlx::{Pool, Postgres};

pub fn register_config(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("disnotconfig")
        .description("Update disnot configuration")
        .create_option(|option| {
            option
                .name("channel")
                .kind(CommandOptionType::SubCommand)
                .description("Update notification channel")
                .create_sub_option(|sub_option| {
                    sub_option
                        .name("channel")
                        .kind(CommandOptionType::Channel)
                        .description("Channel to send notification")
                        .required(true)
                })
        })
}

async fn set_guild_event_notification_channel_id(
    pool: &Pool<Postgres>,
    guild_id: &GuildId,
    event_name: NotificationEvent,
    channel_id: &ChannelId,
) -> sqlx::Result<()> {
    sqlx::query(
        "insert into notification_channels (guild_id, event_name, channel_id) values ($1, $2, $3)",
    )
    .bind(guild_id.0.to_string())
    .bind(event_name.to_string())
    .bind(channel_id.0.to_string())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn run_config(
    pool: &Pool<Postgres>,
    ctx: &Context,
    command: &ApplicationCommandInteraction,
) {
    if let Some(guild_id) = command.guild_id {
        let options = &command.data.options;
        if let Some(option) = options.get(0) {
            if option.name == "channel" {
                if let Some(channel) = option.options.get(0) {
                    if let Some(CommandDataOptionValue::Channel(partial_channel)) =
                        channel.resolved.as_ref()
                    {
                        let channel_id = ChannelId(partial_channel.id.0);
                        set_guild_event_notification_channel_id(
                            pool,
                            &guild_id,
                            NotificationEvent::VoiceJoin,
                            &channel_id,
                        )
                        .await
                        .ok();
                        set_guild_event_notification_channel_id(
                            pool,
                            &guild_id,
                            NotificationEvent::VoiceLeave,
                            &channel_id,
                        )
                        .await
                        .ok();
                        set_guild_event_notification_channel_id(
                            pool,
                            &guild_id,
                            NotificationEvent::VoiceMove,
                            &channel_id,
                        )
                        .await
                        .ok();
                        command
                            .create_interaction_response(ctx, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message.content("Config updated!")
                                    })
                            })
                            .await
                            .ok();
                        return;
                    }
                }
            }
        }
    }
    command
        .create_interaction_response(ctx, |response| {
            response
                .kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| message.content("Invalid command"))
        })
        .await
        .ok();
}
