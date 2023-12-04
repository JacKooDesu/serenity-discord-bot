use std::sync::Arc;

use crate::bot::common::{get_manager, SongBeginNotifier};

use super::super::common::{check_msg, say, try_say, HttpKey, SongEndedNotifier, VolumeKey};
use reqwest::Client;
use serenity::{
    all::ChannelId,
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    http::Http,
    model::channel::Message,
};
use songbird::{
    events::Event,
    input::{AuxMetadata, Compose, Input},
    tracks::TrackHandle,
};
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
            let mut src = YoutubeDl::new(http_client, url);
            let metadata = src.aux_metadata().await;
            let track = handler.enqueue_input(src.into()).await;
            let _ = track.set_volume(volume);
            if let Ok(meta) = metadata {
                create_song_begin_event(meta, send_http, track, channel_id).await;
            }
            try_say(msg.channel_id, ctx, "Song Added!").await;
        }
    } else {
        say(msg.channel_id, ctx, "Not in voice channel!").await;
    }

    Ok(())
}

pub(super) async fn create_song_begin_event(
    meta: AuxMetadata,
    send_http: Arc<Http>,
    track: TrackHandle,
    channel_id: ChannelId,
) {
    if let Some(title) = meta.title {
        let _ = track.add_event(
            Event::Track(TrackEvent::Play),
            SongBeginNotifier {
                channel_id: channel_id,
                cache_http: send_http,
                title: title,
            },
        );
    }
}
