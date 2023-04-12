use serenity::{
    model::{
        id::{ChannelId, GuildId},
        voice::VoiceState,
    },
    prelude::Context,
    utils::Color,
};
use sqlx::{Pool, Postgres};
use std::fmt;

pub enum NotificationEvent {
    VoiceJoin,
    VoiceLeave,
    VoiceMove,
}

impl fmt::Display for NotificationEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            NotificationEvent::VoiceJoin => write!(f, "voice_join"),
            NotificationEvent::VoiceLeave => write!(f, "voice_leave"),
            NotificationEvent::VoiceMove => write!(f, "voice_move"),
        }
    }
}

async fn get_guild_event_notification_channel_id_from_db(
    pool: &Pool<Postgres>,
    guild_id: &GuildId,
    event_name: NotificationEvent,
) -> sqlx::Result<Option<ChannelId>> {
    let row: Option<(String,)> = sqlx::query_as(
        "select channel_id from notification_channels where guild_id = $1 and event_name = $2",
    )
    .bind(guild_id.0.to_string())
    .bind(event_name.to_string())
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
        if let Some(member) = &new.member {
            let member_display_name = member.display_name();
            let member_user_id = member.user.id;
            let member_avatar_url = member.face();
            if let Some(old_vs) = old {
                if let Some(old_vc_channel_id) = old_vs.channel_id {
                    let old_vc_channel_id_num = old_vc_channel_id.0;
                    if let Some(new_vc_channel_id) = new.channel_id {
                        let new_vc_channel_id_num = new_vc_channel_id.0;
                        if old_vc_channel_id != new_vc_channel_id {
                            let channel_id_res = get_guild_event_notification_channel_id_from_db(
                                pool,
                                &guild_id,
                                NotificationEvent::VoiceMove,
                            )
                            .await;
                            if let Ok(Some(channel_id)) = channel_id_res {
                                channel_id
                                    .send_message(ctx, |m| m.add_embed(|e| {
                                        e.title(
                                            format!("{member_display_name} moved VC!")
                                        )
                                        .description(
                                            format!("<@{member_user_id}> moved from <#{old_vc_channel_id_num}> to <#{new_vc_channel_id_num}>!")
                                        )
                                        .color(Color::from_rgb(23, 162, 184))
                                        .thumbnail(&member_avatar_url)
                                    }))
                                    .await
                                    .ok();
                            }
                        }
                    } else {
                        let channel_id_res = get_guild_event_notification_channel_id_from_db(
                            pool,
                            &guild_id,
                            NotificationEvent::VoiceLeave,
                        )
                        .await;
                        if let Ok(Some(channel_id)) = channel_id_res {
                            channel_id
                                .send_message(ctx, |m| {
                                    m.add_embed(|e| {
                                        e.title(format!("{member_display_name} left VC!"))
                                            .description(format!(
                                                "<@{member_user_id}> left <#{old_vc_channel_id_num}>!"
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
                    let new_vc_channel_id_num = new_vc_channel_id.0;
                    let channel_id_res = get_guild_event_notification_channel_id_from_db(
                        pool,
                        &guild_id,
                        NotificationEvent::VoiceJoin,
                    )
                    .await;
                    if let Ok(Some(channel_id)) = channel_id_res {
                        channel_id
                            .send_message(ctx, |m| {
                                m.add_embed(|e| {
                                    e.title(format!("{member_display_name} joined VC!"))
                                        .description(format!(
                                            "<@{member_user_id}> joined <#{new_vc_channel_id_num}>!"
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
