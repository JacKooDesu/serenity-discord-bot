use std::{collections::HashMap, env, sync::Arc, time::Duration};

use async_recursion::async_recursion;
use invidious::{
    hidden::SearchItem, ClientAsync as YtClient, ClientAsyncTrait, CommonChannel, CommonVideo,
};
use serenity::{
    all::{Message, ReactionType, User},
    builder::{CreateEmbed, CreateEmbedAuthor, CreateMessage, EditMessage},
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
};
use songbird::{input::YoutubeDl, typemap::TypeMapKey};

use crate::bot::{
    common::{add_song, say, try_say, HttpKey},
    constants::{BACK_EMOJI, INVIDIOUS_INSTANCE_KEY, NEXT_EMOJI, NUM_EMOJI, REGION_KEY},
};

use super::play::create_song_begin_event;

const ARTIST_RESULT_LEN: usize = 3;
#[command]
#[only_in(guilds)]
pub async fn artist(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        say(msg.channel_id, ctx, "Args missing!").await;
        return Ok(());
    }

    let mut msg_builder = CreateMessage::new().content("Artist Founded!");
    let mut action_map = HashMap::new();
    if let Some(yt_client) = ctx.data.read().await.get::<YtClientKey>() {
        let mut param = format!("type=channel&q={}", args.message());
        if let Ok(region) = env::var(REGION_KEY) {
            let _ = &param.push_str(format!("&region={}", region).as_str());
        }
        if let Ok(result) = yt_client.search(Some(param.as_str())).await {
            let len = usize::min(result.items.len(), ARTIST_RESULT_LEN);

            for i in 0..len {
                let origin = result.items.get(i);
                let embed = PrettyChannel::new(origin.cloned());
                if let Some(item) = embed.to_embed() {
                    msg_builder = msg_builder.add_embed(item);
                    if let Some(channel) = embed.item {
                        action_map.insert(NUM_EMOJI[i], NextAction::ExploreVideos(channel.id, 0));
                    }
                }
            }
            // msg_builder = msg_builder.content(format!("Founded {} artists", len));
        } else {
            msg_builder = msg_builder.content("Cannot call search api!");
        }
    }

    if let Ok(reply) = msg.channel_id.send_message(ctx, msg_builder).await {
        let collector = ReactionCollector::create(action_map);
        match collector
            .wait_reaction(&msg.author, reply.clone(), ctx)
            .await
        {
            Some(NextAction::ExploreVideos(id, page)) => {
                explore_videos(ctx, reply.clone(), msg.author.clone(), &id, page, None).await
            }
            Some(_) | None => None,
        };
    }
    Ok(())
}

#[async_recursion]
async fn explore_videos(
    ctx: &Context,
    msg: Message,
    user: User,
    id: &str,
    offset: usize,
    buffer: Option<Vec<CommonVideo>>,
) -> Option<()> {
    const PAGE_SIZE: usize = 5;
    let mut msg_builder = EditMessage::new().content("").embeds(Vec::new());
    let mut action_map = HashMap::new();
    let videos: Vec<CommonVideo> = if let Some(_buffer) = buffer {
        _buffer
    } else if let Some(yt_client) = ctx.data.read().await.get::<YtClientKey>() {
        if let Ok(videos) = yt_client.channel_videos(id, None).await {
            videos.videos
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    if (offset + PAGE_SIZE) < videos.len() {
        action_map.insert(
            NEXT_EMOJI,
            NextAction::ExploreVideos(id.to_string(), offset + PAGE_SIZE),
        );
    }

    if offset >= PAGE_SIZE {
        action_map.insert(
            BACK_EMOJI,
            NextAction::ExploreVideos(id.to_string(), offset - PAGE_SIZE),
        );
    }

    let len = usize::min(videos.len() - offset, PAGE_SIZE);

    for i in 0..len {
        let origin = videos.get(i + offset);
        let embed = PrettyVideo::video(origin.cloned());
        if let Some(mut item) = embed.to_embed() {
            item = item.author(CreateEmbedAuthor::new(NUM_EMOJI[i]));
            msg_builder = msg_builder.add_embed(item);
            if let Some(x) = embed.item {
                action_map.insert(NUM_EMOJI[i], NextAction::Finished(x.id));
            }
        }
    }
    let _ = msg.delete_reactions(ctx).await;

    if let Ok(_) = msg.clone().edit(ctx, msg_builder).await {
        let collector = ReactionCollector::create(action_map);
        match collector.wait_reaction(&user, msg.clone(), ctx).await {
            Some(NextAction::Finished(id)) => selected_video(ctx, msg, id.as_str()).await,
            Some(NextAction::ExploreVideos(id, page)) => {
                explore_videos(ctx, msg, user, id.as_str(), page, Some(videos)).await;
            }
            None => (),
        }
    };
    Some(())
}

async fn selected_video(ctx: &Context, msg: Message, id: &str) {
    let http_client = {
        let data = ctx.data.read().await;
        let http_client = data
            .get::<HttpKey>()
            .cloned()
            .expect("Cannot get http clinet!");
        http_client
    };
    let src = YoutubeDl::new(http_client, format!("https://youtu.be/{}", id));
    if let Some(result) = add_song(ctx, msg.clone().guild_id.unwrap(), src).await {
        if let Some(_) = result.1 {
            create_song_begin_event(
                ctx.http.clone(),
                Arc::new(ctx.clone()),
                result.0,
                msg.channel_id,
            )
            .await;
        }
        try_say(msg.channel_id, ctx, "Song Added!").await;
    } else {
        say(msg.channel_id, ctx, "Not in voice channel!").await;
    }
}

pub struct ReactionCollector<'a> {
    pub(self) action_map: HashMap<&'a str, NextAction>,
}

impl<'a> ReactionCollector<'a> {
    pub(self) fn create(map: HashMap<&'a str, NextAction>) -> Self {
        let x = ReactionCollector { action_map: map };
        x
    }

    pub(self) fn add_action(mut self, s: &'a str, action: NextAction) -> Self {
        self.action_map.insert(s, action);
        self
    }

    async fn wait_reaction(self, user: &User, msg: Message, ctx: &Context) -> Option<NextAction> {
        for (reaction, _) in &self.action_map {
            let _ = msg
                .react(ctx, ReactionType::Unicode(reaction.to_string()))
                .await;
        }

        let collector = msg
            .await_reaction(&ctx.shard)
            .timeout(Duration::from_secs(10_u64))
            .author_id(user.id);

        let reaction = collector.await;
        if let Some(ReactionType::Unicode(emoji)) = reaction.map(|x| x.emoji) {
            if let Some(action) = self.action_map.get(emoji.as_str()).cloned() {
                return Some(action);
            }
        }
        None
    }
}

pub async fn init_yt_client() -> YtClient {
    if let Ok(instance) = env::var(INVIDIOUS_INSTANCE_KEY) {
        YtClient::new(instance, invidious::MethodAsync::default())
    } else {
        YtClient::default()
    }
}

#[derive(Clone)]
enum NextAction {
    ExploreVideos(String, usize),
    Finished(String), // Error(String),
}

pub struct YtClientKey {}
impl TypeMapKey for YtClientKey {
    type Value = YtClient;
}

pub struct PrettyChannel {
    item: Option<CommonChannel>,
}

pub struct PrettyVideo {
    item: Option<CommonVideo>,
}

pub trait EmbedCreator {
    fn to_embed(&self) -> Option<CreateEmbed>;
}

impl PrettyChannel {
    pub fn new(item: Option<SearchItem>) -> Self {
        let mut x = PrettyChannel { item: None };
        if let Some(SearchItem::Channel(channel)) = item {
            x.item = Some(channel);
        }
        x
    }
}

impl PrettyVideo {
    pub fn new(item: Option<SearchItem>) -> Self {
        let mut x = PrettyVideo { item: None };
        if let Some(SearchItem::Video(video)) = item {
            x.item = Some(video);
        }
        x
    }

    pub fn video(video: Option<CommonVideo>) -> Self {
        PrettyVideo { item: video }
    }
}

impl EmbedCreator for PrettyChannel {
    fn to_embed(&self) -> Option<CreateEmbed> {
        if let Some(target) = &self.item {
            let mut embed = CreateEmbed::new();
            if let Some(thumbnail) = &target.thumbnails.last() {
                let mut url = thumbnail.url.clone();
                if !url.starts_with("https:") {
                    url.insert_str(0, "https:");
                }
                embed = embed.thumbnail(url);
            } else {
                // todo: add github fallback image
                const FALLBACK: &str = "";
                embed = embed.thumbnail(FALLBACK);
            }
            embed = embed.title(&target.name);
            embed = embed.url(format!("http://youtube.com{}", &target.url));
            embed = embed.description(&target.description_html);

            return Some(embed);
        }
        None
    }
}

impl EmbedCreator for PrettyVideo {
    fn to_embed(&self) -> Option<CreateEmbed> {
        if let Some(target) = &self.item {
            let mut embed = CreateEmbed::new();

            if let Some(thumbnail) = target.thumbnails.first() {
                embed = embed.thumbnail(thumbnail.url.as_str());
            } else {
                // todo: add github fallback image
                const FALLBACK: &str = "";
                embed = embed.thumbnail(FALLBACK);
            }
            embed = embed.title(target.title.as_str());
            embed = embed.url(format!("http://youtube.com/{}", &target.id));
            let mut author = CreateEmbedAuthor::new(target.author.clone());
            {
                author = author.url(format!("http://youtube.com{}", target.author_url));
            }
            embed = embed.author(author);

            return Some(embed);
        }
        None
    }
}
