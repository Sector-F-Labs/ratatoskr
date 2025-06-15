#!/bin/bash

# Source the environment setup script
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/setup_env.sh" || {
  echo "Error: Failed to source setup_env.sh"
  exit 1
}

# Additional Kafka settings for this script
KAFKA_OUT_TOPIC=${KAFKA_OUT_TOPIC:-"com.sectorflabs.ratatoskr.out"}

echo "Testing auto-organized button functionality..."
echo "============================================="
echo "NOTE: These examples show how buttons WOULD be organized with our new auto-organization logic"
echo "Each test shows the manual organization that matches our 26-character limit algorithm"

# Test 1: Short buttons that should fit on one row
echo -e "\n1. Testing short buttons (should fit on one row):"
echo "Algorithm: A(1) + B(1) + C(1) + D(1) = 4 chars ≤ 26 → All on one row"
TMP_FILE1=$(mktemp)
cat > "$TMP_FILE1" << EOF
{
  "trace_id": "$(uuidgen | tr '[:upper:]' '[:lower:]')",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Test 1: Short buttons (A, B, C, D) - Auto-organized into ONE row",
      "buttons": [
        [
          {"text": "A", "callback_data": "a"},
          {"text": "B", "callback_data": "b"},
          {"text": "C", "callback_data": "c"},
          {"text": "D", "callback_data": "d"}
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

echo "$CHAT_ID:$(cat "$TMP_FILE1" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"
rm "$TMP_FILE1"
sleep 2

# Test 2: Mixed length buttons that should split into multiple rows
echo -e "\n2. Testing mixed length buttons (should split into rows):"
echo "Algorithm: Short(5) + Medium Length(13) = 18 ≤ 26 → Row 1"
echo "          Very Long Button Text(21) + X(1) = 22 ≤ 26 → Row 2"
TMP_FILE2=$(mktemp)
cat > "$TMP_FILE2" << EOF
{
  "trace_id": "$(uuidgen | tr '[:upper:]' '[:lower:]')",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Test 2: Mixed buttons - Auto-organized into TWO rows based on 26-char limit",
      "buttons": [
        [
          {"text": "Short", "callback_data": "short"},
          {"text": "Medium Length", "callback_data": "medium"}
        ],
        [
          {"text": "Very Long Button Text", "callback_data": "long"},
          {"text": "X", "callback_data": "x"}
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

echo "$CHAT_ID:$(cat "$TMP_FILE2" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"
rm "$TMP_FILE2"
sleep 2

# Test 3: Single very long button
echo -e "\n3. Testing single very long button:"
echo "Algorithm: Button exceeds 26 chars → Gets its own row"
TMP_FILE3=$(mktemp)
cat > "$TMP_FILE3" << EOF
{
  "trace_id": "$(uuidgen | tr '[:upper:]' '[:lower:]')",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Test 3: Single very long button - Auto-organized on its own row",
      "buttons": [
        [
          {"text": "This is an extremely long button text that exceeds the 26 character limit", "callback_data": "very_long"}
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

echo "$CHAT_ID:$(cat "$TMP_FILE3" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"
rm "$TMP_FILE3"
sleep 2

# Test 4: Many small buttons
echo -e "\n4. Testing many small buttons (should create multiple rows):"
echo "Algorithm: Pack as many single/double digit numbers per row within 26 chars"
TMP_FILE4=$(mktemp)
cat > "$TMP_FILE4" << EOF
{
  "trace_id": "$(uuidgen | tr '[:upper:]' '[:lower:]')",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Test 4: Numbers 1-12 - Auto-organized efficiently across multiple rows",
      "buttons": [
        [
          {"text": "1", "callback_data": "1"},
          {"text": "2", "callback_data": "2"},
          {"text": "3", "callback_data": "3"},
          {"text": "4", "callback_data": "4"},
          {"text": "5", "callback_data": "5"},
          {"text": "6", "callback_data": "6"},
          {"text": "7", "callback_data": "7"},
          {"text": "8", "callback_data": "8"},
          {"text": "9", "callback_data": "9"}
        ],
        [
          {"text": "10", "callback_data": "10"},
          {"text": "11", "callback_data": "11"},
          {"text": "12", "callback_data": "12"}
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

echo "$CHAT_ID:$(cat "$TMP_FILE4" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"
rm "$TMP_FILE4"
sleep 2

# Test 5: Edge case - exactly at the limit
echo -e "\n5. Testing buttons exactly at the limit:"
echo "Algorithm: Exactly26CharactersHere(23) + OneMore(7) = 30 > 26 → Split into 2 rows"
TMP_FILE5=$(mktemp)
cat > "$TMP_FILE5" << EOF
{
  "trace_id": "$(uuidgen | tr '[:upper:]' '[:lower:]')",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Test 5: Edge case - buttons split when combined length exceeds 26 chars",
      "buttons": [
        [
          {"text": "Exactly26CharactersHere", "callback_data": "exact26"}
        ],
        [
          {"text": "OneMore", "callback_data": "onemore"}
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

echo "$CHAT_ID:$(cat "$TMP_FILE5" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"
rm "$TMP_FILE5"

echo -e "\n============================================="
echo "Auto-button organization demonstration completed!"
echo "Check your Telegram chat to see the organized buttons."
echo ""
echo "What you're seeing:"
echo "- Each test shows how our auto-organization algorithm WOULD arrange buttons"
echo "- The manual organization in these messages matches our 26-character limit logic"
echo "- Test 1: All short buttons fit on one row (4 chars total)"
echo "- Test 2: Mixed lengths split appropriately (18 chars row 1, 22 chars row 2)"
echo "- Test 3: Long button gets its own row (exceeds 26 chars)"
echo "- Test 4: Small buttons packed efficiently across rows"
echo "- Test 5: Buttons split when combined length > 26 chars"
echo ""
echo "To actually USE the auto-organization functions:"
echo "- ButtonInfo::create_inline_keyboard(flat_buttons) → organized_rows"
echo "- OutgoingMessage::text_with_auto_buttons(text, flat_buttons, target)"
echo "- Unit tests in src/kafka_processing/outgoing.rs verify the logic works"
