#!/bin/bash

# Script to test typing indicator functionality
# Usage: ./scripts/produce_typing.sh

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
    echo "Usage: CHAT_ID=123456789 ./scripts/produce_typing.sh"
    exit 1
fi

# Generate timestamp
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Create temporary file for the message
TMP_FILE=$(mktemp)
trap "rm -f $TMP_FILE" EXIT

# Create typing message JSON
cat > "$TMP_FILE" <<EOF
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

echo "Sending typing indicator to chat $CHAT_ID..."
echo "Message: $(cat "$TMP_FILE")"

# Send to Kafka
# Compact the JSON to a single line to avoid kafka-console-producer treating each line as separate message
echo "$CHAT_ID:$(cat "$TMP_FILE" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"

echo "Typing indicator sent successfully!"
echo "The bot should now show 'typing...' in chat $CHAT_ID"
