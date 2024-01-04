use invidious::CommonPlaylist;
use serenity::builder::{CreateEmbed, CreateEmbedAuthor};

use super::prettier::EmbedCreator;

pub struct PrettyPlaylist {
    pub item: Option<CommonPlaylist>,
}

impl PrettyPlaylist {
    pub fn new(item: Option<CommonPlaylist>) -> Self {
        PrettyPlaylist { item }
    }
}

impl EmbedCreator for PrettyPlaylist {
    fn to_embed(&self) -> Option<CreateEmbed> {
        if let Some(target) = &self.item {
            let mut embed = CreateEmbed::new();
            embed = embed.title(target.title.as_str());
            embed = embed.thumbnail(target.thumbnail.as_str());
            embed = embed.url(format!("https://youtube.com/playlist?list={}", &target.id));

            let mut author = CreateEmbedAuthor::new(target.author.clone());
            {
                author = author.url(format!("http://youtube.com/channel/{}", target.author_id));
            }
            embed = embed.author(author);

            return Some(embed);
        }

        None
    }
}
