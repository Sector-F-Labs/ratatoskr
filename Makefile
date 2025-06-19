# Makefile for Deno Telegram <-> Kafka Bot

.PHONY: help install setup run dev stop produce test_buttons test_image test_callback test_typing test_typing_demo install-service uninstall-service start-service stop-service status-service

# Broker configuration (supports both Kafka and MQTT)
BROKER_TYPE?=kafka

# Kafka config
KAFKA_BROKER?=localhost:9092
KAFKA_IN_TOPIC?=com.sectorflabs.ratatoskr.in
KAFKA_OUT_TOPIC?=com.sectorflabs.ratatoskr.out

# MQTT config
MQTT_BROKER?=localhost:1883
MQTT_IN_TOPIC?=com.sectorflabs.ratatoskr.in
MQTT_OUT_TOPIC?=com.sectorflabs.ratatoskr.out

# Export variables so scripts can use them
export BROKER_TYPE
export KAFKA_BROKER
export KAFKA_IN_TOPIC
export KAFKA_OUT_TOPIC
export MQTT_BROKER
export MQTT_IN_TOPIC
export MQTT_OUT_TOPIC

# Default target - show help
help:
	@echo "Ratatoskr - Telegram <-> Kafka Bot"
	@echo ""
	@echo "Build targets:"
	@echo "  main          Build the project with cargo"
	@echo ""
	@echo "Setup targets:"
	@echo "  install       Install binary with cargo"
	@echo "  setup         Start Kafka and create topics"
	@echo ""
	@echo "Service targets:"
	@echo "  install-service   Install ratatoskr as macOS LaunchAgent"
	@echo "  uninstall-service Remove ratatoskr LaunchAgent"
	@echo "  start-service     Start the ratatoskr service"
	@echo "  stop-service      Stop the ratatoskr service"
	@echo "  status-service    Check service status"
	@echo ""
	@echo "Runtime targets:"
	@echo "  run           Run the bot"
	@echo "  dev           Run the bot with auto-reload (cargo watch)"
	@echo "  stop          Stop Kafka container"
	@echo ""
	@echo "Testing targets (Unified - work with both brokers):"
	@echo "  test_text     Send text message (TEXT=\"your message\")"
	@echo "  test_buttons  Send message with buttons (TEXT=\"your message\")"
	@echo "  test_typing   Send typing indicator"
	@echo "  consume       Monitor incoming messages (N=number)"
	@echo "  test_all_message_types  Test all unified message types with current BROKER_TYPE"
	@echo "  test_comprehensive      Run comprehensive tests on both Kafka and MQTT"
	@echo "  test_both_brokers       Quick test of both brokers with same messages"
	@echo ""
	@echo "Broker-specific testing:"
	@echo "  test_*_kafka  Run test with Kafka broker (BROKER_TYPE=kafka)"
	@echo "  test_*_mqtt   Run test with MQTT broker (BROKER_TYPE=mqtt)"
	@echo "  setup_kafka   Start Kafka stack and create topics"
	@echo "  setup_mqtt    Start MQTT stack"
	@echo ""
	@echo "Legacy testing targets (Kafka only):"
	@echo "  test_all_message_types_legacy  Test all message types with legacy Kafka scripts"
	@echo "  test_auto_buttons  Test auto-organized button functionality"
	@echo "  test_cafe_buttons  Test auto-organization with real cafe buttons"
	@echo "  test_image    Send image message (IMAGE_PATH=path CAPTION=\"caption\")"
	@echo "  test_callback Simulate callback (MESSAGE_ID=123 CALLBACK_DATA=\"data\")"
	@echo "  test_keyboard Send message with reply keyboard (TEXT=\"your message\")"
	@echo "  test_location Send location request with reply keyboard"
	@echo "  test_typing_demo Send typing indicator followed by message"
	@echo "  test_markdown Send complex markdown formatting test messages"
	@echo "  test_simple_markdown Send single markdown test message (TEXT=\"message\")"
	@echo "  test_markdown_edge_cases Send MarkdownV2 edge case test messages"
	@echo "  test_markdown_fallback Test markdown fallback to plain text functionality"
	@echo "  test_backward_compatibility Test legacy messages without trace_id"
	@echo ""
	@echo "Note: Unified scripts support both Kafka and MQTT. Legacy scripts are Kafka-only."
	@echo ""
	@echo "Environment variables:"
	@echo "  BROKER_TYPE        - Broker type: kafka (default) or mqtt"
	@echo "  CHAT_ID            - Target Telegram chat ID (required for testing)"
	@echo "  KAFKA_BROKER       - Kafka broker address (default: localhost:9092)"
	@echo "  MQTT_BROKER        - MQTT broker address (default: localhost:1883)"
	@echo "  See .envrc.example for all configuration options"

main:
	cargo build

install:
	cargo install --path .

setup: setup_kafka

setup_kafka:
	./scripts/setup_env.sh

setup_mqtt:
	docker-compose -f mqtt-stack.yml up -d
	BROKER_TYPE=mqtt ./scripts/setup_env_unified.sh

run:
	cargo run
dev:
	cargo watch -x run

stop:
	pkill -f "kafka" || true

# Unified targets (work with both Kafka and MQTT based on BROKER_TYPE)
consume:
	BROKER_TYPE=$(BROKER_TYPE) ./scripts/consume_unified.sh "" $(N)

test_text:
	BROKER_TYPE=$(BROKER_TYPE) ./scripts/produce_unified.sh "$(TEXT)"

test_buttons:
	BROKER_TYPE=$(BROKER_TYPE) ./scripts/produce_with_buttons_unified.sh "$(TEXT)"

test_typing:
	BROKER_TYPE=$(BROKER_TYPE) ./scripts/produce_typing_unified.sh

# Kafka-specific targets
test_text_kafka:
	BROKER_TYPE=kafka ./scripts/produce_unified.sh "$(TEXT)"

test_buttons_kafka:
	BROKER_TYPE=kafka ./scripts/produce_with_buttons_unified.sh "$(TEXT)"

test_typing_kafka:
	BROKER_TYPE=kafka ./scripts/produce_typing_unified.sh

consume_kafka:
	BROKER_TYPE=kafka ./scripts/consume_unified.sh "" $(N)

# MQTT-specific targets
test_text_mqtt:
	BROKER_TYPE=mqtt ./scripts/produce_unified.sh "$(TEXT)"

test_buttons_mqtt:
	BROKER_TYPE=mqtt ./scripts/produce_with_buttons_unified.sh "$(TEXT)"

test_typing_mqtt:
	BROKER_TYPE=mqtt ./scripts/produce_typing_unified.sh

consume_mqtt:
	BROKER_TYPE=mqtt ./scripts/consume_unified.sh "" $(N)

# Legacy Kafka-only targets (using original scripts)
legacy_test_text:
	KAFKA_BROKER=$(KAFKA_BROKER) KAFKA_OUT_TOPIC=$(KAFKA_OUT_TOPIC) ./scripts/produce.sh "$(TEXT)"

legacy_test_buttons:
	KAFKA_BROKER=$(KAFKA_BROKER) KAFKA_OUT_TOPIC=$(KAFKA_OUT_TOPIC) ./scripts/produce_with_buttons.sh "$(TEXT)"

legacy_consume:
	KAFKA_BROKER=$(KAFKA_BROKER) KAFKA_IN_TOPIC=$(KAFKA_IN_TOPIC) KAFKA_OUT_TOPIC=$(KAFKA_OUT_TOPIC) ./scripts/consume.sh $(N)

test_auto_buttons:
	KAFKA_BROKER=$(KAFKA_BROKER) KAFKA_OUT_TOPIC=$(KAFKA_OUT_TOPIC) ./scripts/test_auto_buttons.sh

test_cafe_buttons:
	KAFKA_BROKER=$(KAFKA_BROKER) KAFKA_OUT_TOPIC=$(KAFKA_OUT_TOPIC) ./scripts/test_cafe_buttons.sh

test_markdown:
	KAFKA_BROKER=$(KAFKA_BROKER) KAFKA_OUT_TOPIC=$(KAFKA_OUT_TOPIC) ./scripts/test_markdown.sh

test_simple_markdown:
	KAFKA_BROKER=$(KAFKA_BROKER) KAFKA_OUT_TOPIC=$(KAFKA_OUT_TOPIC) ./scripts/test_simple_markdown.sh "$(TEXT)"

test_markdown_edge_cases:
	KAFKA_BROKER=$(KAFKA_BROKER) KAFKA_OUT_TOPIC=$(KAFKA_OUT_TOPIC) ./scripts/test_markdown_edge_cases.sh

test_markdown_fallback:
	KAFKA_BROKER=$(KAFKA_BROKER) KAFKA_OUT_TOPIC=$(KAFKA_OUT_TOPIC) ./scripts/test_markdown_fallback.sh

test_image:
	KAFKA_BROKER=$(KAFKA_BROKER) KAFKA_OUT_TOPIC=$(KAFKA_OUT_TOPIC) ./scripts/produce_image.sh "$(IMAGE_PATH)" "$(CAPTION)"

test_typing_demo:
	KAFKA_BROKER=$(KAFKA_BROKER) KAFKA_OUT_TOPIC=$(KAFKA_OUT_TOPIC) ./scripts/produce_typing_demo.sh

# Test all unified message types
test_all_unified: test_text test_buttons test_typing
	@echo "All unified message types tested with $(BROKER_TYPE) broker."

# Test both brokers with same message types
test_both_brokers:
	@echo "Testing with Kafka..."
	$(MAKE) test_text_kafka TEXT="Test message via Kafka"
	$(MAKE) test_buttons_kafka TEXT="Test buttons via Kafka"
	$(MAKE) test_typing_kafka
	@echo ""
	@echo "Testing with MQTT..."
	$(MAKE) test_text_mqtt TEXT="Test message via MQTT"
	$(MAKE) test_buttons_mqtt TEXT="Test buttons via MQTT"
	$(MAKE) test_typing_mqtt
	@echo ""
	@echo "Both brokers tested successfully!"

# Comprehensive test of both brokers with detailed output
test_comprehensive:
	./scripts/test_both_brokers.sh

# Test all unified message types with current broker type
test_all_message_types: test_text test_buttons test_typing
	@echo "All unified message types tested with $(BROKER_TYPE) broker."
	@echo "Note: For additional message types (image, markdown, etc.), use legacy targets or create unified versions."

# Legacy test target (Kafka only) - includes all message types
test_all_message_types_legacy: legacy_test_text legacy_test_buttons test_image legacy_typing test_markdown test_markdown_edge_cases test_markdown_fallback
	@echo "All message types tested (legacy Kafka scripts)."

# Kafka-only legacy typing target (for legacy test suite)
legacy_typing:
	KAFKA_BROKER=$(KAFKA_BROKER) KAFKA_OUT_TOPIC=$(KAFKA_OUT_TOPIC) ./scripts/produce_typing.sh

debug:
	@echo "Make variables:"
	@echo "  BROKER_TYPE=$(BROKER_TYPE)"
	@echo "  KAFKA_BROKER=$(KAFKA_BROKER)"
	@echo "  KAFKA_IN_TOPIC=$(KAFKA_IN_TOPIC)"
	@echo "  KAFKA_OUT_TOPIC=$(KAFKA_OUT_TOPIC)"
	@echo "  MQTT_BROKER=$(MQTT_BROKER)"
	@echo "  MQTT_IN_TOPIC=$(MQTT_IN_TOPIC)"
	@echo "  MQTT_OUT_TOPIC=$(MQTT_OUT_TOPIC)"
	@echo "Environment variables:"
	@echo "  BROKER_TYPE=$$BROKER_TYPE"
	@echo "  KAFKA_BROKER=$$KAFKA_BROKER"
	@echo "  KAFKA_IN_TOPIC=$$KAFKA_IN_TOPIC"
	@echo "  KAFKA_OUT_TOPIC=$$KAFKA_OUT_TOPIC"
	@echo "  MQTT_BROKER=$$MQTT_BROKER"
	@echo "  MQTT_IN_TOPIC=$$MQTT_IN_TOPIC"
	@echo "  MQTT_OUT_TOPIC=$$MQTT_OUT_TOPIC"
	@echo "  CHAT_ID=$$CHAT_ID"

test_callback:
	KAFKA_BROKER=$(KAFKA_BROKER) KAFKA_IN_TOPIC=$(KAFKA_IN_TOPIC) ./scripts/simulate_callback.sh $(MESSAGE_ID) "$(CALLBACK_DATA)"

test_keyboard:
	KAFKA_BROKER=$(KAFKA_BROKER) KAFKA_OUT_TOPIC=$(KAFKA_OUT_TOPIC) ./scripts/produce_reply_keyboard.sh "$(TEXT)"

test_location:
	KAFKA_BROKER=$(KAFKA_BROKER) KAFKA_OUT_TOPIC=$(KAFKA_OUT_TOPIC) ./scripts/produce_location_request.sh "$(TEXT)"

test_backward_compatibility:
	KAFKA_BROKER=$(KAFKA_BROKER) KAFKA_OUT_TOPIC=$(KAFKA_OUT_TOPIC) ./scripts/test_backward_compatibility.sh

# Push to remote server (configurable via .envrc or environment variables)
REMOTE_HOST?=$(shell echo $$REMOTE_HOST)
REMOTE_USER?=$(shell echo $$REMOTE_USER)
REMOTE_PATH?=$(shell echo $$REMOTE_PATH)

push:
	@if [ -z "$(REMOTE_HOST)" ]; then \
		echo "Error: REMOTE_HOST not set. Please set it in .envrc or environment."; \
		echo "Example: export REMOTE_HOST=myserver.example.com"; \
		exit 1; \
	fi
	@if [ -z "$(REMOTE_USER)" ]; then \
		echo "Error: REMOTE_USER not set. Please set it in .envrc or environment."; \
		echo "Example: export REMOTE_USER=username"; \
		exit 1; \
	fi
	@if [ -z "$(REMOTE_PATH)" ]; then \
		echo "Error: REMOTE_PATH not set. Please set it in .envrc or environment."; \
		echo "Example: export REMOTE_PATH=~/src/ratatoskr"; \
		exit 1; \
	fi
	@echo "Pushing to remote server $(REMOTE_USER)@$(REMOTE_HOST):$(REMOTE_PATH)..."
	@rsync -av --delete \
		Cargo.toml Cargo.lock \
		src scripts \
		Makefile \
		docker-compose.yml Dockerfile \
		$(REMOTE_USER)@$(REMOTE_HOST):$(REMOTE_PATH)
	@echo "Installing binary on remote server..."
	@ssh $(REMOTE_USER)@$(REMOTE_HOST) "cd $(REMOTE_PATH) && cargo install --path ."
	@echo "Detecting OS and installing appropriate service..."
	@ssh $(REMOTE_USER)@$(REMOTE_HOST) "cd $(REMOTE_PATH) && \
		if [[ \"\$$OSTYPE\" == \"darwin\"* ]]; then \
			echo 'Detected macOS, installing LaunchAgent...'; \
			./scripts/install-service-macos.sh; \
			launchctl load ~/Library/LaunchAgents/com.sectorflabs.ratatoskr.plist; \
		elif [[ \"\$$OSTYPE\" == \"linux\"* ]] || command -v systemctl >/dev/null 2>&1; then \
			echo 'Detected Linux, installing systemd service...'; \
			./scripts/install-service-linux.sh; \
			sudo systemctl start ratatoskr; \
		else \
			echo 'Unknown OS, skipping service installation'; \
		fi"
	@echo "Push and service installed. Use 'make start-service' to start it."

uninstall-service: stop-service
	@echo "Uninstalling ratatoskr LaunchAgent..."
	@rm -f ~/Library/LaunchAgents/com.sectorflabs.ratatoskr.plist
	@echo "Service uninstalled."

start-service:
	@echo "Starting ratatoskr service..."
	@launchctl load ~/Library/LaunchAgents/com.sectorflabs.ratatoskr.plist
	@echo "Service started."

stop-service:
	@echo "Stopping ratatoskr service..."
	@launchctl unload ~/Library/LaunchAgents/com.sectorflabs.ratatoskr.plist 2>/dev/null || true
	@echo "Service stopped."

status-service:
	@echo "Checking ratatoskr service status..."
	@launchctl list | grep com.sectorflabs.ratatoskr || echo "Service not running"
	@echo ""
	@echo "Recent logs:"
	@tail -10 /usr/local/var/log/ratatoskr.log 2>/dev/null || echo "No logs found"
