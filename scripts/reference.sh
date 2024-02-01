#!/bin/bash

# IPC Quick Start Script

# Exit on any error
set -e

# Step 1: Prepare your system
echo "Installing dependencies..."
sudo apt update && sudo apt install build-essential libssl-dev mesa-opencl-icd ocl-icd-opencl-dev gcc git bzr jq pkg-config curl clang hwloc libhwloc-dev wget ca-certificates gnupg -y

# Step 2: Initialise your config
echo "Initialising IPC config..."
ipc-cli config init

# Step 3: Set up your wallets
echo "Creating wallets..."
wallet_addresses=()
for i in {1..3}
do
    addr=$(ipc-cli wallet new -w evm)
    wallet_addresses+=($addr)
    echo "Wallet $i address: $addr"
done

# Set default wallet (Manual intervention required here)
# ipc-cli wallet set-default --address <DEFAULT_ETH_ADDR> -w evm

# Step 4: Create a child subnet
# Note: Requires manual entry of SUBNET_ID after creation
echo "Creating a child subnet..."
subnet_id=$(ipc-cli subnet create --parent /r314159 --min-validators 3 --min-validator-stake 1 --bottomup-check-period 30)
echo "Subnet ID: $subnet_id"

# Step 5: Join the subnet
# Note: This requires manual interaction to retrieve and use public keys
for addr in "${wallet_addresses[@]}"
do
    pubkey=$(ipc-cli wallet pub-key -w evm --address $addr)
    echo "Joining subnet with address $addr and public key $pubkey"
    ipc-cli subnet join --from=$addr --subnet=/r314159/$subnet_id --collateral=10 --public-key=$pubkey --initial-balance 1
done

# Step 6: Deploy the infrastructure
# Note: This step requires extensive manual setup and intervention. Scripting this part fully is complex and context-specific.
echo "Deploying infrastructure... (Manual steps required)"

# Step 7: Configure your subnet in the IPC CLI
# Note: Manual editing of ~/.ipc/config.toml is required

# Step 8: Interact with your the ETH RPC
# No specific script commands required

# Step 9 (optional): Run a relayer
# Uncomment below if you want to run a relayer
# ipc-cli checkpoint relayer --subnet $subnet_id
# ipc-cli checkpoint relayer --subnet $subnet_id --submitter <RELAYER_ADDR>

# Step 10: What now?
echo "Script complete. Please refer to further documentation for additional steps and configurations."