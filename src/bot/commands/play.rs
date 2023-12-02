use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::bot::common::get_manager;

use super::super::common::{check_msg, say, try_say, HttpKey, SongEndedNotifier, VolumeKey};
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    json,
    model::channel::Message,
};
use songbird::events::Event;
use songbird::{input::YoutubeDl, TrackEvent};

#[command]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
            let songs = flat_list(url.as_str()).await;
            for s in songs {
                let src = YoutubeDl::new(http_client.clone(), s);
                let track = handler.enqueue_input(src.into()).await;
                let _ = track.set_volume(volume);

                let _ = track.add_event(
                    Event::Track(TrackEvent::End),
                    SongEndedNotifier {
                        channel_id: channel_id,
                        http: send_http.clone(),
                        contex: Arc::new(ctx.clone()),
                    },
                );
            }
            try_say(msg.channel_id, ctx, "Playlist added!").await;
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

pub async fn flat_list(list: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let args = [list, "--flat-playlist", "-j"];
    if let Ok(output) = tokio::process::Command::new("yt-dlp")
        .args(args)
        .output()
        .await
    {
        let _ = String::from_utf8(output.stdout.to_vec())
            .unwrap_or(String::new())
            .split("\n")
            .collect::<Vec<&str>>()
            .iter()
            .for_each(|x| {
                if let Ok(o) = json::from_str::<Output>(*x) {
                    if let Some(url) = o.url {
                        result.push(url)
                    }
                }
            });
    }
    result
}
#[derive(Deserialize, Serialize, Debug)]
struct Output {
    pub url: Option<String>,
}
