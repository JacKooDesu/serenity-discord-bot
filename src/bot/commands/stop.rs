use serenity::{
    all::Message,
    client::Context,
    framework::standard::{macros::command, CommandResult},
};

use crate::bot::common::{say, try_say, QueueKey};

#[command]
#[only_in(guilds)]
async fn stop(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird not initialized!")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        queue.stop();

        try_say(msg.channel_id, ctx, "Queue cleared!").await;

        if let Some(vec) = ctx.data.write().await.get_mut::<QueueKey>() {
            vec.clear()
        }
    } else {
        say(msg.channel_id, ctx, "Not in voice channel!").await
    }
    Ok(())
}
