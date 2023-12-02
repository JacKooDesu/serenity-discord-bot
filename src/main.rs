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
    commands::GENERAL_GROUP,
    common::{create_config, CommonConfigKey, HttpKey, VolumeKey},
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
