# Makefile for USDC Token Bridge on Calibration Network

# Include environment variables from .env file
include .env
export

.PHONY: help deploy-usdctest mint-usdc check-balance deploy-replica-implementation deploy-replica-proxy deploy-controller-implementation deploy-controller-proxy initialize-replica initialize-controller deposit-usdc withdraw-usdc check-replica-balance

help:
	@echo "Available targets:"
	@echo "  install                              - Install dependencies."
	@echo "  deploy-usdctest                      - Deploy the USDCTest contract on the Calibration Network."
	@echo "  mint-usdc                            - Mint 1000 USDCTest tokens to the wallet specified in the .env file."
	@echo "  check-balance                        - Check the balance of USDCTest tokens in the wallet."
	@echo "  deploy-replica-implementation        - Deploy the token replica contract implementation on the subnet."
	@echo "  deploy-replica-proxy                 - Deploy the token replica contract proxy on the subnet."
	@echo "  deploy-controller-implementation     - Deploy the token controller contract on the Calibration Network."
	@echo "  deploy-controller-proxy              - Deploy the token controller contract on the Calibration Network."
	@echo "  initialize-replica                   - Initialize the replica with the token controller's address."
	@echo "  initialize-controller                - Initialize the controller with the token replica's address."
	@echo "  approve-usdc                         - Approve USDC to allow for transfer to the subnet."
	@echo "  deposit-usdc                         - Deposit USDC to the subnet."
	@echo "  check-replica-balance                - Check Replica Token balance."
	@echo "  withdraw-usdc                        - Withdraw USDC from the subnet."
	@echo "  upgrade-controller-proxy             - Upgrade the token controller proxy to a new implementation. \n\t\t\t\tPlease run make deploy-controller-implementation to deploy a new implementation and upgrade config.json before running this command."
	@echo "  upgrade-replica-proxy                - Upgrade the token replica proxy to a new implementation. \n\t\t\t\tPlease run make deploy-replica-implementation to deploy a new implementation and upgrade config.json before running this command."
	@echo ""
	@echo "Usage:"
	@echo "  make <target>                        - Run a specific target."
	@echo "  make help                            - Display this help message."


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
	forge script script/DeployIpcTokenReplica.s.sol:DeployIpcTokenReplica --skip-simulation --rpc-url $$SUBNET_RPC_URL --private-key $$SUBNET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "deployIpcTokenReplicaProxy(address)" -- $$LinkedTokenReplicaImplementation

deploy-controller-implementation:
	@echo "Deploying token controller implementation contract on Calibnet..."
	forge script script/DeployIpcTokenController.s.sol:DeployIpcTokenController --skip-simulation --rpc-url $$CALIBNET_RPC_URL --private-key $$CALIBNET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "deployIpcTokenController()"

deploy-controller-proxy:
	@echo "Deploying token controller proxy contract on Calibnet..."
	@LinkedTokenControllerImplementation=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenControllerImplementation'); \
	forge script script/DeployIpcTokenController.s.sol:DeployIpcTokenController --skip-simulation --rpc-url $$CALIBNET_RPC_URL --private-key $$CALIBNET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "deployIpcTokenControllerProxy(address)" -- $$LinkedTokenControllerImplementation

initialize-replica:
	@echo "Updating replica with controller's address..."
	@REPLICA_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplica'); \
	USDCTEST_ADDR=$$(cat config.json | jq -r '.LinkedToken.USDCTest'); \
	CONTROLLER_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenController'); \
	forge script script/DeployIpcTokenReplica.s.sol:DeployIpcTokenReplica --skip-simulation --rpc-url $$SUBNET_RPC_URL --private-key $$SUBNET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "initializeIpcTokenReplica(address,address,address,uint64,address[],address)" -- $$REPLICA_ADDR $$SUBNET_GATEWAY $$USDCTEST_ADDR $$CALIBNET_CHAIN_ID '[]' $$CONTROLLER_ADDR

initialize-controller:
	@echo "Updating controller with replica's address..."
	@CONTROLLER_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenController'); \
	USDCTEST_ADDR=$$(cat config.json | jq -r '.LinkedToken.USDCTest'); \
	REPLICA_ADDR=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplica'); \
	forge script script/DeployIpcTokenController.s.sol:DeployIpcTokenController --skip-simulation --rpc-url $$CALIBNET_RPC_URL --private-key $$CALIBNET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "initializeIpcTokenController(address,address,address,uint64,address[],address)" -- $$CONTROLLER_ADDR $$CALIBNET_GATEWAY $$USDCTEST_ADDR $$CALIBNET_CHAIN_ID "[$$SUBNET_ROUTE_IN_ETH_FORMAT]" $$REPLICA_ADDR

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

upgrade-controller-proxy:
	@echo "Upgrading token controller proxy contract on Calibnet... Please run make deploy-controller-implementation to deploy a new implementation and upgrade config.json before running this command."
	@LinkedTokenControllerImplementation=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenControllerImplementation'); \
	LinkedTokenControllerProxy=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenController'); \
	forge script script/DeployIpcTokenController.s.sol:DeployIpcTokenController --skip-simulation --rpc-url $$CALIBNET_RPC_URL --private-key $$CALIBNET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "upgradeIpcTokenController(address,address)" -- $$LinkedTokenControllerProxy $$LinkedTokenControllerImplementation

upgrade-replica-proxy:
	@echo "Upgrading token replica proxy contract on IPC... Please run make deploy-replica-implementation to deploy a new implementation and upgrade config.json before running this command."
	@LinkedTokenReplicaImplementation=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplicaImplementation'); \
	LinkedTokenReplicaProxy=$$(cat config.json | jq -r '.LinkedToken.LinkedTokenReplica'); \
	forge script script/DeployIpcTokenReplica.s.sol:DeployIpcTokenReplica --skip-simulation --rpc-url $$CALIBNET_RPC_URL --private-key $$CALIBNET_PRIVATE_KEY --broadcast -vvvv --ffi --sig "upgradeIpcTokenReplica(address,addrese)" -- $$LinkedTokenReplicaProxy $$LinkedTokenReplicaImplementation
