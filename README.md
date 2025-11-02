# Big Bangers Bot 🤩

This Discord starboard bot is designed to support large servers with multiple channels of varying activity levels. It provides users with the ability to prioritize specific channels for starboard posting while ignoring others. Built with efficiency in mind.

- Channel Prioritization: Specify which channels to prioritize when scanning.
- Channel Ignoring: Choose which channels to ignore.
- Efficient Resource Usage: Consuming less than 12MB of RAM for a discord server with 30 channels and 1.7 million messages

## Setup 🛠️

Create a `config.json` file based on [`config.example.json`](https://github.com/AnnsAnns/bigbangersbot/blob/main/config.example.json)

### Configuration Options

- `discordChannel`: The ID of the channel where starred messages will be posted (starboard channel)
- `discordServer`: The ID of your Discord server
- `discordToken`: Your Discord bot token
- `threshold`: Minimum number of reactions required for a message to be posted to the starboard
- `reply`: Whether the bot should reply to starred messages with a configured response
- `replies`: Array of possible reply messages the bot can send (randomly selected if `reply` is true)
- `enableChannelWhitelist`: When true, only monitors channels listed in the `channels` array
- `channels`: Array of channel objects to whitelist (only used if `enableChannelWhitelist` is true)
    - `id`: Channel ID to include in the whitelist
- `persistenceFile`: (Optional) Path to the file where approved messages are saved (defaults to `approved_messages.json`)

### Persistence

The bot automatically saves the mapping of original messages to starboard posts when shutting down. This data is loaded on startup, allowing the bot to:
- Update existing starboard posts when reactions change
- Avoid duplicate posts after restarts
- Maintain continuity across bot restarts

## Run 🏃

### Using Cargo

```bash
cargo run
```

### Using Docker

```bash
# Option 1: Use the pre-built image from GitHub Container Registry
docker run -v $(pwd)/config.json:/app/config.json:ro \
           -v $(pwd)/approved_messages.json:/app/approved_messages.json \
           ghcr.io/annsanns/bigbangersbot:latest

# Option 2: Build locally
docker build -t big-bangers-bot .
docker run -v $(pwd)/config.json:/app/config.json:ro \
           -v $(pwd)/approved_messages.json:/app/approved_messages.json \
           big-bangers-bot
```

**Note**: The `approved_messages.json` volume mount is needed for persistence. The file will be created automatically on first run.

### Using Docker Compose

By default the `docker-compose.yml` is set to use the pre-built image from GitHub Container Registry. If you want to build the image locally instead, comment out the `image` line and uncomment the `build` line.

```bash
# Make sure config.json exists in the project directory
docker-compose up -d
```

The `docker-compose.yml` file already includes the persistence file mount.

## Build 🏗️

### Native Build

```bash
cargo build --release
```

### Docker Build

```bash
docker build -t big-bangers-bot .
```

## Contributions 🤝

Contributions are welcome! If you have suggestions for improvements, new features, or encounter any issues, feel free to open an issue or pull request on GitHub.

## License 📜

This project is licensed under the EUPL license - see the [LICENSE](LICENSE) file for details
