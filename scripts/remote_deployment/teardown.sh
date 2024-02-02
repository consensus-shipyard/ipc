#!/bin/bash

# Teardown the deployment created by deploy.sh

# Exit on any error
set -e

# Print commands as we execute
set -x

PREFIX='------'
IPC_FOLDER=${HOME}/ipc
IPC_CLI=${IPC_FOLDER}/target/release/ipc-cli
IPC_CONFIG_FOLDER=${HOME}/.ipc

CMT_P2P_HOST_PORTS=(26656 26756 26856)
CMT_RPC_HOST_PORTS=(26657 26757 26857)
ETHAPI_HOST_PORTS=(8545 8645 8745)
RESOLVER_HOST_PORTS=(26655 26755 26855)

subnet_id=$(cat ~/running_subnet_id)
bootstrap_node_id=$(cat ~/running_bootstrap_node_id)

# Step 1: Read parent net gateway address and registry address
echo "$PREFIX Reading parent gateway and registry address"
parent_gateway_address=$(toml get ${IPC_CONFIG_FOLDER}/config.toml subnets[0].config.gateway_addr | tr -d '"')
parent_registry_address=$(toml get ${IPC_CONFIG_FOLDER}/config.toml subnets[0].config.registry_addr | tr -d '"')

# Step 2: Teardown the bootstrap validator node
echo "$PREFIX Teardown the bootstrap validator node"
cd ${IPC_FOLDER}
cargo make --makefile infra/fendermint/Makefile.toml \
        -e NODE_NAME=validator-1 \
        -e PRIVATE_KEY_PATH=${IPC_CONFIG_FOLDER}/validator_1.sk \
        -e SUBNET_ID=${subnet_id} \
        -e CMT_P2P_HOST_PORT=${CMT_P2P_HOST_PORTS[0]} \
        -e CMT_RPC_HOST_PORT=${CMT_RPC_HOST_PORTS[0]} \
        -e ETHAPI_HOST_PORT=${ETHAPI_HOST_PORTS[0]} \
        -e RESOLVER_HOST_PORT=${RESOLVER_HOST_PORTS[0]} \
        -e PARENT_REGISTRY=${parent_registry_address} \
        -e PARENT_GATEWAY=${parent_gateway_address} \
        -e FM_PULL_SKIP=1 \
        child-validator-down

# Step 3: Teardown other validator nodes
echo "$PREFIX Start the other validator nodes"
cd ${IPC_FOLDER}
for i in {1..2}
do
  cargo make --makefile infra/fendermint/Makefile.toml \
      -e NODE_NAME=validator-$(($i+1)) \
      -e PRIVATE_KEY_PATH=${IPC_CONFIG_FOLDER}/validator_1.sk \
      -e SUBNET_ID=${subnet_id} \
      -e CMT_P2P_HOST_PORT=${CMT_P2P_HOST_PORTS[i]} \
      -e CMT_RPC_HOST_PORT=${CMT_RPC_HOST_PORTS[i]} \
      -e ETHAPI_HOST_PORT=${ETHAPI_HOST_PORTS[i]} \
      -e RESOLVER_HOST_PORT=${RESOLVER_HOST_PORTS[i]} \
      -e BOOTSTRAPS=${bootstrap_node_id} \
      -e PARENT_REGISTRY=${parent_registry_address} \
      -e PARENT_GATEWAY=${parent_gateway_address} \
      -e FM_PULL_SKIP=1 \
      child-validator-down
done
