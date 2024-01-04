use std::{env, sync::Arc, time::Duration};

use serenity::{
    all::{Message, ReactionType, User},
    builder::CreateMessage,
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
};
use songbird::input::YoutubeDl;

use crate::bot::{
    commands::play::create_song_begin_event,
    common::{add_song, say, try_say},
    constants::{self, NUM_EMOJI}, clients::{HttpKey, get_search_result},
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
