# Ratatoskr Test Scripts

This directory contains scripts for testing different message types with Ratatoskr's Kafka message handling. These scripts help developers test message sending without needing complex setups.

## Prerequisites

- [Redpanda CLI (`rpk`)](https://docs.redpanda.com/docs/reference/rpk-commands/) or Kafka CLI tools
- `jq` for JSON formatting (install with `brew install jq` on macOS or `apt install jq` on Debian/Ubuntu)
- Environment variable `CHAT_ID` set to your Telegram chat ID

## Available Scripts

### 1. Basic Text Message

```bash
make produce TEXT="Your message here"
# or directly:
./scripts/produce.sh "Your message here"
```

Sends a simple text message to Telegram.

### 2. Message with Buttons

```bash
make test_buttons TEXT="Choose an option"
# or directly:
./scripts/produce_with_buttons.sh "Choose an option"
```

Sends a message with inline keyboard buttons to Telegram.

### 3. Image Message

```bash
make test_image IMAGE_PATH="path/to/image.jpg" CAPTION="Image caption"
# or directly:
./scripts/produce_image.sh "path/to/image.jpg" "Image caption"
```

Sends an image with caption and buttons to Telegram.

### 4. Simulate Button Click

```bash
make test_callback MESSAGE_ID=123 CALLBACK_DATA="button_action"
# or directly:
./scripts/simulate_callback.sh 123 "button_action"
```

Simulates a user clicking a button by sending a callback query message.

## Environment Variables

All scripts use these environment variables:

- `CHAT_ID`: Telegram chat ID (required)
- `USER_ID`: Telegram user ID (defaults to CHAT_ID if not set)
- `KAFKA_BROKER`: Kafka broker address (default: "localhost:9092")
- `KAFKA_IN_TOPIC`: Input topic name (default: "com.sectorflabs.ratatoskr.in")
- `KAFKA_OUT_TOPIC`: Output topic name (default: "com.sectorflabs.ratatoskr.out")

Set `CHAT_ID` in your `.envrc` file or export it:

```bash
export CHAT_ID=123456789
```

## Monitoring Messages

Use the Redpanda Console UI to monitor Kafka topics instead of command-line scripts. The console provides a better interface for viewing message contents and topic activity.

## Making Scripts Executable

Before running scripts directly, make them executable:

```bash
chmod +x scripts/*.sh
```