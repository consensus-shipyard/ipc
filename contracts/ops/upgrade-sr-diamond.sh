#!/bin/bash
# Upgrades IPC Subnet Registry Diamond Facets on an EVM-compatible subnet using hardhat
set -eu
set -o pipefail

if [ $# -ne 1 ]
then
    echo "Expected a single argument with the name of the network to deploy (localnet, calibrationnet, mainnet)"
    exit 1
fi

NETWORK="$1"

if [ "$NETWORK" = "auto" ]; then
  echo "[*] Automatically getting chainID for network"
  source ops/chain-id.sh
fi


pnpm exec hardhat upgrade-sr-diamond --network "${NETWORK}"
