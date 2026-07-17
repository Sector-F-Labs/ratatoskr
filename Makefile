.PHONY: help build install run dev test coverage kafka-up kafka-down

help:
	@echo "Ratatoskr - Telegram <-> Kafka bridge"
	@echo "Targets:"
	@echo "  build      - cargo build"
	@echo "  install    - cargo install --path ."
	@echo "  run        - cargo run -- serve"
	@echo "  dev        - cargo watch -x 'run -- serve'"
	@echo "  test       - cargo test"
	@echo "  coverage   - run tests with llvm-cov and show coverage report"
	@echo "  kafka-up   - start Kafka/Zookeeper/AKHQ via docker-compose"
	@echo "  kafka-down - stop the docker-compose stack"

build:
	cargo build

install:
	cargo install --path .

run:
	cargo run -- serve

dev:
	cargo watch -x 'run -- serve'

test:
	cargo test

coverage:
	cargo llvm-cov --summary-only
	@echo ""
	@echo "For a detailed HTML report: cargo llvm-cov --html --open"

kafka-up:
	docker-compose up -d

kafka-down:
	docker-compose down
