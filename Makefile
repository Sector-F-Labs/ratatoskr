# Makefile for Deno Telegram <-> Kafka (Redpanda) Bot

.PHONY: help install setup run dev stop produce test_buttons test_image test_callback

# Redpanda config
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
	@echo "  install       Install Redpanda using Homebrew"
	@echo "  setup         Start Redpanda and create topics"
	@echo ""
	@echo "Runtime targets:"
	@echo "  run           Run the bot"
	@echo "  dev           Run the bot with auto-reload (cargo watch)"
	@echo "  stop          Stop Redpanda container"
	@echo ""
	@echo "Testing targets:"
	@echo "  produce       Send text message (TEXT=\"your message\")"
	@echo "  test_buttons  Send message with buttons (TEXT=\"your message\")"
	@echo "  test_image    Send image message (IMAGE_PATH=path CAPTION=\"caption\")"
	@echo "  test_callback Simulate callback (MESSAGE_ID=123 CALLBACK_DATA=\"data\")"
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
	pkill -f "redpanda start" || true

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

# Push to remote server (configurable via environment variables)
REMOTE_HOST?=plan10
REMOTE_PATH?=~/src/ratatoskr

push:
	rsync -av --delete \
		Cargo.toml Cargo.lock \
		src scripts \
		Makefile \
		$(REMOTE_HOST):$(REMOTE_PATH)
