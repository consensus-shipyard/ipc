.PHONY: all build test lint license check-fmt check-clippy

all: test build

build:
	cargo build --release

test:
	cargo test --release --workspace

clean:
	cargo clean

lint: \
	license \
	check-fmt \
	check-clippy

license:
	./scripts/add_license.sh

check-fmt:
	cargo fmt --all --check

check-clippy:
	cargo clippy --all --tests -- -D clippy::all
