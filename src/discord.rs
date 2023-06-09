mod cmd;
mod notification;

use serenity::{
    model::{
        application::{
            command::Command,
            interaction::{Interaction, InteractionResponseType},
        },
        gateway::Ready,
        voice::VoiceState,
    },
    prelude::{Context, EventHandler},
};
use sqlx::{Pool, Postgres};

pub struct DisNotHandler {
    pub db_pool: Pool<Postgres>,
}

#[serenity::async_trait]
impl EventHandler for DisNotHandler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            match command.data.name.as_str() {
                "disnotconfig" => {
                    let config_result = cmd::run_config(&self.db_pool, &ctx, &command).await;
                    match config_result {
                        Err(cmd::ConfigError::DbError(message)) => {
                            cmd::create_command_response(&ctx, &command, &message)
                                .await
                                .ok();
                        }
                        Err(cmd::ConfigError::InvalidCommand(message)) => {
                            cmd::create_command_response(&ctx, &command, &message)
                                .await
                                .ok();
                        }
                        _ => {}
                    }
                }
                _ => {
                    command
                        .create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| {
                                    message.content("Invalid command - command not found")
                                })
                        })
                        .await
                        .ok();
                }
            }
        }
    }

    async fn voice_state_update(&self, ctx: Context, old: Option<VoiceState>, new: VoiceState) {
        notification::voice_state_notification(&self.db_pool, &ctx, &old, &new).await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("Connected as {}", ready.user.name);

        Command::set_global_application_commands(&ctx.http, |commands| {
            commands.create_application_command(|command| cmd::register_config(command))
        })
        .await
        .expect("Slash command settings failed!");
    }
}
