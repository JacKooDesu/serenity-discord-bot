use std::env;

use serenity::{
    async_trait,
    client::{Context, EventHandler},
    framework::StandardFramework,
    model::gateway::Ready,
    prelude::GatewayIntents,
};
use songbird::SerenityInit;

use reqwest::Client as HttpClient;

mod bot;
use bot::{
    commands::search::*,
    commands::GENERAL_GROUP,
    common::{create_config, CommonConfigKey, HttpKey, VolumeKey},
    constants::*,
};

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
    let args: Vec<_> = env::args().collect();
    if let 1 = args.len() {
        ();
    } else {
        println!("set token: {:?}", args[1].to_string());
        env::set_var(DISCORD_TOKEN_KEY, args[1].to_string());
        if let Some(yt_key) = args.get(2) {
            env::set_var(YT_API_KEY, yt_key.to_string());
            println!("set youtube api key: {:?}", yt_key);
        } else {
            println!("Not set youtube api key!");
        }
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
        .type_map_insert::<YtHubKey>(
            init_yt_hub("AIzaSyBRKrk1103KrPygaOu7kb8Yd-yBeowYgN4".to_string()).await,
        )
        .await
        .expect("Error on creating client");

    tokio::spawn(async move {
        let _ = client.start().await.map_err(|err| println!("{:?}", err));
    });
    let _signal_err = tokio::signal::ctrl_c().await;
    println!("client stopped!");
}
