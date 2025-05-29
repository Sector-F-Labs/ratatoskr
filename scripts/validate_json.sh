#!/bin/bash

# Script to validate JSON messages against Ratatoskr's expected format
# This helps ensure test scripts produce valid messages

# Source the environment setup script
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/setup_env.sh" || {
  echo "Error: Failed to source setup_env.sh"
  exit 1
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [JSON_FILE|JSON_STRING] [TYPE]"
    echo ""
    echo "Arguments:"
    echo "  JSON_FILE/STRING  JSON file path or JSON string to validate"
    echo "  TYPE             Message type: 'incoming' or 'outgoing' (default: outgoing)"
    echo ""
    echo "Examples:"
    echo "  $0 message.json outgoing"
    echo "  $0 '{\"message_type\":{\"type\":\"TextMessage\"...}}' outgoing"
    echo "  $0 --test-outgoing  # Test with sample outgoing message"
    echo "  $0 --test-incoming  # Test with sample incoming message"
}

# Check for help flag
if [[ "$1" == "-h" || "$1" == "--help" ]]; then
    show_usage
    exit 0
fi

# Handle test flags
if [[ "$1" == "--test-outgoing" ]]; then
    JSON_INPUT='{"message_type":{"type":"TextMessage","data":{"text":"Test message","buttons":null,"parse_mode":null,"disable_web_page_preview":null}},"timestamp":"2023-12-01T10:30:00Z","target":{"platform":"telegram","chat_id":123456789,"thread_id":null}}'
    MESSAGE_TYPE="outgoing"
elif [[ "$1" == "--test-incoming" ]]; then
    JSON_INPUT='{"message_type":{"type":"CallbackQuery","data":{"chat_id":123456789,"user_id":987654321,"message_id":42,"callback_data":"test_action","callback_query_id":"123456789"}},"timestamp":"2023-12-01T10:30:00Z","source":{"platform":"telegram","bot_id":null,"bot_username":null}}'
    MESSAGE_TYPE="incoming"
else
    JSON_INPUT="$1"
    MESSAGE_TYPE="${2:-outgoing}"
fi

# Validate arguments
if [ -z "$JSON_INPUT" ]; then
    echo "Error: No JSON input provided"
    show_usage
    exit 1
fi

# Check if input is a file or string
if [ -f "$JSON_INPUT" ]; then
    echo "Validating JSON file: $JSON_INPUT"
    JSON_CONTENT=$(cat "$JSON_INPUT")
else
    echo "Validating JSON string"
    JSON_CONTENT="$JSON_INPUT"
fi

echo "Message type: $MESSAGE_TYPE"
echo ""

# Basic JSON syntax validation
echo "1. Checking JSON syntax..."
echo "$JSON_CONTENT" | jq . > /dev/null 2>&1
if [ $? -eq 0 ]; then
    echo "   ✅ Valid JSON syntax"
else
    echo "   ❌ Invalid JSON syntax"
    exit 1
fi

# Structure validation based on type
echo "2. Checking message structure..."

if [ "$MESSAGE_TYPE" == "outgoing" ]; then
    # Validate outgoing message structure
    HAS_MESSAGE_TYPE=$(echo "$JSON_CONTENT" | jq -r 'has("message_type")' 2>/dev/null)
    HAS_TIMESTAMP=$(echo "$JSON_CONTENT" | jq -r 'has("timestamp")' 2>/dev/null)
    HAS_TARGET=$(echo "$JSON_CONTENT" | jq -r 'has("target")' 2>/dev/null)
    
    if [ "$HAS_MESSAGE_TYPE" == "true" ] && [ "$HAS_TIMESTAMP" == "true" ] && [ "$HAS_TARGET" == "true" ]; then
        echo "   ✅ Has required fields: message_type, timestamp, target"
    else
        echo "   ❌ Missing required fields for outgoing message"
        echo "      Required: message_type, timestamp, target"
        exit 1
    fi
    
    # Check target structure
    TARGET_PLATFORM=$(echo "$JSON_CONTENT" | jq -r '.target.platform' 2>/dev/null)
    TARGET_CHAT_ID=$(echo "$JSON_CONTENT" | jq -r '.target.chat_id' 2>/dev/null)
    
    if [ "$TARGET_PLATFORM" != "null" ] && [ "$TARGET_CHAT_ID" != "null" ]; then
        echo "   ✅ Target has platform and chat_id"
    else
        echo "   ❌ Target missing platform or chat_id"
        exit 1
    fi
    
    # Check message_type structure
    MSG_TYPE=$(echo "$JSON_CONTENT" | jq -r '.message_type.type' 2>/dev/null)
    MSG_DATA=$(echo "$JSON_CONTENT" | jq -r '.message_type.data' 2>/dev/null)
    
    if [ "$MSG_TYPE" != "null" ] && [ "$MSG_DATA" != "null" ]; then
        echo "   ✅ Message type has type and data fields"
        echo "      Message type: $MSG_TYPE"
    else
        echo "   ❌ Message type missing type or data fields"
        exit 1
    fi

elif [ "$MESSAGE_TYPE" == "incoming" ]; then
    # Validate incoming message structure
    HAS_MESSAGE_TYPE=$(echo "$JSON_CONTENT" | jq -r 'has("message_type")' 2>/dev/null)
    HAS_TIMESTAMP=$(echo "$JSON_CONTENT" | jq -r 'has("timestamp")' 2>/dev/null)
    HAS_SOURCE=$(echo "$JSON_CONTENT" | jq -r 'has("source")' 2>/dev/null)
    
    if [ "$HAS_MESSAGE_TYPE" == "true" ] && [ "$HAS_TIMESTAMP" == "true" ] && [ "$HAS_SOURCE" == "true" ]; then
        echo "   ✅ Has required fields: message_type, timestamp, source"
    else
        echo "   ❌ Missing required fields for incoming message"
        echo "      Required: message_type, timestamp, source"
        exit 1
    fi
    
    # Check source structure
    SOURCE_PLATFORM=$(echo "$JSON_CONTENT" | jq -r '.source.platform' 2>/dev/null)
    
    if [ "$SOURCE_PLATFORM" != "null" ]; then
        echo "   ✅ Source has platform field"
    else
        echo "   ❌ Source missing platform field"
        exit 1
    fi
    
    # Check message_type structure
    MSG_TYPE=$(echo "$JSON_CONTENT" | jq -r '.message_type.type' 2>/dev/null)
    MSG_DATA=$(echo "$JSON_CONTENT" | jq -r '.message_type.data' 2>/dev/null)
    
    if [ "$MSG_TYPE" != "null" ] && [ "$MSG_DATA" != "null" ]; then
        echo "   ✅ Message type has type and data fields"
        echo "      Message type: $MSG_TYPE"
    else
        echo "   ❌ Message type missing type or data fields"
        exit 1
    fi
else
    echo "   ❌ Invalid message type: $MESSAGE_TYPE (must be 'incoming' or 'outgoing')"
    exit 1
fi

echo ""
echo "3. Formatted JSON:"
echo "$JSON_CONTENT" | jq .

echo ""
echo "✅ JSON validation passed!"