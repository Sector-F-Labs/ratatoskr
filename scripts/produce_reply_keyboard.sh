#!/bin/bash

# Source the environment setup script
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/setup_env.sh" || {
  echo "Error: Failed to source setup_env.sh"
  exit 1
}

# Additional Kafka settings for this script
KAFKA_OUT_TOPIC=${KAFKA_OUT_TOPIC:-"com.sectorflabs.ratatoskr.out"}

# Set a default message text
MESSAGE_TEXT=${1:-"Please share your location or contact using the buttons below:"}

# Create a temporary file for the message
TMP_FILE=$(mktemp)

# Write the correctly formatted JSON to the temporary file
cat > "$TMP_FILE" << EOF
{
  "trace_id": "$(uuidgen | tr '[:upper:]' '[:lower:]')",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "$MESSAGE_TEXT",
      "buttons": null,
      "reply_keyboard": {
        "keyboard": [
          [
            {
              "text": "📍 Share Location",
              "request_location": true
            },
            {
              "text": "📞 Share Contact",
              "request_contact": true
            }
          ],
          [
            {
              "text": "📊 Create Poll",
              "request_poll": {
                "type": "regular"
              }
            }
          ],

          [
            {
              "text": "❌ Hide Keyboard"
            }
          ]
        ],
        "resize_keyboard": true,
        "one_time_keyboard": true,
        "input_field_placeholder": "Use the buttons or type a message..."
      },
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
echo "Producing message with reply keyboard to $KAFKA_OUT_TOPIC:"
cat "$TMP_FILE" | jq 2>/dev/null || cat "$TMP_FILE"

# Send the message to Kafka directly from the file with chat_id as key for proper partitioning
# Compact the JSON to a single line to avoid kafka-console-producer treating each line as separate message
echo "$CHAT_ID:$(cat "$TMP_FILE" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"

# Clean up
rm "$TMP_FILE"

echo "Message with reply keyboard sent!"
