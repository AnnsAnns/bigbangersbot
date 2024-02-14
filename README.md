# Big Bangers Bot 🤩

This Discord starboard bot is designed to support large servers with multiple channels of varying activity levels. It provides users with the ability to prioritize specific channels for starboard posting while ignoring others. Built with efficiency in mind.

- Channel Prioritization: Specify which channels to prioritize when scanning.
- Channel Ignoring: Choose which channels to ignore.
- Efficient Resource Usage: Consuming less than 12MB of RAM for a discord server with 30 channels and 1.7 million messages

## Setup 🛠️

Create a `config.json` file based on [`config.example.json`](https://github.com/AnnsAnns/bigbangersbot/blob/main/config.example.json)

*Priority options: `Low`, `Medium` & `High`*

## Run 🏃

```bash
cargo run
```

## Build 🏗️

```bash
cargo build --release
```

## Contributions 🤝

Contributions are welcome! If you have suggestions for improvements, new features, or encounter any issues, feel free to open an issue or pull request on GitHub.

## License 📜

This project is licensed under the EUPL license - see the [LICENSE](LICENSE) file for details
