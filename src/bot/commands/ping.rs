use serenity::{
    all::Message,
    client::Context,
    framework::standard::{macros::command, CommandResult},
};

#[command]
pub async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "Pong!").await?;
    Ok(())
}
