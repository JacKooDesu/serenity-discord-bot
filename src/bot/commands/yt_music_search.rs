use google_youtube3::api::Channel;
use invidious::{hidden::SearchItem, ClientAsync as YtClient, ClientAsyncTrait, CommonChannel};
use serenity::{
    all::Message,
    builder::{CreateEmbed, CreateMessage},
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
};
use songbird::typemap::TypeMapKey;
use tracing_subscriber::fmt::format::{self, Pretty};

use crate::bot::common::say;

const ARTIST_RESULT_LEN: usize = 3;
#[command]
#[only_in(guilds)]
pub async fn artist(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        say(msg.channel_id, ctx, "Args missing!").await;
        return Ok(());
    }

    let mut msg_builder = CreateMessage::new().content("Artist Founded!");
    if let Some(yt_client) = ctx.data.read().await.get::<YtClientKey>() {
        let param = format!("type=channel&q={}", args.message());
        if let Ok(result) = yt_client.search(Some(param.as_str())).await {
            let len = usize::min(result.items.len(), ARTIST_RESULT_LEN);

            for i in 0..len {
                let embed = PrettyChannel::new(result.items.get(i).cloned());
                if let Some(item) = embed.to_embed() {
                    msg_builder = msg_builder.add_embed(item);
                }
            }
            // msg_builder = msg_builder.content(format!("Founded {} artists", len));
        } else {
            msg_builder = msg_builder.content("Cannot call search api!");
        }
        let _ = msg.channel_id.send_message(ctx, msg_builder).await;
    }
    Ok(())
}

pub async fn init_yt_client() -> YtClient {
    YtClient::default()
}

pub struct YtClientKey {}
impl TypeMapKey for YtClientKey {
    type Value = YtClient;
}

#[derive(Default)]
pub struct PrettyChannel {
    item: Option<CommonChannel>,
}
pub trait EmbedCreator {
    fn to_embed(&self) -> Option<CreateEmbed>;
}

impl PrettyChannel {
    pub fn new(item: Option<SearchItem>) -> Self {
        let mut x = Self::default();
        if let Some(SearchItem::Channel(channel)) = item {
            x.item = Some(channel);
        }
        x
    }
}
impl EmbedCreator for PrettyChannel {
    fn to_embed(&self) -> Option<CreateEmbed> {
        if let Some(target) = &self.item {
            let mut embed = CreateEmbed::new();
            if let Some(thumbnail) = &target.thumbnails.last() {
                embed = embed.thumbnail(format!("https:{}", thumbnail.url));
            } else {
                // todo: add github fallback image
                const FALLBACK: &str = "";
                embed = embed.thumbnail(FALLBACK);
            }
            embed = embed.title(&target.name);
            embed = embed.url(format!("http://youtube.com{}", &target.url));
            embed = embed.description(&target.description);

            return Some(embed);
        }
        None
    }
}
