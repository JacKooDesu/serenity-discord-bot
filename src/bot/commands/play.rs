use std::sync::Arc;

use crate::bot::common::get_manager;

use super::super::common::{check_msg, say, try_say, HttpKey, SongEndedNotifier, VolumeKey};
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use songbird::events::Event;
use songbird::{input::YoutubeDl, TrackEvent};

#[command]
#[only_in(guilds)]
pub(super) async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let url = match args.single::<String>() {
        Ok(url) => url,
        Err(_) => {
            check_msg(msg.channel_id.say(&ctx.http, "URL unknow!!").await);
            return Ok(());
        }
    };

    if !url.starts_with("http") {
        check_msg(msg.channel_id.say(&ctx.http, "Invalid URL!!").await);
        return Ok(());
    }

    let guild_id = msg.guild_id.unwrap();
    println!("try playing: {:?}", &url);
    let (http_client, volume) = {
        let data = ctx.data.read().await;
        let http_client = data
            .get::<HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.");

        let volume = data.get::<VolumeKey>().cloned().unwrap_or(1_f32);

        (http_client, volume)
    };

    let manager = get_manager(ctx).await;

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let channel_id = msg.channel_id;
        let send_http = ctx.http.clone();

        if url.contains("list") {
            try_say(
                msg.channel_id,
                ctx,
                "Playlist can only played by using `playlist` command!",
            )
            .await;
        } else {
            let src = YoutubeDl::new(http_client, url);
            let track = handler.enqueue_input(src.into()).await;
            let _ = track.set_volume(volume);

            let _ = track.add_event(
                Event::Track(TrackEvent::End),
                SongEndedNotifier {
                    channel_id: channel_id,
                    http: send_http,
                    contex: Arc::new(ctx.clone()),
                },
            );
            try_say(msg.channel_id, ctx, "Playing!").await;
        }
    } else {
        say(msg.channel_id, ctx, "Not in voice channel!").await;
    }

    Ok(())
}
