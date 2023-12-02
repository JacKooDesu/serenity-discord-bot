use core::time;
use std::{borrow::BorrowMut, default, env, string, sync::Arc};

// use serenity::async_trait;
use serenity::{
    all::ChannelId,
    async_trait,
    client::{Client, Context, EventHandler},
    framework::{
        standard::{
            macros::{command, group},
            Args, CommandResult,
        },
        StandardFramework,
    },
    http::Http,
    model::{channel::Message, gateway::Ready},
    prelude::{GatewayIntents, TypeMapKey},
    Result as SerenityResult,
};

use songbird::input::YoutubeDl;
use songbird::SerenityInit;
use songbird::{
    events::{Event, EventContext, EventHandler as VoiceEventHandler, TrackEvent},
    Config,
};

use reqwest::Client as HttpClient;
use tracing_subscriber::fmt::format;

#[group]
#[commands(ping, play, join, set_volume, stop, skip, dont_spam)]
struct General;

struct HttpKey;
impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

struct VolumeKey;
impl TypeMapKey for VolumeKey {
    type Value = f32;
}

struct CommonConfigKey;
impl TypeMapKey for CommonConfigKey {
    type Value = CommonConfig;
}
struct CommonConfig {
    dont_spam: bool,
}
fn create_config(dont_spam: Option<bool>) -> CommonConfig {
    CommonConfig {
        dont_spam: dont_spam.unwrap_or(false),
    }
}

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let framework = StandardFramework::new().group(&GENERAL_GROUP);
    // .configure(|c:Configuration| c.prefix("!"));
    let args: Vec<_> = env::args().collect();
    let token = match env::var("DISCORD_TOKEN") {
        Ok(s) => s,
        Err(_) => {
            if let 0 = args.len() {
                panic!("{:?}", "cannot get token")
            } else {
                println!("token: {:?}", args[1].to_string());
                args[1].to_string()
            }
        }
    };
    let config = create_config(None);
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = serenity::Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
        .type_map_insert::<VolumeKey>(1_f32)
        .type_map_insert::<CommonConfigKey>(config)
        .await
        .expect("Error on creating client");

    tokio::spawn(async move {
        let _ = client.start().await.map_err(|err| println!("{:?}", err));
    });
    let _signal_err = tokio::signal::ctrl_c().await;
    println!("client stopped!");
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let (guild_id, channel_id) = {
        let guild = msg.guild(&ctx.cache).unwrap();
        let channel = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|state| state.channel_id);

        (guild.id, channel)
    };

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            try_say(msg.channel_id, ctx, "You're not in voice channel!!").await;
            return Ok(());
        }
    };

    let manager = songbird::get(&ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Ok(handler_lock) = manager.join(guild_id, connect_to).await {
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier)
    }

    Ok(())
}

struct TrackErrorNotifier;

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

#[command]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
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
    println!("song playing: {:?}", &url);
    let (http_client, volume) = {
        let data = ctx.data.read().await;
        let http_client = data
            .get::<HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.");

        let volume = data.get::<VolumeKey>().cloned().unwrap_or(1_f32);

        (http_client, volume)
    };

    let manager = songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let channel_id = msg.channel_id;
        let send_http = ctx.http.clone();

        let src = YoutubeDl::new(http_client, url);
        let track = handler.enqueue_input(src.into()).await;
        let _ = track.set_volume(volume);

        let _ = track.add_event(
            Event::Track(TrackEvent::End),
            SongEndedNotifier {
                channel_id: channel_id,
                http: send_http,
                contex: Arc::new(ctx.clone()),
            },
        );
        try_say(msg.channel_id, ctx, "Playing!").await;
    } else {
        say(msg.channel_id, ctx, "Not in voice channel!").await;
    }

    Ok(())
}

struct SongEndedNotifier {
    channel_id: ChannelId,
    http: Arc<Http>,
    contex: Arc<Context>,
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

#[command]
#[only_in(guilds)]
async fn stop(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird not initialized!")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        queue.stop();

        try_say(msg.channel_id, ctx, "Queue cleared!").await
    } else {
        say(msg.channel_id, ctx, "Not in voice channel!").await
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn skip(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird not initailzied!")
        .clone();

    if let Some(handler_lock) = manager.get(guild_id) {
        let handler = handler_lock.lock().await;
        let queue = handler.queue();
        let _ = queue.skip();

        try_say(msg.channel_id, ctx, "Skipped!").await
    } else {
        say(msg.channel_id, ctx, "Not in voice channel!").await
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
async fn set_volume(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let manager = songbird::get(ctx)
        .await
        .expect("Songbird not initiazlied!")
        .clone();
    let volume = match args.single::<String>() {
        Ok(s) => match s.parse::<f32>() {
            Ok(f) => f.clamp(0_f32, 1_f32),
            Err(_) => {
                say(msg.channel_id, ctx, "Volume value invalid!").await;
                return Ok(());
            }
        },
        Err(_) => {
            say(msg.channel_id, ctx, "Volume value invalid!").await;
            return Ok(());
        }
    };

    if let Some(volume_key) = ctx.data.write().await.get_mut::<VolumeKey>() {
        let log = format!("Set volume to {:?}", volume);
        println!("{:?}", log);
        try_say(msg.channel_id, ctx, log.as_str()).await;
        *volume_key = volume
    };

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let _ = handler.add_global_event(
            Event::Delayed(time::Duration::new(0, 0)),
            VolumeSetter { value: volume },
        );
    }
    Ok(())
}
struct VolumeSetter {
    value: f32,
}
#[async_trait]
impl VoiceEventHandler for VolumeSetter {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(&[(_, track)]) = ctx {
            let _ = track.set_volume(self.value);
        }
        Some(Event::Cancel)
    }
}

#[command]
#[only_in(guilds)]
async fn dont_spam(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let result = match args.single::<String>() {
        Ok(s) => match s.as_str() {
            "on" => Ok(true),
            "off" => Ok(false),
            _ => Err(()),
        },
        Err(_) => Err(()),
    };

    match result {
        Ok(b) => {
            say(
                msg.channel_id,
                ctx,
                format!("Set spam setting to {:?}!", b).as_str(),
            )
            .await;
            if let Some(config) = ctx.data.write().await.get_mut::<CommonConfigKey>() {
                (*config).dont_spam = b
            };
        }
        Err(_) => {
            say(msg.channel_id, ctx, "Value invalid!").await;
            return Ok(());
        }
    };
    Ok(())
}

async fn say(channel: ChannelId, ctx: &Context, text: &str) {
    check_msg(channel.say(&ctx.http, text).await)
}

async fn try_say(channel: ChannelId, ctx: &Context, text: &str) {
    if let Some(config) = ctx.data.read().await.get::<CommonConfigKey>() {
        if !config.dont_spam {
            check_msg(channel.say(&ctx.http, text).await)
        }
    }
}

fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}
