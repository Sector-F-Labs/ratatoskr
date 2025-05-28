# Ratatoskr

A lightweight Telegram <-> Kafka bridge written in **Rust**, designed to decouple message ingestion from processing logic.

![Logo](docs/logo-256.png)

[![GitHub Repository](https://img.shields.io/badge/GitHub-Repository-blue.svg)](https://github.com/yourusername/ratatoskr)

## 🚀 Features

* Uses [`teloxide`](https://github.com/teloxide/teloxide) for Telegram bot integration
* Uses [`rdkafka`](https://github.com/fede1024/rust-rdkafka) for Kafka connectivity
* Forwards full Telegram message objects to Kafka
* Listens for outbound messages on a Kafka topic and sends them back to Telegram
* Minimal, event-driven, and easy to extend

## 📦 Prerequisites

* [Rust](https://www.rust-lang.org/tools/install)
* A Kafka broker (default: `localhost:9092`)
* A Telegram bot token from [@BotFather](https://t.me/BotFather)

## ⚙️ Setup

1. **Clone the repository:**

   ```sh
   git clone https://github.com/yourusername/ratatoskr.git
   cd ratatoskr
   ```

   Alternatively, you can download the source code directly from the [GitHub repository](https://github.com/yourusername/ratatoskr/releases).

2. **Set environment variables:**

   * `TELEGRAM_BOT_TOKEN` (**required**)
   * `KAFKA_BROKER` (optional, default: `localhost:9092`)
   * `KAFKA_IN_TOPIC` (optional, default: `com.sectorflabs.ratatoskr.in`)
   * `KAFKA_OUT_TOPIC` (optional, default: `com.sectorflabs.ratatoskr.out`)

   You can place these in a `.env` file or export them in your shell. A `.env.example` file is provided as a template.

3. **Build and run the bot:**

   ```sh
   cargo build --release
   ./target/release/ratatoskr
   ```

   Or simply:
   
   ```sh
   cargo run --release
   ```

## 🔄 Development

For development with auto-reload:

```sh
cargo install cargo-watch
cargo watch -x run
```

To run tests:

```sh
cargo test
```

## 🐳 Containerization

### Using Docker

To containerize the application for deployment:

1. **Using the provided Dockerfile:**

   The project includes a Dockerfile that sets up a multi-stage build for a lightweight container:

   ```dockerfile
   FROM rust:1.75-slim as builder
   WORKDIR /app
   COPY . .
   RUN cargo build --release

   FROM debian:bullseye-slim
   RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
   WORKDIR /app
   COPY --from=builder /app/target/release/ratatoskr /app/ratatoskr
   COPY --from=builder /app/.env* /app/
   ENV RUST_LOG=info
   CMD ["./ratatoskr"]
   ```

2. **Build and run the Docker image:**

   ```sh
   docker build -t ratatoskr:latest .
   docker run -d --name ratatoskr \
     -e TELEGRAM_BOT_TOKEN=your_token_here \
     -e KAFKA_BROKER=kafka:9092 \
     ratatoskr:latest
   ```

### Using Docker Compose

For a complete development environment with Kafka, Zookeeper, and Kafdrop (a Kafka UI):

1. **Run with docker-compose:**

   ```sh
   # Make sure TELEGRAM_BOT_TOKEN is set in your environment or .env file
   docker-compose up -d
   ```

2. **Access services:**
   - Ratatoskr: Running in container
   - Kafka: localhost:9092
   - Kafdrop (Kafka UI): http://localhost:9000


---

## 📤 Kafka Message Formats

This section describes the JSON message formats used by Ratatoskr when communicating with Kafka.

### Incoming Messages to `KAFKA_IN_TOPIC` (e.g., `com.sectorflabs.ratatoskr.in`)

Messages sent *from* Telegram *to* Kafka are placed on this topic. There are two main types of messages:

#### 1. Standard Messages

These are standard Telegram messages received by the bot.
*   **Kafka Key:** "message"
*   **Payload:** The full Telegram `Message` object, serialized as JSON.

```json
{
  "message_id": 123,
  "from": { "id": 456, "first_name": "User", "is_bot": false, "username": "testuser" },
  "chat": { "id": 789, "type": "private", "first_name": "User", "username": "testuser" },
  "date": 1678901234,
  "text": "Hello bot!"
  // ... other fields from the Telegram Message object
}
```

#### 2. Callback Query Messages (Button Clicks)

These messages are generated when a user clicks an inline button sent by the bot.
*   **Kafka Key:** "callback_query"
*   **Payload:** A custom JSON structure containing details about the button click.

```json
{
    "chat_id": 123456789,
    "user_id": 987654321,
    "message_id": 54321,
    "callback_data": "action_1",
    "callback_query_id": "1234567890123456789"
}
```

**Field Descriptions:**
*   `chat_id`: ID of the chat where the button was clicked.
*   `user_id`: ID of the user who clicked the button.
*   `message_id`: ID of the message to which the button was attached.
*   `callback_data`: The `callback_data` string associated with the clicked button.
*   `callback_query_id`: The unique ID for this callback query, useful for responding or acknowledging.

### Outgoing Messages from `KAFKA_OUT_TOPIC` (e.g., `com.sectorflabs.ratatoskr.out`)

Messages sent *from* Kafka *to* Telegram are placed on this topic.
*   **Kafka Key:** Not specified (can be anything, typically ignored by Ratatoskr).
*   **Payload:** A JSON object defining the message to be sent.

```json
{
  "chat_id": 123456789,
  "text": "Hello from Ratatoskr! This message can also have buttons.",
  "buttons": [
      [
          {"text": "Button 1 Label", "callback_data": "action_1"},
          {"text": "Button 2 Label", "callback_data": "action_2"}
      ],
      [
          {"text": "Another Row Button", "callback_data": "action_3"}
      ]
  ]
}
```
The `buttons` field is optional. If included, it must be a `Vec<Vec<ButtonInfo>>` (a list of lists of button information objects). Each `ButtonInfo` object has the following structure:
*   `text`: The text label displayed on the button.
*   `callback_data`: The string data that will be sent back to the bot when this button is clicked.

If `buttons` is not provided or is `null`, a plain text message will be sent.
Example without buttons:
```json
{
  "chat_id": 123456789,
  "text": "Hello from Ratatoskr! This is a plain message."
}
```

## 🧠 Why Ratatoskr?

Inspired by the mythical squirrel that relays messages across realms, Ratatoskr is built to relay messages between users and intelligent systems, using Kafka as the messaging backbone.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the project
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📃 License

This project is licensed under the BSD 3-Clause License - see the LICENSE.md file for details.
