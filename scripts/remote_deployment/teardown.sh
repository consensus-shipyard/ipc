#!/bin/bash

# Teardown the deployment created by deploy.sh

# Exit on any error
set -e

# Print commands as we execute
set -x

PREFIX='------'
IPC_FOLDER=${HOME}/ipc

subnet_id=$(cat ~/running_subnet_id)

# Step 1: Teardown the bootstrap validator node
echo "$PREFIX Teardown the bootstrap validator node"
cd ${IPC_FOLDER}
cargo make --makefile infra/fendermint/Makefile.toml \
        -e NODE_NAME=validator-1 \
        -e SUBNET_ID=${subnet_id} \
        -e FM_PULL_SKIP=1 \
        bootstrap-down

# Step 2: Teardown other validator nodes
echo "$PREFIX Start the other validator nodes"
cd ${IPC_FOLDER}
for i in {1..2}
do
  cargo make --makefile infra/fendermint/Makefile.toml \
      -e NODE_NAME=validator-$(($i+1)) \
      -e SUBNET_ID=${subnet_id} \
      -e FM_PULL_SKIP=1 \
      child-validator-down
done
