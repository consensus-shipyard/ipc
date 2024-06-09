#!/bin/bash

set -euo pipefail

DASHES='------'
if [[ ! -v IPC_FOLDER ]]; then
    IPC_FOLDER=${HOME}/ipc
else
    IPC_FOLDER=${IPC_FOLDER}
fi
IPC_CONFIG_FOLDER=${HOME}/.ipc

CMT_P2P_HOST_PORTS=(26656 26756 26856)
CMT_RPC_HOST_PORTS=(26657 26757 26857)
ETHAPI_HOST_PORTS=(8545 8645 8745)
RESOLVER_HOST_PORTS=(26655 26755 26855)
PROXY_HOST_PORTS=(8001 8002 8003)
IPFS_SWARM_HOST_PORTS=(4001 4002 4003)
IPFS_RPC_HOST_PORTS=(5001 5002 5003)
IPFS_GATEWAY_HOST_PORTS=(8080 8081 8082)

# Use "dummy" subnet
subnet_id="/r314159/t410f726d2jv6uj4mpkcbgg5ndlpp3l7dd5rlcpgzkoi"
echo "Use existing subnet id: $subnet_id"
subnet_folder=$IPC_CONFIG_FOLDER/$(echo $subnet_id | sed 's|^/||;s|/|-|g')

# Step 1: Restart validators
# Step 1.1: Rebuild fendermint docker
echo "$DASHES Rebuild fendermint docker"
cd ${IPC_FOLDER}/fendermint
make clean
make docker-build

# Step 1.2: Start other validator node
echo "$DASHES Restart validator nodes"
cd ${IPC_FOLDER}
for i in {0..2}
do
  cargo make --makefile infra/fendermint/Makefile.toml \
      -e NODE_NAME=validator-${i} \
      -e PRIVATE_KEY_PATH=${IPC_CONFIG_FOLDER}/validator_${i}.sk \
      -e SUBNET_ID=${subnet_id} \
      -e CMT_P2P_HOST_PORT=${CMT_P2P_HOST_PORTS[i]} \
      -e CMT_RPC_HOST_PORT=${CMT_RPC_HOST_PORTS[i]} \
      -e ETHAPI_HOST_PORT=${ETHAPI_HOST_PORTS[i]} \
      -e RESOLVER_HOST_PORT=${RESOLVER_HOST_PORTS[i]} \
      -e PROXY_HOST_PORT=${PROXY_HOST_PORTS[i]} \
      -e IPFS_SWARM_HOST_PORT=${IPFS_SWARM_HOST_PORTS[i]} \
      -e IPFS_RPC_HOST_PORT=${IPFS_RPC_HOST_PORTS[i]} \
      -e IPFS_GATEWAY_HOST_PORT=${IPFS_GATEWAY_HOST_PORTS[i]} \
      -e IPFS_PROFILE="local-discovery" \
      -e FM_PULL_SKIP=1 \
      -e FM_LOG_LEVEL="info,fendermint=debug" \
      child-validator-restart-no-parent
done

# Step 2: Test
# Step 2.1: Test ETH API endpoint
echo "$DASHES Test ETH API endpoints of validator nodes"
for i in {0..2}
do
  curl --location http://localhost:${ETHAPI_HOST_PORTS[i]} \
  --header 'Content-Type: application/json' \
  --data '{
    "jsonrpc":"2.0",
    "method":"eth_blockNumber",
    "params":[],
    "id":83
  }'
done

# Step 2.2: Test proxy endpoint
printf "\n$DASHES Test proxy endpoints of validator nodes\n"
for i in {0..2}
do
  curl --location http://localhost:${PROXY_HOST_PORTS[i]}/health
done

# Step 3: Print a summary of the deployment
cat << EOF
############################
#                          #
# IPC deployment ready! 🚀 #
#                          #
############################
Subnet ID:
$subnet_id

Proxy API:
http://localhost:${PROXY_HOST_PORTS[0]}
http://localhost:${PROXY_HOST_PORTS[1]}
http://localhost:${PROXY_HOST_PORTS[2]}

IPFS API:
http://localhost:${IPFS_RPC_HOST_PORTS[0]}
http://localhost:${IPFS_RPC_HOST_PORTS[1]}
http://localhost:${IPFS_RPC_HOST_PORTS[2]}

ETH API:
http://localhost:${ETHAPI_HOST_PORTS[0]}
http://localhost:${ETHAPI_HOST_PORTS[1]}
http://localhost:${ETHAPI_HOST_PORTS[2]}

Accounts:
$(jq -r '.app_state.accounts[] | "\(.meta.Account.owner): \(.balance) coin units"' ${subnet_folder}/validator-0/genesis.json)

Private keys (hex ready to import in MetaMask):
$(cat ${IPC_CONFIG_FOLDER}/validator_0.sk | base64 -d | xxd -p -c 1000000)
$(cat ${IPC_CONFIG_FOLDER}/validator_1.sk | base64 -d | xxd -p -c 1000000)
$(cat ${IPC_CONFIG_FOLDER}/validator_2.sk | base64 -d | xxd -p -c 1000000)

Chain ID:
$(curl -s --location --request POST 'http://localhost:8645/' --header 'Content-Type: application/json' --data-raw '{ "jsonrpc":"2.0", "method":"eth_chainId", "params":[], "id":1 }' | jq -r '.result' | xargs printf "%d")

Fendermint API:
http://localhost:26658

CometBFT API:
http://localhost:${CMT_RPC_HOST_PORTS[0]}
EOF
