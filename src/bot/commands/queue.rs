use serenity::{
    all::Message,
    client::Context,
    framework::standard::{macros::command, CommandResult},
};

use crate::bot::common::{try_say, QueueKey};

const QUERY_COUNT: usize = 10;
#[command]
pub async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    if let Some(vec) = ctx.data.read().await.get::<QueueKey>() {
        let len = usize::clamp(QUERY_COUNT, 0, vec.len());

        if len == 0 {
            try_say(msg.channel_id, ctx, "Nothing in the queue!").await;
            return Ok(());
        }

        let mut text = String::new();
        text.push_str(format!("First {} songs in queue! ( {} in total )", len, vec.len()).as_str());
        let arr = vec.range(..len);
        {
            for (index, element) in arr.enumerate() {
                text.push_str("```");
                if let Some(title) = &element.title {
                    let info = format!("{}. {}", index + 1, title);
                    text.push_str(info.as_str());
                }
                text.push_str("```");
            }
        }

        try_say(msg.channel_id, ctx, text.as_str()).await
    }
    Ok(())
}
