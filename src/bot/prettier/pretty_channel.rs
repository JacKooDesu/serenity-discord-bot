use super::prettier::EmbedCreator;
use invidious::{hidden::SearchItem, CommonChannel};
use serenity::builder::CreateEmbed;

pub struct PrettyChannel {
    pub item: Option<CommonChannel>,
}

impl PrettyChannel {
    pub fn new(item: Option<SearchItem>) -> Self {
        let mut x = PrettyChannel { item: None };
        if let Some(SearchItem::Channel(channel)) = item {
            x.item = Some(channel);
        }
        x
    }
}

impl EmbedCreator for PrettyChannel {
    fn to_embed(&self) -> Option<CreateEmbed> {
        if let Some(target) = &self.item {
            let mut embed = CreateEmbed::new();
            if let Some(thumbnail) = &target.thumbnails.last() {
                let mut url = thumbnail.url.clone();
                if !url.starts_with("https:") {
                    url.insert_str(0, "https:");
                }
                embed = embed.thumbnail(url);
            } else {
                // todo: add github fallback image
                const FALLBACK: &str = "";
                embed = embed.thumbnail(FALLBACK);
            }
            embed = embed.title(&target.name);
            embed = embed.url(format!("http://youtube.com{}", &target.url));
            embed = embed.description(&target.description_html);

            return Some(embed);
        }
        None
    }
}
