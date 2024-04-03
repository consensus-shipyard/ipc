# Makefile for IPC Token Replica Bridge

# Include environment variables from .env file
-include .env
export

.PHONY: help fmt deploy-usdctest mint-usdc check-balance deploy-replica deploy-controller initialize-replica initialize-controller approve-token deposit withdraw check-replica-balance

help:
	@echo "Available targets:"
	@echo "  install                                - Install dependencies."
	@echo "  deploy-usdctest                        - Deploy the USDCTest contract to be used as the Original Token."
	@echo "  mint-usdc                              - Mint 1000 USDCTest tokens to the specified wallet in .env file."
	@echo "  check-balance                          - Check the Original Token token balance in the wallet."
	@echo "  deploy-replica-implementation          - Deploy the token replica contract implementation."
	@echo "  deploy-replica-proxy                   - Deploy the token replica proxy."
	@echo "  deploy-controller-implementation       - Deploy the token controller contract implementation."
	@echo "  deploy-controller-proxy                - Deploy the token controller proxy."
	@echo "  link-replica                           - Link the replica with the token controller address."
	@echo "  link-controller                        - Link the controller with the token replica address."
	@echo "  approve-token                          - Approve Original Token for transfer."
	@echo "  deposit-token                          - Deposit Original Token to the subnet."
	@echo "  check-replica-balance                  - Check the token replica balance."
	@echo "  withdraw-token                         - Withdraw Original Token from the subnet."
	@echo "  deploy-replica-implementation-v2       - Deploy the token replica contract implementation v2 for upgrading. Overwrites the config file with the new implementation address."
	@echo "  deploy-controller-implementation-v2    - Deploy the token controller contract implementation v2 for upgrading. Overwrites the config file with the new implementation address."
	@echo "  upgrade-replica-proxy                  - Upgrade the token replica proxy to a new implementation."
	@echo "  upgrade-controller-proxy               - Upgrade the token controller proxy to a new implementation."
	@echo ""
	@echo "Usage:"
	@echo "  make <target>                          - Run a specific make target."
	@echo "  make help                              - Display this help message."

fmt:
	npm install --silent --no-save
	npx prettier --check -w 'src/**/**/*.sol' 'script/**/**/*.sol' 'test/**/**/*.sol' 'test/**/**/*.t.sol' '**/*.{js,jsx,ts,tsx,json,css}'

install:
	@echo "Installing dependencies..."
	forge install

deploy-usdctest:
	@echo "Deploying USDCTest contract..."
	forge script script/DeployUSDCTest.s.sol:DeployUSDCTest --broadcast --rpc-url $$ORIGIN_NET_RPC_URL --private-key $$ORIGIN_NET_PRIVATE_KEY --skip-simulation -vvv

mint-usdc:
	@echo "Minting 1000 USDCTest tokens..."
	@USDCTEST_ADDR=$$(cat config.json | jq -r '.LinkedToken.OriginalToken'); \
    cast send $$USDCTEST_ADDR "$$(cast calldata "mint(uint256)" $$AMOUNT)" --rpc-url $$ORIGIN_NET_RPC_URL --private-key $$ORIGIN_NET_PRIVATE_KEY

check-balance:
	@echo "Checking wallet balance..."
	@WALLET_ADDRESS=$$(cast wallet address --private-key $$ORIGIN_NET_PRIVATE_KEY); \
	USDCTEST_ADDR=$$(cat config.json | jq -r '.LinkedToken.OriginalToken'); \
	cast call $$USDCTEST_ADDR "$$(cast calldata 'balanceOf(address)' $$WALLET_ADDRESS)" --rpc-url $$ORIGIN_NET_RPC_URL

deploy-replica-implementation:
	@echo "Deploying token replica contract on subnet..."
	forge script script/DeployIpcTokenReplica.s.sol:DeployIpcTokenReplica --skip-simulation --rpc-url $$SUBNET_RPC_URL --private-key $$SUBNET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "deployIpcTokenReplica()"

deploy-replica-proxy:
	@echo "Deploying token replica proxy contract on Origin Net..."
	@LinkedTokenReplicaImplementation=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplicaImplementation'); \
	USDCTEST_ADDR=$$(cat config.json | jq -r '.LinkedToken.OriginalToken'); \
	forge script script/DeployIpcTokenReplica.s.sol:DeployIpcTokenReplica --skip-simulation --rpc-url $$SUBNET_RPC_URL --private-key $$SUBNET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "deployIpcTokenReplicaProxy(address,address,address,uint64,address[],string,string,uint8)" -- $$LinkedTokenReplicaImplementation $$SUBNET_GATEWAY $$USDCTEST_ADDR $$ORIGIN_NET_CHAIN_ID '[]' $$REPLICA_TOKEN_NAME $$REPLICA_TOKEN_SYMBOL $$REPLICA_TOKEN_DECIMALS


deploy-controller-implementation:
	@echo "Deploying token controller implementation contract on Origin Net..."
	forge script script/DeployIpcTokenController.s.sol:DeployIpcTokenController --skip-simulation --rpc-url $$ORIGIN_NET_RPC_URL --private-key $$ORIGIN_NET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "deployIpcTokenController()"

deploy-controller-proxy:
	@echo "Deploying token controller proxy contract on Origin Net..."
	@LinkedTokenControllerImplementation=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenControllerImplementation'); \
	USDCTEST_ADDR=$$(cat config.json | jq -r '.LinkedToken.OriginalToken'); \
	forge script script/DeployIpcTokenController.s.sol:DeployIpcTokenController --skip-simulation --rpc-url $$ORIGIN_NET_RPC_URL --private-key $$ORIGIN_NET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "deployIpcTokenControllerProxy(address,address,address,uint64,address[])" -- $$LinkedTokenControllerImplementation $$ORIGIN_NET_GATEWAY $$USDCTEST_ADDR $$ORIGIN_NET_CHAIN_ID "[$$SUBNET_ROUTE_IN_ETH_FORMAT]"

link-controller:
	@echo "Updating controller with replica's address..."
	@CONTROLLER_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenControllerProxy'); \
	REPLICA_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplicaProxy'); \
	cast send $$CONTROLLER_ADDR "setLinkedContract(address)" --rpc-url $$ORIGIN_NET_RPC_URL --private-key $$ORIGIN_NET_PRIVATE_KEY --  $$REPLICA_ADDR

link-replica:
	@echo "Updating controller with replica's address..."
	@CONTROLLER_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenControllerProxy'); \
	REPLICA_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplicaProxy'); \
	cast send $$REPLICA_ADDR "setLinkedContract(address)" --rpc-url $$SUBNET_RPC_URL --private-key $$SUBNET_PRIVATE_KEY -- $$CONTROLLER_ADDR


approve-usdc:
	@echo "Approving USDC for Controller Contract..."
	@CONTROLLER_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenControllerProxy'); \
	USDCTEST_ADDR=$$(cat config.json | jq -r '.LinkedToken.OriginalToken'); \
	cast send $$USDCTEST_ADDR "approve(address,uint256)" --rpc-url $$ORIGIN_NET_RPC_URL --private-key $$ORIGIN_NET_PRIVATE_KEY -- $$CONTROLLER_ADDR $$AMOUNT

deposit-usdc:
	@echo "Depositing USDC to subnet..."
	@WALLET_ADDRESS=$$(cast wallet address --private-key $$ORIGIN_NET_PRIVATE_KEY); \
	CONTROLLER_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenControllerProxy'); \
	cast send $$CONTROLLER_ADDR "linkedTransfer(address,uint256)" $$WALLET_ADDRESS $$AMOUNT --rpc-url $$ORIGIN_NET_RPC_URL --private-key $$ORIGIN_NET_PRIVATE_KEY

check-replica-balance:
	@echo "Checking Replica Token balance..."
	@WALLET_ADDRESS=$$(cast wallet address --private-key $$SUBNET_PRIVATE_KEY); \
	REPLICA_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplicaProxy'); \
	cast call $$REPLICA_ADDR "$$(cast calldata 'balanceOf(address)' $$WALLET_ADDRESS)" --rpc-url $$SUBNET_RPC_URL

withdraw-usdc:
	@echo "Withdrawing USDC from subnet..."
	@WALLET_ADDRESS=$$(cast wallet address --private-key $$SUBNET_PRIVATE_KEY); \
	REPLICA_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplicaProxy'); \
	cast send $$REPLICA_ADDR "linkedTransfer(address,uint256)" $$WALLET_ADDRESS $$AMOUNT --rpc-url $$SUBNET_RPC_URL --private-key $$SUBNET_PRIVATE_KEY

deploy-replica-implementation:
	@echo "Deploying token replica contract v2 on subnet..."
	forge script script/DeployIpcTokenReplica.s.sol:DeployIpcTokenReplica --skip-simulation --rpc-url $$SUBNET_RPC_URL --private-key $$SUBNET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "deployIpcTokenReplicaV2()"

deploy-controller-implementation:
	@echo "Deploying token controller v2 implementation contract on Origin Net..."
	forge script script/DeployIpcTokenController.s.sol:DeployIpcTokenController --skip-simulation --rpc-url $$ORIGIN_NET_RPC_URL --private-key $$ORIGIN_NET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "deployIpcTokenControllerV2()"


upgrade-replica-proxy:
	@echo "Upgrading token replica proxy contract on IPC... Please run make deploy-replica-implementation-v2 to deploy a new implementation and upgrade config.json before running this command."
	@LinkedTokenReplicaImplementation=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplicaImplementation'); \
	USDCTEST_ADDR=$$(cat config.json | jq -r '.LinkedToken.OriginalToken'); \
	LinkedTokenReplicaProxy=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplicaProxy'); \
	LinkedTokenControllerProxy=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenControllerProxy'); \
	forge script script/DeployIpcTokenReplica.s.sol:DeployIpcTokenReplica --skip-simulation --rpc-url $$SUBNET_RPC_URL --private-key $$SUBNET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "upgradeIpcTokenReplica(address,address,address,address,uint64,address[],address,string,string,uint8)" -- $$LinkedTokenReplicaProxy $$LinkedTokenReplicaImplementation $$SUBNET_GATEWAY $$USDCTEST_ADDR $$ORIGIN_NET_CHAIN_ID '[]' $$LinkedTokenControllerProxy $$REPLICA_TOKEN_NAME $$REPLICA_TOKEN_SYMBOL $$REPLICA_TOKEN_DECIMALS

upgrade-controller-proxy:
	@echo "Upgrading token replica proxy contract on IPC... Please run make deploy-replica-implementation-v2 to deploy a new implementation and upgrade config.json before running this command."
	@LinkedTokenControllerImplementation=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenControllerImplementation'); \
	USDCTEST_ADDR=$$(cat config.json | jq -r '.LinkedToken.OriginalToken'); \
	LinkedTokenReplicaProxy=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplicaProxy'); \
	LinkedTokenControllerProxy=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenControllerProxy'); \
	forge script script/DeployIpcTokenController.s.sol:DeployIpcTokenController --skip-simulation --rpc-url $$ORIGIN_NET_RPC_URL --private-key $$ORIGIN_NET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "upgradeIpcTokenController(address,address,address,address,uint64,address[],address)" -- $$LinkedTokenControllerProxy $$LinkedTokenControllerImplementation $$ORIGIN_NET_GATEWAY $$USDCTEST_ADDR $$ORIGIN_NET_CHAIN_ID "[$$SUBNET_ROUTE_IN_ETH_FORMAT]" $$LinkedTokenReplicaProxy
