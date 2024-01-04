use invidious::{hidden::SearchItem, CommonVideo};
use serenity::builder::{CreateEmbed, CreateEmbedAuthor};

use super::prettier::EmbedCreator;

pub struct PrettyVideo {
    pub item: Option<CommonVideo>,
}

impl PrettyVideo {
    pub fn new(item: Option<SearchItem>) -> Self {
        let mut x = PrettyVideo { item: None };
        if let Some(SearchItem::Video(video)) = item {
            x.item = Some(video);
        }
        x
    }

    pub fn video(video: Option<CommonVideo>) -> Self {
        PrettyVideo { item: video }
    }
}

impl EmbedCreator for PrettyVideo {
    fn to_embed(&self) -> Option<CreateEmbed> {
        if let Some(target) = &self.item {
            let mut embed = CreateEmbed::new();

            if let Some(thumbnail) = target.thumbnails.first() {
                embed = embed.thumbnail(thumbnail.url.as_str());
            } else {
                // todo: add github fallback image
                const FALLBACK: &str = "";
                embed = embed.thumbnail(FALLBACK);
            }
            embed = embed.title(target.title.as_str());
            embed = embed.url(format!("http://youtu.be/{}", &target.id));
            let mut author = CreateEmbedAuthor::new(target.author.clone());
            {
                author = author.url(format!("http://youtube.com{}", target.author_url));
            }
            embed = embed.author(author);

            return Some(embed);
        }
        None
    }
}
