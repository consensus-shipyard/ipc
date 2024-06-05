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

if [[ ! -v PARENT_HTTP_AUTH_TOKEN ]]; then
    echo "PARENT_HTTP_AUTH_TOKEN is not set"
    exit 1
fi

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
OBJECT_API_HOST_PORTS=(8001 8002 8003)
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

# Step 1.5: Install node
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

# Step 4: Deploy IPC contracts to parent net (calibration net)
# Step 4.1: Export validator private keys into files
for i in {0..2}
do
  ipc-cli wallet export --wallet-type evm --address ${wallet_addresses[i]} --hex > ${IPC_CONFIG_FOLDER}/validator_${i}.sk
  echo "Export private key for ${wallet_addresses[i]} to ${IPC_CONFIG_FOLDER}/validator_${i}.sk"
done

# Step 4.2: Deploy IPC contracts
cd ${IPC_FOLDER}/contracts
npm install
export RPC_URL=https://calibration.filfox.info/rpc/v1
export PRIVATE_KEY=$(cat ${IPC_CONFIG_FOLDER}/validator_0.sk)
deploy_contracts_output=$(make deploy-ipc NETWORK=calibrationnet)

parent_gateway_address=$(echo "$deploy_contracts_output" | grep '"Gateway"' | awk -F'"' '{print $4}')
parent_registry_address=$(echo "$deploy_contracts_output" | grep '"SubnetRegistry"' | awk -F'"' '{print $4}')
echo "New parent gateway address: $parent_gateway_address"
echo "New parent registry address: $parent_registry_address"

# Step 4.3: Use the new parent gateway and registry address to update IPC config file
toml set ${IPC_CONFIG_FOLDER}/config.toml subnets[0].config.gateway_addr $parent_gateway_address > /tmp/config.toml.1
toml set /tmp/config.toml.1 subnets[0].config.registry_addr $parent_registry_address > /tmp/config.toml.2
cp /tmp/config.toml.2 ${IPC_CONFIG_FOLDER}/config.toml

# Step 5: Create a subnet
echo "$DASHES Creating a child subnet..."
create_subnet_output=$(ipc-cli subnet create --parent /r314159 --min-validators 3 --min-validator-stake 1 --bottomup-check-period 600 --from $default_wallet_address --permission-mode collateral --supply-source-kind native 2>&1)
echo $create_subnet_output
subnet_id=$(echo $create_subnet_output | sed 's/.*with id: \([^ ]*\).*/\1/')
echo "Created new subnet id: $subnet_id"
subnet_folder=$IPC_CONFIG_FOLDER/$(echo $subnet_id | sed 's|^/||;s|/|-|g')

# Step 5 (alternative): Use an already-created subnet
#subnet_id=/r314159/t410flp4jf7keqcf5bqogrkx4wpkygiskykcvpaigicq
#echo "Use existing subnet id: $subnet_id"

# Step 6: Use the new subnet ID to update IPC config file
toml set ${IPC_CONFIG_FOLDER}/config.toml subnets[1].id $subnet_id > /tmp/config.toml.3
cp /tmp/config.toml.3 ${IPC_CONFIG_FOLDER}/config.toml

# Step 7: Join subnet for addresses in wallet
echo "$DASHES Join subnet for addresses in wallet..."
for i in {0..2}
do
  echo "Joining subnet ${subnet_id} for address ${wallet_addresses[i]}"
  ipc-cli subnet join --from ${wallet_addresses[i]} --subnet $subnet_id --initial-balance 1 --collateral 10
done

# Step 8: Start validators
# Step 8.1 (optional): Rebuild fendermint docker
cd ${IPC_FOLDER}/fendermint
make clean
make docker-build

# Step 8.2: Start the bootstrap validator node
echo "$DASHES Start the first validator node as bootstrap"
echo "First we need to force a wait to make sure the subnet is confirmed as created in the parent contracts"
echo "Wait for 30 seconds"
sleep 30
echo "Finished waiting"

cd ${IPC_FOLDER}
bootstrap_output=$(cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=validator-0 \
    -e PRIVATE_KEY_PATH=${IPC_CONFIG_FOLDER}/validator_0.sk \
    -e SUBNET_ID=${subnet_id} \
    -e CMT_P2P_HOST_PORT=${CMT_P2P_HOST_PORTS[0]} \
    -e CMT_RPC_HOST_PORT=${CMT_RPC_HOST_PORTS[0]} \
    -e ETHAPI_HOST_PORT=${ETHAPI_HOST_PORTS[0]} \
    -e RESOLVER_HOST_PORT=${RESOLVER_HOST_PORTS[0]} \
    -e OBJECT_API_HOST_PORT=${OBJECT_API_HOST_PORTS[0]} \
    -e IPFS_SWARM_HOST_PORT=${IPFS_SWARM_HOST_PORTS[0]} \
    -e IPFS_RPC_HOST_PORT=${IPFS_RPC_HOST_PORTS[0]} \
    -e IPFS_GATEWAY_HOST_PORT=${IPFS_GATEWAY_HOST_PORTS[0]} \
    -e IPFS_PROFILE="local-discovery" \
    -e PARENT_HTTP_AUTH_TOKEN=${PARENT_HTTP_AUTH_TOKEN} \
    -e PARENT_REGISTRY=${parent_registry_address} \
    -e PARENT_GATEWAY=${parent_gateway_address} \
    -e FM_PULL_SKIP=1 \
    -e FM_LOG_LEVEL="info,fendermint=debug" \
    child-validator 2>&1)
echo "$bootstrap_output"
bootstrap_node_id=$(echo "$bootstrap_output" | sed -n '/CometBFT node ID:/ {n;p;}' | tr -d "[:blank:]")
bootstrap_peer_id=$(echo "$bootstrap_output" | sed -n '/IPLD Resolver Multiaddress:/ {n;p;}' | tr -d "[:blank:]" | sed 's/.*\/p2p\///')
echo "Bootstrap node started. Node id ${bootstrap_node_id}, peer id ${bootstrap_peer_id}"

bootstrap_node_endpoint=${bootstrap_node_id}@validator-0-cometbft:${CMT_P2P_HOST_PORTS[0]}
echo "Bootstrap node endpoint: ${bootstrap_node_endpoint}"
bootstrap_resolver_endpoint="/dns/validator-0-fendermint/tcp/${RESOLVER_HOST_PORTS[0]}/p2p/${bootstrap_peer_id}"
echo "Bootstrap resolver endpoint: ${bootstrap_resolver_endpoint}"

# Step 8.3: Start other validator node
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
      -e OBJECT_API_HOST_PORT=${OBJECT_API_HOST_PORTS[i]} \
      -e IPFS_SWARM_HOST_PORT=${IPFS_SWARM_HOST_PORTS[i]} \
      -e IPFS_RPC_HOST_PORT=${IPFS_RPC_HOST_PORTS[i]} \
      -e IPFS_GATEWAY_HOST_PORT=${IPFS_GATEWAY_HOST_PORTS[i]} \
      -e IPFS_PROFILE="local-discovery" \
      -e RESOLVER_BOOTSTRAPS=${bootstrap_resolver_endpoint} \
      -e BOOTSTRAPS=${bootstrap_node_endpoint} \
      -e PARENT_HTTP_AUTH_TOKEN=${PARENT_HTTP_AUTH_TOKEN} \
      -e PARENT_REGISTRY=${parent_registry_address} \
      -e PARENT_GATEWAY=${parent_gateway_address} \
      -e FM_PULL_SKIP=1 \
      -e FM_LOG_LEVEL="info,fendermint=debug" \
      child-validator
done

# Step 8.4: Fund proxy wallet in the subnet
echo "$DASHES Fund proxy wallets in the subnet"
for i in {0..2}
do
  proxy_key=$(cat ${IPC_CONFIG_FOLDER}/evm_keystore_proxy.json | jq .[$i].private_key | tr -d '"' | tr -d '\n')
  proxy_address=$(cat ${IPC_CONFIG_FOLDER}/evm_keystore_proxy.json | jq .[$i].address | tr -d '"' | tr -d '\n')
  ipc-cli wallet import --wallet-type evm --private-key ${proxy_key}
  ipc-cli cross-msg fund --from ${proxy_address} --subnet ${subnet_id} 1
  out=${subnet_folder}/validator-${i}/validator-${i}/keys/proxy_key.sk
  ipc-cli wallet export --wallet-type evm --address ${proxy_address} --fendermint | tr -d '\n' > ${out}
  chmod 600 ${subnet_folder}/validator-${i}/validator-${i}/keys/proxy_key.sk
  ipc-cli wallet remove --wallet-type evm --address ${proxy_address}
done

# Step 9: Test
# Step 9.1: Test ETH API endpoint
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

# Step 9.2: Test object API endpoint
printf "\n$DASHES Test object API endpoints of validator nodes\n"
for i in {0..2}
do
  curl --location http://localhost:${OBJECT_API_HOST_PORTS[i]}/health
done

# Step 10: Start a relayer process
# Kill existing relayer if there's one
pkill -f "relayer" || true
# Start relayer
echo "$DASHES Start relayer process (in the background)"
nohup ipc-cli checkpoint relayer --subnet $subnet_id --submitter $default_wallet_address > nohup.out 2> nohup.err < /dev/null &

# Step 11: Print a summary of the deployment
cat << EOF
############################
#                          #
# IPC deployment ready! 🚀 #
#                          #
############################
Subnet ID:
$subnet_id

OBJECT API:
http://localhost:${OBJECT_API_HOST_PORTS[0]}
http://localhost:${OBJECT_API_HOST_PORTS[1]}
http://localhost:${OBJECT_API_HOST_PORTS[2]}

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
