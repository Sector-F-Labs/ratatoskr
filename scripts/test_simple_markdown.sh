#!/bin/bash

# Source the environment setup script
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/setup_env.sh" || {
  echo "Error: Failed to source setup_env.sh"
  exit 1
}

# Additional Kafka settings for this script
KAFKA_OUT_TOPIC=${KAFKA_OUT_TOPIC:-"com.sectorflabs.ratatoskr.out"}

# Get message text from command line argument or use default
MESSAGE_TEXT=${1:-"Quick test: *bold* _italic_ \`code\` [link](https://example.com) and special chars: ()[]{}#+-=|.!"}

echo "Testing simple markdown formatting..."
echo "Message: $MESSAGE_TEXT"

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
      "parse_mode": "Markdown",
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
echo "Producing markdown message to $KAFKA_OUT_TOPIC:"
echo "Raw JSON:"
cat "$TMP_FILE" | jq . 2>/dev/null || cat "$TMP_FILE"

# Send the message to Kafka
echo "$CHAT_ID:$(cat "$TMP_FILE" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"

# Clean up
rm "$TMP_FILE"

echo ""
echo "Simple markdown test message sent!"
echo "Usage: $0 \"Your *markdown* message here\""
echo "Check your Telegram to see if the formatting is applied correctly."
