#!/bin/bash

# Source the environment setup script
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/setup_env.sh" || {
  echo "Error: Failed to source setup_env.sh"
  exit 1
}

# Additional Kafka settings for this script
KAFKA_OUT_TOPIC=${KAFKA_OUT_TOPIC:-"com.sectorflabs.ratatoskr.out"}

echo "Testing auto-organization with cafe buttons (real scenario)..."
echo "=============================================="

# Create a temporary file for the message
TMP_FILE=$(mktemp)

# Write the cafe message with buttons that need auto-organization
cat > "$TMP_FILE" << EOF
{
  "trace_id": "$(uuidgen | tr '[:upper:]' '[:lower:]')",
  "message_type": {
    "type": "TextMessage",
    "data": {
      "text": "Here are some cafes you might be interested in:\n\n1. *Starfish & Coffee* - Galoppvägen, 187 54 Täby, Sweden\n2. *Bageri & Konditori Kardemumma Täby Kyrkby* - Margaretavägen 3, 187 74 Täby, Sweden\n3. *Viggbyholms Stationskafé* - Södervägen 5, 183 69 Täby, Sweden\n4. *Cafe Startboxen* - Boulevarden, 183 74 Täby, Sweden\n5. *Manufactura* - Täby Torg 4, 183 34 Täby, Sweden\n\nThese buttons should be auto-organized based on text length!",
      "buttons": [
        [
          {
            "text": "1. Starfish & coffee",
            "callback_data": "callback:PlaceSearchCapability:select:70661797:0"
          },
          {
            "text": "2. Bageri & Konditori Kardemumma Täby Kyrkby",
            "callback_data": "callback:PlaceSearchCapability:select:70661797:1"
          },
          {
            "text": "3. Viggbyholms Stationskafé",
            "callback_data": "callback:PlaceSearchCapability:select:70661797:2"
          },
          {
            "text": "4. Cafe Startboxen",
            "callback_data": "callback:PlaceSearchCapability:select:70661797:3"
          },
          {
            "text": "5. Manufactura",
            "callback_data": "callback:PlaceSearchCapability:select:70661797:4"
          }
        ]
      ],
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
echo "Sending cafe buttons that should be auto-organized:"
echo ""
echo "Original single row with buttons:"
echo "- '1. Starfish & coffee' (20 chars)"
echo "- '2. Bageri & Konditori Kardemumma Täby Kyrkby' (45 chars)"
echo "- '3. Viggbyholms Stationskafé' (28 chars)"
echo "- '4. Cafe Startboxen' (18 chars)"
echo "- '5. Manufactura' (14 chars)"
echo ""
echo "Expected auto-organization (26 char limit):"
echo "Row 1: '1. Starfish & coffee' (20 chars) - fits alone"
echo "Row 2: '2. Bageri & Konditori Kardemumma Täby Kyrkby' (45 chars) - exceeds limit, gets own row"
echo "Row 3: '3. Viggbyholms Stationskafé' (28 chars) - exceeds limit, gets own row"
echo "Row 4: '4. Cafe Startboxen' (18 chars) - fits alone or could combine with next"
echo "Row 5: '5. Manufactura' (14 chars) - fits"
echo ""

# Send the message to Kafka
echo "Producing to $KAFKA_OUT_TOPIC..."
echo "$CHAT_ID:$(cat "$TMP_FILE" | jq -c .)" | kafka-console-producer --bootstrap-server "$KAFKA_BROKER" --topic "$KAFKA_OUT_TOPIC" --property "key.separator=:" --property "parse.key=true"

# Clean up
rm "$TMP_FILE"

echo ""
echo "=============================================="
echo "Cafe button auto-organization test sent!"
echo "Check your Telegram chat to see if the buttons are organized better."
echo ""
echo "If auto-organization is working, you should see:"
echo "- Buttons split across multiple rows instead of all on one row"
echo "- Long button names get their own rows"
echo "- Short button names might be combined where they fit"
