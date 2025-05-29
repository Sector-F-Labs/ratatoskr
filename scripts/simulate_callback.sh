#!/bin/bash

# Source the environment setup script
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/setup_env.sh" || {
  echo "Error: Failed to source setup_env.sh"
  exit 1
}

# Additional Kafka settings for this script
KAFKA_IN_TOPIC=${KAFKA_IN_TOPIC:-"com.sectorflabs.ratatoskr.in"}

# Set default message ID and callback data
MESSAGE_ID=${1:-"1234"}
CALLBACK_DATA=${2:-"button1_action"}
USER_ID=${USER_ID:-"$CHAT_ID"}  # Default to using the same ID as the chat
CALLBACK_QUERY_ID=${3:-"$(date +%s)_$(openssl rand -hex 4)"}

# Create a temporary file for the message
TMP_FILE=$(mktemp)

# Write the correctly formatted JSON to the temporary file
cat > "$TMP_FILE" << EOF
{"chat_id":$CHAT_ID,"user_id":$USER_ID,"message_id":$MESSAGE_ID,"callback_data":"$CALLBACK_DATA","callback_query_id":"$CALLBACK_QUERY_ID"}
EOF

# Set the key for the Kafka message to "callback_query"
KEY="callback_query"

echo "Simulating callback query to $KAFKA_IN_TOPIC:"
cat "$TMP_FILE" | jq 2>/dev/null || cat "$TMP_FILE"

# Use rpk to produce the message with the specified key
# Compact the JSON to a single line to avoid rpk treating each line as separate message
cat "$TMP_FILE" | jq -c . | rpk topic produce $KAFKA_IN_TOPIC --brokers $KAFKA_BROKER -k "$KEY"

# Clean up
rm "$TMP_FILE"

echo "Callback query simulation sent!"