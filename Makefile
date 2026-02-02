.PHONY: help build install run dev test pipe coverage

PIPE_OUTBOUND_PATH ?= ./ratatoskr_out.pipe

help:
	@echo "Ratatoskr - Telegram <-> handler bridge (pipe mode only)"
	@echo "Targets:"
	@echo "  build     - cargo build"
	@echo "  install   - cargo install --path ."
	@echo "  run       - cargo run"
	@echo "  dev       - cargo watch -x run"
	@echo "  test      - cargo test"
	@echo "  pipe      - mkfifo $${PIPE_OUTBOUND_PATH} if missing"
	@echo "  coverage  - run tests with llvm-cov and show coverage report"
	@echo "  test_pipe - write a sample OutgoingMessage JSON into the pipe (requires CHAT_ID)"

build:
	cargo build

install:
	cargo install --path .

run:
	cargo run

dev:
	cargo watch -x run

test:
	cargo test

coverage:
	cargo llvm-cov --summary-only
	@echo ""
	@echo "For a detailed HTML report: cargo llvm-cov --html --open"

pipe:
	@if [ ! -p "$(PIPE_OUTBOUND_PATH)" ]; then \
		mkfifo "$(PIPE_OUTBOUND_PATH)"; \
		echo "Created pipe at $(PIPE_OUTBOUND_PATH)"; \
	else \
		echo "Pipe already exists at $(PIPE_OUTBOUND_PATH)"; \
	fi

test_pipe:
	PIPE_OUTBOUND_PATH=$(PIPE_OUTBOUND_PATH) ./scripts/test_pipe_outbound.sh
