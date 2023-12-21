CRATE   := primitives
PACKAGE := $(patsubst %, --package %, $(CRATE))

test:
	cargo test --release $(PACKAGE)

check-fmt:
	cargo +nightly fmt $(PACKAGE) --check

check-clippy:
	cargo clippy $(PACKAGE) --tests --no-deps -- -D clippy::all


lint: \
	check-fmt \
	check-clippy
