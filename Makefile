# Makefile for Deno Telegram <-> Kafka (Redpanda) Bot

.PHONY: help install setup run dev stop consume produce test_flow test_buttons test_image test_callback show_topics validate_json

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
	@echo "  consume       Consume messages from IN topic (N=number of messages)"
	@echo "  produce       Send text message (TEXT=\"your message\")"
	@echo "  test_buttons  Send message with buttons (TEXT=\"your message\")"
	@echo "  test_image    Send image message (IMAGE_PATH=path CAPTION=\"caption\")"
	@echo "  test_callback Simulate callback (MESSAGE_ID=123 CALLBACK_DATA=\"data\")"
	@echo "  test_flow     Run full integration test"
	@echo ""
	@echo "Debug targets:"
	@echo "  show_topics   Show recent topic messages (N=number of messages)"
	@echo "  validate_json Validate JSON message format (JSON=\"...\" TYPE=incoming|outgoing)"
	@echo ""
	@echo "Environment variables:"
	@echo "  CHAT_ID            - Target Telegram chat ID (required for testing)"
	@echo "  KAFKA_BROKER       - Kafka broker address (default: localhost:9092)"
	@echo "  KAFKA_IN_TOPIC     - Input topic name"
	@echo "  KAFKA_OUT_TOPIC    - Output topic name"

main:
	cargo build

# Install Redpanda natively (macOS/Linux with Homebrew)
install:
	brew install redpanda-data/tap/redpanda

# Setup Redpanda natively and create topics
setup:
	rpk container start
	rpk topic create $(KAFKA_IN_TOPIC) || true; \
	rpk topic create $(KAFKA_OUT_TOPIC) || true; \
	echo "Redpanda and topics are ready."

run:
	cargo run
dev:
	cargo watch -x run

stop:
	pkill -f "redpanda start" || true 

consume:
	./scripts/consume.sh $(N)

produce:
	./scripts/produce.sh "$(TEXT)"

test_buttons:
	./scripts/produce_with_buttons.sh "$(TEXT)"

test_image:
	./scripts/produce_image.sh "$(IMAGE_PATH)" "$(CAPTION)"

test_callback:
	./scripts/simulate_callback.sh $(MESSAGE_ID) "$(CALLBACK_DATA)"

test_flow:
	./scripts/test_full_flow.sh

show_topics:
	./scripts/show_topics.sh $(N)

validate_json:
	./scripts/validate_json.sh "$(JSON)" "$(TYPE)"

pushpi:
	rsync -av --delete \
		Cargo.toml Cargo.lock \
		src scripts \
		divanvisagie@heimdallr:~/src/ratatoskr
