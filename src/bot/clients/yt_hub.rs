use std::fmt;

use google_youtube3::{hyper::client::HttpConnector, hyper_rustls::HttpsConnector, *};
use serenity::{client::Context, all::ReactionType};
use songbird::typemap::TypeMapKey;

use crate::bot::constants::NUM_EMOJI;

pub struct YtHubKey;
impl TypeMapKey for YtHubKey {
    type Value = YouTube<HttpsConnector<HttpConnector>>;
}

pub async fn init_yt_hub() -> YouTube<HttpsConnector<HttpConnector>> {
    YouTube::new(
        hyper::Client::builder().build(
            hyper_rustls::HttpsConnectorBuilder::new()
                .with_native_roots()
                .https_or_http()
                .enable_http1()
                .build(),
        ),
        "".to_string(),
    )
}

pub async fn get_search_result(ctx: &Context, args: &str, api_key: &str) -> Option<Vec<PrettyResult>> {
    if let Some(hub) = ctx.data.read().await.get::<YtHubKey>() {
        let result = hub
            .search()
            .list(&vec!["snippet".into()])
            .q(args)
            .add_type("video")
            .safe_search("none")
            .param("key", api_key)
            .doit()
            .await;

        match result {
            Err(e) => match e {
                Error::HttpError(_)
                | Error::Io(_)
                | Error::MissingAPIKey
                | Error::MissingToken(_)
                | Error::Cancelled
                | Error::UploadSizeLimitExceeded(_, _)
                | Error::Failure(_)
                | Error::BadRequest(_)
                | Error::FieldClash(_)
                | Error::JsonDecodeError(_, _) => {
                    println!("{}", e);
                    None
                }
            },
            Ok(ok) => {
                let mut pretties: Vec<PrettyResult> = Vec::new();
                let arr = ok.1.items.unwrap();
                for (index, item) in arr.into_iter().enumerate() {
                    pretties.push(PrettyResult(item, index));
                }

                Some(pretties)
            }
        }
    } else {
        None
    }
}

pub struct PrettyResult(api::SearchResult, usize);
impl PrettyResult {
    pub fn get_emoji_reaction(&self) -> ReactionType {
        ReactionType::Unicode(NUM_EMOJI[self.1].to_string())
    }

    pub fn get_video_url(&self) -> String {
        if let Some(Some(id)) = self.0.id.clone().map(|x| x.video_id) {
            format!("https://youtu.be/{}", id)
        } else {
            String::new()
        }
    }
}
impl fmt::Display for PrettyResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let title = &self.0.snippet.clone().unwrap().title.unwrap();
        let str = format!("{0}. {1}", &self.1 + 1, title);
        f.write_str(str.as_str())?;
        Ok(())
    }
}
