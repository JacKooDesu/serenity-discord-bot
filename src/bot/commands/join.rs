use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandResult},
    model::channel::Message,
};
use songbird::TrackEvent;

use crate::bot::common::{try_say, TrackErrorNotifier};

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let (guild_id, channel_id) = {
        let guild = msg.guild(&ctx.cache).unwrap();
        let channel = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|state| state.channel_id);

        (guild.id, channel)
    };

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            try_say(msg.channel_id, ctx, "You're not in voice channel!!").await;
            return Ok(());
        }
    };

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Ok(handler_lock) = manager.join(guild_id, connect_to).await {
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier)
    }

    Ok(())
}
