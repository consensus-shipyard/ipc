.PHONY: all build test lint license check-fmt check-clippy actor-bundle

BUILTIN_ACTORS_TAG    ?= v11.0.0
BUILTIN_ACTORS_DIR    := ../builtin-actors
BUILTIN_ACTORS_CODE   := $(shell find $(BUILTIN_ACTORS_DIR) -type f -name "*.rs" | grep -v target)
BUILTIN_ACTORS_BUNDLE := $(shell pwd)/$(BUILTIN_ACTORS_DIR)/output/bundle.car

# Tag used to disambiguate if there are multiple options.
IPC_ACTORS_TAG				?= dev
IPC_ACTORS_FIND       := scripts/find-ipc-actors.sh $(IPC_ACTORS_TAG)
IPC_ACTORS_CODE       := $(shell find $(shell $(IPC_ACTORS_FIND)) -type f -name "*.sol")
IPC_ACTORS_ABI        := .make/.ipc-actors-abi
# Note that without `:=`, just `=`, it should evaluate it every time it appears in a target.
IPC_ACTORS_DIR         = $(shell $(IPC_ACTORS_FIND))
IPC_ACTORS_OUT         = $(IPC_ACTORS_DIR)/out
FENDERMINT_CODE       := $(shell find . -type f \( -name "*.rs" -o -name "Cargo.toml" \) | grep -v target)

# Override PROFILE env var to choose between `local | ci`
PROFILE ?= local

# Set to `--push` to push the multiarch image during the build.
# Leave on `--load` for local build, but it only works for a single platform.
BUILDX_STORE ?= --load
# Set according to what kind of `--platform` and `--cache` to use.
# Leave empty for local builds, then the platform matches the local one.
BUILDX_FLAGS ?=
# Set to the `<repo>/<image>:<tag>` label the image.
BUILDX_TAG   ?= fendermint:latest

all: test build diagrams

diagrams:
	make -C docs/diagrams diagrams

build:
	cargo build --release

install:
	cargo install --path fendermint/app

# Using --release for testing because wasm can otherwise be slow.
test: $(IPC_ACTORS_ABI) $(BUILTIN_ACTORS_BUNDLE)
	FM_BUILTIN_ACTORS_BUNDLE=$(BUILTIN_ACTORS_BUNDLE) \
	FM_CONTRACTS_DIR=$(IPC_ACTORS_OUT) \
	cargo test --release --workspace --exclude smoke-test

e2e: docker-build
	cd fendermint/testing/smoke-test && cargo make --profile $(PROFILE)

clean:
	cargo clean
	cd $(BUILTIN_ACTORS_DIR) && cargo clean
	rm $(BUILTIN_ACTORS_BUNDLE)
	rm -rf .make

lint: \
	license \
	check-fmt \
	check-clippy

license:
	./scripts/add_license.sh

check-fmt:
	@# `nightly` is required to support ignore list in rustfmt.toml
	cargo +nightly fmt --all --check

check-clippy:
	cargo clippy --all --tests -- -D clippy::all

docker-deps: $(BUILTIN_ACTORS_BUNDLE) $(FENDERMINT_CODE) $(IPC_ACTORS_ABI)
	rm -rf docker/.artifacts
	mkdir -p docker/.artifacts/contracts
	cp -r $(IPC_ACTORS_OUT)/* docker/.artifacts/contracts
	cp $(BUILTIN_ACTORS_BUNDLE) docker/.artifacts

docker-build: docker-deps
	if [ "$(PROFILE)" = "ci" ]; then \
		cat docker/builder.ci.Dockerfile docker/runner.Dockerfile > docker/Dockerfile ; \
		docker buildx build \
			$(BUILDX_STORE) \
			$(BUILDX_FLAGS) \
			-f docker/Dockerfile \
			-t $(BUILDX_TAG) $(PWD); \
	else \
		cat docker/builder.local.Dockerfile docker/runner.Dockerfile > docker/Dockerfile ; \
		DOCKER_BUILDKIT=1 \
		docker build \
			-f docker/Dockerfile \
			-t fendermint:latest $(PWD); \
	fi


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
	git checkout $(BUILTIN_ACTORS_TAG) && \
	rustup target add wasm32-unknown-unknown && \
	cargo run --release -- -o output/$(shell basename $@)


# Compile the ABI artifacts of the IPC Solidity actors.
ipc-actors-abi: $(IPC_ACTORS_ABI)

# Check out the IPC Solidity actors if necessary and compile the ABI, putting down a marker at the end.
# Doing a recursive call if the checkouts haven't been done before because of how $(shell) is already evaluated.
$(IPC_ACTORS_ABI): $(IPC_ACTORS_CODE) | forge
	if [ -z $(IPC_ACTORS_DIR) ]; then \
		cargo fetch; \
		$(MAKE) ipc-actors-abi; \
	else \
		$(MAKE) -C $(IPC_ACTORS_DIR) compile-abi; \
	fi
	mkdir -p $(dir $@) && touch $@

# Forge is used by the ipc-solidity-actors compilation steps.
.PHONY: forge
forge:
	@if [ -z "$(shell which forge)" ]; then \
		echo "Please install Foundry. See https://book.getfoundry.sh/getting-started/installation"; \
		exit 1; \
	fi
