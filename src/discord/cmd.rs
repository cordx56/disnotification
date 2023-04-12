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

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command.name("disnot").create_option(|option| {
        option
            .name("config")
            .kind(CommandOptionType::SubCommand)
            .create_sub_option(|sub_option| {
                sub_option
                    .name("channel")
                    .kind(CommandOptionType::SubCommand)
                    .create_sub_option(|sub_option| {
                        sub_option
                            .name("channel")
                            .kind(CommandOptionType::Channel)
                            .required(true)
                    })
            })
    })
}

async fn set_guild_notification_channel_id(
    pool: &Pool<Postgres>,
    guild_id: &GuildId,
    channel_id: &ChannelId,
) -> sqlx::Result<()> {
    sqlx::query("insert into notification_channels (guild_id, channel_id) values ($1, $2)")
        .bind(guild_id.0.to_string())
        .bind(channel_id.0.to_string())
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn run(pool: &Pool<Postgres>, ctx: &Context, command: &ApplicationCommandInteraction) {
    if let Some(guild_id) = command.guild_id {
        let options = &command.data.options;
        if let Some(option) = options.get(0) {
            if option.name == "config" {
                if let Some(sub_option) = option.options.get(0) {
                    if sub_option.name == "channel" {
                        if let Some(channel) = sub_option.options.get(0) {
                            if let Some(CommandDataOptionValue::Channel(partial_channel)) =
                                channel.resolved.as_ref()
                            {
                                if set_guild_notification_channel_id(
                                    pool,
                                    &guild_id,
                                    &ChannelId(partial_channel.id.0),
                                )
                                .await
                                .is_ok()
                                {
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
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
