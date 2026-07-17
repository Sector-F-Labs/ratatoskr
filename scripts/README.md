# Ratatoskr Test Scripts

This directory contains scripts for testing different message types with Ratatoskr's Kafka message handling. These scripts help developers test message sending without needing complex setups.

## Prerequisites

- Kafka CLI tools (`kafka-console-producer`, `kafka-topics`, etc.)
- `jq` for JSON formatting (install with `brew install jq` on macOS or `apt install jq` on Debian/Ubuntu)
- Environment variable `CHAT_ID` set to your Telegram chat ID

## Available Scripts

### 1. Basic Text Message

```bash
./scripts/produce.sh "Your message here"
```

Sends a simple text message to Telegram.

### 2. Message with Buttons

```bash
./scripts/produce_with_buttons.sh "Choose an option"
```

Sends a message with inline keyboard buttons to Telegram.

### 3. Image Message

```bash
./scripts/produce_image.sh "path/to/image.jpg" "Image caption"
```

Sends an image with caption and buttons to Telegram.

### 4. Simulate Button Click

```bash
./scripts/simulate_callback.sh 123 "button_action"
```

Simulates a user clicking a button by sending a callback query message.

## Environment Variables

All scripts use these environment variables:

- `CHAT_ID`: Telegram chat ID (required)
- `USER_ID`: Telegram user ID (defaults to CHAT_ID if not set)
- `KAFKA_BROKERS`: Kafka broker address (default: "localhost:9092")
- `KAFKA_TOPIC_PREFIX`: Topic prefix (default: "ratatoskr"); topics are `{prefix}.in` / `{prefix}.out`
- `KAFKA_IN_TOPIC`: Input topic name (default: "ratatoskr.in")
- `KAFKA_OUT_TOPIC`: Output topic name (default: "ratatoskr.out")

Set `CHAT_ID` in your `.envrc` file or export it:

```bash
export CHAT_ID=123456789
```

## Monitoring Messages

Use Kafka monitoring tools or web UIs to monitor Kafka topics instead of command-line scripts. Tools like Kafka UI, Kafdrop, or the command-line `kafka-console-consumer` provide interfaces for viewing message contents and topic activity.

## Making Scripts Executable

Before running scripts directly, make them executable:

```bash
chmod +x scripts/*.sh
```