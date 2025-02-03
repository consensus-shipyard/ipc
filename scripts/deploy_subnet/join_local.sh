#!/bin/bash

set -euo pipefail

DASHES='------'

if [[ -z "${BOOTSTRAP_NODE_ID:-}" ]]; then
  echo "BOOTSTRAP_NODE_ID is not set"
  exit 1
fi

if [[ -z "${BOOTSTRAP_PEER_ID:-}" ]]; then
  echo "BOOTSTRAP_PEER_ID is not set"
  exit 1
fi

subnet_id="/r31337/t410f6gbdxrbehnaeeo4mrq7wc5hgq6smnefys4qanwi"
subnet_eth_addr="0x56639dB16Ac50A89228026e42a316B30179A5376"
validator_addr="0x90f79bf6eb2c4f870365e785982e1f101e93b906"

PARENT_GATEWAY_ADDRESS="0x9A676e781A523b5d0C0e43731313A708CB607508"
PARENT_REGISTRY_ADDRESS="0x4ed7c70F96B99c776995fB64377f0d4aB3B0e1C1"
SUPPLY_SOURCE_ADDRESS="0xa85233C63b9Ee964Add6F2cffe00Fd84eb32338f"
VALIDATOR_GATER_ADDRESS="0x851356ae760d987E095750cCeb3bC6014560891C"

dir=$(dirname -- "$(readlink -f -- "${BASH_SOURCE[0]}")")
IPC_FOLDER=$(readlink -f -- "$dir"/../..)
IPC_CONFIG_FOLDER=${HOME}/.ipc

CMT_P2P_HOST_PORT=26956
CMT_RPC_HOST_PORT=26957
ETHAPI_HOST_PORT=8945
RESOLVER_HOST_PORT=26955
OBJECTS_HOST_PORT=8004
IROH_RPC_HOST_PORT=4924

FENDERMINT_METRICS_HOST_PORT=9188
IROH_METRICS_HOST_PORT=9094
PROMTAIL_AGENT_HOST_PORT=9083

ANVIL_HOST_PORT=8545
PARENT_ENDPOINT="http://anvil:${ANVIL_HOST_PORT}"

# Prepare code repo
git submodule sync
git submodule update --init --recursive

echo "New validator address: $validator_addr"

# Export validator private key into files
ipc-cli wallet export --wallet-type evm --address "$validator_addr" --hex > "${IPC_CONFIG_FOLDER}"/validator_3.sk
echo "Export private key for $validator_addr to ${IPC_CONFIG_FOLDER}/validator_3.sk"

echo "$DASHES Approve new validator to stake $DASHES"
pk=$(cat "${IPC_CONFIG_FOLDER}"/validator_0.sk)
cd "${IPC_FOLDER}/recall-contracts"
# approved power min 1 RECALL max 10 RECALL
cast send "$VALIDATOR_GATER_ADDRESS" "approve(address,uint256,uint256)" "$validator_addr" 1000000000000000000 100000000000000000000 --rpc-url "http://localhost:${ANVIL_HOST_PORT}" --private-key "$pk"
echo "Approved new validator to stake on anvil rootnet"
cd "$IPC_FOLDER"

echo "$DASHES Join subnet for new validator $DASHES"
echo "Joining subnet ${subnet_id} for validator $validator_addr}"
# Approve subnet contract to lock up to 10 RECALL from collateral contract (which is also the supply source contract)
vpk=$(cat "${IPC_CONFIG_FOLDER}"/validator_3.sk)
cast send "$SUPPLY_SOURCE_ADDRESS" "approve(address,uint256)" "$subnet_eth_addr" 10000000000000000000 --private-key "$vpk"
# Join and stake 10 RECALL
ipc-cli subnet join --from "$validator_addr" --subnet "$subnet_id" --collateral 10

bootstrap_node_endpoint=${BOOTSTRAP_NODE_ID}@validator-0-cometbft:26656
echo "Bootstrap node endpoint: ${bootstrap_node_endpoint}"
bootstrap_resolver_endpoint="/dns/validator-0-fendermint/tcp/26655/p2p/${BOOTSTRAP_PEER_ID}"
echo "Bootstrap resolver endpoint: ${bootstrap_resolver_endpoint}"

# Force a wait to make sure the subnet is confirmed as created in the parent contracts
echo "Wait for validator changes..."
sleep 30
echo "Finished waiting"

# Start new validator node
echo "$DASHES Start new validator node"
cd "${IPC_FOLDER}"
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=validator-3 \
    -e PRIVATE_KEY_PATH="${IPC_CONFIG_FOLDER}"/validator_3.sk \
    -e SUBNET_ID="${subnet_id}" \
    -e PARENT_ENDPOINT="${PARENT_ENDPOINT}" \
    -e CMT_P2P_HOST_PORT="${CMT_P2P_HOST_PORT}" \
    -e CMT_RPC_HOST_PORT="${CMT_RPC_HOST_PORT}" \
    -e ETHAPI_HOST_PORT="${ETHAPI_HOST_PORT}" \
    -e RESOLVER_HOST_PORT="${RESOLVER_HOST_PORT}" \
    -e OBJECTS_HOST_PORT="${OBJECTS_HOST_PORT}" \
    -e IROH_RPC_HOST_PORT="${IROH_RPC_HOST_PORT}" \
    -e FENDERMINT_METRICS_HOST_PORT="${FENDERMINT_METRICS_HOST_PORT}" \
    -e IROH_METRICS_HOST_PORT="${IROH_METRICS_HOST_PORT}" \
    -e PROMTAIL_AGENT_HOST_PORT="${PROMTAIL_AGENT_HOST_PORT}" \
    -e PROMTAIL_CONFIG_FOLDER="${IPC_CONFIG_FOLDER}" \
    -e IROH_CONFIG_FOLDER="${IPC_FOLDER}/infra/iroh/" \
    -e RESOLVER_BOOTSTRAPS="${bootstrap_resolver_endpoint}" \
    -e BOOTSTRAPS="${bootstrap_node_endpoint}" \
    -e PARENT_HTTP_AUTH_TOKEN="" \
    -e PARENT_AUTH_FLAG="" \
    -e PARENT_REGISTRY="${PARENT_REGISTRY_ADDRESS}" \
    -e PARENT_GATEWAY="${PARENT_GATEWAY_ADDRESS}" \
    -e FM_PULL_SKIP=1 \
    child-validator

echo "Done"
exit 0
