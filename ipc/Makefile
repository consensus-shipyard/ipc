.PHONY: all build build-ui build-with-ui test lint license check-fmt check-clippy diagrams

CRATE   := ipc-cli ipc-wallet ipc-provider ipc-api ipc-types
PACKAGE := $(patsubst %, --package %, $(CRATE))

all: test build

build:
	cargo build --locked --release

build-ui:
	@echo "Building frontend assets..."
	cd ../ipc-ui/frontend && pnpm install && pnpm run build
	@echo "Frontend build complete!"

build-with-ui: build-ui
	@echo "Building IPC CLI with embedded frontend assets..."
	cargo build --locked --release
	@echo "IPC CLI build complete with UI assets!"

install:
	cargo install --locked --path cli

test:
	cargo test --locked --release $(PACKAGE)

# itest:
# 	cargo test -p itest --test checkpoint -- --nocapture

# e2e:
# 	cargo test --release -p ipc_e2e

clean:
	cargo clean

lint: \
	check-fmt \
	check-clippy

check-fmt:
	@# `nightly` is required to support ignore list in rustfmt.toml
	cargo +nightly fmt $(PACKAGE) --check

check-clippy:
	cargo clippy $(PACKAGE) --tests --no-deps -- -D clippy::all

diagrams:
	$(MAKE) -C docs/diagrams

check-diagrams: diagrams
	if git diff --name-only docs/diagrams | grep .png; then \
		echo "There are uncommitted changes to the diagrams"; \
		exit 1; \
	fi
