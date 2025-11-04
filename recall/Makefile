.PHONY: all build test clean lint check-fmt check-clippy

CRATE   := recall_actor_sdk recall_kernel recall_kernel_ops recall_syscalls iroh_manager
PACKAGE := $(patsubst %, --package %, $(CRATE))

all: test build

build:
	cargo build --locked --release

test:
	cargo test --locked --release $(PACKAGE)

clean:
	cargo clean

lint: \
	check-fmt \
	check-clippy

check-fmt:
	@# `nightly` is required to support ignore list in rustfmt.toml
	rustup install nightly-2024-09-20
	rustup component add --toolchain nightly-2024-09-20 rustfmt
	cargo +nightly-2024-09-20 fmt $(PACKAGE) --check

check-clippy:
	cargo clippy $(PACKAGE) --tests --no-deps -- -D clippy::all
