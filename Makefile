NETWORK ?= localnet

check:
	slither . --config-file ./slither.config.json

lint:
	solhint 'src/**/*.sol'

check-gateway:
	docker run --rm -v $(shell pwd):/app -w /app mythril/myth:latest -v4 analyze --solc-json remappings.json ./src/Gateway.sol --solv 0.8.19

check-subnet:
	docker run --rm -v $(shell pwd):/app -w /app mythril/myth:latest -v4 analyze --solc-json remappings.json ./src/SubnetActor.sol --solv 0.8.19

deploy-ipc:
	./ops/deploy.sh $(NETWORK)

.PHONY: deploy-ipc check lint check-subnet check-gateway