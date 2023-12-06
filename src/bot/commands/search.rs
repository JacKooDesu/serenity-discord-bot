use core::fmt;
use std::{env, sync::Arc, time::Duration};

use google_youtube3::{hyper::client::HttpConnector, hyper_rustls::HttpsConnector, *};

use serenity::{
    all::{Message, ReactionType, User},
    builder::CreateMessage,
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
};
use songbird::{input::YoutubeDl, typemap::TypeMapKey};

use crate::bot::{
    commands::play::create_song_begin_event,
    common::{add_song, say, try_say, HttpKey},
    constants::{self, NUM_EMOJI},
};

#[command]
pub async fn search(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        say(msg.channel_id, ctx, "Args missing!").await;
        return Ok(());
    }
    let api_key = env::var(constants::YT_API_KEY).expect("Cannot get yt api key!");

    if let Some(pretties) = get_search_result(ctx, args.message(), api_key.as_str()).await {
        let http_client = {
            let data = ctx.data.read().await;
            let http_client = data
                .get::<HttpKey>()
                .cloned()
                .expect("Guaranteed to exist in the typemap.");

            http_client
        };

        let mut text = String::new();
        let mut reactions = Vec::<ReactionType>::new();

        for p in &pretties {
            text.push_str("```");
            text.push_str(p.to_string().as_str());
            text.push_str("```\n");
            reactions.push(p.get_emoji_reaction());
        }

        let guild_id = msg.guild_id.unwrap();
        let channel_id = msg.channel_id;
        let msg_builder = CreateMessage::new().content(text).reactions(reactions);

        if let Ok(waiting_msg) = msg.channel_id.send_message(ctx, msg_builder).await {
            if let Some(index) = wait_reaction(&msg.author, waiting_msg, ctx).await {
                let src = YoutubeDl::new(http_client.clone(), pretties[index].get_video_url());
                if let Some(result) = add_song(ctx, guild_id, src).await {
                    if let Some(_) = result.1 {
                        create_song_begin_event(
                            ctx.http.clone(),
                            Arc::new(ctx.clone()),
                            result.0,
                            channel_id,
                        )
                        .await;
                    }
                    try_say(msg.channel_id, ctx, "Song Added!").await;
                } else {
                    say(msg.channel_id, ctx, "Not in voice channel!").await;
                }
            } else {
                say(msg.channel_id, ctx, "Unknown reaction!").await
            }
        }
    }

    Ok(())
}

async fn wait_reaction(user: &User, msg: Message, ctx: &Context) -> Option<usize> {
    let collector = msg
        .await_reaction(&ctx.shard)
        .timeout(Duration::from_secs(10_u64))
        .author_id(user.id);

    let reaction = collector.await;
    if let Some(ReactionType::Unicode(emoji)) = reaction.map(|x| x.emoji) {
        if let Some(index) = NUM_EMOJI.iter().position(|&x| x == emoji.as_str()) {
            Some(index)
        } else {
            None
        }
    } else {
        None
    }
}

async fn get_search_result(ctx: &Context, args: &str, api_key: &str) -> Option<Vec<PrettyResult>> {
    if let Some(hub) = ctx.data.read().await.get::<YtHubKey>() {
        let result = hub
            .search()
            .list(&vec!["snippet".into()])
            .q(args)
            .add_type("video")
            .safe_search("none")
            .param("key", api_key)
            .doit()
            .await;

        match result {
            Err(e) => match e {
                Error::HttpError(_)
                | Error::Io(_)
                | Error::MissingAPIKey
                | Error::MissingToken(_)
                | Error::Cancelled
                | Error::UploadSizeLimitExceeded(_, _)
                | Error::Failure(_)
                | Error::BadRequest(_)
                | Error::FieldClash(_)
                | Error::JsonDecodeError(_, _) => {
                    println!("{}", e);
                    None
                }
            },
            Ok(ok) => {
                let mut pretties: Vec<PrettyResult> = Vec::new();
                let arr = ok.1.items.unwrap();
                for (index, item) in arr.into_iter().enumerate() {
                    pretties.push(PrettyResult(item, index));
                }

                Some(pretties)
            }
        }
    } else {
        None
    }
}

pub struct YtHubKey;
impl TypeMapKey for YtHubKey {
    type Value = YouTube<HttpsConnector<HttpConnector>>;
}

pub async fn init_yt_hub() -> YouTube<HttpsConnector<HttpConnector>> {
    YouTube::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .build(),
        ),
        "".to_string(),
    )
}

struct PrettyResult(api::SearchResult, usize);
impl PrettyResult {
    fn get_emoji_reaction(&self) -> ReactionType {
        ReactionType::Unicode(NUM_EMOJI[self.1].to_string())
    }

    fn get_video_url(&self) -> String {
        if let Some(Some(id)) = self.0.id.clone().map(|x| x.video_id) {
            format!("https://youtu.be/{}", id)
        } else {
            String::new()
        }
    }
}
impl fmt::Display for PrettyResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title = &self.0.snippet.clone().unwrap().title.unwrap();
        let str = format!("{0}. {1}", &self.1 + 1, title);
        f.write_str(str.as_str())?;
        Ok(())
    }
}
