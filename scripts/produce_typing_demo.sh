#!/bin/bash

# Demo script showing typing indicator followed by a message
# Usage: CHAT_ID=123456789 TEXT="Hello!" ./scripts/produce_typing_demo.sh

set -e

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | xargs)
fi

# Configuration
KAFKA_BROKER=${KAFKA_BROKER:-localhost:9092}
KAFKA_OUT_TOPIC=${KAFKA_OUT_TOPIC:-com.sectorflabs.ratatoskr.out}

# Check if CHAT_ID is set
if [ -z "$CHAT_ID" ]; then
    echo "Error: CHAT_ID environment variable is required"
    echo "Usage: CHAT_ID=123456789 TEXT=\"Hello!\" ./scripts/produce_typing_demo.sh"
    exit 1
fi

# Default message if not provided
TEXT=${TEXT:-"Hello! I was just typing..."}

echo "Demo: Sending typing indicator followed by message to chat $CHAT_ID"

# Step 1: Send typing indicator
echo "1. Sending typing indicator..."
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Create temporary file for typing message
TMP_FILE_TYPING=$(mktemp)
trap "rm -f $TMP_FILE_TYPING" EXIT

cat > "$TMP_FILE_TYPING" <<EOF
{
  "trace_id": "$(uuidgen | tr '[:upper:]' '[:lower:]')",
  "message_type": {
    "type": "TypingMessage",
    "data": {
      "action": "typing"
    }
  },
  "timestamp": "$TIMESTAMP",
  "target": {
    "platform": "telegram",
    "chat_id": $CHAT_ID,
    "thread_id": null
  }
}
EOF

# Send typing message to Kafka
echo "$CHAT_ID:$(cat "$TMP_FILE_TYPING" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"

echo "   ✓ Typing indicator sent"

# Step 2: Wait a moment to simulate processing
echo "2. Simulating processing time (3 seconds)..."
sleep 3

# Step 3: Send actual message
echo "3. Sending text message..."
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Create temporary file for text message
TMP_FILE_TEXT=$(mktemp)
trap "rm -f $TMP_FILE_TEXT" EXIT

cat > "$TMP_FILE_TEXT" <<EOF
{
  "trace_id": "$(uuidgen | tr '[:upper:]' '[:lower:]')",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "$TEXT",
      "parse_mode": "HTML"
    }
  },
  "timestamp": "$TIMESTAMP",
  "target": {
    "platform": "telegram",
    "chat_id": $CHAT_ID,
    "thread_id": null
  }
}
EOF

# Send text message to Kafka
echo "$CHAT_ID:$(cat "$TMP_FILE_TEXT" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"

echo "   ✓ Text message sent"
echo ""
echo "Demo complete! Check your Telegram chat to see the typing indicator followed by the message."
