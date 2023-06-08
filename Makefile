.PHONY: all build test lint license check-fmt check-clippy diagrams install_infra

all: test build

build:
	cargo build -Z unstable-options --release --out-dir ./bin

test:
	cargo test --release --workspace --exclude ipc_e2e itest

itest:
	cargo test -p itest --test checkpoint -- --nocapture

e2e:
	cargo test --release -p ipc_e2e

clean:
	cargo clean

lint: \
	license \
	check-fmt \
	check-clippy

license:
	./scripts/add_license.sh

install-infra:
	./scripts/install_infra.sh

check-fmt:
	cargo fmt --all --check

check-clippy:
	cargo clippy --all --tests -- -D clippy::all

diagrams:
	$(MAKE) -C docs/diagrams

check-diagrams: diagrams
	if git diff --name-only docs/diagrams | grep .png; then \
		echo "There are uncommitted changes to the diagrams"; \
		exit 1; \
	fi
