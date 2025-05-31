#!/bin/bash

# Source the environment setup script
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"
source "$SCRIPT_DIR/setup_env.sh" || {
  echo "Error: Failed to source setup_env.sh"
  exit 1
}

# Set default values
TOPIC=${1:-"$KAFKA_OUT_TOPIC"}
NUM_MESSAGES=${2:-""}

# Function to show usage
show_usage() {
    echo "Usage: $0 [TOPIC] [NUM_MESSAGES]"
    echo ""
    echo "Arguments:"
    echo "  TOPIC        Kafka topic to consume from (default: \$KAFKA_OUT_TOPIC)"
    echo "  NUM_MESSAGES Number of messages to consume (default: consume indefinitely)"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Consume from output topic indefinitely"
    echo "  $0 com.sectorflabs.ratatoskr.in     # Consume from input topic"
    echo "  $0 com.sectorflabs.ratatoskr.out 10 # Consume 10 messages from output topic"
    echo ""
    echo "Environment variables:"
    echo "  KAFKA_BROKER     - Kafka broker address (default: localhost:9092)"
    echo "  KAFKA_IN_TOPIC   - Input topic name"
    echo "  KAFKA_OUT_TOPIC  - Output topic name"
}

# Check for help flag
if [[ "$1" == "-h" || "$1" == "--help" ]]; then
    show_usage
    exit 0
fi

echo "Consuming messages from topic: $TOPIC"
echo "Kafka broker: $KAFKA_BROKER"

if [ -n "$NUM_MESSAGES" ]; then
    echo "Number of messages: $NUM_MESSAGES"
    kafka-console-consumer --bootstrap-server "$KAFKA_BROKER" \
                          --topic "$TOPIC" \
                          --from-beginning \
                          --max-messages "$NUM_MESSAGES" \
                          --property print.key=true \
                          --property key.separator=":" \
                          --formatter kafka.tools.DefaultMessageFormatter \
                          --property print.timestamp=true
else
    echo "Consuming indefinitely (press Ctrl+C to stop)"
    kafka-console-consumer --bootstrap-server "$KAFKA_BROKER" \
                          --topic "$TOPIC" \
                          --from-beginning \
                          --property print.key=true \
                          --property key.separator=":" \
                          --formatter kafka.tools.DefaultMessageFormatter \
                          --property print.timestamp=true
fi