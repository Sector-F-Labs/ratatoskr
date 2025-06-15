# Makefile for Deno Telegram <-> Kafka Bot

.PHONY: help install setup run dev stop produce test_buttons test_image test_callback test_typing test_typing_demo install-service uninstall-service start-service stop-service status-service

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
	@echo "  test_typing   Send typing indicator"
	@echo "  test_typing_demo Send typing indicator followed by message"
	@echo ""
	@echo "Environment variables:"
	@echo "  See .envrc.example for all configuration options"
	@echo "  Copy .envrc.example to .envrc and customize"
	@echo "  CHAT_ID            - Target Telegram chat ID (required for testing)"
	@echo "  REMOTE_HOST        - Remote host for deployment"
	@echo "  REMOTE_USER        - Remote user for deployment"
	@echo "  REMOTE_PATH        - Remote path for deployment"

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

test_typing:
	./scripts/produce_typing.sh

test_typing_demo:
	./scripts/produce_typing_demo.sh

test_all_message_types: test_text test_buttons test_image test_typing
	echo "All message types tested."

test_callback:
	./scripts/simulate_callback.sh $(MESSAGE_ID) "$(CALLBACK_DATA)"

test_keyboard:
	./scripts/produce_reply_keyboard.sh "$(TEXT)"

test_location:
	./scripts/produce_location_request.sh "$(TEXT)"

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
