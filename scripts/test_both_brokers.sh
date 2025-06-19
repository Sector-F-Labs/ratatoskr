#!/bin/bash

# Comprehensive test script for both Kafka and MQTT brokers
# Tests the same message types against both brokers to ensure compatibility
# Usage: ./scripts/test_both_brokers.sh

set -e

# Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Test configuration
TEST_TEXT_MESSAGE="Test message from both brokers script"
TEST_BUTTON_MESSAGE="Test buttons from both brokers script"
SCRIPT_DIR="$(dirname "$(readlink -f "$0")")"

# Function to print section headers
print_section() {
    echo -e "\n${CYAN}========================================${NC}"
    echo -e "${CYAN}$1${NC}"
    echo -e "${CYAN}========================================${NC}"
}

# Function to print test headers
print_test() {
    echo -e "\n${BLUE}--- $1 ---${NC}"
}

# Function to print success message
print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# Function to print error message
print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Function to print warning message
print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Function to test broker connectivity
test_broker_connectivity() {
    local broker_type=$1

    print_test "Testing $broker_type connectivity"

    if BROKER_TYPE=$broker_type "$SCRIPT_DIR/setup_env_unified.sh" &>/dev/null; then
        print_success "$broker_type broker is reachable"
        return 0
    else
        print_error "$broker_type broker is not reachable"
        return 1
    fi
}

# Function to send test message
send_test_message() {
    local broker_type=$1
    local script_name=$2
    local message_text=$3

    print_test "Sending $script_name via $broker_type"

    if BROKER_TYPE=$broker_type "$SCRIPT_DIR/$script_name" "$message_text" &>/dev/null; then
        print_success "Message sent successfully via $broker_type"
        return 0
    else
        print_error "Failed to send message via $broker_type"
        return 1
    fi
}

# Function to run comprehensive test suite
run_test_suite() {
    local broker_type=$1
    local failures=0

    print_section "Testing $broker_type Broker"

    # Test connectivity first
    if ! test_broker_connectivity "$broker_type"; then
        print_error "Skipping $broker_type tests due to connectivity issues"
        return 1
    fi

    # Test basic text message
    if ! send_test_message "$broker_type" "produce_unified.sh" "$TEST_TEXT_MESSAGE"; then
        ((failures++))
    fi

    # Test message with buttons
    if ! send_test_message "$broker_type" "produce_with_buttons_unified.sh" "$TEST_BUTTON_MESSAGE"; then
        ((failures++))
    fi

    # Test typing indicator
    print_test "Sending typing indicator via $broker_type"
    if BROKER_TYPE=$broker_type "$SCRIPT_DIR/produce_typing_unified.sh" &>/dev/null; then
        print_success "Typing indicator sent successfully via $broker_type"
    else
        print_error "Failed to send typing indicator via $broker_type"
        ((failures++))
    fi

    # Summary for this broker
    if [ $failures -eq 0 ]; then
        print_success "All tests passed for $broker_type broker"
    else
        print_error "$failures test(s) failed for $broker_type broker"
    fi

    return $failures
}

# Function to check prerequisites
check_prerequisites() {
    print_section "Checking Prerequisites"

    local missing_tools=0

    # Check for required tools
    if ! command -v jq &>/dev/null; then
        print_error "jq is required but not installed"
        ((missing_tools++))
    else
        print_success "jq is available"
    fi

    if ! command -v uuidgen &>/dev/null; then
        print_error "uuidgen is required but not installed"
        ((missing_tools++))
    else
        print_success "uuidgen is available"
    fi

    # Check Kafka tools
    if command -v kafka-console-producer &>/dev/null || command -v kafka-console-producer.sh &>/dev/null; then
        print_success "Kafka tools are available"
    else
        print_warning "Kafka tools not found - Kafka tests will be skipped"
        ((missing_tools++))
    fi

    # Check MQTT tools
    if command -v mosquitto_pub &>/dev/null && command -v mosquitto_sub &>/dev/null; then
        print_success "MQTT tools are available"
    else
        print_warning "MQTT tools not found - MQTT tests will be skipped"
        ((missing_tools++))
    fi

    # Check CHAT_ID
    if [ -z "$CHAT_ID" ]; then
        print_error "CHAT_ID environment variable is not set"
        echo "Please set your Telegram chat ID: export CHAT_ID=123456789"
        ((missing_tools++))
    else
        print_success "CHAT_ID is set: $CHAT_ID"
    fi

    if [ $missing_tools -gt 0 ]; then
        print_error "Some prerequisites are missing. Please install required tools."
        exit 1
    fi

    print_success "All prerequisites are met"
}

# Function to show test results summary
show_summary() {
    local kafka_result=$1
    local mqtt_result=$2

    print_section "Test Results Summary"

    echo -e "Kafka Tests: $([ $kafka_result -eq 0 ] && echo -e "${GREEN}PASSED${NC}" || echo -e "${RED}FAILED${NC}")"
    echo -e "MQTT Tests:  $([ $mqtt_result -eq 0 ] && echo -e "${GREEN}PASSED${NC}" || echo -e "${RED}FAILED${NC}")"

    if [ $kafka_result -eq 0 ] && [ $mqtt_result -eq 0 ]; then
        print_success "All tests passed! Both brokers are working correctly."
        echo ""
        echo "Next steps:"
        echo "  - Start consuming messages: make consume BROKER_TYPE=kafka"
        echo "  - Or with MQTT: make consume BROKER_TYPE=mqtt"
        echo "  - Run your Ratatoskr bot to see the messages delivered to Telegram"
        return 0
    else
        print_error "Some tests failed. Check the output above for details."
        echo ""
        echo "Troubleshooting:"
        if [ $kafka_result -ne 0 ]; then
            echo "  - Kafka: Check if Kafka is running (make setup_kafka)"
        fi
        if [ $mqtt_result -ne 0 ]; then
            echo "  - MQTT: Check if MQTT broker is running (make setup_mqtt)"
        fi
        echo "  - Verify CHAT_ID is correctly set"
        echo "  - Check broker connectivity manually"
        return 1
    fi
}

# Function to demonstrate usage examples
show_usage_examples() {
    print_section "Usage Examples"

    echo "After running this test, you can use these commands:"
    echo ""
    echo "Send messages via Kafka:"
    echo "  make test_text BROKER_TYPE=kafka TEXT=\"Hello via Kafka\""
    echo "  make test_buttons BROKER_TYPE=kafka TEXT=\"Buttons via Kafka\""
    echo ""
    echo "Send messages via MQTT:"
    echo "  make test_text BROKER_TYPE=mqtt TEXT=\"Hello via MQTT\""
    echo "  make test_buttons BROKER_TYPE=mqtt TEXT=\"Buttons via MQTT\""
    echo ""
    echo "Monitor messages:"
    echo "  make consume BROKER_TYPE=kafka"
    echo "  make consume BROKER_TYPE=mqtt"
    echo ""
    echo "Test both brokers quickly:"
    echo "  make test_both_brokers"
}

# Main execution
main() {
    print_section "Ratatoskr Dual Broker Test Suite"
    echo "Testing both Kafka and MQTT message delivery..."

    # Check prerequisites
    check_prerequisites

    # Initialize results
    kafka_result=1
    mqtt_result=1

    # Test Kafka
    if command -v kafka-console-producer &>/dev/null || command -v kafka-console-producer.sh &>/dev/null; then
        run_test_suite "kafka"
        kafka_result=$?
    else
        print_warning "Skipping Kafka tests - tools not available"
        kafka_result=0  # Don't fail if tools aren't installed
    fi

    # Test MQTT
    if command -v mosquitto_pub &>/dev/null; then
        run_test_suite "mqtt"
        mqtt_result=$?
    else
        print_warning "Skipping MQTT tests - tools not available"
        mqtt_result=0  # Don't fail if tools aren't installed
    fi

    # Show summary
    show_summary $kafka_result $mqtt_result
    result=$?

    # Show usage examples
    show_usage_examples

    exit $result
}

# Handle script interruption
trap 'echo -e "\n${YELLOW}Test interrupted by user${NC}"; exit 130' INT

# Run main function
main "$@"
