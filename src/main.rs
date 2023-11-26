use std::env;

// use serenity::async_trait;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{CommandResult, StandardFramework};
use serenity::model::channel::Message;
use serenity::prelude::*;

#[group]
#[commands(ping)]
struct General;

struct Handler;

impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        .group(&GENERAL_GROUP)
        .configure(|c| c.prefix("!"));

    let token = match env::var("DISCORD_TOKEN") {
        Ok(s) => s,
        Err(_) => panic!("{:?}", "cannot get token"),
    };
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error on creating client");

    if let Err(err) = client.start().await {
        println!("[Error] {:?}", err)
    }
}

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;
    Ok(())
}
