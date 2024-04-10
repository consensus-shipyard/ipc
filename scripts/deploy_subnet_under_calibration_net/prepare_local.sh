#!/bin/bash

# A script which resets your local .ipc folder so we can run the deploy.sh script for a local deployment
#
# To run:
# 1. Run `./prepare_local.sh`
# 2. Run `./deploy.sh`
#
# Known issues:
# - The deploy.sh script does not run "make docker-build" which can cause errors if its outdated on your
#    machine. To fix this, run `make docker-build` in ipc/fendermint folder and rerun the deploy.sh script

set -eo pipefail

SCRIPT_DIR=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
echo "SCRIPT_DIR: $SCRIPT_DIR"

echo "Installing dependencies..."
cargo install toml-cli

echo "Removing existing .ipc folder"
rm -rf ~/.ipc

echo "Copying new .ipc folder"
cp -r $SCRIPT_DIR/.ipc ~/.ipc

echo "Fetching info about newest current deployment contracts"
hash=$(curl -s https://raw.githubusercontent.com/consensus-shipyard/ipc/cd/contracts/deployments/r314159.json)
echo $hash | jq

echo "Parsing gateway and registry address..."
gateway_addr=$(echo $hash | jq -r '.gateway_addr')
echo "- Gateway address: $gateway_addr"
registry_addr=$(echo $hash | jq -r '.registry_addr')
echo "- Registry address: $registry_addr"

echo "Updating config with new gateway and registry address"
toml set ~/.ipc/config.toml subnets[0].config.gateway_addr $gateway_addr > ~/.ipc/config.toml.tmp
toml set ~/.ipc/config.toml.tmp subnets[0].config.registry_addr $registry_addr > ~/.ipc/config.toml.tmp2
cp ~/.ipc/config.toml.tmp2 ~/.ipc/config.toml

echo "Setting up wallets"
wallet1=$(cargo run -q -p ipc-cli --release -- wallet new --wallet-type evm | tr -d '"')
echo "- Wallet1: $wallet1"
wallet2=$(cargo run -q -p ipc-cli --release -- wallet new --wallet-type evm | tr -d '"')
echo "- Wallet2: $wallet2"
wallet3=$(cargo run -q -p ipc-cli --release -- wallet new --wallet-type evm | tr -d '"')
echo "- Wallet3: $wallet3"

echo "--- GO TO Calibration faucet (https://faucet.calibnet.chainsafe-fil.io/) and get some tokens for the wallets ---"
