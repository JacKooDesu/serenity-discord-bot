use std::{sync::Arc, collections::VecDeque};

use serenity::{
    all::ChannelId,
    async_trait,
    client::Context,
    http::{CacheHttp, Http},
    model::channel::Message,
    prelude::TypeMapKey,
    Result as SerenityResult,
};
use songbird::{
    events::{Event, EventContext, EventHandler as VoiceEventHandler},
    input::AuxMetadata,
    Songbird,
};

use reqwest::Client as HttpClient;

pub struct HttpKey;
impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

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
    pub queue_id: usize,
}
#[async_trait]
impl VoiceEventHandler for SongBeginNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(_) = ctx {
            if let Some(vec) = self.contex.data.write().await.get_mut::<QueueKey>() {
                if let Some(title) = vec.get(0).and_then(|m| m.title.clone()) {
                    let s = format!("🎶  Current Playing  🎶```{}```", title);
                    say(self.channel_id, self.cache_http.http(), s.as_str()).await;
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
