#!/bin/bash

set -euo pipefail

if [[ ! -v IPC_FOLDER ]]; then
    IPC_FOLDER="$PWD"
fi
IPC_CONFIG_FOLDER=${HOME}/.ipc

# Use "dummy" subnet
subnet_id="/r314159/t410f726d2jv6uj4mpkcbgg5ndlpp3l7dd5rlcpgzkoi"
subnet_folder=$IPC_CONFIG_FOLDER/$(echo $subnet_id | sed 's|^/||;s|/|-|g')
rm -rf "$subnet_folder"

# Stop validators
cd "$IPC_FOLDER"
for i in {0..2}
do
  cargo make --makefile infra/fendermint/Makefile.toml \
      -e NODE_NAME=validator-"$i" \
      -e SUBNET_ID="$subnet_id" \
      child-validator-down
done
