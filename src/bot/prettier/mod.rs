mod pretty_playlist;
mod pretty_aux_metadata;
mod pretty_channel;
mod pretty_video;

pub use pretty_playlist::*;
pub use pretty_aux_metadata::*;
pub use pretty_channel::*;
pub use pretty_video::*;

pub mod prettier {
    use serenity::builder::CreateEmbed;
    pub trait EmbedCreator {
        fn to_embed(&self) -> Option<CreateEmbed>;
    }
}
