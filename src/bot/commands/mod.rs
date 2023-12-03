use serenity::framework::standard::macros::group;

pub mod join;
pub mod ping;
pub mod play;
pub mod playlist;
pub mod search;
pub mod set_volume;
pub mod skip;
pub mod stop;

use join::JOIN_COMMAND;
use ping::PING_COMMAND;
use play::PLAY_COMMAND;
use playlist::PLAYLIST_COMMAND;
use search::SEARCH_COMMAND;
use set_volume::SET_VOLUME_COMMAND;
use skip::SKIP_COMMAND;
use stop::STOP_COMMAND;

#[group]
#[commands(play, ping, join, skip, set_volume, stop, search, playlist)]
pub struct General;
