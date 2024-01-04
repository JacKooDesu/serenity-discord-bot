use std::{collections::VecDeque, sync::Arc, usize};

use serenity::{
    all::{ChannelId, GuildId},
    async_trait,
    builder::{CreateEmbed, CreateEmbedAuthor, CreateMessage},
    client::Context,
    http::{CacheHttp, Http},
    model::channel::Message,
    prelude::TypeMapKey,
    Result as SerenityResult,
};
use songbird::{
    events::{Event, EventContext, EventHandler as VoiceEventHandler},
    input::{AuxMetadata, Compose, Input},
    tracks::TrackHandle,
    Songbird,
};

pub struct CommonConfigKey;
impl TypeMapKey for CommonConfigKey {
    type Value = CommonConfig;
}
pub struct CommonConfig {
    pub dont_spam: bool,
}
pub fn create_config(dont_spam: Option<bool>) -> CommonConfig {
    CommonConfig {
        dont_spam: dont_spam.unwrap_or(false),
    }
}

pub struct VolumeKey;
impl TypeMapKey for VolumeKey {
    type Value = f32;
}

pub struct QueueKey;
impl TypeMapKey for QueueKey {
    type Value = VecDeque<AuxMetadata>;
}

pub struct SongBeginNotifier {
    pub channel_id: ChannelId,
    pub cache_http: Arc<Http>,
    pub contex: Arc<Context>,
    // pub queue_id: usize,
}
#[async_trait]
impl VoiceEventHandler for SongBeginNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(_) = ctx {
            if let Some(vec) = self.contex.data.write().await.get_mut::<QueueKey>() {
                if let Some(metadata) = vec.get(0) {
                    let mut embed = CreateEmbed::new();
                    if let Some(title) = &metadata.title {
                        embed = embed.title(title);
                        if let Some(url) = &metadata.source_url {
                            embed = embed.url(url);
                        }
                    }
                    if let Some(thumbnail) = &metadata.thumbnail {
                        embed = embed.thumbnail(thumbnail);
                    }
                    if let Some(channel) = &metadata.channel {
                        let author = CreateEmbedAuthor::new(channel);
                        embed = embed.author(author);
                    }

                    let mut builder = CreateMessage::new();
                    builder = builder.content("🎶  Current Playing  🎶").embed(embed);
                    let msg = self
                        .channel_id
                        .send_message(self.cache_http.http(), builder)
                        .await;

                    if let Err(_) = msg {
                        if let Some(title) = &metadata.title {
                            let s = format!("🎶  Current Playing  🎶```{}```", title);
                            say(self.channel_id, self.cache_http.http(), s.as_str()).await;
                        }
                    }
                }
                vec.pop_front();
            }
        }
        None
    }
}

pub struct SongEndedNotifier {
    pub channel_id: ChannelId,
    pub http: Arc<Http>,
    pub contex: Arc<Context>,
}
#[async_trait]
impl VoiceEventHandler for SongEndedNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let Some(config) = &self.contex.data.read().await.get::<CommonConfigKey>() {
            if config.dont_spam {
                return None;
            }
        }
        if let EventContext::Track(list) = ctx {
            if 0 == list.len() {
                return None;
            }
            check_msg(
                self.channel_id
                    .say(&self.http, &format!("Next track!"))
                    .await,
            );
        }
        None
    }
}

pub struct TrackErrorNotifier;

#[async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                println!(
                    "Track {:?} encountered an error: {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }

        None
    }
}
pub struct TrackEndNotifier {
    pub guild_id: GuildId,
    pub songbird: Arc<Songbird>,
    pub context: Arc<Context>,
}

#[async_trait]
impl VoiceEventHandler for TrackEndNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(_) = ctx {
            if let usize::MIN = self
                .context
                .data
                .read()
                .await
                .get::<QueueKey>()
                .map_or(usize::MIN, |vec| vec.len())
            {
                let _ = self.songbird.remove(self.guild_id).await;
            }
        }
        None
    }
}

pub async fn add_song(
    ctx: &Context,
    guild_id: GuildId,
    mut input: impl Into<Input> + Compose,
) -> Option<(TrackHandle, Option<AuxMetadata>)> {
    let manager = get_manager(ctx).await;
    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        let metadata = input.aux_metadata().await;
        let track = handler.enqueue_input(input.into()).await;

        if let Some(volume) = ctx.data.read().await.get::<VolumeKey>() {
            let _ = track.set_volume(volume.clone());
        }

        if let Ok(meta) = metadata {
            if let Some(vec) = ctx.data.write().await.get_mut::<QueueKey>() {
                vec.push_back(meta.clone());
                return Some((track, Some(meta)));
            }
        }
        return Some((track, None));
    }
    None
}

pub async fn say(channel: ChannelId, ctx: impl CacheHttp, text: &str) {
    check_msg(channel.say(ctx, text).await)
}

pub async fn try_say(channel: ChannelId, ctx: &Context, text: &str) {
    if let Some(config) = ctx.data.read().await.get::<CommonConfigKey>() {
        if !config.dont_spam {
            check_msg(channel.say(&ctx.http, text).await)
        }
    }
}

pub(super) fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

pub async fn get_manager(ctx: &Context) -> Arc<Songbird> {
    songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone()
}
