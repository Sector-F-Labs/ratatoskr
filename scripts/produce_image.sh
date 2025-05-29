#!/bin/bash

# Source the environment setup script
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/setup_env.sh" || {
  echo "Error: Failed to source setup_env.sh"
  exit 1
}

# Additional Kafka settings for this script
KAFKA_OUT_TOPIC=${KAFKA_OUT_TOPIC:-"com.sectorflabs.ratatoskr.out"}

# Function to show usage
show_usage() {
    echo "Usage: $0 [IMAGE_PATH] [CAPTION]"
    echo ""
    echo "Arguments:"
    echo "  IMAGE_PATH  Path to image file (default: docs/logo.png)"
    echo "  CAPTION     Caption for the image (default: 'Here's the Ratatoskr logo!')"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Use default logo and caption"
    echo "  $0 /path/to/image.jpg                # Custom image, default caption"
    echo "  $0 /path/to/image.jpg \"My caption\"   # Custom image and caption"
    echo ""
    echo "Environment variables:"
    echo "  CHAT_ID     - Target Telegram chat ID (required)"
    echo "  KAFKA_BROKER - Kafka broker address (default: localhost:9092)"
    echo "  KAFKA_OUT_TOPIC - Kafka output topic (default: com.sectorflabs.ratatoskr.out)"
}

# Check for help flag
if [[ "$1" == "-h" || "$1" == "--help" ]]; then
    show_usage
    exit 0
fi

# Set default image path and caption
IMAGE_PATH=${1:-"$(dirname "$SCRIPT_DIR")/docs/logo.png"}
CAPTION=${2:-"Here's the Ratatoskr logo!"}

# Check if image file exists
if [ ! -f "$IMAGE_PATH" ]; then
    echo "Error: Image file not found at $IMAGE_PATH"
    echo ""
    show_usage
    exit 1
fi

# Convert to absolute path
IMAGE_PATH=$(realpath "$IMAGE_PATH")

# Create a temporary file for the message
TMP_FILE=$(mktemp)

# Write the unified outgoing message format JSON to the temporary file
cat > "$TMP_FILE" << EOF
{
  "message_type": {
    "type": "ImageMessage",
    "data": {
      "image_path": "$IMAGE_PATH",
      "caption": "$CAPTION",
      "buttons": [
        [
          {"text": "👍 Like", "callback_data": "like_image"},
          {"text": "👎 Dislike", "callback_data": "dislike_image"}
        ],
        [
          {"text": "ℹ️ Info", "callback_data": "image_info"}
        ]
      ]
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
echo "Producing image message to $KAFKA_OUT_TOPIC:"
echo "Image: $IMAGE_PATH"
echo "Caption: $CAPTION"
cat "$TMP_FILE" | jq 2>/dev/null || cat "$TMP_FILE"

# Send the message to Kafka directly from the file with chat_id as key for proper partitioning
# Compact the JSON to a single line to avoid rpk treating each line as separate message
cat "$TMP_FILE" | jq -c . | rpk topic produce "$KAFKA_OUT_TOPIC" --brokers "$KAFKA_BROKER" -k "$CHAT_ID"

# Clean up
rm "$TMP_FILE"

echo "Image message sent!"