use std::sync::Arc;

use serenity::{
    all::{ChannelId, GuildId, User},
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};
use songbird::TrackEvent;

use crate::bot::common::{get_manager, try_say, TrackEndNotifier, TrackErrorNotifier};

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    if let Err(Some(err_msg)) = join_voice(ctx, JoinActionEnum::ByMessage(msg.clone())).await {
        try_say(msg.channel_id, ctx, err_msg).await;
    }
    Ok(())
}

pub(crate) async fn join_voice(ctx: &Context, action: JoinActionEnum) -> Result<(), Option<&str>> {
    let manager = get_manager(&ctx).await;

    let action_result: Result<(GuildId, ChannelId), _> = match action {
        JoinActionEnum::Direct(guild_id, channel) => Ok((guild_id, channel)),
        JoinActionEnum::ByMessage(msg) => {
            let args = msg.guild(&ctx.cache).and_then(|g| {
                if let Some(channel) = g
                    .voice_states
                    .get(&msg.author.id)
                    .and_then(|s| s.channel_id)
                {
                    Some((g.id, channel))
                } else {
                    None
                }
            });

            if let Some((guild, channel)) = args {
                Ok((guild, channel))
            } else {
                Err("You're not in voice channel!")
            }
        }
        JoinActionEnum::ByUser(guild_id, user) => {
            if let Some(channel) = ctx
                .cache
                .guild(guild_id)
                .and_then(|g| g.voice_states.get(&user.id).and_then(|s| s.channel_id))
            {
                Ok((guild_id, channel))
            } else {
                Err("You're not in voice channel!")
            }
        }
    };

    if let Ok((guild, channel)) = action_result {
        if manager.get(guild).is_some() {
            return Ok(());
        }

        if let Ok(handler_lock) = manager.join(guild, channel).await {
            let mut handler = handler_lock.lock().await;
            handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
            handler.add_global_event(
                TrackEvent::End.into(),
                TrackEndNotifier {
                    guild_id: guild,
                    songbird: manager,
                    context: Arc::new(ctx.clone()),
                },
            );
            return Ok(());
        }
    } else {
        if let Some(err) = action_result.err() {
            return Err(Some(err));
        }
    }

    Err(None)
}

pub(crate) enum JoinActionEnum {
    Direct(GuildId, ChannelId),
    ByMessage(Message),
    ByUser(GuildId, User),
}
