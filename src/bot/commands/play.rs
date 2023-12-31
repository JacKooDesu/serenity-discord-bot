use std::sync::Arc;

use crate::bot::{
    commands::join::{join_voice, JoinActionEnum},
    common::{add_song, SongBeginNotifier},
};

use super::super::common::{check_msg, try_say, HttpKey};
use serenity::{
    all::ChannelId,
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    http::Http,
    model::channel::Message,
};
use songbird::{events::Event, tracks::TrackHandle};
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

    if url.contains("list") {
        try_say(
            msg.channel_id,
            ctx,
            "Playlist can only played by using `playlist` command!",
        )
        .await;
    } else {
        let src = YoutubeDl::new(http_client, url);

        if let Err(Some(err_msg)) = join_voice(ctx, JoinActionEnum::ByMessage(msg.clone())).await {
            try_say(msg.channel_id, ctx, err_msg).await;
            return Ok(());
        }

        if let Some(result) = add_song(ctx, guild_id, src).await {
            if let Some(_) = result.1 {
                create_song_begin_event(send_http, Arc::new(ctx.clone()), result.0, channel_id)
                    .await;
            }
            try_say(msg.channel_id, ctx, "Song Added!").await;
        }
    }

    Ok(())
}

pub(super) async fn create_song_begin_event(
    send_http: Arc<Http>,
    ctx: Arc<Context>,
    track: TrackHandle,
    channel_id: ChannelId,
) {
    let _ = track.add_event(
        Event::Track(TrackEvent::Play),
        SongBeginNotifier {
            channel_id: channel_id,
            cache_http: send_http,
            contex: ctx,
        },
    );
}
