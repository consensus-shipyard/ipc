.PHONY: all build test lint license check-fmt check-clippy actor-bundle

BUILTIN_ACTORS_DIR:=../builtin-actors
BUILTIN_ACTORS_CODE:=$(shell find $(BUILTIN_ACTORS_DIR) -type f -name "*.rs")
BUILTIN_ACTORS_BUNDLE:=$(shell pwd)/$(BUILTIN_ACTORS_DIR)/output/bundle.car

all: test build

build:
	cargo build --release

# Using --release for testing because wasm can otherwise be slow.
test: $(BUILTIN_ACTORS_BUNDLE)
	BUILTIN_ACTORS_BUNDLE=$(BUILTIN_ACTORS_BUNDLE) cargo test --release

clean:
	cargo clean
	cd $(BUILTIN_ACTORS_DIR) && cargo clean
	rm $(BUILTIN_ACTORS_BUNDLE)

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

# Build a bundle CAR; this is so we don't have to have a project reference,
# which means we are not tied to the release cycle of both FVM _and_ actors;
# so long as they work together.
actor-bundle: $(BUILTIN_ACTORS_BUNDLE)

$(BUILTIN_ACTORS_BUNDLE): $(BUILTIN_ACTORS_CODE)
	if [ ! -d $(BUILTIN_ACTORS_DIR) ]; then \
		mkdir -p $(BUILTIN_ACTORS_DIR) && \
		cd $(BUILTIN_ACTORS_DIR) && \
		cd .. && \
		git clone https://github.com/filecoin-project/builtin-actors.git; \
	fi
	cd $(BUILTIN_ACTORS_DIR) && \
	git checkout next && \
	git pull && \
	rustup target add wasm32-unknown-unknown && \
	cargo run --release -- -o output/$(shell basename $@)
