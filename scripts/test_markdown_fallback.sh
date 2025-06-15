#!/bin/bash

# Source the environment setup script
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/setup_env.sh" || {
  echo "Error: Failed to source setup_env.sh"
  exit 1
}

# Additional Kafka settings for this script
KAFKA_OUT_TOPIC=${KAFKA_OUT_TOPIC:-"com.sectorflabs.ratatoskr.out"}

echo "Testing markdown fallback functionality..."
echo "=========================================="
echo "This script tests messages that should fail in markdown mode and fallback to plain text"

# Helper function to send a test message
send_test_message() {
    local test_name="$1"
    local message_text="$2"
    local description="$3"

    echo -e "\n$test_name:"
    echo "Description: $description"

    TMP_FILE=$(mktemp)
    cat > "$TMP_FILE" << EOF
{
  "trace_id": "$(uuidgen | tr '[:upper:]' '[:lower:]')",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "$message_text",
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

    echo "Sending: $message_text"
    echo "$CHAT_ID:$(cat "$TMP_FILE" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"
    rm "$TMP_FILE"
    sleep 3
}

# Test 1: Badly formatted markdown that should fail and fallback
send_test_message "Test 1: Invalid Markdown Syntax" \
"This message has *unclosed bold and _unclosed italic with [broken link](no-url and `unclosed code" \
"Invalid markdown syntax that should fail parsing and fallback to plain text"

# Test 2: Conflicting markdown formats
send_test_message "Test 2: Conflicting Formats" \
"This has *bold with _italic that crosses* boundaries_ which breaks markdown parsing" \
"Overlapping markdown formats that violate MarkdownV2 rules"

# Test 3: Unescaped special characters in markdown mode
send_test_message "Test 3: Unescaped Special Characters" \
"Message with unescaped chars: user@domain.com costs $100 (50% off) & more! #hashtag" \
"Special characters that need escaping in MarkdownV2 but aren't escaped"

# Test 4: Image message with bad caption
echo -e "\nTest 4: Image with Invalid Caption:"
echo "Description: Image message with markdown caption that should fail and fallback"

TMP_FILE=$(mktemp)
cat > "$TMP_FILE" << EOF
{
  "trace_id": "$(uuidgen | tr '[:upper:]' '[:lower:]')",
  "message_type": {
    "type": "ImageMessage",
    "data": {
      "image_path": "/tmp/test_image.jpg",
      "caption": "This caption has *unclosed bold and _broken formatting with unescaped & symbols",
      "buttons": null,
      "reply_keyboard": null
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

# Create a test image if it doesn't exist
if [ ! -f "/tmp/test_image.jpg" ]; then
    # Create a simple test image using ImageMagick if available
    if command -v convert >/dev/null 2>&1; then
        convert -size 100x100 xc:blue /tmp/test_image.jpg
    else
        # Create a dummy file if ImageMagick not available
        echo "Test image data" > /tmp/test_image.jpg
    fi
fi

echo "Sending image with invalid caption..."
echo "$CHAT_ID:$(cat "$TMP_FILE" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"
rm "$TMP_FILE"
sleep 3

# Test 5: Edit message with bad markdown
echo -e "\nTest 5: Edit Message with Invalid Markdown:"
echo "Description: Edit message with markdown that should fail and fallback"

# First send a message to edit
ORIGINAL_MSG_TMP=$(mktemp)
cat > "$ORIGINAL_MSG_TMP" << EOF
{
  "trace_id": "$(uuidgen | tr '[:upper:]' '[:lower:]')",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Original message that will be edited",
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

echo "Sending original message to edit..."
echo "$CHAT_ID:$(cat "$ORIGINAL_MSG_TMP" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"
rm "$ORIGINAL_MSG_TMP"
sleep 3

# Note: In a real test, you'd need the message ID from the sent message
# For this demo, we'll use a placeholder ID
echo "Note: Edit message test requires a real message ID from Telegram API"
echo "This is a demonstration of what the edit message would look like:"

TMP_FILE=$(mktemp)
cat > "$TMP_FILE" << EOF
{
  "trace_id": "$(uuidgen | tr '[:upper:]' '[:lower:]')",
  "message_type": {
    "type": "EditMessage",
    "data": {
      "message_id": 12345,
      "new_text": "Edited text with *bad markdown and _broken formatting & unescaped chars",
      "new_buttons": null
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

echo "Edit message payload (would need real message ID):"
cat "$TMP_FILE" | jq .
rm "$TMP_FILE"

# Test 6: Valid markdown that should NOT fallback
send_test_message "Test 6: Valid Markdown (Control Test)" \
"This message has *proper bold* and _proper italic_ with [valid link](https://example.com) and \`proper code\`" \
"Valid markdown that should NOT trigger fallback - used as control test"

echo -e "\n=========================================="
echo "Markdown fallback testing completed!"
echo ""
echo "What to check in your Telegram chat:"
echo ""
echo "✅ SUCCESS indicators:"
echo "- All test messages are delivered (even the badly formatted ones)"
echo "- Tests 1-4: Messages appear as plain text (no formatting applied)"
echo "- Test 6: Message appears with proper formatting (bold, italic, links, code)"
echo "- Check logs for fallback warnings like 'retrying with plain text'"
echo ""
echo "❌ FAILURE indicators:"
echo "- Any test messages fail to send completely"
echo "- Tests 1-4: Messages appear with broken/garbled formatting"
echo "- Test 6: Message appears as plain text instead of formatted"
echo ""
echo "Log messages to look for:"
echo "- 'Failed to send [message_type] with formatting, retrying with plain text'"
echo "- These should appear for Tests 1-4 but NOT for Test 6"
echo ""
echo "The fallback mechanism ensures message delivery even when markdown parsing fails!"
