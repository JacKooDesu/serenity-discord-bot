use std::sync::Arc;

use async_recursion::async_recursion;
use invidious::{ClientAsync as YtClient, CommonPlaylist, PublicItems};
use serde::{Deserialize, Serialize};
use serenity::{
    all::{Message, User},
    builder::{CreateEmbed, CreateEmbedAuthor, CreateMessage, EditMessage},
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
};
use songbird::input::YoutubeDl;

use crate::bot::{
    commands::artist::{find_artist, EmbedCreator, YtClientKey},
    common::{add_song, say, try_say, HttpKey},
    constants::{BACK_EMOJI, NEXT_EMOJI, NUM_EMOJI},
    utils::reaction_collector::{ActionEnumTrait, ReactionCollector},
};

use super::{play::create_song_begin_event, playlist::flat_list};

#[command]
#[only_in(guilds)]
pub async fn album(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    if args.is_empty() {
        say(msg.channel_id, ctx, "Args missing!").await;
        return Ok(());
    }

    let mut msg_builder = CreateMessage::new().content("Artist Founded!");
    let mut action_map = Vec::new();

    if let Some(channels) = find_artist(ctx, args).await {
        let mut iter = 0;
        for channel in channels {
            if let Some(embed) = channel.to_embed() {
                msg_builder = msg_builder.add_embed(embed);
                if let Some(channel) = channel.item {
                    action_map.push((NUM_EMOJI[iter], NextAction::ExploreVideos(channel.id, 0)));
                    iter += 1;
                }
            }
        }
    } else {
        msg_builder = msg_builder.content("Cannot call search api!");
    }

    if let Ok(reply) = msg.channel_id.send_message(ctx, msg_builder).await {
        let collector = ReactionCollector::create(action_map);
        match collector
            .wait_reaction(&msg.author, reply.clone(), ctx)
            .await
        {
            Some(NextAction::ExploreVideos(id, page)) => {
                explore_albums(ctx, reply.clone(), msg.author.clone(), &id, page, None).await
            }
            Some(_) | None => None,
        };
    }
    Ok(())
}

#[async_recursion]
async fn explore_albums(
    ctx: &Context,
    msg: Message,
    user: User,
    id: &str,
    offset: usize,
    buffer: Option<Vec<CommonPlaylist>>,
) -> Option<()> {
    const PAGE_SIZE: usize = 5;
    let mut msg_builder = EditMessage::new().content("").embeds(Vec::new());
    let mut action_map = Vec::new();
    let videos: Vec<CommonPlaylist> = if let Some(_buffer) = buffer {
        _buffer
    } else if let Some(yt_client) = ctx.data.read().await.get::<YtClientKey>() {
        if let Ok(videos) = channel_releases(yt_client, id, None).await {
            videos.playlists
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let len = usize::min(videos.len() - offset, PAGE_SIZE);

    for i in 0..len {
        let origin = videos.get(i + offset);
        let embed = PrettyPlaylist::new(origin.cloned());
        if let Some(mut item) = embed.to_embed() {
            item = item.author(CreateEmbedAuthor::new(NUM_EMOJI[i]));
            msg_builder = msg_builder.add_embed(item);
            if let Some(x) = embed.item {
                action_map.push((NUM_EMOJI[i], NextAction::Finished(x.id)));
            }
        }
    }

    if (offset + PAGE_SIZE) < videos.len() {
        action_map.push((
            NEXT_EMOJI,
            NextAction::ExploreVideos(id.to_string(), offset + PAGE_SIZE),
        ));
    }

    if offset >= PAGE_SIZE {
        action_map.insert(
            0,
            (
                BACK_EMOJI,
                NextAction::ExploreVideos(id.to_string(), offset - PAGE_SIZE),
            ),
        );
    }

    let _ = msg.delete_reactions(ctx).await;

    if let Ok(_) = msg.clone().edit(ctx, msg_builder).await {
        let collector = ReactionCollector::create(action_map);
        match collector.wait_reaction(&user, msg.clone(), ctx).await {
            Some(NextAction::Finished(id)) => selected_album(ctx, msg, id.as_str()).await,
            Some(NextAction::ExploreVideos(id, page)) => {
                explore_albums(ctx, msg, user, id.as_str(), page, Some(videos)).await;
            }
            _ => (),
        }
    };
    Some(())
}

async fn selected_album(ctx: &Context, msg: Message, id: &str) {
    let guild_id = msg.guild_id.unwrap();

    let http_client = {
        let data = ctx.data.read().await;
        let http_client = data
            .get::<HttpKey>()
            .cloned()
            .expect("Cannot get http clinet!");
        http_client
    };

    let channel_id = msg.channel_id;
    let send_http = ctx.http.clone();

    let songs = flat_list(format!("https://youtube.com/playlist?list={}", id).as_str()).await;
    for s in songs {
        let src = YoutubeDl::new(http_client.clone(), s);
        if let Some(result) = add_song(ctx, guild_id, src).await {
            if let Some(_) = result.1 {
                create_song_begin_event(
                    send_http.clone(),
                    Arc::new(ctx.clone()),
                    result.0,
                    channel_id,
                )
                .await;
            }
        }
    }
    try_say(channel_id, ctx, "Playlist added!").await;
}

#[derive(Clone)]
pub(crate) enum NextAction {
    ExploreVideos(String, usize),
    Finished(String), // Error(String),
    Error,
}

impl ActionEnumTrait for NextAction {
    fn fallback_action() -> Self {
        NextAction::Error
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChannelReleases {
    pub playlists: Vec<CommonPlaylist>,
}

impl PublicItems for ChannelReleases {
    fn url(args: String) -> String {
        format!("api/v1/channels/{args}/releases")
    }
}

pub async fn channel_releases(
    client: &YtClient,
    id: &str,
    params: Option<&str>,
) -> Result<ChannelReleases, invidious::InvidiousError> {
    ChannelReleases::fetch_async(client, Some(id), params).await
}

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
