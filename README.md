# Rust Bot

## What is this

使用 serenity/songbird 為基底的 discord 音樂機器人！

A simple serenity/songbird based discord music bot written in rust!

## Setup

Required dependency [yt-dlp](https://github.com/yt-dlp/yt-dlp) to play songs with songbird.

Then edit `.env` file or add it if not in the binary folder.

### Relative docs

- [discord-bot-token](https://discord.com/developers/docs/getting-started#configuring-your-bot)
- [youtube-api-key](https://developers.google.com/youtube/registering_an_application#create_project)

```toml
# The configuration file must named with `.env`

# Keys below are required
# ===================================================== #
DISCORD_TOKEN = ""
YT_API = ""
# ===================================================== #


# Keys below are optional
# ===================================================== #
# Set custom invidious instance below, default:
# INVIDIOUS_INSTANCE = "https://vid.puffyan.us"
# INVIDIOUS_INSTANCE = "https://onion.tube"

# Set your searching region
# REGION = "TW"

# Set custom command prefix, default is `~`
# COMMAND_PREFIX = "~"
# ===================================================== #
```

## Known Issue

## TODO
