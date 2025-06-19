#!/bin/bash

# Unified producer script for Ratatoskr
# Supports both Kafka and MQTT brokers based on BROKER_TYPE environment variable
# Usage: ./scripts/produce_unified.sh [MESSAGE_TEXT]

set -e

# Source the unified environment setup script
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/setup_env_unified.sh" || {
  echo "Error: Failed to source setup_env_unified.sh"
  exit 1
}

# Set a default message text
MESSAGE_TEXT=${1:-"Hello from Ratatoskr unified test script!"}

# Create a temporary file for the message
TMP_FILE=$(mktemp)
trap "rm -f $TMP_FILE" EXIT

# Write the correctly formatted JSON to the temporary file
cat > "$TMP_FILE" << EOF
{
  "trace_id": "$(uuidgen | tr '[:upper:]' '[:lower:]')",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "$MESSAGE_TEXT",
      "buttons": null,
      "parse_mode": null,
      "disable_web_page_preview": false
    }
  },
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "target": {
    "platform": "telegram",
    "chat_id": $CHAT_ID,
    "thread_id": null
  }
}
EOF

# Display the message being sent
echo "Producing message to $OUT_TOPIC ($BROKER_TYPE):"
cat "$TMP_FILE" | jq 2>/dev/null || cat "$TMP_FILE"

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

echo "Message sent successfully via $BROKER_TYPE!"
