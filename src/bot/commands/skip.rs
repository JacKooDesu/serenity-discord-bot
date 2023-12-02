use serenity::{
    all::Message,
    client::Context,
    framework::standard::{macros::command, CommandResult},
};

use crate::bot::common::{say, try_say};

#[command]
#[only_in(guilds)]
async fn skip(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird not initailzied!")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.skip();

        try_say(msg.channel_id, ctx, "Skipped!").await
    } else {
        say(msg.channel_id, ctx, "Not in voice channel!").await
    }
    Ok(())
}
