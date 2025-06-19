#!/bin/bash

# Unified environment setup for Ratatoskr scripts
# Supports both Kafka and MQTT brokers based on BROKER_TYPE environment variable
# This script can be sourced by other scripts to verify environment

# Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Set default broker type
BROKER_TYPE=${BROKER_TYPE:-"kafka"}

# Function to check if a command exists
command_exists() {
    command -v "$1" &> /dev/null
}

# Function to check Kafka tools
check_kafka_tools() {
    if ! command_exists kafka-topics && ! command_exists kafka-topics.sh; then
        echo -e "${RED}Error: Kafka command-line tools not found${NC}"
        echo "Please install Kafka and ensure kafka-topics.sh is in your PATH"
        echo "You can install with: brew install kafka"
        echo "Or download from: https://kafka.apache.org/downloads"
        return 1
    fi

    # Determine which kafka command to use
    if command_exists kafka-topics.sh; then
        export KAFKA_TOPICS_CMD="kafka-topics.sh"
        export KAFKA_CONSOLE_PRODUCER_CMD="kafka-console-producer.sh"
        export KAFKA_CONSOLE_CONSUMER_CMD="kafka-console-consumer.sh"
    else
        export KAFKA_TOPICS_CMD="kafka-topics"
        export KAFKA_CONSOLE_PRODUCER_CMD="kafka-console-producer"
        export KAFKA_CONSOLE_CONSUMER_CMD="kafka-console-consumer"
    fi
    return 0
}

# Function to check MQTT tools
check_mqtt_tools() {
    if ! command_exists mosquitto_pub || ! command_exists mosquitto_sub; then
        echo -e "${RED}Error: MQTT command-line tools not found${NC}"
        echo "Please install Mosquitto client tools:"
        echo "  macOS: brew install mosquitto"
        echo "  Ubuntu/Debian: apt install mosquitto-clients"
        echo "  RHEL/CentOS: yum install mosquitto"
        return 1
    fi

    export MQTT_PUB_CMD="mosquitto_pub"
    export MQTT_SUB_CMD="mosquitto_sub"
    return 0
}

# Check for required common tools
if ! command_exists jq; then
    echo -e "${RED}Error: jq not found${NC}"
    echo "Please install it with: brew install jq (macOS) or apt install jq (Debian/Ubuntu)"
    exit 1
fi

if ! command_exists uuidgen; then
    echo -e "${RED}Error: uuidgen not found${NC}"
    echo "Please install it (usually part of util-linux package)"
    exit 1
fi

# Configure broker-specific settings
case "$BROKER_TYPE" in
    "kafka")
        echo -e "${BLUE}Configuring for Kafka broker...${NC}"

        # Check Kafka tools
        if ! check_kafka_tools; then
            exit 1
        fi

        # Set Kafka defaults
        export KAFKA_BROKER=${KAFKA_BROKER:-"localhost:9092"}
        export KAFKA_IN_TOPIC=${KAFKA_IN_TOPIC:-"com.sectorflabs.ratatoskr.in"}
        export KAFKA_OUT_TOPIC=${KAFKA_OUT_TOPIC:-"com.sectorflabs.ratatoskr.out"}

        # Set unified broker variables
        export BROKER_HOST="$KAFKA_BROKER"
        export IN_TOPIC="$KAFKA_IN_TOPIC"
        export OUT_TOPIC="$KAFKA_OUT_TOPIC"
        ;;

    "mqtt")
        echo -e "${BLUE}Configuring for MQTT broker...${NC}"

        # Check MQTT tools
        if ! check_mqtt_tools; then
            exit 1
        fi

        # Set MQTT defaults
        export MQTT_BROKER=${MQTT_BROKER:-"localhost:1883"}
        export MQTT_IN_TOPIC=${MQTT_IN_TOPIC:-"com.sectorflabs.ratatoskr.in"}
        export MQTT_OUT_TOPIC=${MQTT_OUT_TOPIC:-"com.sectorflabs.ratatoskr.out"}

        # Parse MQTT broker into host and port
        IFS=':' read -r MQTT_HOST MQTT_PORT <<< "$MQTT_BROKER"
        export MQTT_HOST=${MQTT_HOST:-"localhost"}
        export MQTT_PORT=${MQTT_PORT:-"1883"}

        # Set unified broker variables
        export BROKER_HOST="$MQTT_BROKER"
        export IN_TOPIC="$MQTT_IN_TOPIC"
        export OUT_TOPIC="$MQTT_OUT_TOPIC"
        ;;

    *)
        echo -e "${RED}Error: Unsupported BROKER_TYPE: $BROKER_TYPE${NC}"
        echo "Supported values: kafka, mqtt"
        exit 1
        ;;
esac

# Check if CHAT_ID is set
if [ -z "$CHAT_ID" ]; then
    echo -e "${YELLOW}Warning: CHAT_ID environment variable is not set${NC}"
    echo "Please set it to your Telegram chat ID, for example:"
    echo "export CHAT_ID=123456789"
    echo "This is needed for most test scripts to work properly."

    # Allow script to continue if this was just sourced (not directly executed)
    if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
        exit 1
    fi
fi

# Function to test broker connectivity
test_broker_connectivity() {
    case "$BROKER_TYPE" in
        "kafka")
            echo -e "${BLUE}Testing Kafka connectivity...${NC}"
            if $KAFKA_TOPICS_CMD --bootstrap-server "$KAFKA_BROKER" --list &>/dev/null; then
                echo -e "${GREEN}✓ Kafka broker reachable at $KAFKA_BROKER${NC}"
                return 0
            else
                echo -e "${RED}✗ Failed to connect to Kafka broker at $KAFKA_BROKER${NC}"
                return 1
            fi
            ;;

        "mqtt")
            echo -e "${BLUE}Testing MQTT connectivity...${NC}"
            # Test MQTT connection with a simple publish (will fail if broker is unreachable)
            if timeout 5s mosquitto_pub -h "$MQTT_HOST" -p "$MQTT_PORT" -t "test/connectivity" -m "test" &>/dev/null; then
                echo -e "${GREEN}✓ MQTT broker reachable at $MQTT_HOST:$MQTT_PORT${NC}"
                return 0
            else
                echo -e "${YELLOW}⚠ MQTT broker may not be reachable at $MQTT_HOST:$MQTT_PORT${NC}"
                echo -e "${YELLOW}  This might be OK if the broker requires authentication${NC}"
                return 0  # Don't fail completely for MQTT
            fi
            ;;
    esac
}

# Function to create/verify topics (Kafka only)
setup_topics() {
    if [ "$BROKER_TYPE" = "kafka" ]; then
        echo -e "\n${BLUE}Checking/Creating Kafka topics:${NC}"

        # Check if input topic exists
        if ! $KAFKA_TOPICS_CMD --bootstrap-server "$KAFKA_BROKER" --list | grep -q "^$KAFKA_IN_TOPIC$"; then
            echo -e "Creating topic: ${YELLOW}$KAFKA_IN_TOPIC${NC}"
            $KAFKA_TOPICS_CMD --bootstrap-server "$KAFKA_BROKER" --create --topic "$KAFKA_IN_TOPIC" --partitions 1 --replication-factor 1
        else
            echo -e "Topic exists: ${GREEN}$KAFKA_IN_TOPIC${NC}"
        fi

        # Check if output topic exists
        if ! $KAFKA_TOPICS_CMD --bootstrap-server "$KAFKA_BROKER" --list | grep -q "^$KAFKA_OUT_TOPIC$"; then
            echo -e "Creating topic: ${YELLOW}$KAFKA_OUT_TOPIC${NC}"
            $KAFKA_TOPICS_CMD --bootstrap-server "$KAFKA_BROKER" --create --topic "$KAFKA_OUT_TOPIC" --partitions 1 --replication-factor 1
        else
            echo -e "Topic exists: ${GREEN}$KAFKA_OUT_TOPIC${NC}"
        fi
    else
        echo -e "\n${BLUE}MQTT topics are created automatically on first publish${NC}"
        echo -e "Using topics: ${GREEN}$MQTT_IN_TOPIC${NC}, ${GREEN}$MQTT_OUT_TOPIC${NC}"
    fi
}

# Function to publish message (broker-agnostic)
publish_message() {
    local topic="$1"
    local message="$2"
    local key="$3"  # Optional, only used for Kafka

    case "$BROKER_TYPE" in
        "kafka")
            if [ -n "$key" ]; then
                echo "$key:$message" | $KAFKA_CONSOLE_PRODUCER_CMD --bootstrap-server "$KAFKA_BROKER" --topic "$topic" --property "key.separator=:" --property "parse.key=true"
            else
                echo "$message" | $KAFKA_CONSOLE_PRODUCER_CMD --bootstrap-server "$KAFKA_BROKER" --topic "$topic"
            fi
            ;;

        "mqtt")
            echo "$message" | $MQTT_PUB_CMD -h "$MQTT_HOST" -p "$MQTT_PORT" -t "$topic" -l
            ;;
    esac
}

# Print environment settings if being run directly (not sourced)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    echo -e "${BLUE}=== Ratatoskr Unified Environment ===${NC}"
    echo -e "BROKER_TYPE: ${YELLOW}$BROKER_TYPE${NC}"
    echo -e "CHAT_ID: ${YELLOW}$CHAT_ID${NC}"
    echo -e "BROKER_HOST: ${GREEN}$BROKER_HOST${NC}"
    echo -e "IN_TOPIC: ${GREEN}$IN_TOPIC${NC}"
    echo -e "OUT_TOPIC: ${GREEN}$OUT_TOPIC${NC}"

    if [ "$BROKER_TYPE" = "kafka" ]; then
        echo -e "KAFKA_BROKER: ${GREEN}$KAFKA_BROKER${NC}"
        echo -e "KAFKA_IN_TOPIC: ${GREEN}$KAFKA_IN_TOPIC${NC}"
        echo -e "KAFKA_OUT_TOPIC: ${GREEN}$KAFKA_OUT_TOPIC${NC}"
    else
        echo -e "MQTT_BROKER: ${GREEN}$MQTT_BROKER${NC}"
        echo -e "MQTT_HOST: ${GREEN}$MQTT_HOST${NC}"
        echo -e "MQTT_PORT: ${GREEN}$MQTT_PORT${NC}"
        echo -e "MQTT_IN_TOPIC: ${GREEN}$MQTT_IN_TOPIC${NC}"
        echo -e "MQTT_OUT_TOPIC: ${GREEN}$MQTT_OUT_TOPIC${NC}"
    fi

    # Test connectivity
    test_broker_connectivity

    # Setup topics
    setup_topics

    echo -e "\n${GREEN}Environment is ready for Ratatoskr scripts with $BROKER_TYPE broker${NC}"
else
    # When sourced, show which broker we're using
    echo -e "${BLUE}Using $BROKER_TYPE broker: ${GREEN}$BROKER_HOST${NC}"
fi
