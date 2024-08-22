.PHONY: all build test lint license check-fmt check-clippy actor-bundle

BUILTIN_ACTORS_FOLDER ?= $(PWD)/../builtin-actors
BUILTIN_ACTORS_BUNDLE := $(PWD)/builtin-actors/output/bundle.car
CUSTOM_ACTORS_BUNDLE  := $(PWD)/actors/output/custom_actors_bundle.car

IPC_ACTORS_DIR        := $(PWD)/../contracts
IPC_ACTORS_OUT        := $(IPC_ACTORS_DIR)/out
IPC_ACTORS_CODE       := $(shell find $(IPC_ACTORS_DIR) -type f \( -name "*.sol" \))
IPC_ACTORS_GEN        := .contracts-gen

FENDERMINT_CODE       := $(shell find . -type f \( -name "*.rs" -o -name "Cargo.toml" \) | grep -v target)

CRATE   := $(shell cargo test -p 2>&1 | grep fendermint)
PACKAGE := $(patsubst %, --package %, $(CRATE))

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

all: lint build test diagrams docker-build

diagrams:
	make -C  ../docs/fendermint/diagrams diagrams

build: $(IPC_ACTORS_GEN) | protoc npm
	cargo build --locked --release

install: $(IPC_ACTORS_GEN)
	cargo install --locked --path app

# Using --release for testing because wasm can otherwise be slow.
# Excluding the materializer tests because they rely on docker; we could test with --lib
test: $(BUILTIN_ACTORS_BUNDLE) $(IPC_ACTORS_GEN)
	FM_BUILTIN_ACTORS_BUNDLE=$(BUILTIN_ACTORS_BUNDLE) \
	FM_CUSTOM_ACTORS_BUNDLE=$(CUSTOM_ACTORS_BUNDLE) \
	FM_CONTRACTS_DIR=$(IPC_ACTORS_OUT) \
	cargo test --locked --release \
		-p fendermint_abci \
		-p fendermint_app \
		-p fendermint_actor_objectstore \
		-p fendermint_app_options \
		-p fendermint_crypto \
		-p fendermint_testing \
		-p fendermint_vm_actor_interface \
		-p fendermint_vm_genesis \
		-p fendermint_vm_core \
		-p fendermint_vm_encoding \
		-p fendermint_app_settings \
		-p fendermint_vm_topdown \
		-p fendermint_vm_event \
		-p fendermint_eth_api \
		-p fendermint_rpc \
		-p fendermint_actor_accumulator \
		-p fendermint_vm_message \
		-p fendermint_rocksdb \
		-p fendermint_storage \
		-p fendermint_actor_chainmetadata \
		-p fendermint_actors \
		-p fendermint_eth_hardhat \
		-p fendermint_vm_iroh_resolver \
		-p fendermint_vm_resolver \
		-p fendermint_graph_test \
		-p fendermint_smoke_test \
		-p fendermint_snapshot_test
	FM_BUILTIN_ACTORS_BUNDLE=$(BUILTIN_ACTORS_BUNDLE) \
	FM_CUSTOM_ACTORS_BUNDLE=$(CUSTOM_ACTORS_BUNDLE) \
	FM_CONTRACTS_DIR=$(IPC_ACTORS_OUT) \
	cargo test --locked --release \
		-p fendermint_actor_eam \
		-p fendermint_vm_interpreter \
		-p fendermint_vm_snapshot \
		-p fendermint_contract_test

# Not using --release beause the build has been done in docker and the wasm code runs inside the container.
e2e: docker-build | cargo-make
	cd testing/smoke-test    && cargo make --profile $(PROFILE)
	cd testing/snapshot-test && cargo make --profile $(PROFILE)
	cd testing/graph-test    && cargo make --profile $(PROFILE)
	PROFILE=$(PROFILE) cargo test --locked --package fendermint_materializer

clean:
	cargo clean
	rm -f $(BUILTIN_ACTORS_BUNDLE)
	rm -f $(CUSTOM_ACTORS_BUNDLE)
	rm -rf testing/materializer/tests/docker-materializer

lint: \
	check-fmt \
	check-clippy

check-fmt:
	@# `nightly` is required to support ignore list in rustfmt.toml
	cargo +nightly fmt $(PACKAGE) --check

check-clippy: $(IPC_ACTORS_GEN)
	@# --all actually runs on the whole workspace but since fendermint depends on all, it's okay.
	@# We could have a separate top level job that does `fmt` and `clippy` check on the whole project.
	cargo clippy $(PACKAGE) --no-deps --tests -- -D clippy::all

docker-deps: $(BUILTIN_ACTORS_BUNDLE) $(CUSTOM_ACTORS_BUNDLE) $(IPC_ACTORS_GEN)
	rm -rf docker/.artifacts
	mkdir -p docker/.artifacts/contracts
	cp -r $(IPC_ACTORS_OUT)/* docker/.artifacts/contracts
	cp $(BUILTIN_ACTORS_BUNDLE) docker/.artifacts
	cp $(CUSTOM_ACTORS_BUNDLE) docker/.artifacts

# To use `buildx` locally to produce multiplatform images, one needs to run `docker buildx create --use`.
# After that it looks like even the regular docker build needs the `--load` parameter, which hopefully
# it can handle for anyone with `DOCKER_BUILDKIT=1`.
docker-build: docker-deps $(FENDERMINT_CODE)
	if [ "$(PROFILE)" = "ci" ]; then \
		cat docker/builder.ci.Dockerfile docker/runner.Dockerfile > docker/Dockerfile ; \
		docker buildx build --ssh default \
			$(BUILDX_STORE) \
			$(BUILDX_FLAGS) \
			-f docker/Dockerfile \
			-t $(BUILDX_TAG) $(PWD)/..; \
	else \
		cat docker/builder.local.Dockerfile docker/runner.Dockerfile > docker/Dockerfile ; \
		DOCKER_BUILDKIT=1 \
		docker build --ssh default \
			$(BUILDX_STORE) \
			-f docker/Dockerfile \
			-t fendermint:latest $(PWD)/..; \
	fi


# Build a bundle CAR; this is so we don't have to have a project reference,
# which means we are not tied to the release cycle of both FVM _and_ actors;
# so long as they work together.
actor-bundle: $(BUILTIN_ACTORS_BUNDLE) $(CUSTOM_ACTORS_BUNDLE)

compile-abi: $(IPC_ACTORS_GEN)

# Download a released builtin-actors bundle CAR file.
$(BUILTIN_ACTORS_BUNDLE):
	mkdir -p $(dir $@)
	(cd $(BUILTIN_ACTORS_FOLDER) && make bundle-mainnet)
	cp $(BUILTIN_ACTORS_FOLDER)/output/builtin-actors-mainnet.car $@

# Build a bundle CAR for the custom actors in this repo.
$(CUSTOM_ACTORS_BUNDLE):
	cargo build --locked --release -p fendermint_actors

# Regenerate the ABI artifacts if we don't have them already, or they changed.
$(IPC_ACTORS_GEN): $(IPC_ACTORS_CODE)
		cd $(IPC_ACTORS_DIR) && make compile-abi
		touch $@

.PHONY: protoc
protoc:
	@if [ -z "$(shell which protoc)" ]; then \
		echo "Please install the Protobuf Compiler. See https://grpc.io/docs/protoc-installation/"; \
		exit 1; \
	fi

.PHONY: npm
npm:
	@if [ -z "$(shell which npm)" ]; then \
		echo "Please install npm. See https://docs.npmjs.com/downloading-and-installing-node-js-and-npm"; \
		exit 1; \
	fi

.PHONY: cargo-make
cargo-make:
	@if [ -z "$(shell cargo --list | grep make)" ]; then \
		cargo install cargo-make; \
	fi
