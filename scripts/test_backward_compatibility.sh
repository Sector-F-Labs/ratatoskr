#!/bin/bash

# Source the environment setup script
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/setup_env.sh" || {
  echo "Error: Failed to source setup_env.sh"
  exit 1
}

# Additional Kafka settings for this script
KAFKA_OUT_TOPIC=${KAFKA_OUT_TOPIC:-"com.sectorflabs.ratatoskr.out"}

echo "Testing backward compatibility for trace IDs..."
echo "KAFKA_BROKER: $KAFKA_BROKER"
echo "KAFKA_OUT_TOPIC: $KAFKA_OUT_TOPIC"
echo "CHAT_ID: $CHAT_ID"
echo ""

# Test 1: Message WITH trace_id (current format)
echo "=== Test 1: Message with trace_id ==="
TMP_FILE_WITH_TRACE=$(mktemp)
cat > "$TMP_FILE_WITH_TRACE" << EOF
{
  "trace_id": "$(uuidgen | tr '[:upper:]' '[:lower:]')",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "✅ This message HAS a trace_id field",
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

echo "Sending message WITH trace_id:"
cat "$TMP_FILE_WITH_TRACE" | jq .
echo ""

echo "$CHAT_ID:$(cat "$TMP_FILE_WITH_TRACE" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"

if [ $? -eq 0 ]; then
    echo "✅ Message with trace_id sent successfully"
else
    echo "❌ Failed to send message with trace_id"
fi
echo ""

# Clean up first temp file
rm "$TMP_FILE_WITH_TRACE"

# Wait a moment between messages
sleep 2

# Test 2: Message WITHOUT trace_id (legacy format)
echo "=== Test 2: Message without trace_id (legacy) ==="
TMP_FILE_WITHOUT_TRACE=$(mktemp)
cat > "$TMP_FILE_WITHOUT_TRACE" << EOF
{
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "🔄 This message LACKS a trace_id field (legacy format)",
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

echo "Sending message WITHOUT trace_id (should auto-generate one):"
cat "$TMP_FILE_WITHOUT_TRACE" | jq .
echo ""

echo "$CHAT_ID:$(cat "$TMP_FILE_WITHOUT_TRACE" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"

if [ $? -eq 0 ]; then
    echo "✅ Message without trace_id sent successfully"
else
    echo "❌ Failed to send message without trace_id"
fi
echo ""

# Clean up second temp file
rm "$TMP_FILE_WITHOUT_TRACE"

# Wait a moment between messages
sleep 2

# Test 3: Message with buttons and no trace_id
echo "=== Test 3: Complex message without trace_id ==="
TMP_FILE_COMPLEX=$(mktemp)
cat > "$TMP_FILE_COMPLEX" << EOF
{
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "🎛️ Complex legacy message with buttons but no trace_id",
      "buttons": [
        [
          {"text": "Legacy Button 1", "callback_data": "legacy_1"},
          {"text": "Legacy Button 2", "callback_data": "legacy_2"}
        ],
        [
          {"text": "Legacy Button 3", "callback_data": "legacy_3"}
        ]
      ],
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

echo "Sending complex message WITHOUT trace_id:"
cat "$TMP_FILE_COMPLEX" | jq .
echo ""

echo "$CHAT_ID:$(cat "$TMP_FILE_COMPLEX" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"

if [ $? -eq 0 ]; then
    echo "✅ Complex message without trace_id sent successfully"
else
    echo "❌ Failed to send complex message without trace_id"
fi
echo ""

# Clean up third temp file
rm "$TMP_FILE_COMPLEX"

echo "=== Backward Compatibility Test Complete ==="
echo ""
echo "Summary:"
echo "  ✅ Messages with trace_id should work normally"
echo "  🔄 Messages without trace_id should auto-generate one"
echo "  🎛️ Complex messages without trace_id should work"
echo ""
echo "Check your Telegram chat to verify all messages were delivered."
echo "Check the bot logs to see trace_id generation for legacy messages."
