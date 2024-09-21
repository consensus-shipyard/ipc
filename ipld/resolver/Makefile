.PHONY: all build test lint license check-fmt check-clippy diagrams

CRATE   := ipc_ipld_resolver
PACKAGE := $(patsubst %, --package %, $(CRATE))

all: test build

build:
	cargo build -Z unstable-options --release

test:
	cargo test --release $(PACKAGE)

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

diagrams:
	$(MAKE) -C docs/diagrams

check-diagrams: diagrams
	if git diff --name-only docs/diagrams | grep .png; then \
		echo "There are uncommitted changes to the diagrams"; \
		exit 1; \
	fi
