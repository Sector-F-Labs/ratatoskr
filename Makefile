# Makefile for Deno Telegram <-> Kafka Bot

.PHONY: help install setup run dev stop produce test_buttons test_image test_callback install-service uninstall-service start-service stop-service status-service

# Kafka config
KAFKA_BROKER=localhost:9092
KAFKA_IN_TOPIC?=com.sectorflabs.ratatoskr.in
KAFKA_OUT_TOPIC?=com.sectorflabs.ratatoskr.out

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
	@echo "Testing targets:"
	@echo "  produce       Send text message (TEXT=\"your message\")"
	@echo "  test_buttons  Send message with buttons (TEXT=\"your message\")"
	@echo "  test_image    Send image message (IMAGE_PATH=path CAPTION=\"caption\")"
	@echo "  test_callback Simulate callback (MESSAGE_ID=123 CALLBACK_DATA=\"data\")"
	@echo "  test_keyboard Send message with reply keyboard (TEXT=\"your message\")"
	@echo "  test_location Send location request with reply keyboard"
	@echo ""
	@echo "Environment variables:"
	@echo "  CHAT_ID            - Target Telegram chat ID (required for testing)"
	@echo "  KAFKA_BROKER       - Kafka broker address (default: localhost:9092)"
	@echo "  KAFKA_IN_TOPIC     - Input topic name"
	@echo "  KAFKA_OUT_TOPIC    - Output topic name"
	@echo "  REMOTE_HOST        - Remote host for pushpi (default: divanvisagie@heimdallr)"
	@echo "  REMOTE_PATH        - Remote path for pushpi (default: ~/src/ratatoskr)"

main:
	cargo build

install:
	cargo install --path .

setup:
	./scripts/setup_env.sh

run:
	cargo run
dev:
	cargo watch -x run

stop:
	pkill -f "kafka" || true

consume:
	./scripts/consume.sh $(N)

test_text:
	./scripts/produce.sh "$(TEXT)"

test_buttons:
	./scripts/produce_with_buttons.sh "$(TEXT)"

test_image:
	./scripts/produce_image.sh "$(IMAGE_PATH)" "$(CAPTION)"

test_all_message_types: test_text test_buttons test_image
	echo "All message types tested."

test_callback:
	./scripts/simulate_callback.sh $(MESSAGE_ID) "$(CALLBACK_DATA)"

test_keyboard:
	./scripts/produce_reply_keyboard.sh "$(TEXT)"

test_location:
	./scripts/produce_location_request.sh "$(TEXT)"

# Push to remote server (configurable via environment variables)
REMOTE_HOST?=plan10
REMOTE_PATH?=~/src/ratatoskr

push:
	@echo "Pushing to remote server $(REMOTE_HOST):$(REMOTE_PATH)..."
	@rsync -av --delete \
		Cargo.toml Cargo.lock \
		src scripts \
		Makefile \
		$(REMOTE_HOST):$(REMOTE_PATH)
	@ssh $(REMOTE_HOST) "cd $(REMOTE_PATH) && cargo install --path ."
	@ssh $(REMOTE_HOST) "cd $(REMOTE_PATH) && make install-service"
	@ssh $(REMOTE_HOST) "cd $(REMOTE_PATH) && make start-service"


# Service management targets
install-service: install
	@echo "Installing ratatoskr as macOS LaunchAgent..."
	@mkdir -p /usr/local/var/ratatoskr
	@mkdir -p /usr/local/var/log
	@cp scripts/com.sectorflabs.ratatoskr.plist ~/Library/LaunchAgents/
	@echo "Service installed. Use 'make start-service' to start it."

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
