use std::sync::Arc;

use crate::bot::{
    commands::{
        join::{join_voice, JoinActionEnum},
        play::create_song_begin_event,
    },
    common::{add_song, check_msg, try_say}, clients::HttpKey,
};
use serde::{Deserialize, Serialize};
use serenity::{
    all::Message,
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    json,
};

use songbird::input::YoutubeDl;

#[command]
pub async fn playlist(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
    println!("try playlist: {:?}", &url);
    let http_client = {
        let data = ctx.data.read().await;
        let http_client = data
            .get::<HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.");

        http_client
    };

    let channel_id = msg.channel_id;
    let send_http = ctx.http.clone();

    if !url.contains("list") {
        try_say(
            msg.channel_id,
            ctx,
            "Song can only played by using `play` command!",
        )
        .await;
    } else {
        if let Err(Some(err_msg)) = join_voice(ctx, JoinActionEnum::ByMessage(msg.clone())).await {
            try_say(msg.channel_id, ctx, err_msg).await;
            return Ok(());
        }

        let songs = flat_list(url.as_str()).await;
        for s in songs {
            let src = YoutubeDl::new(http_client.clone(), s);
            if let Some(result) = add_song(ctx, guild_id, src).await {
                if let Some(_) = result.1 {
                    create_song_begin_event(
                        send_http.clone(),
                        Arc::new(ctx.clone()),
                        result.0,
                        channel_id,
                    )
                    .await;
                }
            }
        }
        try_say(channel_id, ctx, "Playlist added!").await;
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
