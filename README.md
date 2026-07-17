# Ratatoskr

A Telegram <-> Kafka bridge written in **Rust**. It streams inbound Telegram updates onto a Kafka topic and delivers outbound messages read from another Kafka topic back to Telegram, with optional per-user authorization.

![Logo](docs/logo-256.png)

[![GitHub Repository](https://img.shields.io/badge/GitHub-Repository-blue.svg)](https://github.com/sector-f-labs/ratatoskr)

## 🚀 Features

* Uses [`teloxide`](https://github.com/teloxide/teloxide) for Telegram bot integration
* Publishes inbound Telegram updates as JSON to the `{prefix}.in` Kafka topic
* Consumes `OutgoingMessage` JSON from the `{prefix}.out` Kafka topic and delivers it to Telegram
* Supports text, image, audio, voice, video, video note, document, sticker, and animation messages, plus message edit/delete and typing indicators
* Optional per-user auth (`users.toml`) that gates which Telegram users/IDs may talk to the bot
* CLI with `serve`, `users` (add/remove/list), and `send` (publish a message straight to Kafka) subcommands

## 📦 Prerequisites

* [Rust](https://www.rust-lang.org/tools/install)
* A Telegram bot token from [@BotFather](https://t.me/BotFather)
* A running Kafka broker (a `docker-compose.yml` with Kafka, Zookeeper, and an AKHQ UI is provided)

## ⚙️ Setup

1. **Clone the repository:**

   ```sh
   git clone https://github.com/sector-f-labs/ratatoskr.git
   cd ratatoskr
   ```

2. **Start Kafka:**

   ```sh
   docker-compose up -d
   ```

   This brings up Kafka on `localhost:9092`, Zookeeper, and an AKHQ UI at `http://localhost:8080` for inspecting topics.

3. **Set environment variables:**

   * `TELEGRAM_BOT_TOKEN` (**required**)
   * `KAFKA_BROKERS` (optional, default: `localhost:9092`)
   * `KAFKA_TOPIC_PREFIX` (optional, default: `ratatoskr` — topics become `{prefix}.in` / `{prefix}.out`)

   You can place these in a `.env` file (loaded automatically via `dotenv`) or export them in your shell.

4. **(Optional) Configure allowed users:**

   By default, with no `users.toml`, auth is disabled and all messages pass through. To restrict who can talk to the bot:

   ```sh
   cargo run --release -- users add --system-user alice --username alice_tg --promote
   ```

   This writes to `/etc/ratatoskr/users.toml` by default; pass `--users-file <path>` (before the subcommand) to use a different location.

5. **Build and run the bot:**

   ```sh
   cargo build --release
   ./target/release/ratatoskr serve
   ```

   Or simply:

   ```sh
   cargo run --release -- serve
   ```

## 🔄 Development

For development with auto-reload:

```sh
cargo install cargo-watch
cargo watch -x 'run -- serve'
```

To run tests:

```sh
cargo test
```

### CLI commands

```sh
ratatoskr serve                       # Run the bot: consumes Telegram updates, bridges to/from Kafka
ratatoskr users add --system-user <name> [--username <tg_username>]... [--promote]
ratatoskr users remove --system-user <name>
ratatoskr users list
ratatoskr send --chat-id <id> [--parse-mode HTML|Markdown] [--thread-id <id>] <message text>
```

`send` publishes an `OutgoingMessage` directly to the `{prefix}.out` topic (reading from stdin and/or a trailing positional message), which is useful for testing without going through your own producer.

### Inspecting topics directly

With Kafka running, `scripts/produce.sh` and `scripts/consume.sh` are convenience wrappers around `kafka-console-producer`/`kafka-console-consumer` for manually publishing/reading messages on the topics (see `scripts/setup_env.sh` for the env vars they expect).

----

## 📤 Unified Message Types

Ratatoskr uses a unified message type system for consistent handling of all communications. For detailed documentation, see [Unified Message Types](docs/unified_message_types.md).

### 🔧 Client Type Generation

Ratatoskr provides TypeScript definitions that can be used with [quicktype](https://quicktype.io/) to generate client types in your preferred programming language:

```bash
# Install quicktype
npm install -g quicktype

# Generate Python types
quicktype --src docs/types/ratatoskr-types.ts --lang python --out ratatoskr_types.py

# Generate Java types
quicktype --src docs/types/ratatoskr-types.ts --lang java --out RatatoskrTypes.java

# Generate Go types
quicktype --src docs/types/ratatoskr-types.ts --lang go --out ratatoskr_types.go
```

Supported languages include Python, Java, C#, Go, Rust, Kotlin, Swift, Dart, and more. For complete instructions and examples, see [Type Generation Guide](docs/types/README.md).

You can also use the provided generation script:
```bash
cd docs/types
./generate-types.sh  # Generates types for all supported languages
```

### Incoming messages published to the `{prefix}.in` Kafka topic

All messages from Telegram are wrapped in the unified `IncomingMessage` type:

#### Telegram Message Example
```json
{
  "message_type": {
    "type": "TelegramMessage",
    "data": {
      "message": {
        "message_id": 123,
        "from": { "id": 456, "first_name": "User", "username": "testuser" },
        "chat": { "id": 789, "type": "private" },
        "date": 1678901234,
        "text": "Hello bot!"
      },
      "file_attachments": [
        {
          "file_id": "AgACAgIAAxkDAAIC_mF...",
          "file_unique_id": "abc123def456",
          "file_type": "Photo",
          "file_size": 245760,
          "file_url": "https://api.telegram.org/file/bot<token>/photos/file_1.jpg",
          "metadata": { "type": "Photo", "width": 1920, "height": 1080 }
        }
      ]
    }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "source": {
    "platform": "telegram",
    "bot_id": null,
    "bot_username": null
  }
}
```

#### Callback Query Example
```json
{
  "message_type": {
    "type": "CallbackQuery",
    "data": {
      "chat_id": 123456789,
      "user_id": 987654321,
      "message_id": 54321,
      "callback_data": "action_1",
      "callback_query_id": "1234567890123456789"
    }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "source": {
    "platform": "telegram",
    "bot_id": null,
    "bot_username": null
  }
}
```

#### Edited Message Example
```json
{
  "message_type": {
    "type": "EditedMessage",
    "data": {
      "message": {
        "message_id": 123,
        "from": { "id": 456, "first_name": "User", "username": "testuser" },
        "chat": { "id": 789, "type": "private" },
        "date": 1678901234,
        "edit_date": 1678901300,
        "text": "Hello bot! (edited)"
      },
      "file_attachments": [],
      "edit_date": 1678901300
    }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "source": {
    "platform": "telegram",
    "bot_id": null,
    "bot_username": null
  }
}
```

### Outgoing messages read from the `{prefix}.out` Kafka topic

All messages to Telegram use the unified `OutgoingMessage` type:

#### Typing Indicator Example
```json
{
  "message_type": {
    "type": "TypingMessage",
    "data": {
      "action": "typing"
    }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "target": {
    "platform": "telegram",
    "chat_id": 123456789,
    "thread_id": null
  }
}
```

This message will cause the bot to display the "typing..." indicator in the specified chat, letting users know the bot is busy processing.

#### Text Message Example
```json
{
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Hello from Ratatoskr! This message can have buttons.",
      "buttons": [
        [
          {"text": "Button 1", "callback_data": "action_1"},
          {"text": "Button 2", "callback_data": "action_2"}
        ]
      ],
      "parse_mode": "HTML",
      "disable_web_page_preview": false
    }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "target": {
    "platform": "telegram",
    "chat_id": 123456789,
    "thread_id": null
  }
}
```

#### Image Message Example
```json
{
  "message_type": {
    "type": "ImageMessage",
    "data": {
      "image_path": "/path/to/image.jpg",
      "caption": "Check out this image!",
      "buttons": [
        [{"text": "Like", "callback_data": "like_image"}]
      ]
    }
  },
  "timestamp": "2023-12-01T10:30:00Z",
  "target": {
    "platform": "telegram",
    "chat_id": 123456789,
    "thread_id": null
  }
}
```

### Supported Message Types

- **TextMessage** - Send text with optional formatting and buttons
- **ImageMessage** - Send images from local filesystem
- **AudioMessage** - Send audio files
- **VoiceMessage** - Send voice notes
- **VideoMessage** - Send videos
- **VideoNoteMessage** - Send round video notes
- **DocumentMessage** - Send documents/files from local filesystem
- **StickerMessage** - Send stickers
- **AnimationMessage** - Send GIFs/animations
- **EditMessage** - Edit previously sent messages
- **DeleteMessage** - Delete messages from chat
- **TypingMessage** - Show typing indicator (bot is busy)

For complete documentation, see [Unified Message Types](docs/unified_message_types.md).
For practical examples and usage patterns, see [Examples](docs/examples.md).
For troubleshooting common issues, see [Troubleshooting Guide](docs/troubleshooting.md).

## 🧠 Why Ratatoskr?

Inspired by the mythical squirrel that relays messages across realms, Ratatoskr is built to relay messages between users and intelligent systems, using simple pipe-based messaging as the backbone.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the project
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📃 License

This project is licensed under the BSD 3-Clause License - see the LICENSE.md file for details.
