#!/bin/bash

#set -euo pipefail

DASHES='------'
dir=$(dirname -- "$(readlink -f -- "${BASH_SOURCE[0]}")")
IPC_FOLDER="$dir"/../..
IPC_CONFIG_FOLDER=${HOME}/.ipc

CMT_P2P_HOST_PORTS=(26656 26756 26856)
CMT_RPC_HOST_PORTS=(26657 26757 26857)
ETHAPI_HOST_PORTS=(8645 8745 8845)
RESOLVER_HOST_PORTS=(26655 26755 26855)
OBJECTS_HOST_PORTS=(8001 8002 8003)
IROH_RPC_HOST_PORTS=(4921 4922 4923)

FENDERMINT_METRICS_HOST_PORTS=(9184 9185 9186)
IROH_METRICS_HOST_PORTS=(9091 9092 9093)
PROMTAIL_AGENT_HOST_PORTS=(9080 9081 9082)

ANVIL_HOST_PORT=8545
PROMETHEUS_HOST_PORT=9090
LOKI_HOST_PORT=3100
GRAFANA_HOST_PORT=3000

if [[ ! -v SKIP_BUILD ]]; then 
  # Build IPC contracts
  cd "$IPC_FOLDER"/contracts
  make gen

  # Rebuild fendermint docker
  cd "$IPC_FOLDER"/fendermint
  make clean
  make docker-build
fi

# # Rebuild fendermint docker
# cd "$IPC_FOLDER"/fendermint
# make clean
# make docker-build

# Prepare wallet by using existing wallet json file
wallet_addresses=()
for i in {0..2}
do
  addr=$(jq .["$i"].address < "$IPC_CONFIG_FOLDER"/evm_keystore.json | tr -d '"')
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

# Prepare wallet by using existing wallet json file
wallet_addresses=()
for i in {0..2}
do
  addr=$(jq .["$i"].address "$IPC_CONFIG_FOLDER"/evm_keystore.json | tr -d '"')

echo "adding address: " $addr
  wallet_addresses+=("$addr")
done

# Init ipc cli config
ipc-cli config init

# Export validator private keys into files
for i in {0..2}
do
  ipc-cli wallet export --wallet-type evm --address "${wallet_addresses[i]}" --hex > "$IPC_CONFIG_FOLDER"/validator_"$i".sk
done


# Start anvil node and deploy contracts
echo "starting anvil"
# Step 1 Start Anvil
cd "$IPC_FOLDER"
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=anvil \
    -e ANVIL_HOST_PORT="${ANVIL_HOST_PORT}" \
    anvil-start

echo "started anvil"

# Step 2: Deploy IPC contracts
cd ${IPC_FOLDER}/contracts
npm install

export RPC_URL=http://localhost:8545
export PRIVATE_KEY=$(cat ${IPC_CONFIG_FOLDER}/validator_0.sk)

deploy_contracts_output=$(make deploy-ipc NETWORK="localnet")

echo "**************************************************"
echo "             deploy_contracts_output"
echo "$deploy_contracts_output"
echo ""
echo ""


parent_gateway_address=$(echo "$deploy_contracts_output" | grep '"Gateway"' | awk -F'"' '{print $4}')
parent_registry_address=$(echo "$deploy_contracts_output" | grep '"SubnetRegistry"' | awk -F'"' '{print $4}')
echo "New parent gateway address: $parent_gateway_address"
echo "New parent registry address: $parent_registry_address"
echo ""

# Step 3: Use the new parent gateway and registry address to update IPC config file
parent_id=/r31337

toml set ${IPC_CONFIG_FOLDER}/config.toml subnets[0].config.gateway_addr $parent_gateway_address > /tmp/config.toml.1
toml set /tmp/config.toml.1 subnets[0].config.registry_addr $parent_registry_address > /tmp/config.toml.2
toml set /tmp/config.toml.2 subnets[0].config.network_type "fevm" > /tmp/config.toml.3
toml set /tmp/config.toml.3 subnets[0].config.provider_http "http://localhost:8545" > /tmp/config.toml.4
toml set /tmp/config.toml.4 subnets[0].id "$parent_id" > /tmp/config.toml.5
cp /tmp/config.toml.5 ${IPC_CONFIG_FOLDER}/config.toml


# Step 5: Create a subnet

default_wallet_address=${wallet_addresses[0]}
echo "Default wallet address: $default_wallet_address"

echo "$DASHES Creating a child subnet... $DASHES"
create_subnet_output=$(ipc-cli subnet create --parent $parent_id --min-validators 3 --min-validator-stake 1 --bottomup-check-period 600 --from $default_wallet_address --permission-mode collateral --supply-source-kind native 2>&1)
echo "$create_subnet_output"
subnet_id=$(echo $create_subnet_output | sed 's/.*with id: \([^ ]*\).*/\1/')
echo "Created new subnet id: $subnet_id"

subnet_folder=$IPC_CONFIG_FOLDER/$(echo $subnet_id | sed 's|^/||;s|/|-|g')
rm -rf "$subnet_folder"

# Take down any existing validators and init from scratch
cd "$IPC_FOLDER"
for i in {0..2}
do
  cargo make --makefile infra/fendermint/Makefile.toml \
      -e NODE_NAME=validator-"$i" \
      -e SUBNET_ID="$subnet_id" \
      -e FM_PULL_SKIP=1 \
      child-validator-no-parent-init
done


# Step 6: Use the new subnet ID to update IPC config file
echo "[[subnets]]" >> /tmp/config.toml.5
toml set /tmp/config.toml.5 subnets[1].id $subnet_id > /tmp/config.toml.6
echo "" >> /tmp/config.toml.6
echo "[subnets.config]" >> /tmp/config.toml.6
echo "network_type = \"fevm\"" >> /tmp/config.toml.6
toml set /tmp/config.toml.6 subnets[1].config.provider_http "http://localhost:8545" > /tmp/config.toml.7
toml set /tmp/config.toml.7 subnets[1].config.gateway_addr $parent_gateway_address > /tmp/config.toml.8
toml set /tmp/config.toml.8 subnets[1].config.registry_addr $parent_registry_address > /tmp/config.toml.9

cp /tmp/config.toml.9 ${IPC_CONFIG_FOLDER}/config.toml



# Step 7: Join subnet for addresses in wallet
echo "$DASHES Join subnet for addresses in wallet..."
for i in {0..2}
do
  echo "Joining subnet ${subnet_id} for address ${wallet_addresses[i]}"
  ipc-cli subnet join --from ${wallet_addresses[i]} --subnet $subnet_id --initial-balance 1 --collateral 10
done


# Copy genesis file into each validator
for i in {0..2}
do
  cp "$IPC_CONFIG_FOLDER"/genesis.json "$subnet_folder"/validator-"$i"
done

# Start bootstrap validator
cd "$IPC_FOLDER"
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
cd "$IPC_FOLDER"
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
cd "$IPC_FOLDER"
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=prometheus \
    -e SUBNET_ID="$subnet_id" \
    -e PROMETHEUS_HOST_PORT="${PROMETHEUS_HOST_PORT}" \
    -e PROMETHEUS_CONFIG_FOLDER="${IPC_CONFIG_FOLDER}" \
    prometheus-start

# Start grafana
cd "$IPC_FOLDER"
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=grafana \
    -e SUBNET_ID="$subnet_id" \
    -e GRAFANA_HOST_PORT="${GRAFANA_HOST_PORT}" \
    grafana-start

# Start loki
cd "$IPC_FOLDER"
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
$(jq -r '.app_state.accounts[] | "\(.meta.Account.owner): \(.balance) coin units"' < "$subnet_folder"/validator-0/genesis.json)

Private keys (hex ready to use with ADM SDK/CLI):
$(jq .[0].private_key < "$IPC_CONFIG_FOLDER"/evm_keystore.json | tr -d '"')
$(jq .[1].private_key < "$IPC_CONFIG_FOLDER"/evm_keystore.json | tr -d '"')
$(jq .[2].private_key < "$IPC_CONFIG_FOLDER"/evm_keystore.json | tr -d '"')
EOF
