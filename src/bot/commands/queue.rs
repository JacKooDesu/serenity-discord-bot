use async_recursion::async_recursion;
use serenity::{
    all::{Message, User},
    builder::{CreateMessage, EditMessage},
    client::Context,
    framework::standard::{macros::command, CommandResult},
};

use crate::bot::{
    common::{try_say, QueueKey},
    constants::{BACK_EMOJI, NEXT_EMOJI},
    utils::reaction_collector::{ActionEnumTrait, ReactionCollector}, prettier::{PrettyAuxMetadata, prettier::EmbedCreator},
};

const QUERY_COUNT: usize = 10;
#[command]
pub async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    if let Ok(message) = msg
        .channel_id
        .send_message(ctx, CreateMessage::new().content("Loading queue..."))
        .await
    {
        explore_queue(ctx, message, msg.author.clone(), 0, None).await;
    }
    Ok(())
}

#[async_recursion]
async fn explore_queue(
    ctx: &Context,
    msg: Message,
    user: User,
    offset: usize,
    buffer: Option<Vec<PrettyAuxMetadata>>,
) -> Option<()> {
    let mut action_map = Vec::new();
    let queue: Vec<PrettyAuxMetadata>;
    if let Some(vec) = buffer {
        queue = vec;
    } else {
        if let Some(vec) = ctx.data.read().await.get::<QueueKey>() {
            queue = vec
                .iter()
                .map(|x| PrettyAuxMetadata {
                    item: Some(x.clone()),
                })
                .collect();
        } else {
            queue = Vec::new();
        }
    }

    let len = usize::min(queue.len() - offset, QUERY_COUNT);
    let mut msg_builder = EditMessage::new()
        .content(
            format!(
                "{} - {} ( {} in total )",
                offset + 1,
                offset + len,
                queue.len()
            )
            .as_str(),
        )
        .embeds(Vec::new());

    if len == 0 {
        try_say(msg.channel_id, ctx, "Nothing in the queue!").await;
        return None;
    }

    for i in 0..len {
        if let Some(embed) = queue[i].to_embed() {
            // embed = embed.author(CreateEmbedAuthor::new((i + offset + 1).to_string()));
            msg_builder = msg_builder.add_embed(embed);
        }
    }

    if (offset + QUERY_COUNT) < queue.len() {
        action_map.push((NEXT_EMOJI, NextAction::ExploreQueue(offset + QUERY_COUNT)))
    }

    if offset >= QUERY_COUNT {
        action_map.insert(
            0,
            (BACK_EMOJI, NextAction::ExploreQueue(offset - QUERY_COUNT)),
        )
    }

    let _ = msg.delete_reactions(ctx).await;

    if let Ok(_) = msg.clone().edit(ctx, msg_builder).await {
        let collector = ReactionCollector::create(action_map);
        match collector.wait_reaction(&user, msg.clone(), ctx).await {
            Some(NextAction::ExploreQueue(offset)) => {
                explore_queue(ctx, msg, user, offset, Some(queue)).await;
            }
            _ => (),
        }
    }

    None
}

#[derive(Clone)]
enum NextAction {
    ExploreQueue(usize),
    Error,
}

impl ActionEnumTrait for NextAction {
    fn fallback_action() -> Self {
        NextAction::Error
    }
}
