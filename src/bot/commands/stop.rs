use serenity::{
    all::Message,
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
};

use crate::bot::common::{say, try_say};

#[command]
#[only_in(guilds)]
async fn stop(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird not initialized!")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        queue.stop();

        try_say(msg.channel_id, ctx, "Queue cleared!").await
    } else {
        say(msg.channel_id, ctx, "Not in voice channel!").await
    }
    Ok(())
}
