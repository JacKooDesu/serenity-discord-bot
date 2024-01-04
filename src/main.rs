use std::{collections::VecDeque, env};

use serenity::{
    async_trait,
    client::{Context, EventHandler},
    framework::{standard::Configuration, StandardFramework},
    model::gateway::Ready,
    prelude::GatewayIntents,
};
use songbird::SerenityInit;

use reqwest::Client as HttpClient;

mod bot;
use bot::{
    commands::GENERAL_GROUP,
    common::{create_config, CommonConfigKey, QueueKey, VolumeKey},
    constants::*,
};

use crate::bot::clients::{init_yt_client, YtClientKey, YtHubKey, init_yt_hub, HttpKey};

struct Handler;
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    env_init();

    tracing_subscriber::fmt::init();

    let framework = StandardFramework::new().group(&GENERAL_GROUP);
    if let Ok(prefix) = env::var(COMMAND_PREFIX_KEY) {
        framework.configure(Configuration::new().prefix(prefix.as_str()))
    }

    let token = match env::var(DISCORD_TOKEN_KEY) {
        Ok(s) => s,
        Err(_) => panic!("{:?}", "cannot get token"),
    };
    let config = create_config(None);
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS;
    let mut client = serenity::Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
        .type_map_insert::<VolumeKey>(1_f32)
        .type_map_insert::<CommonConfigKey>(config)
        .type_map_insert::<YtHubKey>(init_yt_hub().await)
        .type_map_insert::<YtClientKey>(init_yt_client().await)
        .type_map_insert::<QueueKey>(VecDeque::new())
        .await
        .expect("Error on creating client");

    tokio::spawn(async move {
        let _ = client.start().await.map_err(|err| println!("{:?}", err));
    });
    let _signal_err = tokio::signal::ctrl_c().await;
    println!("client stopped!");
}

fn env_init() {
    #[cfg(not(debug_assertions))]
    if let Err(err) = dotenv::dotenv() {
        panic!("dotenv initialized failed!!\n{}", err)
    }

    for key in REQUIRED_KEYS {
        if let Err(_) = env::var(key) {
            panic!("required key missing `{}`!!", key);
        }
    }

    for key in OPTIONAL_KEYS {
        if let Err(_) = env::var(key) {
            println!("optional key `{}` not found!!", key)
        }
    }
}
