use std::env;

use invidious::ClientAsync as YtClient;
use songbird::typemap::TypeMapKey;

use crate::bot::constants::INVIDIOUS_INSTANCE_KEY;

pub struct YtClientKey {}
impl TypeMapKey for YtClientKey {
    type Value = YtClient;
}

pub async fn init_yt_client() -> YtClient {
    if let Ok(instance) = env::var(INVIDIOUS_INSTANCE_KEY) {
        YtClient::new(instance, invidious::MethodAsync::default())
    } else {
        YtClient::default()
    }
}
