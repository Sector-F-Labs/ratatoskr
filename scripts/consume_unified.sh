#!/bin/bash

# Unified consumer script for Ratatoskr
# Supports both Kafka and MQTT brokers based on BROKER_TYPE environment variable
# Usage: ./scripts/consume_unified.sh [TOPIC] [MAX_MESSAGES]

set -e

# Source the unified environment setup script
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/setup_env_unified.sh" || {
  echo "Error: Failed to source setup_env_unified.sh"
  exit 1
}

# Set topic and message limit
TOPIC=${1:-"$IN_TOPIC"}
MAX_MESSAGES=${2:-""}

echo "Starting consumer for $BROKER_TYPE broker..."
echo "Topic: $TOPIC"
echo "Broker: $BROKER_HOST"

if [ -n "$MAX_MESSAGES" ]; then
    echo "Max messages: $MAX_MESSAGES"
fi

echo "Press Ctrl+C to stop..."
echo ""

# Start consuming based on broker type
case "$BROKER_TYPE" in
    "kafka")
        echo "=== Consuming from Kafka topic: $TOPIC ==="

        KAFKA_CONSUMER_ARGS="--bootstrap-server $KAFKA_BROKER --topic $TOPIC --from-beginning"

        if [ -n "$MAX_MESSAGES" ]; then
            KAFKA_CONSUMER_ARGS="$KAFKA_CONSUMER_ARGS --max-messages $MAX_MESSAGES"
        fi

        # Use kafka console consumer with pretty printing
        $KAFKA_CONSOLE_CONSUMER_CMD $KAFKA_CONSUMER_ARGS --property print.key=true --property key.separator=: | while IFS=: read -r key message; do
            echo "----------------------------------------"
            echo "Key: $key"
            echo "Timestamp: $(date)"
            echo "Message:"
            echo "$message" | jq . 2>/dev/null || echo "$message"
            echo ""
        done
        ;;

    "mqtt")
        echo "=== Consuming from MQTT topic: $TOPIC ==="

        MQTT_SUB_ARGS="-h $MQTT_HOST -p $MQTT_PORT -t $TOPIC"

        if [ -n "$MAX_MESSAGES" ]; then
            MQTT_SUB_ARGS="$MQTT_SUB_ARGS -C $MAX_MESSAGES"
        fi

        # Use mosquitto subscriber with pretty printing
        $MQTT_SUB_CMD $MQTT_SUB_ARGS | while read -r message; do
            echo "----------------------------------------"
            echo "Timestamp: $(date)"
            echo "Topic: $TOPIC"
            echo "Message:"
            echo "$message" | jq . 2>/dev/null || echo "$message"
            echo ""
        done
        ;;

    *)
        echo "Error: Unsupported broker type: $BROKER_TYPE"
        exit 1
        ;;
esac

echo "Consumer stopped."
