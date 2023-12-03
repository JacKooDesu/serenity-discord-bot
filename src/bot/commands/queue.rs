use serenity::{
    all::Message,
    client::Context,
    framework::standard::{macros::command, CommandResult},
};

#[command]
pub async fn queue(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}
