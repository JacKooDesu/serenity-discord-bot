use serenity::framework::standard::macros::group;

pub mod ping;
pub mod play;
pub mod stop;
pub mod skip;
pub mod set_volume;
pub mod join;

use join::JOIN_COMMAND;
use play::PLAY_COMMAND;
use ping::PING_COMMAND;
use stop::STOP_COMMAND;
use set_volume::SET_VOLUME_COMMAND;
use skip::SKIP_COMMAND;

#[group]
#[commands(play,ping,join,skip,set_volume,stop)]
pub struct General;
