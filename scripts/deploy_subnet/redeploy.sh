#!/bin/bash

set -euo pipefail

eval "$(ssh-agent -s)"
ssh-add
ssh-add "${HOME}"/.ssh/id_rsa.ipc

if [[ ! -v PARENT_HTTP_AUTH_TOKEN ]]; then
    echo "PARENT_HTTP_AUTH_TOKEN is not set"
    exit 1
fi

DASHES='------'
if [[ ! -v IPC_FOLDER ]]; then
    IPC_FOLDER=${HOME}/ipc
fi
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

# Prepare code repo and build ipc-cli
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

# Use already-created subnet
subnet_id=$(toml get -r "${IPC_CONFIG_FOLDER}"/config.toml subnets[1].id)
echo "Use existing subnet id: $subnet_id"
subnet_folder=$IPC_CONFIG_FOLDER/$(echo "$subnet_id" | sed 's|^/||;s|/|-|g')
parent_gateway_address=$(toml get -r "${IPC_CONFIG_FOLDER}"/config.toml subnets[0].config.gateway_addr)
parent_registry_address=$(toml get -r "${IPC_CONFIG_FOLDER}"/config.toml subnets[0].config.registry_addr)

# Rebuild fendermint docker
echo "$DASHES Rebuild fendermint docker"
cd "${IPC_FOLDER}"/fendermint
make clean
make docker-build

# Start first validator node as bootstrap
echo "$DASHES Start first validator node as bootstrap"
cd "${IPC_FOLDER}"
bootstrap_output=$(cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=validator-0 \
    -e PRIVATE_KEY_PATH="${IPC_CONFIG_FOLDER}"/validator_0.sk \
    -e SUBNET_ID="${subnet_id}" \
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
    -e PARENT_HTTP_AUTH_TOKEN="${PARENT_HTTP_AUTH_TOKEN}" \
    -e PARENT_REGISTRY="${parent_registry_address}" \
    -e PARENT_GATEWAY="${parent_gateway_address}" \
    -e FM_PULL_SKIP=1 \
    -e FM_LOG_LEVEL="info,fendermint=debug" \
    child-validator-restart 2>&1)
echo "$bootstrap_output"
bootstrap_node_id=$(echo "$bootstrap_output" | sed -n '/CometBFT node ID:/ {n;p;}' | tr -d "[:blank:]")
bootstrap_peer_id=$(echo "$bootstrap_output" | sed -n '/IPLD Resolver Multiaddress:/ {n;p;}' | tr -d "[:blank:]" | sed 's/.*\/p2p\///')
echo "Bootstrap node started. Node id ${bootstrap_node_id}, peer id ${bootstrap_peer_id}"
bootstrap_node_endpoint=${bootstrap_node_id}@validator-0-cometbft:${CMT_P2P_HOST_PORTS[0]}
echo "Bootstrap node endpoint: ${bootstrap_node_endpoint}"
bootstrap_resolver_endpoint="/dns/validator-0-fendermint/tcp/${RESOLVER_HOST_PORTS[0]}/p2p/${bootstrap_peer_id}"
echo "Bootstrap resolver endpoint: ${bootstrap_resolver_endpoint}"

# Start other validator node
echo "$DASHES Start the other validator nodes"
cd "$IPC_FOLDER"
for i in {1..2}
do
  cargo make --makefile infra/fendermint/Makefile.toml \
      -e NODE_NAME=validator-"${i}" \
      -e PRIVATE_KEY_PATH="${IPC_CONFIG_FOLDER}"/validator_"${i}".sk \
      -e SUBNET_ID="${subnet_id}" \
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
      -e RESOLVER_BOOTSTRAPS="${bootstrap_resolver_endpoint}" \
      -e BOOTSTRAPS="${bootstrap_node_endpoint}" \
      -e PARENT_HTTP_AUTH_TOKEN="${PARENT_HTTP_AUTH_TOKEN}" \
      -e PARENT_REGISTRY="${parent_registry_address}" \
      -e PARENT_GATEWAY="${parent_gateway_address}" \
      -e FM_PULL_SKIP=1 \
      -e FM_LOG_LEVEL="info,fendermint=debug" \
      child-validator-restart
done

# Test ETH API endpoint
echo "$DASHES Test ETH API endpoints of validator nodes"
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
printf "\n%s Test Object API endpoints of validator nodes\n" $DASHES
for i in {0..2}
do
  curl --location http://localhost:"${OBJECTS_HOST_PORTS[i]}"/health
done

# Kill existing relayer if there's one
pkill -f "relayer" || true
# Start relayer
echo "$DASHES Start relayer process (in the background)"
nohup ipc-cli checkpoint relayer --subnet "$subnet_id" --submitter 0xA08aE9E8c038CAf9765D7Db725CA63a92FCf12Ce > relayer.log &

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

Accounts:
$(jq -r '.accounts[] | "\(.meta.Account.owner): \(.balance) coin units"' < "${subnet_folder}"/validator-0/genesis.json)
EOF
