#!/bin/bash

set -euo pipefail

dir=$(dirname -- "$(readlink -f -- "${BASH_SOURCE[0]}")")
IPC_FOLDER="$dir"/../..
IPC_CONFIG_FOLDER=${HOME}/.ipc

CMT_P2P_HOST_PORTS=(26656 26756 26856)
CMT_RPC_HOST_PORTS=(26657 26757 26857)
ETHAPI_HOST_PORTS=(8545 8645 8745)
RESOLVER_HOST_PORTS=(26655 26755 26855)
OBJECTS_HOST_PORTS=(8001 8002 8003)
IROH_RPC_HOST_PORTS=(4921 4922 4923)

FENDERMINT_METRICS_HOST_PORTS=(9184 9185 9186)
IROH_METRICS_HOST_PORTS=(9091 9092 9093)
PROMTAIL_AGENT_HOST_PORTS=(9080 9081 9082)

PROMETHEUS_HOST_PORT=9090
LOKI_HOST_PORT=3100
GRAFANA_HOST_PORT=3000

# Use "dummy" subnet
subnet_id="/r314159/t410f726d2jv6uj4mpkcbgg5ndlpp3l7dd5rlcpgzkoi"
subnet_folder=$IPC_CONFIG_FOLDER/$(echo $subnet_id | sed 's|^/||;s|/|-|g')
rm -rf "$subnet_folder"

# Build IPC contracts
cd "$IPC_FOLDER"/contracts
make gen

# Rebuild fendermint docker
cd "$IPC_FOLDER"/fendermint
make clean
make docker-build

# Prepare wallet by using existing wallet json file
wallet_addresses=()
for i in {0..2}
do
  addr=$(jq .["$i"].address "$IPC_CONFIG_FOLDER"/evm_keystore.json | tr -d '"')
  wallet_addresses+=("$addr")
done

# Export validator private keys into files
for i in {0..2}
do
  ipc-cli wallet export --wallet-type evm --address "${wallet_addresses[i]}" --hex > "$IPC_CONFIG_FOLDER"/validator_"$i".sk
done

# Init validators
cd "$IPC_FOLDER"
for i in {0..2}
do
  cargo make --makefile infra/fendermint/Makefile.toml \
      -e NODE_NAME=validator-"$i" \
      -e SUBNET_ID="$subnet_id" \
      -e FM_PULL_SKIP=1 \
      child-validator-no-parent-init
done

# Copy genesis file into each validator
for i in {0..2}
do
  cp "$IPC_CONFIG_FOLDER"/genesis.json "$subnet_folder"/validator-"$i"
done

# Start bootstrap validator
bootstrap_output=$(cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=validator-0 \
    -e PRIVATE_KEY_PATH="$IPC_CONFIG_FOLDER"/validator_0.sk \
    -e SUBNET_ID="$subnet_id" \
    -e CMT_P2P_HOST_PORT="${CMT_P2P_HOST_PORTS[0]}" \
    -e CMT_RPC_HOST_PORT="${CMT_RPC_HOST_PORTS[0]}" \
    -e ETHAPI_HOST_PORT="${ETHAPI_HOST_PORTS[0]}" \
    -e RESOLVER_HOST_PORT="${RESOLVER_HOST_PORTS[0]}" \
    -e OBJECTS_HOST_PORT="${OBJECTS_HOST_PORTS[0]}" \
    -e IROH_RPC_HOST_PORT="${IROH_RPC_HOST_PORTS[0]}" \
    -e FENDERMINT_METRICS_HOST_PORT="${FENDERMINT_METRICS_HOST_PORTS[0]}" \
    -e IROH_METRICS_HOST_PORT="${IROH_METRICS_HOST_PORTS[0]}" \
    -e PROMTAIL_AGENT_HOST_PORT="${PROMTAIL_AGENT_HOST_PORTS[0]}" \
    -e PROMTAIL_CONFIG_FOLDER="${IPC_CONFIG_FOLDER}" \
    -e FM_PULL_SKIP=1 \
    -e FM_LOG_LEVEL="info,fendermint=debug" \
    child-validator-no-parent 2>&1)
echo "$bootstrap_output"
bootstrap_node_id=$(echo "$bootstrap_output" | sed -n '/CometBFT node ID:/ {n;p;}' | tr -d "[:blank:]")
bootstrap_peer_id=$(echo "$bootstrap_output" | sed -n '/IPLD Resolver Multiaddress:/ {n;p;}' | tr -d "[:blank:]" | sed 's/.*\/p2p\///')
bootstrap_node_endpoint=${bootstrap_node_id}@validator-0-cometbft:${CMT_P2P_HOST_PORTS[0]}
bootstrap_resolver_endpoint="/dns/validator-0-fendermint/tcp/${RESOLVER_HOST_PORTS[0]}/p2p/${bootstrap_peer_id}"

# Start other validators
for i in {1..2}
do
  cargo make --makefile infra/fendermint/Makefile.toml \
      -e NODE_NAME=validator-"$i" \
      -e PRIVATE_KEY_PATH="$IPC_CONFIG_FOLDER"/validator_"$i".sk \
      -e SUBNET_ID="$subnet_id" \
      -e CMT_P2P_HOST_PORT="${CMT_P2P_HOST_PORTS[i]}" \
      -e CMT_RPC_HOST_PORT="${CMT_RPC_HOST_PORTS[i]}" \
      -e ETHAPI_HOST_PORT="${ETHAPI_HOST_PORTS[i]}" \
      -e RESOLVER_HOST_PORT="${RESOLVER_HOST_PORTS[i]}" \
      -e OBJECTS_HOST_PORT="${OBJECTS_HOST_PORTS[i]}" \
      -e IROH_RPC_HOST_PORT="${IROH_RPC_HOST_PORTS[i]}" \
      -e FENDERMINT_METRICS_HOST_PORT="${FENDERMINT_METRICS_HOST_PORTS[i]}" \
      -e IROH_METRICS_HOST_PORT="${IROH_METRICS_HOST_PORTS[i]}" \
      -e PROMTAIL_AGENT_HOST_PORT="${PROMTAIL_AGENT_HOST_PORTS[i]}" \
      -e PROMTAIL_CONFIG_FOLDER="${IPC_CONFIG_FOLDER}" \
      -e RESOLVER_BOOTSTRAPS="$bootstrap_resolver_endpoint" \
      -e BOOTSTRAPS="$bootstrap_node_endpoint" \
      -e FM_PULL_SKIP=1 \
      -e FM_LOG_LEVEL="info,fendermint=debug" \
      child-validator-no-parent
done

# Start prometheus
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=prometheus \
    -e SUBNET_ID="$subnet_id" \
    -e PROMETHEUS_HOST_PORT="${PROMETHEUS_HOST_PORT}" \
    -e PROMETHEUS_CONFIG_FOLDER="${IPC_CONFIG_FOLDER}" \
    prometheus-start

# Start grafana
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=grafana \
    -e SUBNET_ID="$subnet_id" \
    -e GRAFANA_HOST_PORT="${GRAFANA_HOST_PORT}" \
    grafana-start

# Start loki
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=loki \
    -e SUBNET_ID="$subnet_id" \
    -e LOKI_HOST_PORT="${LOKI_HOST_PORT}" \
    -e LOKI_CONFIG_FOLDER="${IPC_CONFIG_FOLDER}" \
    loki-start

# Test ETH API endpoint
for i in {0..2}
do
  curl --location http://localhost:"${ETHAPI_HOST_PORTS[i]}" \
  --header 'Content-Type: application/json' \
  --data '{
    "jsonrpc":"2.0",
    "method":"eth_blockNumber",
    "params":[],
    "id":83
  }'
done

# Test Object API endpoint
for i in {0..2}
do
  curl --location http://localhost:"${OBJECTS_HOST_PORTS[i]}"/health
done

# Test Prometheus endpoints
curl --location http://localhost:"${PROMETHEUS_HOST_PORT}"/graph
for i in {0..2}
do
  curl --location http://localhost:"${FENDERMINT_METRICS_HOST_PORTS[i]}"/metrics
done

# Print a summary of the deployment
cat << EOF
############################
#                          #
# IPC deployment ready! 🚀 #
#                          #
############################
Subnet ID:
$subnet_id

Chain ID:
$(curl -s --location --request POST http://localhost:"${ETHAPI_HOST_PORTS[0]}" --header 'Content-Type: application/json' --data-raw '{ "jsonrpc":"2.0", "method":"eth_chainId", "params":[], "id":1 }' | jq -r '.result' | xargs printf "%d")

Object API:
http://localhost:${OBJECTS_HOST_PORTS[0]}
http://localhost:${OBJECTS_HOST_PORTS[1]}
http://localhost:${OBJECTS_HOST_PORTS[2]}

Iroh API:
http://localhost:${IROH_RPC_HOST_PORTS[0]}
http://localhost:${IROH_RPC_HOST_PORTS[1]}
http://localhost:${IROH_RPC_HOST_PORTS[2]}

ETH API:
http://localhost:${ETHAPI_HOST_PORTS[0]}
http://localhost:${ETHAPI_HOST_PORTS[1]}
http://localhost:${ETHAPI_HOST_PORTS[2]}

CometBFT API:
http://localhost:${CMT_RPC_HOST_PORTS[0]}
http://localhost:${CMT_RPC_HOST_PORTS[1]}
http://localhost:${CMT_RPC_HOST_PORTS[2]}

Prometheus API:
http://localhost:${PROMETHEUS_HOST_PORT}

Loki API:
http://localhost:${LOKI_HOST_PORT}

Grafana API:
http://localhost:${GRAFANA_HOST_PORT}

Accounts:
$(jq -r '.app_state.accounts[] | "\(.meta.Account.owner): \(.balance) coin units"' "$subnet_folder"/validator-0/genesis.json)

Private keys (hex ready to use with ADM SDK/CLI):
$(jq .[0].private_key "$IPC_CONFIG_FOLDER"/evm_keystore.json | tr -d '"')
$(jq .[1].private_key "$IPC_CONFIG_FOLDER"/evm_keystore.json | tr -d '"')
$(jq .[2].private_key "$IPC_CONFIG_FOLDER"/evm_keystore.json | tr -d '"')
EOF
