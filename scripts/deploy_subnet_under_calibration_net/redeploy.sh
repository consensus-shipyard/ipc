#!/bin/bash

# IPC Quick Start Script
# See also https://github.com/consensus-shipyard/ipc/blob/main/docs/ipc/quickstart-calibration.md

# Known issues:
# 1. Need to previously manual enable sudo without password on the host
# 2. You may need to rerun the script after docker installation for the first time
# 2. You may need to manually install nodejs and npm on the host

set -euo pipefail

eval `ssh-agent -s`
ssh-add
ssh-add ${HOME}/.ssh/id_rsa.ipc

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

if (($# != 1)); then
  echo "Arguments: <Specify github remote branch name to use to deploy. Or use 'local' (without quote) to indicate using local repo instead. If not provided, will default to main branch"
  head_ref=main
  local_deploy=false
else
  if [ $1 = "local" ]; then
    local_deploy=true
  else
    local_deploy=false
    head_ref=$1
  fi
fi

# Step 1: Prepare system for building and running IPC

# Step 1.1: Install build dependencies
echo "${DASHES} Installing build dependencies..."
sudo apt update && sudo apt install build-essential libssl-dev mesa-opencl-icd ocl-icd-opencl-dev gcc git bzr jq pkg-config curl clang hwloc libhwloc-dev wget ca-certificates gnupg -y

# Step 1.2: Install rust + cargo
echo "$DASHES Check rustc & cargo..."
if which cargo ; then
  echo "$DASHES rustc & cargo already installed."
else
  echo "$DASHES Need to install rustc & cargo"
  curl https://sh.rustup.rs -sSf | sh -s -- -y
  # Refresh env
  source ${HOME}/.bashrc
fi

# Step 1.3: Install cargo-make and toml-cli
# Install cargo make
echo "$DASHES Installing cargo-make"
cargo install cargo-make
# Install toml-cli
echo "$DASHES Installing toml-cli"
cargo install toml-cli

# Step 1.4: Install docker
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
  sudo usermod -aG docker $USER
  newgrp docker

  # Test running docker without sudo
  docker ps
fi

# Make sure we re-read the latest env before finishing dependency installation.
set +u
source ${HOME}/.bashrc
set -u

# Step 2: Prepare code repo and build ipc-cli
if ! $local_deploy ; then
  echo "$DASHES Preparing ipc repo..."
  if ! ls $IPC_FOLDER ; then
    git clone --recurse-submodules -j8 git@github.com-ipc:amazingdatamachine/ipc.git ${IPC_FOLDER}
  fi
  cd ${IPC_FOLDER}
  git fetch
  git stash
  git checkout $head_ref
  git pull --rebase origin $head_ref
  git submodule sync
  git submodule update --init --recursive
fi

# Step 3: Use already-created subnet
subnet_id=$(toml get -r ${IPC_CONFIG_FOLDER}/config.toml subnets[1].id)
echo "Use existing subnet id: $subnet_id"
subnet_folder=$IPC_CONFIG_FOLDER/$(echo $subnet_id | sed 's|^/||;s|/|-|g')
parent_gateway_address=$(toml get -r ${IPC_CONFIG_FOLDER}/config.toml subnets[0].config.gateway_addr)
parent_registry_address=$(toml get -r ${IPC_CONFIG_FOLDER}/config.toml subnets[0].config.registry_addr)

# Step 4: Restart validators
# Step 4.1: Rebuild fendermint docker
echo "$DASHES Rebuild fendermint docker"
cd ${IPC_FOLDER}/fendermint
make clean
make docker-build

# Step 4.2: Start other validator node
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
      -e PARENT_REGISTRY=${parent_registry_address} \
      -e PARENT_GATEWAY=${parent_gateway_address} \
      -e FM_PULL_SKIP=1 \
      -e FM_LOG_LEVEL="info" \
      child-validator-restart
done

# Step 5: Test
# Step 5.1: Test ETH API endpoint
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

# Step 5.2: Test proxy endpoint
printf "\n$DASHES Test proxy endpoints of validator nodes\n"
for i in {0..2}
do
  curl --location http://localhost:${PROXY_HOST_PORTS[i]}/health
done

# Step 6: Start a relayer process
# Kill existing relayer if there's one
pkill -f "relayer" || true
# Start relayer
echo "$DASHES Start relayer process (in the background)"
nohup ipc-cli checkpoint relayer --subnet $subnet_id --submitter 0xA08aE9E8c038CAf9765D7Db725CA63a92FCf12Ce > nohup.out 2> nohup.err < /dev/null &

# Step 7: Print a summary of the deployment
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
$(jq -r '.accounts[] | "\(.meta.Account.owner): \(.balance) coin units"' ${subnet_folder}/validator-0/genesis.json)

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
