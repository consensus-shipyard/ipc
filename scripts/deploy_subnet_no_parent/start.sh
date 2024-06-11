#!/bin/bash

set -euo pipefail

DASHES='------'
if [[ ! -v IPC_FOLDER ]]; then
    IPC_FOLDER=${HOME}/ipc
else
    IPC_FOLDER=${IPC_FOLDER}
fi
IPC_CONFIG_FOLDER=${HOME}/.ipc

wallet_addresses=()
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

# Step 1.4: Install Foundry
echo "$DASHES Check foundry..."
if which foundryup ; then
  echo "$DASHES foundry is already installed."
else
  echo "$DASHES Need to install foundry"
  curl -L https://foundry.paradigm.xyz | bash
  foundryup
fi

# Step 1.5 Install node
echo "$DASHES Check node..."
if which node ; then
  echo "$DASHES node is already installed."
else
  echo "$DASHES Need to install node"
  curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.3/install.sh | bash
  source "$HOME/.bashrc"
  nvm install --default lts/*
fi

# Step 1.6: Install docker
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
    git clone --recurse-submodules -j8 git@github.com:amazingdatamachine/ipc.git ${IPC_FOLDER}
  fi
  cd ${IPC_FOLDER}
  git fetch
  git stash
  git checkout $head_ref
  git pull --rebase origin $head_ref
  git submodule sync
  git submodule update --init --recursive
fi

echo "$DASHES Building ipc contracts..."
cd ${IPC_FOLDER}/contracts
make build

echo "$DASHES Building ipc-cli..."
cd ${IPC_FOLDER}/ipc
make install

# Step 3: Prepare wallet by using existing wallet json file
echo "$DASHES Using 3 address in wallet..."
for i in {0..2}
do
  addr=$(cat ${IPC_CONFIG_FOLDER}/evm_keystore.json | jq .[$i].address | tr -d '"')
  wallet_addresses+=($addr)
  echo "Wallet $i address: $addr"
done

default_wallet_address=${wallet_addresses[0]}
echo "Default wallet address: $default_wallet_address"

# Step 4: Setup local keys and configs
# Step 4.1: Export validator private keys into files
for i in {0..2}
do
  ipc-cli wallet export --wallet-type evm --address ${wallet_addresses[i]} --hex > ${IPC_CONFIG_FOLDER}/validator_${i}.sk
  echo "Export private key for ${wallet_addresses[i]} to ${IPC_CONFIG_FOLDER}/validator_${i}.sk"
done

# Step 4.2: Setup validators
# Use "dummy" subnet
subnet_id="/r314159/t410f726d2jv6uj4mpkcbgg5ndlpp3l7dd5rlcpgzkoi"
echo "Use existing subnet id: $subnet_id"
subnet_folder=$IPC_CONFIG_FOLDER/$(echo $subnet_id | sed 's|^/||;s|/|-|g')
rm -rf subnet_folder

# Step 5: Init validators
echo "$DASHES Init validators"
cd ${IPC_FOLDER}
for i in {0..2}
do
  cargo make --makefile infra/fendermint/Makefile.toml \
      -e NODE_NAME=validator-${i} \
      -e SUBNET_ID=${subnet_id} \
      -e FM_PULL_SKIP=1 \
      child-validator-no-parent-init
done

# Step 6: Setup proxy wallets
echo "$DASHES Configuring proxy wallets"
for i in {0..2}
do
  # Import wallet
  proxy_key=$(cat ${IPC_CONFIG_FOLDER}/evm_keystore_proxy.json | jq .[$i].private_key | tr -d '"' | tr -d '\n')
  proxy_address=$(cat ${IPC_CONFIG_FOLDER}/evm_keystore_proxy.json | jq .[$i].address | tr -d '"' | tr -d '\n')
  ipc-cli wallet import --wallet-type evm --private-key ${proxy_key}

  # Export private key to proxy
  out=${subnet_folder}/validator-${i}/validator-${i}/keys/proxy_key.sk
  ipc-cli wallet export --wallet-type evm --address ${proxy_address} --fendermint | tr -d '\n' > ${out}
  chmod 600 ${subnet_folder}/validator-${i}/validator-${i}/keys/proxy_key.sk

  # Export public key to proxy
  pubkey=$(ipc-cli wallet pub-key --wallet-type evm --address ${proxy_address})
  echo "Proxy wallet $i address's pubkey: $pubkey"
  echo $pubkey | tr -d '\n' > ${subnet_folder}/validator-${i}/validator-${i}/keys/proxy_key.pk

  # Remove wallet
  ipc-cli wallet remove --wallet-type evm --address ${proxy_address}
done

# Copy genesis file into each validator
for i in {0..2}
do
  cp ${IPC_CONFIG_FOLDER}/genesis.json ${subnet_folder}/validator-${i}
done

# Step 7: Start validators
# Step 7.1 (optional): Rebuild fendermint docker
cd ${IPC_FOLDER}/fendermint
make clean
make docker-build

# Step 7.2: Start the bootstrap validator node
echo "$DASHES Start the first validator node as bootstrap"
cd ${IPC_FOLDER}
bootstrap_output=$(cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=validator-0 \
    -e PRIVATE_KEY_PATH=${IPC_CONFIG_FOLDER}/validator_0.sk \
    -e SUBNET_ID=${subnet_id} \
    -e CMT_P2P_HOST_PORT=${CMT_P2P_HOST_PORTS[0]} \
    -e CMT_RPC_HOST_PORT=${CMT_RPC_HOST_PORTS[0]} \
    -e ETHAPI_HOST_PORT=${ETHAPI_HOST_PORTS[0]} \
    -e RESOLVER_HOST_PORT=${RESOLVER_HOST_PORTS[0]} \
    -e PROXY_HOST_PORT=${PROXY_HOST_PORTS[0]} \
    -e IPFS_SWARM_HOST_PORT=${IPFS_SWARM_HOST_PORTS[0]} \
    -e IPFS_RPC_HOST_PORT=${IPFS_RPC_HOST_PORTS[0]} \
    -e IPFS_GATEWAY_HOST_PORT=${IPFS_GATEWAY_HOST_PORTS[0]} \
    -e IPFS_PROFILE="local-discovery" \
    -e FM_PULL_SKIP=1 \
    -e FM_LOG_LEVEL="info,fendermint=debug" \
    child-validator-no-parent 2>&1)
echo "$bootstrap_output"
bootstrap_node_id=$(echo "$bootstrap_output" | sed -n '/CometBFT node ID:/ {n;p;}' | tr -d "[:blank:]")
bootstrap_peer_id=$(echo "$bootstrap_output" | sed -n '/IPLD Resolver Multiaddress:/ {n;p;}' | tr -d "[:blank:]" | sed 's/.*\/p2p\///')
echo "Bootstrap node started. Node id ${bootstrap_node_id}, peer id ${bootstrap_peer_id}"

bootstrap_node_endpoint=${bootstrap_node_id}@validator-0-cometbft:${CMT_P2P_HOST_PORTS[0]}
echo "Bootstrap node endpoint: ${bootstrap_node_endpoint}"
bootstrap_resolver_endpoint="/dns/validator-0-fendermint/tcp/${RESOLVER_HOST_PORTS[0]}/p2p/${bootstrap_peer_id}"
echo "Bootstrap resolver endpoint: ${bootstrap_resolver_endpoint}"

# Step 7.3: Start other validator node
echo "$DASHES Start the other validator nodes"
cd ${IPC_FOLDER}
for i in {1..2}
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
      -e RESOLVER_BOOTSTRAPS=${bootstrap_resolver_endpoint} \
      -e BOOTSTRAPS=${bootstrap_node_endpoint} \
      -e FM_PULL_SKIP=1 \
      -e FM_LOG_LEVEL="info,fendermint=debug" \
      child-validator-no-parent
done

# Step 8: Test
# Step 8.1: Test ETH API endpoint
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

# Step 8.2: Test proxy endpoint
printf "\n$DASHES Test proxy endpoints of validator nodes\n"
for i in {0..2}
do
  curl --location http://localhost:${PROXY_HOST_PORTS[i]}/health
done

# Step 9: Print a summary of the deployment
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
