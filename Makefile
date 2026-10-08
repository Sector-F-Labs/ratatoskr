.PHONY: help build install run dev test coverage kafka-up kafka-down freebsd-test freebsd-build freebsd-down

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
	@echo "  freebsd-test  - run cargo test in a FreeBSD VM (needs bsdt, see bsdt.toml)"
	@echo "  freebsd-build - cargo build --release in the FreeBSD VM"
	@echo "  freebsd-down  - shut the FreeBSD VM down (bsdt destroy deletes it)"

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

# FreeBSD, through bsdt (https://bsdt.divanv.com). `bsdt up` boots the VM described
# in bsdt.toml, provisioning it on first use, and does nothing if it's already
# running; `bsdt exec` syncs the checkout into it before running the command.
freebsd-test:
	bsdt up
	bsdt exec -- cargo test

freebsd-build:
	bsdt up
	bsdt exec -- cargo build --release

freebsd-down:
	bsdt down
