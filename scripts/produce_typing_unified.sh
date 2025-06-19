#!/bin/bash

# Unified typing indicator producer script for Ratatoskr
# Supports both Kafka and MQTT brokers based on BROKER_TYPE environment variable
# Usage: ./scripts/produce_typing_unified.sh

set -e

# Source the unified environment setup script
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/setup_env_unified.sh" || {
  echo "Error: Failed to source setup_env_unified.sh"
  exit 1
}

# Create a temporary file for the message
TMP_FILE=$(mktemp)
trap "rm -f $TMP_FILE" EXIT

# Generate timestamp
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

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

# Display the message being sent
echo "Sending typing indicator to chat $CHAT_ID via $BROKER_TYPE..."
echo "Message: $(cat "$TMP_FILE" | jq -c . 2>/dev/null || cat "$TMP_FILE")"

# Compact JSON to single line
JSON_MESSAGE=$(cat "$TMP_FILE" | jq -c .)

# Send message based on broker type
case "$BROKER_TYPE" in
    "kafka")
        echo "Sending via Kafka..."
        echo "$CHAT_ID:$JSON_MESSAGE" | $KAFKA_CONSOLE_PRODUCER_CMD --bootstrap-server "$KAFKA_BROKER" --topic "$OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"
        ;;

    "mqtt")
        echo "Sending via MQTT..."
        echo "$JSON_MESSAGE" | $MQTT_PUB_CMD -h "$MQTT_HOST" -p "$MQTT_PORT" -t "$OUT_TOPIC" -l
        ;;

    *)
        echo "Error: Unsupported broker type: $BROKER_TYPE"
        exit 1
        ;;
esac

echo "Typing indicator sent successfully via $BROKER_TYPE!"
echo "The bot should now show 'typing...' in chat $CHAT_ID"
