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