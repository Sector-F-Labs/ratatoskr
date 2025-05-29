#!/bin/bash

# Test script for message reactions (emoji reactions) end-to-end
# This script sends a message asking for reactions and then monitors for them

# Source the environment setup script
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/setup_env.sh" || {
  echo "Error: Failed to source setup_env.sh"
  exit 1
}

# Kafka topics
KAFKA_OUT_TOPIC=${KAFKA_OUT_TOPIC:-"com.sectorflabs.ratatoskr.out"}
KAFKA_IN_TOPIC=${KAFKA_IN_TOPIC:-"com.sectorflabs.ratatoskr.in"}

echo "=== Testing Message Reactions (Emoji Reactions) ==="
echo "This script will:"
echo "1. Send a test message to Telegram"
echo "2. Ask you to add emoji reactions to that message"
echo "3. Monitor and display any reactions received"
echo ""

# Step 1: Send test message
echo "Step 1: Sending test message asking for reactions..."

TMP_FILE=$(mktemp)
cat > "$TMP_FILE" << EOF
{
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "🧪 <b>Message Reaction Test</b>\n\nPlease add emoji reactions to this message to test the reaction handling system!\n\nTry adding different emojis:\n• 👍 👎 ❤️ 🔥 🎉\n• Any other emoji you like\n\nThe system will detect and log all reaction changes.",
      "buttons": null,
      "parse_mode": "HTML",
      "disable_web_page_preview": false
    }
  },
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "target": {
    "platform": "telegram",
    "chat_id": $CHAT_ID,
    "thread_id": null
  }
}
EOF

echo "Sending test message to $KAFKA_OUT_TOPIC..."
cat "$TMP_FILE" | jq -c . | rpk topic produce "$KAFKA_OUT_TOPIC" --brokers "$KAFKA_BROKER" -k "$CHAT_ID"
rm "$TMP_FILE"

echo "✅ Test message sent!"
echo ""

# Step 2: Monitor for message reactions
echo "Step 2: Monitoring for message reactions..."
echo "Please go to your Telegram chat and add emoji reactions to the message above."
echo "Watching $KAFKA_IN_TOPIC for incoming message reactions..."
echo "Press Ctrl+C to stop monitoring."
echo ""

# Function to process incoming messages
process_message() {
    local message="$1"
    
    # Try to extract message type
    MESSAGE_TYPE=$(echo "$message" | jq -r '.message_type.type // empty' 2>/dev/null)
    
    if [ "$MESSAGE_TYPE" = "MessageReaction" ]; then
        echo "🎭 MESSAGE REACTION DETECTED!"
        echo "---"
        
        # Extract reaction details
        USER_ID=$(echo "$message" | jq -r '.message_type.data.user_id // empty')
        CHAT_ID_RX=$(echo "$message" | jq -r '.message_type.data.chat_id // empty')
        MESSAGE_ID=$(echo "$message" | jq -r '.message_type.data.message_id // empty')
        DATE=$(echo "$message" | jq -r '.message_type.data.date // empty')
        OLD_REACTIONS=$(echo "$message" | jq -r '.message_type.data.old_reaction | join(", ") // empty')
        NEW_REACTIONS=$(echo "$message" | jq -r '.message_type.data.new_reaction | join(", ") // empty')
        TIMESTAMP=$(echo "$message" | jq -r '.timestamp // empty')
        
        echo "Timestamp: $TIMESTAMP"
        echo "User ID: $USER_ID"
        echo "Chat ID: $CHAT_ID_RX"
        echo "Message ID: $MESSAGE_ID"
        echo "Reaction Date: $DATE"
        
        # Determine the action and show it clearly
        OLD_COUNT=$(echo "$message" | jq -r '.message_type.data.old_reaction | length')
        NEW_COUNT=$(echo "$message" | jq -r '.message_type.data.new_reaction | length')
        
        if [ "$OLD_COUNT" -eq 0 ] && [ "$NEW_COUNT" -gt 0 ]; then
            echo "Action: ➕ Added first reaction(s): $NEW_REACTIONS"
        elif [ "$OLD_COUNT" -gt 0 ] && [ "$NEW_COUNT" -eq 0 ]; then
            echo "Action: ➖ Removed all reactions (was: $OLD_REACTIONS)"
        elif [ "$OLD_COUNT" -lt "$NEW_COUNT" ]; then
            echo "Action: ➕ Added reaction(s)"
            echo "Before: [$OLD_REACTIONS]"
            echo "After:  [$NEW_REACTIONS]"
        elif [ "$OLD_COUNT" -gt "$NEW_COUNT" ]; then
            echo "Action: ➖ Removed reaction(s)"
            echo "Before: [$OLD_REACTIONS]"
            echo "After:  [$NEW_REACTIONS]"
        else
            echo "Action: 🔄 Changed reaction(s)"
            echo "Before: [$OLD_REACTIONS]"
            echo "After:  [$NEW_REACTIONS]"
        fi
        
        echo "---"
        echo ""
        
        # Send acknowledgment message
        echo "Sending reaction acknowledgment..."
        RESPONSE_FILE=$(mktemp)
        cat > "$RESPONSE_FILE" << EOF
{
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "✅ <b>Reaction Detected!</b>\n\nUser $USER_ID reacted to message $MESSAGE_ID\n\nReaction change: <code>[$OLD_REACTIONS] → [$NEW_REACTIONS]</code>\n\nMessage reaction handling is working! 🎉",
      "buttons": null,
      "parse_mode": "HTML",
      "disable_web_page_preview": false
    }
  },
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "target": {
    "platform": "telegram",
    "chat_id": $CHAT_ID_RX,
    "thread_id": null
  }
}
EOF
        
        cat "$RESPONSE_FILE" | jq -c . | rpk topic produce "$KAFKA_OUT_TOPIC" --brokers "$KAFKA_BROKER" -k "$CHAT_ID_RX"
        rm "$RESPONSE_FILE"
        echo "✅ Acknowledgment sent!"
        echo ""
        
    elif [ "$MESSAGE_TYPE" = "TelegramMessage" ]; then
        TEXT=$(echo "$message" | jq -r '.message_type.data.message.text // empty')
        if [ "$TEXT" != "empty" ] && [ -n "$TEXT" ]; then
            echo "📝 Regular message: $TEXT"
        fi
    fi
}

# Monitor the incoming topic
rpk topic consume "$KAFKA_IN_TOPIC" --brokers "$KAFKA_BROKER" --group "reaction-test-$(date +%s)" --format json | while read -r line; do
    # Extract the message value from rpk output
    if echo "$line" | jq -e . >/dev/null 2>&1; then
        VALUE=$(echo "$line" | jq -r '.value // empty')
        if [ "$VALUE" != "empty" ] && [ -n "$VALUE" ]; then
            # Decode the base64 value if needed
            if echo "$VALUE" | base64 -d >/dev/null 2>&1; then
                MESSAGE=$(echo "$VALUE" | base64 -d)
            else
                MESSAGE="$VALUE"
            fi
            
            # Process the message if it's valid JSON
            if echo "$MESSAGE" | jq -e . >/dev/null 2>&1; then
                process_message "$MESSAGE"
            fi
        fi
    fi
done

echo ""
echo "=== Message Reaction Test Complete ==="