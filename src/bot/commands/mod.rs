use serenity::framework::standard::macros::group;

pub mod join;
pub mod ping;
pub mod play;
pub mod playlist;
pub mod queue;
pub mod search;
pub mod set_volume;
pub mod skip;
pub mod stop;
pub mod artist;

use join::JOIN_COMMAND;
use ping::PING_COMMAND;
use play::PLAY_COMMAND;
use playlist::PLAYLIST_COMMAND;
use queue::QUEUE_COMMAND;
use search::SEARCH_COMMAND;
use set_volume::SET_VOLUME_COMMAND;
use skip::SKIP_COMMAND;
use stop::STOP_COMMAND;
use artist::ARTIST_COMMAND;

#[group]
#[commands(
    play, ping, join, skip, set_volume, stop, search, playlist, queue, artist
)]
pub struct General;
