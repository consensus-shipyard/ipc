# Targets that are commands.
COMMANDS :=  deploy-ipc deploy-subnet-registry deploy-subnet upgrade-gw-diamond upgrade-sa-diamond \
             upgrade-sr-diamond gen compile-abi rust-binding slither lint fmt deps build \
             test install-dev install-npm-package install-eth-abi storage clean coverage \
             prepare build-selector-library forge

# Targets that are prerequisite commands.
PREREQ_COMMANDS := pnpm

# Targets that are not commands.
NON_COMMANDS := node_modules

# Declare commands and prerequisite commands as phony targets.
.PHONY: $(COMMANDS) $(PREREQ_COMMANDS)

# Make all commands and non-command targets dependent on the prerequisites.
$(COMMANDS): $(PREREQ_COMMANDS)
$(NON_COMMANDS): $(PREREQ_COMMANDS)

pnpm:
	@command -v pnpm > /dev/null || { \
		echo "pnpm is not installed. Installing pnpm..."; \
		npm install -g pnpm; \
	}

# ==============================================================================
# Deployment

NETWORK ?= auto

# Output for generated Solidity artifacts.
# It is required by docker builds, but shouldn't be checked into git.
OUTPUT ?= out

deploy-ipc:
	./ops/deploy.sh $(NETWORK)

deploy-subnet-registry:
	./ops/deploy-subnet-registry.sh $(NETWORK)

deploy-subnet:
	./ops/deploy-subnet.sh $(NETWORK)

upgrade-gw-diamond:
	./ops/upgrade-gw-diamond.sh $(NETWORK)

upgrade-sa-diamond:
	./ops/upgrade-sa-diamond.sh $(NETWORK) $(SUBNET_ACTOR_ADDRESS)

upgrade-sr-diamond:
	./ops/upgrade-sr-diamond.sh $(NETWORK)

# ==============================================================================
# Code generation

# Regenerate bindings and ABI artifacts when sources change with the `make .gen` target.
.gen: $(IPC_ACTORS_CODE)
	$(MAKE) gen
	touch $@

gen: compile-abi rust-binding

compile-abi: node_modules | forge
	mkdir -p $(OUTPUT)
	./ops/compile-abi.sh $(OUTPUT)

rust-binding:
	OUTPUT=$(OUTPUT) cargo build --locked --release --manifest-path ./binding/Cargo.toml -p ipc_actors_abis

# ==============================================================================
# Running security checks within the local computer

slither:
	slither . --config-file ./slither.config.json

# ==============================================================================
# Development support

lint: fmt
	pnpm exec solhint 'contracts/**/*.sol' 'sdk/**/**/*.sol'

fmt:
	pnpm install --silent --no-save
	pnpm exec prettier --check -w 'contracts/**/**/*.sol' 'sdk/**/**/*.sol' 'test/**/**/*.sol' 'test/**/**/*.t.sol' '**/*.{js,jsx,ts,tsx,json,css,md}'

deps: node_modules

node_modules: package.json pnpm-lock.yaml
	pnpm install
	touch node_modules

build: node_modules | forge
	forge build
	FOUNDRY_SRC=sdk forge build

test: node_modules | forge
	forge test -vvv --ffi

install-dev: install-npm-package install-eth-abi

install-npm-package: node_modules
	pnpm install --save-dev

install-eth-abi:
	curl -sSL https://bootstrap.pypa.io/get-pip.py -o get-pip.py && python3 get-pip.py && rm get-pip.py && python3 -m pip install eth_abi

storage:
	rm -rf ./cache
	rm -rf ./cache_hardhat
	pnpm exec hardhat storage-layout --update

clean:
	rm -rf ./artifacts
	rm -rf ./cache
	rm -rf ./cache_hardhat
	rm -rf ./typechain

coverage: node_modules | forge
	forge coverage --ffi --report lcov -C ./src
	@if [ "$(shell uname)" = "Darwin" ]; then \
		genhtml -o coverage_report lcov.info --branch-coverage --ignore-errors category; \
	else \
		genhtml -o coverage_report lcov.info --branch-coverage; \
	fi
	./tools/check_coverage.sh

prepare: build-selector-library fmt lint test slither

build-selector-library: | forge
	python3 scripts/python/build_selector_library.py
	pnpm exec prettier -w test/helpers/SelectorLibrary.sol

# Forge is used by the ipc-solidity-actors compilation steps.
forge:
	@if [ -z "$(shell which forge)" ]; then \
		echo "Please install Foundry. See https://book.getfoundry.sh/getting-started/installation"; \
		exit 1; \
	fi

# ==============================================================================