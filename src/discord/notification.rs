use serenity::{
    model::{
        application::{
            command::Command,
            interaction::{Interaction, InteractionResponseType},
        },
        gateway::Ready,
        id::{ChannelId, GuildId},
        voice::VoiceState,
    },
    prelude::{Context, EventHandler},
    utils::Color,
};
use sqlx::{Pool, Postgres};

async fn get_guild_notification_channel_id_from_db(
    pool: &Pool<Postgres>,
    guild_id: &GuildId,
) -> sqlx::Result<Option<ChannelId>> {
    let row: Option<(String,)> =
        sqlx::query_as("select channel_id from notification_channels where guild_id = $1")
            .bind(guild_id.0.to_string())
            .fetch_optional(pool)
            .await?;
    if let Some((channel_id_string,)) = row {
        Ok(Some(ChannelId(channel_id_string.parse().unwrap())))
    } else {
        Ok(None)
    }
}

pub async fn voice_state_notification(
    pool: &Pool<Postgres>,
    ctx: &Context,
    old: &Option<VoiceState>,
    new: &VoiceState,
) {
    if let Some(guild_id) = new.guild_id {
        let channel_id_res = get_guild_notification_channel_id_from_db(pool, &guild_id).await;
        if let Ok(Some(channel_id)) = channel_id_res {
            if let Some(member) = &new.member {
                let member_display_name = member.display_name();
                let member_user_id = member.user.id;
                let member_avatar_url = member.face();
                if let Some(old_vs) = old {
                    if let Some(old_vc_channel_id) = old_vs.channel_id {
                        if let Some(old_vc_channel_name) = old_vc_channel_id.name(&ctx).await {
                            if let Some(new_vc_channel_id) = new.channel_id {
                                if let Some(new_vc_channel_name) =
                                    new_vc_channel_id.name(&ctx).await
                                {
                                    if old_vc_channel_id != new_vc_channel_id {
                                        channel_id
                                            .send_message(ctx, |m| m.add_embed(|e| {
                                                e.title(
                                                    format!("{member_display_name} moved VC!")
                                                )
                                                .description(
                                                    format!("<@{member_user_id}> moved from {old_vc_channel_name} to {new_vc_channel_name}!")
                                                )
                                                .color(Color::from_rgb(23, 162, 184))
                                                .thumbnail(&member_avatar_url)
                                            }))
                                            .await
                                            .ok();
                                    }
                                }
                            } else {
                                channel_id
                                    .send_message(ctx, |m| {
                                        m.add_embed(|e| {
                                            e.title(format!("{member_display_name} left VC!"))
                                                .description(format!(
                                                "<@{member_user_id}> left {old_vc_channel_name}!"
                                            ))
                                                .color(Color::from_rgb(220, 53, 59))
                                                .thumbnail(&member_avatar_url)
                                        })
                                    })
                                    .await
                                    .ok();
                            }
                        }
                    }
                } else {
                    if let Some(new_vc_channel_id) = new.channel_id {
                        if let Some(new_vc_channel_name) = new_vc_channel_id.name(ctx).await {
                            channel_id
                                .send_message(ctx, |m| {
                                    m.add_embed(|e| {
                                        e.title(format!("{member_display_name} joined VC!"))
                                            .description(format!(
                                                "<@{member_user_id}> joined {new_vc_channel_name}!"
                                            ))
                                            .color(Color::from_rgb(40, 167, 69))
                                            .thumbnail(&member_avatar_url)
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
