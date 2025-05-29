#!/bin/bash

# Monitor message reactions (emoji reactions) in real-time
# This script watches the incoming Kafka topic for message reactions only

# Source the environment setup script
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/setup_env.sh" || {
  echo "Error: Failed to source setup_env.sh"
  exit 1
}

# Kafka topics
KAFKA_IN_TOPIC=${KAFKA_IN_TOPIC:-"com.sectorflabs.ratatoskr.in"}

echo "=== Monitoring Message Reactions (Emoji Reactions) ==="
echo "Watching $KAFKA_IN_TOPIC for incoming message reactions..."
echo "Add emoji reactions to messages in your Telegram chat to see them here."
echo "Press Ctrl+C to stop monitoring."
echo ""

# Function to process incoming messages
process_message() {
    local message="$1"
    
    # Try to extract message type
    MESSAGE_TYPE=$(echo "$message" | jq -r '.message_type.type // empty' 2>/dev/null)
    
    if [ "$MESSAGE_TYPE" = "MessageReaction" ]; then
        echo "🎭 MESSAGE REACTION DETECTED! $(date '+%H:%M:%S')"
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
        echo "Old Reactions: [$OLD_REACTIONS]"
        echo "New Reactions: [$NEW_REACTIONS]"
        
        # Determine the action
        OLD_COUNT=$(echo "$message" | jq -r '.message_type.data.old_reaction | length')
        NEW_COUNT=$(echo "$message" | jq -r '.message_type.data.new_reaction | length')
        
        if [ "$OLD_COUNT" -lt "$NEW_COUNT" ]; then
            echo "Action: ➕ Added reaction(s)"
        elif [ "$OLD_COUNT" -gt "$NEW_COUNT" ]; then
            echo "Action: ➖ Removed reaction(s)"
        else
            echo "Action: 🔄 Changed reaction(s)"
        fi
        
        # Show which specific reactions changed
        if [ -n "$OLD_REACTIONS" ] && [ -n "$NEW_REACTIONS" ]; then
            echo "Change: $OLD_REACTIONS → $NEW_REACTIONS"
        elif [ -n "$NEW_REACTIONS" ]; then
            echo "Added: $NEW_REACTIONS"
        elif [ -n "$OLD_REACTIONS" ]; then
            echo "Removed: $OLD_REACTIONS"
        fi
        
        echo "---"
        echo ""
    fi
}

# Monitor the incoming topic
rpk topic consume "$KAFKA_IN_TOPIC" --brokers "$KAFKA_BROKER" --group "reaction-monitor-$(date +%s)" --format json | while read -r line; do
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
echo "=== Message Reaction Monitoring Complete ==="