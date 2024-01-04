use super::prettier::EmbedCreator;
use serenity::builder::{CreateEmbed, CreateEmbedAuthor};
use songbird::input::AuxMetadata;

pub struct PrettyAuxMetadata {
    pub item: Option<AuxMetadata>,
}

impl EmbedCreator for PrettyAuxMetadata {
    fn to_embed(&self) -> Option<CreateEmbed> {
        if let Some(target) = &self.item {
            let mut embed = CreateEmbed::new();
            if let Some(thumbnail) = &target.thumbnail {
                let mut url = thumbnail.clone();
                if !url.starts_with("https:") {
                    url.insert_str(0, "https:");
                }
                embed = embed.thumbnail(url);
            } else {
                // todo: add github fallback image
                const FALLBACK: &str = "";
                embed = embed.thumbnail(FALLBACK);
            }

            embed = embed.title(&target.title.clone().unwrap_or_default());
            embed = embed.url(&target.source_url.clone().unwrap_or_default());

            let author = CreateEmbedAuthor::new(&target.artist.clone().unwrap_or_default());
            embed = embed.author(author);

            return Some(embed);
        }
        None
    }
}
