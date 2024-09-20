#!/bin/bash

set -euo pipefail

eval "$(ssh-agent -s)"
ssh-add
if [ -e "${HOME}"/.ssh/id_rsa.ipc ]; then
  ssh-add "${HOME}"/.ssh/id_rsa.ipc
fi

if [[ ! -v SUPPLY_SOURCE_ADDRESS ]]; then
  echo "SUPPLY_SOURCE_ADDRESS is not set"
  exit 1
fi
if [[ ! -v PARENT_HTTP_AUTH_TOKEN ]]; then
  echo "PARENT_HTTP_AUTH_TOKEN is not set"
  exit 1
fi

DASHES='------'
if [[ ! -v IPC_FOLDER ]]; then
    IPC_FOLDER=${HOME}/ipc
fi
IPC_CONFIG_FOLDER=${HOME}/.ipc

wallet_addresses=()
public_keys=()
CMT_P2P_HOST_PORT=26656
CMT_RPC_HOST_PORT=26657
ETHAPI_HOST_PORT=8545
RESOLVER_HOST_PORT=26655
OBJECTS_HOST_PORT=8001
IROH_RPC_HOST_PORT=4921

FENDERMINT_METRICS_HOST_PORT=9184
IROH_METRICS_HOST_PORT=9091
PROMTAIL_AGENT_HOST_PORT=9080

PROMETHEUS_HOST_PORT=9090
LOKI_HOST_PORT=3100
GRAFANA_HOST_PORT=3000

if (($# != 1)); then
  echo "Arguments: <Specify github remote branch name to use to deploy. Or use 'local' (without quote) to indicate using local repo instead. If not provided, will default to main branch"
  head_ref=main
  local_deploy=false
else
  if [ "$1" = "local" ]; then
    local_deploy=true
  else
    local_deploy=false
    head_ref=$1
  fi
fi

# Install build dependencies
echo "${DASHES} Installing build dependencies..."
sudo apt update && sudo apt install build-essential libssl-dev mesa-opencl-icd ocl-icd-opencl-dev gcc git bzr jq pkg-config curl clang hwloc libhwloc-dev wget ca-certificates gnupg -y

# Install rust + cargo
echo "$DASHES Check rustc & cargo..."
if which cargo ; then
  echo "$DASHES rustc & cargo already installed."
else
  echo "$DASHES Need to install rustc & cargo"
  curl https://sh.rustup.rs -sSf | sh -s -- -y
  # Refresh env
  source "${HOME}"/.bashrc
fi

# Install cargo make
echo "$DASHES Installing cargo-make"
cargo install cargo-make
# Install toml-cli
echo "$DASHES Installing toml-cli"
cargo install toml-cli

# Install Foundry
echo "$DASHES Check foundry..."
if which foundryup ; then
  echo "$DASHES foundry is already installed."
else
  echo "$DASHES Need to install foundry"
  curl -L https://foundry.paradigm.xyz | bash
  foundryup
fi

# Install node
echo "$DASHES Check node..."
if which node ; then
  echo "$DASHES node is already installed."
else
  echo "$DASHES Need to install node"
  curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.3/install.sh | bash
  source "$HOME/.bashrc"
  nvm install --default lts/*
fi

# Install docker
echo "$DASHES check docker"
if which docker ; then
  echo "$DASHES docker is already installed."
else
  echo "$DASHES Need to install docker"
  # Add Docker's official GPG key:
  sudo apt-get update
  sudo apt-get install ca-certificates curl
  sudo install -m 0755 -d /etc/apt/keyrings
  sudo curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
  sudo chmod a+r /etc/apt/keyrings/docker.asc

  # Add the repository to Apt sources:
  echo \
    "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu \
    $(. /etc/os-release && echo "$VERSION_CODENAME") stable" | \
    sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
  sudo apt-get update
  sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

  # Remove the need to use sudo
  getent group docker || sudo groupadd docker
  sudo usermod -aG docker "$USER"
  newgrp docker

  # Test running docker without sudo
  docker ps
fi

# Make sure we re-read the latest env before finishing dependency installation.
set +u
source "${HOME}"/.bashrc
set -u

# Prepare code repo
if ! $local_deploy ; then
  echo "$DASHES Preparing ipc repo..."
  if ! ls "$IPC_FOLDER" ; then
    git clone --recurse-submodules -j8 git@github.com-ipc:amazingdatamachine/ipc.git "${IPC_FOLDER}"
  fi
  cd "${IPC_FOLDER}"
  git fetch
  git stash
  git checkout "$head_ref"
  git pull --rebase origin "$head_ref"
  git submodule sync
  git submodule update --init --recursive
fi

# Stop prometheus
cd "$IPC_FOLDER"
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=prometheus \
    prometheus-destroy

# Stop grafana
cd "$IPC_FOLDER"
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=grafana \
    grafana-destroy

# Stop loki
cd "$IPC_FOLDER"
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=loki \
    loki-destroy

if [ -e "${IPC_CONFIG_FOLDER}" ]; then
    subnet_id=$(toml get -r "${IPC_CONFIG_FOLDER}"/config.toml subnets[1].id)
    echo "Existing subnet id: $subnet_id"
    # Stop validators
    cd "$IPC_FOLDER"
    cargo make --makefile infra/fendermint/Makefile.toml \
        -e NODE_NAME=validator-0 \
        -e SUBNET_ID="$subnet_id" \
        child-validator-down
fi

# Remove existing deployment
rm -rf "$IPC_CONFIG_FOLDER"
mkdir -p "$IPC_CONFIG_FOLDER"

# Copy configs
cp "$HOME"/evm_keystore.json "$IPC_CONFIG_FOLDER"
cp "$IPC_FOLDER"/scripts/deploy_subnet/.ipc-cal/config.toml "$IPC_CONFIG_FOLDER"
cp "$IPC_FOLDER"/infra/prometheus/prometheus.yaml "$IPC_CONFIG_FOLDER"
cp "$IPC_FOLDER"/infra/loki/loki-config.yaml "$IPC_CONFIG_FOLDER"
cp "$IPC_FOLDER"/infra/promtail/promtail-config.yaml "$IPC_CONFIG_FOLDER"

# Build contracts
echo "$DASHES Building ipc contracts..."
cd "${IPC_FOLDER}"/contracts
make build

# Build ipc-cli
echo "$DASHES Building ipc-cli..."
cd "${IPC_FOLDER}"/ipc
make install

# Prepare wallet by using existing wallet json file
echo "$DASHES Using 2 addresses in wallet..."
for i in {0..1}
do
  addr=$(jq .["$i"].address < "${IPC_CONFIG_FOLDER}"/evm_keystore.json | tr -d '"')
  wallet_addresses+=("$addr")
  echo "Wallet $i address: $addr"
  pk=$(ipc-cli wallet pub-key --wallet-type evm --address "$addr" | tr -d '"')
  public_keys+=("$pk")
done
default_wallet_address=${wallet_addresses[0]}
echo "Default wallet address: $default_wallet_address"

# Export validator private key into file
ipc-cli wallet export --wallet-type evm --address "$default_wallet_address" --hex > "${IPC_CONFIG_FOLDER}"/validator_0.sk
echo "Export private key for $default_wallet_address to ${IPC_CONFIG_FOLDER}/validator_0.sk"

# Update IPC config file with parent auth token
toml set "${IPC_CONFIG_FOLDER}"/config.toml subnets[0].config.auth_token "$PARENT_HTTP_AUTH_TOKEN" > /tmp/config.toml.0
cp /tmp/config.toml.0 "${IPC_CONFIG_FOLDER}"/config.toml

# Deploy IPC contracts
if [[ ! -v PARENT_GATEWAY_ADDRESS || ! -v PARENT_REGISTRY_ADDRESS ]]; then
  echo "$DASHES Deploying new IPC contracts..."
  cd "${IPC_FOLDER}"/contracts
  npm install
  export RPC_URL=https://calibration.filfox.info/rpc/v1
  pk=$(cat "${IPC_CONFIG_FOLDER}"/validator_0.sk)
  export PRIVATE_KEY=$pk
  deploy_contracts_output=$(make deploy-ipc NETWORK=calibrationnet)

  PARENT_GATEWAY_ADDRESS=$(echo "$deploy_contracts_output" | grep '"Gateway"' | awk -F'"' '{print $4}')
  PARENT_REGISTRY_ADDRESS=$(echo "$deploy_contracts_output" | grep '"SubnetRegistry"' | awk -F'"' '{print $4}')
fi
echo "Gateway address: $PARENT_GATEWAY_ADDRESS"
echo "Registry address: $PARENT_REGISTRY_ADDRESS"

# Use the parent gateway and registry address to update IPC config file
toml set "${IPC_CONFIG_FOLDER}"/config.toml subnets[0].config.gateway_addr "$PARENT_GATEWAY_ADDRESS" > /tmp/config.toml.1
toml set /tmp/config.toml.1 subnets[0].config.registry_addr "$PARENT_REGISTRY_ADDRESS" > /tmp/config.toml.2
cp /tmp/config.toml.2 "${IPC_CONFIG_FOLDER}"/config.toml

# Create a subnet
echo "$DASHES Creating a child subnet..."
create_subnet_output=$(ipc-cli subnet create --from "$default_wallet_address" --parent /r314159 --min-validators 1 --min-validator-stake 1 --bottomup-check-period 600 --active-validators-limit 2 --permission-mode federated --supply-source-kind erc20 --supply-source-address "$SUPPLY_SOURCE_ADDRESS" 2>&1)
echo "$create_subnet_output"
# shellcheck disable=SC2086
subnet_id=$(echo $create_subnet_output | sed 's/.*with id: \([^ ]*\).*/\1/')
echo "Created new subnet id: $subnet_id"

# Use the new subnet ID to update IPC config file
toml set "${IPC_CONFIG_FOLDER}"/config.toml subnets[1].id "$subnet_id" > /tmp/config.toml.3
cp /tmp/config.toml.3 "${IPC_CONFIG_FOLDER}"/config.toml

# Set federated power
ipc-cli subnet set-federated-power --from "$default_wallet_address" --subnet "$subnet_id" --validator-addresses "${wallet_addresses[@]}" --validator-pubkeys "${public_keys[@]}" --validator-power 1 1

# Rebuild fendermint docker
cd "${IPC_FOLDER}"/fendermint
make clean
make docker-build

# Start the bootstrap validator node
echo "$DASHES Start the first validator node as bootstrap"
echo "First we need to force a wait to make sure the subnet is confirmed as created in the parent contracts"
echo "Wait for 30 seconds"
sleep 30
echo "Finished waiting"
cd "${IPC_FOLDER}"
bootstrap_output=$(cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=validator-0 \
    -e PRIVATE_KEY_PATH="${IPC_CONFIG_FOLDER}"/validator_0.sk \
    -e SUBNET_ID="${subnet_id}" \
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
    -e PARENT_HTTP_AUTH_TOKEN="${PARENT_HTTP_AUTH_TOKEN}" \
    -e PARENT_REGISTRY="${PARENT_REGISTRY_ADDRESS}" \
    -e PARENT_GATEWAY="${PARENT_GATEWAY_ADDRESS}" \
    -e FM_PULL_SKIP=1 \
    -e FM_LOG_LEVEL="info,fendermint=debug" \
    child-validator 2>&1)
echo "$bootstrap_output"
bootstrap_node_id=$(echo "$bootstrap_output" | sed -n '/CometBFT node ID:/ {n;p;}' | tr -d "[:blank:]")
bootstrap_peer_id=$(echo "$bootstrap_output" | sed -n '/IPLD Resolver Multiaddress:/ {n;p;}' | tr -d "[:blank:]" | sed 's/.*\/p2p\///')
echo "Bootstrap node started. Node id ${bootstrap_node_id}, peer id ${bootstrap_peer_id}"
bootstrap_node_endpoint=${bootstrap_node_id}@validator-0-cometbft:${CMT_P2P_HOST_PORT}
echo "Bootstrap node endpoint: ${bootstrap_node_endpoint}"
bootstrap_resolver_endpoint="/dns/validator-0-fendermint/tcp/${RESOLVER_HOST_PORT}/p2p/${bootstrap_peer_id}"
echo "Bootstrap resolver endpoint: ${bootstrap_resolver_endpoint}"

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
echo "$DASHES Test ETH API endpoint of validator node"
curl --location http://localhost:"${ETHAPI_HOST_PORT}" \
--header 'Content-Type: application/json' \
--data '{
  "jsonrpc":"2.0",
  "method":"eth_blockNumber",
  "params":[],
  "id":83
}'

# Test Object API endpoint
echo "$DASHES Test Object API endpoint of validator node"
curl --location http://localhost:"${OBJECTS_HOST_PORT}"/health

# Test Prometheus endpoints
printf "\n%s Test Prometheus endpoints of validator nodes\n" $DASHES
curl --location http://localhost:"${PROMETHEUS_HOST_PORT}"/graph
curl --location http://localhost:"${FENDERMINT_METRICS_HOST_PORT}"/metrics

# Kill existing relayer if there's one
pkill -f "relayer" || true
# Start relayer
echo "$DASHES Start relayer process (in the background)"
nohup ipc-cli checkpoint relayer --subnet "$subnet_id" --submitter "$default_wallet_address" > relayer.log &

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
$(curl -s --location --request POST http://localhost:"${ETHAPI_HOST_PORT}" --header 'Content-Type: application/json' --data-raw '{ "jsonrpc":"2.0", "method":"eth_chainId", "params":[], "id":1 }' | jq -r '.result' | xargs printf "%d")

Object API:
http://localhost:${OBJECTS_HOST_PORT}

Iroh API:
http://localhost:${IROH_RPC_HOST_PORT}

ETH API:
http://localhost:${ETHAPI_HOST_PORT}

CometBFT API:
http://localhost:${CMT_RPC_HOST_PORT}

Prometheus API:
http://localhost:${PROMETHEUS_HOST_PORT}

Loki API:
http://localhost:${LOKI_HOST_PORT}

Grafana API:
http://localhost:${GRAFANA_HOST_PORT}
EOF

echo "Done"
exit 0
