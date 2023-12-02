use std::time;

use serenity::{
    all::Message,
    async_trait,
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
};
use songbird::events::{Event, EventContext, EventHandler as VoiceEventHandler};

use crate::bot::common::{say, try_say, VolumeKey};

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
