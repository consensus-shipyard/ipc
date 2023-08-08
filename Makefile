# ==============================================================================
# Deployment

NETWORK ?= auto
OUTPUT ?= ./out

deploy-ipc:
	./ops/deploy.sh $(NETWORK)

compile-abi:
	./ops/compile-abi.sh $(OUTPUT)
# ==============================================================================
# Running security checks within the local computer

slither:
	slither . --config-file ./slither.config.json

check-gateway:
	docker run --rm -v $(shell pwd):/app -w /app mythril/myth:latest -v4 analyze --solc-json remappings.json ./src/Gateway.sol --solv 0.8.19

check-subnet:
	docker run --rm -v $(shell pwd):/app -w /app mythril/myth:latest -v4 analyze --solc-json remappings.json ./src/SubnetActor.sol --solv 0.8.19

# ==============================================================================
# Development support

lint:
	solhint 'src/**/*.sol'

format:
	npx prettier --check -w 'src/**/*.sol' 'test/*.sol'

build:
	forge build

test:
	forge test -vvv --ffi

storage:
	npx hardhat storage-layout --update

clean:
	rm -rf ./artifacts
	rm -rf ./cache
	rm -rf ./cache_hardhat
	rm -rf ./typechain

prepare: format lint test slither

# ==============================================================================
.PHONY: deploy-ipc lint format check-subnet slither check-gateway test prepare storage build clean
