use serenity::framework::standard::macros::group;

pub mod artist;
pub mod album;
pub mod join;
pub mod ping;
pub mod play;
pub mod playlist;
pub mod queue;
pub mod search;
pub mod set_volume;
pub mod skip;
pub mod stop;

use artist::ARTIST_COMMAND;
use album::ALBUM_COMMAND;
use join::JOIN_COMMAND;
use ping::PING_COMMAND;
use play::PLAY_COMMAND;
use playlist::PLAYLIST_COMMAND;
use queue::QUEUE_COMMAND;
use search::SEARCH_COMMAND;
use set_volume::SET_VOLUME_COMMAND;
use skip::SKIP_COMMAND;
use stop::STOP_COMMAND;

#[group]
#[commands(
    play,
    ping,
    join,
    skip,
    set_volume,
    stop,
    search,
    playlist,
    queue,
    artist,
    album
)]
pub struct General;
