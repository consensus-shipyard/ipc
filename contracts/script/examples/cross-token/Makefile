# Makefile for USDC Token Bridge on Calibration Network

# Include environment variables from .env file
include .env
export

.PHONY: help deploy-usdctest mint-usdc check-balance deploy-replica deploy-controller update-replica deposit-usdc withdraw-usdc check-replica-balance

help:
	@echo "Available targets:"
	@echo "  deploy-usdctest        - Deploy the USDCTest contract on the Calibration Network."
	@echo "  mint-usdc              - Mint 1000 USDCTest tokens to the wallet specified in the .env file."
	@echo "  check-balance          - Check the balance of USDCTest tokens in the wallet."
	@echo "  deploy-replica         - Deploy the token replica contract on the subnet."
	@echo "  deploy-controller      - Deploy the token controller contract on the Calibration Network."
	@echo "  update-replica         - Update the replica with the token controller's address."
	@echo "  approve-usdc           - Approve USDC to allow for transfer to the subnet."
	@echo "  deposit-usdc           - Deposit USDC to the subnet."
	@echo "  check-replica-balance  - Check Replica Token balance."
	@echo "  withdraw-usdc          - Withdraw USDC from the subnet."
	@echo ""
	@echo "Usage:"
	@echo "  make <target>          - Run a specific target."
	@echo "  make help              - Display this help message."


deploy-usdctest:
	@echo "Deploying USDCTest contract..."
	forge script DeployUSDCTest.s.sol --broadcast --rpc-url $$CALIBNET_RPC_URL --private-key $$PRIVATE_KEY --skip-simulation

mint-usdc:
	@echo "Minting 1000 USDCTest tokens..."
	cast send $$CALIBNET_USDC "$$(cast calldata "mint(uint256)" $$AMOUNT)" --rpc-url $$CALIBNET_RPC_URL --private-key $$PRIVATE_KEY

check-balance:
	@echo "Checking wallet balance..."
	cast call $$CALIBNET_USDC "$$(cast calldata 'balanceOf(address)' $$PUBLIC_ADDRESS)" --rpc-url $$CALIBNET_RPC_URL

deploy-replica:
	@echo "Deploying token replica contract on subnet..."
	forge script DeployIpcTokenReplica.s.sol:DeployIpcTokenReplica --skip-simulation --rpc-url $$SUBNET_RPC_URL --private-key $$PRIVATE_KEY --broadcast -vvvv --ffi --sig "run(address,address,uint64,address[])" -- $$SUBNET_GATEWAY 0x0000000000000000000000000000000000000000 $$CALIBNET_CHAIN_ID '[]'

deploy-controller:
	@echo "Deploying token controller contract on Calibnet..."
	forge script DeployIpcTokenController.s.sol:DeployIpcTokenController --skip-simulation --rpc-url $$CALIBNET_RPC_URL --private-key $$PRIVATE_KEY --broadcast -vvvv --ffi --sig "run(address,address,uint64,address[],address)" -- $$CALIBNET_GATEWAY $$CALIBNET_USDC $$CALIBNET_CHAIN_ID "[$$SUBNET_ROUTE_IN_ETH_FORMAT]" $$IPC_TOKEN_REPLICA

update-replica:
	@echo "Updating replica with controller's address..."
	cast send $$IPC_TOKEN_REPLICA "setController(address)" $$IPC_TOKEN_CONTROLLER --rpc-url $$SUBNET_RPC_URL --private-key $$PRIVATE_KEY

approve-usdc:
	@echo "Depositing USDC to Controller Contract..."
	cast send $$CALIBNET_USDC "approve(address,uint256)" --rpc-url $$CALIBNET_RPC_URL --private-key $$PRIVATE_KEY -- $$IPC_TOKEN_CONTROLLER $$AMOUNT

deposit-usdc:
	@echo "Depositing USDC to subnet..."
	forge script CallLockAndTransfer.s.sol:CallLockAndTransfer --skip-simulation --rpc-url $$CALIBNET_RPC_URL --private-key $$PRIVATE_KEY --broadcast -vvvv --ffi --sig "run(address,address,uint256)" -- $$IPC_TOKEN_CONTROLLER $$PUBLIC_ADDRESS $$AMOUNT

check-replica-balance:
	@echo "Checking Replica Token balance..."
	cast call $$IPC_TOKEN_REPLICA "$$(cast calldata 'balanceOf(address)' $$PUBLIC_ADDRESS)" --rpc-url $$SUBNET_RPC_URL

withdraw-usdc:
	@echo "Withdrawing USDC from subnet..."
	# To withdraw 1,000 USDCTest tokens
	forge script CallBurnAndTransfer.s.sol:CallBurnAndTransfer --skip-simulation --rpc-url $$SUBNET_RPC_URL --private-key $$PRIVATE_KEY --broadcast -vvvv --ffi --sig "run(address,address,uint256)" -- $$IPC_TOKEN_REPLICA $$PUBLIC_ADDRESS $$AMOUNT

