.PHONY: all build test lint license check-fmt check-clippy diagrams install-infra clean-infra

IPC_ACTORS_DIR := $(PWD)/../contracts
IPC_ACTORS_OUT := $(IPC_ACTORS_DIR)/out

all: test build

build: $(IPC_ACTORS_OUT)
	cargo build --release -p ipc-cli && mkdir -p bin/ && cp target/release/ipc-cli ./bin/ipc-cli

test: $(IPC_ACTORS_OUT)
	cargo test --release --package 'ipc-*'

itest: $(IPC_ACTORS_OUT)
	cargo test -p itest --test checkpoint -- --nocapture

e2e: $(IPC_ACTORS_OUT)
	cargo test --release -p ipc_e2e

clean:
	cargo clean

lint: \
	check-fmt \
	check-clippy

install-infra:
	./scripts/install_infra.sh

clean-infra:
	rm -rf ./bin/ipc-infra

check-fmt:
	@# `nightly` is required to support ignore list in rustfmt.toml
	cargo +nightly fmt --all --check

check-clippy: $(IPC_ACTORS_OUT)
	cargo clippy --all --tests -- -D clippy::all

diagrams:
	$(MAKE) -C docs/diagrams

check-diagrams: diagrams
	if git diff --name-only docs/diagrams | grep .png; then \
		echo "There are uncommitted changes to the diagrams"; \
		exit 1; \
	fi

# Regenerate the ABI artifacts if we don't have them already, or they changed.
$(IPC_ACTORS_OUT):
	cd $(IPC_ACTORS_DIR) && make compile-abi
