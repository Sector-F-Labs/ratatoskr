#!/bin/bash

# Direct produce script that uses hardcoded values - for testing
KAFKA_BROKER=${KAFKA_BROKER:-"localhost:9092"}
KAFKA_OUT_TOPIC=${KAFKA_OUT_TOPIC:-"com.sectorflabs.ratatoskr.out"}
CHAT_ID=${CHAT_ID:-70661797}

# Create a simple JSON message with a static format
JSON_MESSAGE='{"chat_id":'$CHAT_ID',"text":"This is a test message from test_direct_produce.sh"}'

echo "Sending direct message to $KAFKA_OUT_TOPIC:"
echo "$JSON_MESSAGE" | jq
# Send the message with chat_id as key for proper partitioning
# Compact the JSON to a single line to avoid rpk treating each line as separate message
echo "$JSON_MESSAGE" | jq -c . | rpk topic produce $KAFKA_OUT_TOPIC --brokers $KAFKA_BROKER -k "$CHAT_ID"

echo "Message sent!"