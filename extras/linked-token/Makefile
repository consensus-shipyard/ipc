# Makefile for USDC Token Bridge on Calibration Network

# Include environment variables from .env file
-include .env
export

.PHONY: help fmt deploy-usdctest mint-usdc check-balance deploy-replica deploy-controller initialize-replica initialize-controller deposit-usdc withdraw-usdc check-replica-balance

help:
	@echo "Available targets:"
	@echo "  install                            - Install dependencies."
	@echo "  deploy-usdctest                    - Deploy the USDCTest contract on Calibration Network."
	@echo "  mint-usdc                          - Mint 1000 USDCTest tokens to the specified wallet in .env file."
	@echo "  check-balance                      - Check the USDCTest token balance in the wallet."
	@echo "  deploy-replica-implementation      - Deploy the token replica contract implementation."
	@echo "  deploy-replica-proxy               - Deploy the token replica proxy."
	@echo "  deploy-controller-implementation   - Deploy the token controller contract implementation."
	@echo "  deploy-controller-proxy            - Deploy the token controller proxy."
	@echo "  link-replica                       - Link the replica with the token controller address."
	@echo "  link-controller                    - Link the controller with the token replica address."
	@echo "  approve-usdc                       - Approve USDC for transfer."
	@echo "  deposit-usdc                       - Deposit USDC to the subnet."
	@echo "  check-replica-balance              - Check the token replica balance."
	@echo "  withdraw-usdc                      - Withdraw USDC from the subnet."
	@echo "  upgrade-replica-proxy              - Upgrade the token replica proxy to a new implementation."
	@echo "  upgrade-controller-proxy           - Upgrade the token controller proxy to a new implementation."
	@echo ""
	@echo "Usage:"
	@echo "  make <target>                      - Run a specific make target."
	@echo "  make help                          - Display this help message."

fmt:
	npm install --silent --no-save
	npx prettier --check -w 'src/**/**/*.sol' 'script/**/**/*.sol' 'test/**/**/*.sol' 'test/**/**/*.t.sol' '**/*.{js,jsx,ts,tsx,json,css}'

install:
	@echo "Installing dependencies..."
	forge install

deploy-usdctest:
	@echo "Deploying USDCTest contract..."
	forge script script/DeployUSDCTest.s.sol:DeployUSDCTest --broadcast --rpc-url $$CALIBNET_RPC_URL --private-key $$CALIBNET_PRIVATE_KEY --skip-simulation -vvv

mint-usdc:
	@echo "Minting 1000 USDCTest tokens..."
	@USDCTEST_ADDR=$$(cat config.json | jq -r '.LinkedToken.USDCTest'); \
    cast send $$USDCTEST_ADDR "$$(cast calldata "mint(uint256)" $$AMOUNT)" --rpc-url $$CALIBNET_RPC_URL --private-key $$CALIBNET_PRIVATE_KEY

check-balance:
	@echo "Checking wallet balance..."
	@WALLET_ADDRESS=$$(cast wallet address --private-key $$CALIBNET_PRIVATE_KEY); \
	USDCTEST_ADDR=$$(cat config.json | jq -r '.LinkedToken.USDCTest'); \
	cast call $$USDCTEST_ADDR "$$(cast calldata 'balanceOf(address)' $$WALLET_ADDRESS)" --rpc-url $$CALIBNET_RPC_URL

deploy-replica-implementation:
	@echo "Deploying token replica contract on subnet..."
	forge script script/DeployIpcTokenReplica.s.sol:DeployIpcTokenReplica --skip-simulation --rpc-url $$SUBNET_RPC_URL --private-key $$SUBNET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "deployIpcTokenReplica()"

deploy-replica-proxy:
	@echo "Deploying token replica proxy contract on Calibnet..."
	@LinkedTokenReplicaImplementation=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplicaImplementation'); \
	USDCTEST_ADDR=$$(cat config.json | jq -r '.LinkedToken.USDCTest'); \
	forge script script/DeployIpcTokenReplica.s.sol:DeployIpcTokenReplica --skip-simulation --rpc-url $$SUBNET_RPC_URL --private-key $$SUBNET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "deployIpcTokenReplicaProxy(address,address,address,uint64,address[],string,string,uint8)" -- $$LinkedTokenReplicaImplementation $$SUBNET_GATEWAY $$USDCTEST_ADDR $$CALIBNET_CHAIN_ID '[]' $$REPLICA_TOKEN_NAME $$REPLICA_TOKEN_SYMBOL $$REPLICA_TOKEN_DECIMALS


deploy-controller-implementation:
	@echo "Deploying token controller implementation contract on Calibnet..."
	forge script script/DeployIpcTokenController.s.sol:DeployIpcTokenController --skip-simulation --rpc-url $$CALIBNET_RPC_URL --private-key $$CALIBNET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "deployIpcTokenController()"

deploy-controller-proxy:
	@echo "Deploying token controller proxy contract on Calibnet..."
	@LinkedTokenControllerImplementation=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenControllerImplementation'); \
	USDCTEST_ADDR=$$(cat config.json | jq -r '.LinkedToken.USDCTest'); \
	forge script script/DeployIpcTokenController.s.sol:DeployIpcTokenController --skip-simulation --rpc-url $$CALIBNET_RPC_URL --private-key $$CALIBNET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "deployIpcTokenControllerProxy(address,address,address,uint64,address[])" -- $$LinkedTokenControllerImplementation $$CALIBNET_GATEWAY $$USDCTEST_ADDR $$CALIBNET_CHAIN_ID "[$$SUBNET_ROUTE_IN_ETH_FORMAT]"

link-controller:
	@echo "Updating controller with replica's address..."
	@CONTROLLER_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenController'); \
	REPLICA_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplica'); \
	cast send $$CONTROLLER_ADDR "setLinkedContract(address)" --rpc-url $$CALIBNET_RPC_URL --private-key $$CALIBNET_PRIVATE_KEY --  $$REPLICA_ADDR

link-replica:
	@echo "Updating controller with replica's address..."
	@CONTROLLER_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenController'); \
	REPLICA_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplica'); \
	cast send $$REPLICA_ADDR "setLinkedContract(address)" --rpc-url $$SUBNET_RPC_URL --private-key $$SUBNET_PRIVATE_KEY -- $$CONTROLLER_ADDR


approve-usdc:
	@echo "Approving USDC for Controller Contract..."
	@CONTROLLER_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenController'); \
	USDCTEST_ADDR=$$(cat config.json | jq -r '.LinkedToken.USDCTest'); \
	cast send $$USDCTEST_ADDR "approve(address,uint256)" --rpc-url $$CALIBNET_RPC_URL --private-key $$CALIBNET_PRIVATE_KEY -- $$CONTROLLER_ADDR $$AMOUNT

deposit-usdc:
	@echo "Depositing USDC to subnet..."
	@WALLET_ADDRESS=$$(cast wallet address --private-key $$CALIBNET_PRIVATE_KEY); \
	CONTROLLER_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenController'); \
	cast send $$CONTROLLER_ADDR "linkedTransfer(address,uint256)" $$WALLET_ADDRESS $$AMOUNT --rpc-url $$CALIBNET_RPC_URL --private-key $$CALIBNET_PRIVATE_KEY

check-replica-balance:
	@echo "Checking Replica Token balance..."
	@WALLET_ADDRESS=$$(cast wallet address --private-key $$SUBNET_PRIVATE_KEY); \
	REPLICA_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplica'); \
	cast call $$REPLICA_ADDR "$$(cast calldata 'balanceOf(address)' $$WALLET_ADDRESS)" --rpc-url $$SUBNET_RPC_URL

withdraw-usdc:
	@echo "Withdrawing USDC from subnet..."
	@WALLET_ADDRESS=$$(cast wallet address --private-key $$SUBNET_PRIVATE_KEY); \
	REPLICA_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplica'); \
	cast send $$REPLICA_ADDR "linkedTransfer(address,uint256)" $$WALLET_ADDRESS $$AMOUNT --rpc-url $$SUBNET_RPC_URL --private-key $$SUBNET_PRIVATE_KEY

upgrade-replica-proxy:
	@echo "Upgrading token replica proxy contract on IPC... Please run make deploy-replica-implementation to deploy a new implementation and upgrade config.json before running this command."
	@LinkedTokenReplicaImplementation=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplicaImplementation'); \
	USDCTEST_ADDR=$$(cat config.json | jq -r '.LinkedToken.USDCTest'); \
	LinkedTokenReplicaProxy=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplica'); \
	LinkedTokenControllerProxy=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenController'); \
	forge script script/DeployIpcTokenReplica.s.sol:DeployIpcTokenReplica --skip-simulation --rpc-url $$SUBNET_RPC_URL --private-key $$SUBNET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "upgradeIpcTokenReplica(address,address,address,address,uint64,address[],address,string,string,uint8)" -- $$LinkedTokenReplicaProxy $$LinkedTokenReplicaImplementation $$SUBNET_GATEWAY $$USDCTEST_ADDR $$CALIBNET_CHAIN_ID '[]' $$LinkedTokenControllerProxy $$REPLICA_TOKEN_NAME $$REPLICA_TOKEN_SYMBOL $$REPLICA_TOKEN_DECIMALS

upgrade-controller-proxy:
	@echo "Upgrading token replica proxy contract on IPC... Please run make deploy-replica-implementation to deploy a new implementation and upgrade config.json before running this command."
	@LinkedTokenControllerImplementation=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenControllerImplementation'); \
	USDCTEST_ADDR=$$(cat config.json | jq -r '.LinkedToken.USDCTest'); \
	LinkedTokenReplicaProxy=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplica'); \
	LinkedTokenControllerProxy=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenController'); \
	forge script script/DeployIpcTokenController.s.sol:DeployIpcTokenController --skip-simulation --rpc-url $$CALIBNET_RPC_URL --private-key $$CALIBNET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "upgradeIpcTokenController(address,address,address,address,uint64,address[],address)" -- $$LinkedTokenControllerProxy $$LinkedTokenControllerImplementation $$CALIBNET_GATEWAY $$USDCTEST_ADDR $$CALIBNET_CHAIN_ID "[$$SUBNET_ROUTE_IN_ETH_FORMAT]" $$LinkedTokenReplicaProxy
