#!/bin/bash

set -euo pipefail

DASHES='------'

if (($# != 1)); then
  echo "Arguments: <Specify GitHub remote branch name to use to deploy. Or use 'local' (without quote) to indicate using local repo instead. If not provided, will default to main branch"
  head_ref=main
  local_deploy=false
else
  if [[ "$1" = "local" || "$1" = "localnet" ]]; then
    echo "$DASHES deploying to localnet $DASHES"
    local_deploy=true
  else
    local_deploy=false
    head_ref=$1
  fi
fi

if ! $local_deploy ; then
  if [[ -z "${SUPPLY_SOURCE_ADDRESS:-}" ]]; then
    echo "SUPPLY_SOURCE_ADDRESS is not set"
    exit 1
  fi

  if [[ -z "${VALIDATOR_REWARDER_ADDRESS:-}" ]]; then
    echo "VALIDATOR_REWARDER_ADDRESS is not set"
    exit 1
  fi

  if [[ -z "${PARENT_HTTP_AUTH_TOKEN:-}" ]]; then
    echo "PARENT_HTTP_AUTH_TOKEN is not set"
    exit 1
  fi
  PARENT_AUTH_FLAG="--parent-auth-token ${PARENT_HTTP_AUTH_TOKEN}"
else
  # For local deployment, we'll set these variables later
  SUPPLY_SOURCE_ADDRESS=""
  VALIDATOR_REWARDER_ADDRESS=""
  PARENT_HTTP_AUTH_TOKEN=""
  PARENT_AUTH_FLAG=""
fi

if [[ $local_deploy = true ]]; then
  dir=$(dirname -- "$(readlink -f -- "${BASH_SOURCE[0]}")")
  IPC_FOLDER=$(readlink -f -- "$dir"/../..)
  # we can't pass an auth token if the parent doesn't require one
  PARENT_AUTH_FLAG=""
fi
if [[ -z "${IPC_FOLDER:-}" ]]; then
  IPC_FOLDER=${HOME}/ipc
fi
IPC_CONFIG_FOLDER=${HOME}/.ipc
if [[ -z "${FM_LOG_LEVEL:-}" ]]; then
  FM_LOG_LEVEL="info"
fi
if [[ -z "${FM_LOG_DOMAINS:-}" ]]; then
  FM_LOG_DOMAINS=Bottomup,Consensus,Execution,Mpool,System,Topdown
fi

echo "$DASHES starting with env $DASHES"
echo "IPC_FOLDER $IPC_FOLDER"
echo "IPC_CONFIG_FOLDER $IPC_CONFIG_FOLDER"
echo "FM_LOG_LEVEL $FM_LOG_LEVEL"
echo "FM_LOG_DOMAINS $FM_LOG_DOMAINS"

wallet_addresses=()
CMT_P2P_HOST_PORTS=(26656 26756 26856)
CMT_RPC_HOST_PORTS=(26657 26757 26857)
ETHAPI_HOST_PORTS=(8645 8745 8845)
RESOLVER_HOST_PORTS=(26655 26755 26855)
OBJECTS_HOST_PORTS=(8001 8002 8003)
IROH_RPC_HOST_PORTS=(4921 4922 4923)

FENDERMINT_METRICS_HOST_PORTS=(9184 9185 9186)
IROH_METRICS_HOST_PORTS=(9091 9092 9093)
PROMTAIL_AGENT_HOST_PORTS=(9080 9081 9082)

PROMETHEUS_HOST_PORT=9090
LOKI_HOST_PORT=3100
GRAFANA_HOST_PORT=3000
ANVIL_HOST_PORT=8545
RELAYER_METRICS_HOST_PORT=9187

if [[ -z ${PARENT_ENDPOINT+x} ]]; then
  if [[ $local_deploy == true ]]; then
    PARENT_ENDPOINT="http://anvil:${ANVIL_HOST_PORT}"
  else
    PARENT_ENDPOINT="https://calibration.node.glif.io/archive/lotus/rpc/v1"
  fi
fi

# Install build dependencies
if [[ -z ${SKIP_DEPENDENCIES+x} || "$SKIP_DEPENDENCIES" == "" || "$SKIP_DEPENDENCIES" == "false" ]]; then
  echo "${DASHES} Installing build dependencies..."
  if [[ $(uname -s) == "Linux" ]]; then
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
  elif [[ $(uname -s) == "Darwin" ]]; then
    echo "WARNING: Installing build dependencies not supported for MacOS"
    echo "$DASHES Checking if dependencies already installed..."
    missing_dependencies=()
    # Check rust + cargo
    echo "$DASHES Check rustc & cargo..."
    if which cargo &> /dev/null ; then
      echo "$DASHES rustc & cargo already installed."
    else
      echo "$DASHES Need to install rustc & cargo"
      missing_dependencies+=("rustc & cargo")
    fi
    # Check Foundry
    echo "$DASHES Check foundry..."
    if which anvil &> /dev/null ; then
      echo "$DASHES foundry is already installed."
    else
      echo "$DASHES Need to install foundry"
      missing_dependencies+=("foundry")
    fi
    # Check node
    echo "$DASHES Check node..."
    if which node &> /dev/null ; then
      echo "$DASHES node is already installed."
    else
      echo "$DASHES Need to install node"
      missing_dependencies+=("node")
    fi
    # Check docker
    echo "$DASHES check docker"
    if which docker &> /dev/null ; then
      echo "$DASHES docker is already installed."
    else
      echo "$DASHES Need to install docker"
      missing_dependencies+=("docker")
    fi  
    # Check jq
    echo "$DASHES Check jq..."
    if which jq &> /dev/null ; then
      echo "$DASHES jq is already installed."
    else
      echo "$DASHES Need to install jq"
      missing_dependencies+=("jq")
    fi
    # Check `toml`
    echo "$DASHES Check toml..."
    if which toml &> /dev/null ; then
      echo "$DASHES toml-cli is already installed."
    else
      echo "$DASHES Need to install toml-cli"
      missing_dependencies+=("toml-cli")
    fi
    if [ ${#missing_dependencies[@]} -gt 0 ]; then
      echo "$DASHES Missing dependencies: ${missing_dependencies[*]}"
      exit 1
    else
      echo "$DASHES All dependencies are installed"
    fi
  else
    echo "${DASHES} Unsupported OS: $(uname -s)"
    exit 1
  fi
else
  echo "$DASHES skipping dependencies installation $DASHES"
fi

# Prepare code repo
echo "$DASHES Preparing ipc repo..."
if ! $local_deploy ; then
  dir=$(dirname -- "$(readlink -f -- "${BASH_SOURCE[0]}")")
  source "$dir/ssh_util.sh"
  setup_ssh_config
  add_ssh_keys
  if ! ls "$IPC_FOLDER" ; then
    git clone git@github.com-ipc:hokunet/ipc.git "${IPC_FOLDER}"
  fi
  cd "${IPC_FOLDER}"
  git fetch
  git stash
  git checkout "$head_ref"
  git pull --rebase origin "$head_ref"
  update_gitmodules
  git submodule sync
  git submodule update --init --recursive
  revert_gitmodules
else
  git submodule sync
  git submodule update --init --recursive
fi

# Stop relayer
cd "$IPC_FOLDER"
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=relayer \
    relayer-destroy

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

# Shut down any existing validator nodes
if [ -e "${IPC_CONFIG_FOLDER}/config.toml" ]; then
    subnet_id=$(toml get -r "${IPC_CONFIG_FOLDER}"/config.toml 'subnets[1].id')
    echo "Existing subnet id: $subnet_id"
    # Stop validators
    cd "$IPC_FOLDER"
    for i in {0..2}
    do
      cargo make --makefile infra/fendermint/Makefile.toml \
          -e NODE_NAME=validator-"$i" \
          -e SUBNET_ID="$subnet_id" \
          child-validator-down
    done
fi

# Remove existing deployment
rm -rf "$IPC_CONFIG_FOLDER"
mkdir -p "$IPC_CONFIG_FOLDER"

# Copy configs
if ! $local_deploy ; then
  echo "$DASHES using calibration net config $DASHES"
  cp "$HOME"/evm_keystore.json "$IPC_CONFIG_FOLDER"
  cp "$IPC_FOLDER"/scripts/deploy_subnet/.ipc-cal/config.toml "$IPC_CONFIG_FOLDER"
else
  echo "$DASHES using localnet config $DASHES"
  cp "$IPC_FOLDER"/scripts/deploy_subnet/.ipc-local/config.toml "$IPC_CONFIG_FOLDER"
fi
cp "$IPC_FOLDER"/infra/prometheus/prometheus.yaml "$IPC_CONFIG_FOLDER"
cp "$IPC_FOLDER"/infra/loki/loki-config.yaml "$IPC_CONFIG_FOLDER"
cp "$IPC_FOLDER"/infra/promtail/promtail-config.yaml "$IPC_CONFIG_FOLDER"
cp "$IPC_FOLDER"/infra/iroh/iroh.config.toml "$IPC_CONFIG_FOLDER"

# Explicitly set the chain ID if not provided
if [[ -z ${CHAIN_ID+x} ]]; then
  if [[ $local_deploy == true ]]; then
    CHAIN_ID=248163216
  else
    CHAIN_ID=2481632
  fi
fi
echo "Using chain ID: $CHAIN_ID"

if [[ -z ${SKIP_BUILD+x} || "$SKIP_BUILD" == "" || "$SKIP_BUILD" == "false" ]]; then
  # Build contracts
  echo "$DASHES Building ipc contracts..."
  cd "${IPC_FOLDER}"/contracts
  make gen

  # Build ipc-cli
  echo "$DASHES Building ipc-cli..."
  cd "${IPC_FOLDER}"/ipc
  make install

  # Pull foundry image
  if [[ $local_deploy = true ]]; then
    echo "$DASHES Pulling foundry image..."
    cd "$IPC_FOLDER"
    cargo make --makefile infra/fendermint/Makefile.toml \
      anvil-pull
  fi
fi

if [[ $local_deploy = true ]]; then
  # note: the subnet hasn't been created yet, but it's always the same value and we need it for the docker network name
  subnet_id="/r31337/t410f6gbdxrbehnaeeo4mrq7wc5hgq6smnefys4qanwi"
  cd "$IPC_FOLDER"
  cargo make --makefile infra/fendermint/Makefile.toml \
      -e NODE_NAME=anvil \
      anvil-destroy

  cd "$IPC_FOLDER"
  cargo make --makefile infra/fendermint/Makefile.toml \
      -e NODE_NAME=anvil \
      -e SUBNET_ID="$subnet_id" \
      -e ANVIL_HOST_PORT="${ANVIL_HOST_PORT}" \
      anvil-start

  # the `ipc-cli wallet import` writes keys to the file non-deterministically.
  # instead, we just create the file with the predictable ordering & expected format.
  # provide the first ten anvil preloaded key pairs
  anvil_private_keys=(
    ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
    59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d
    5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a
    7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6
    47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a
    8b3a350cf5c34c9194ca85829a2df0ec3153be0318b5e2d3348e872092edffba
    92db14e403b83dfe3df233f83dfa3a0d7096f21ca9b0d6d6b8d88b2b4ec1564e
    4bbbf85ce3377467afe5d46f804f221813b2bb87f24d81f60f1fcdbf7cbf4356
    dbda1821b80551c9d65939329250298aa3472ba22feea921c0cf5d620ea67b97
    2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6
  )
  # lowercased addresses in matching order (`ipc-cli` expects lowercase)
  anvil_public_keys=(
    0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266
    0x70997970c51812dc3a010c7d01b50e0d17dc79c8
    0x3c44cdddb6a900fa2b585dd299e03d12fa4293bc
    0x90f79bf6eb2c4f870365e785982e1f101e93b906
    0x15d34aaf54267db7d7c367839aaf71a00a2c6a65
    0x9965507d1a55bcc2695c58ba16fb37d819b0a4dc
    0x976ea74026e726554db657fa54763abd0c3a0aa9
    0x14dc79964da2c08b23698b3d3cc7ca32193d9955
    0x23618e81e3f5cdf7f54c3d65f7fbc0abf5b21e8f
    0xa0ee7a142d267c1f36714e4a8f75612f20a79720
  )
  evm_keystore_json=$(jq -n '[]')
  for ((i=0; i<${#anvil_private_keys[@]}; i++))
  do
    evm_keystore_json=$(echo "$evm_keystore_json" | jq --arg address "${anvil_public_keys[i]}" --arg private_key "${anvil_private_keys[i]}" '. += [{address: $address, private_key: $private_key}]')
  done
  echo "$evm_keystore_json" > "${IPC_CONFIG_FOLDER}/evm_keystore.json"
fi

# Prepare wallet by using existing wallet json file
echo "$DASHES Using 3 addresses in wallet..."
for i in {0..2}
do
  addr=$(jq .["$i"].address < "${IPC_CONFIG_FOLDER}"/evm_keystore.json | tr -d '"')
  wallet_addresses+=("$addr")
  echo "Wallet $i address: $addr"
done

default_wallet_address=${wallet_addresses[0]}
echo "Default wallet address: $default_wallet_address"

# Export validator private keys into files
for i in {0..2}
do
  ipc-cli wallet export --wallet-type evm --address "${wallet_addresses[i]}" --hex > "${IPC_CONFIG_FOLDER}"/validator_"${i}".sk
  echo "Export private key for ${wallet_addresses[i]} to ${IPC_CONFIG_FOLDER}/validator_${i}.sk"
done

# Update IPC config file with parent auth token
if ! $local_deploy ; then
  toml set "${IPC_CONFIG_FOLDER}"/config.toml 'subnets[0].config.auth_token' "$PARENT_HTTP_AUTH_TOKEN" > /tmp/config.toml.0
  cp /tmp/config.toml.0 "${IPC_CONFIG_FOLDER}"/config.toml
fi

# Setup Hoku contracts
cd "${IPC_FOLDER}/hoku-contracts"
# need to run clean or we hit upgradeable safety validation errors resulting
# from contracts with the same name
forge clean
if [[ -z ${SKIP_BUILD+x} || "$SKIP_BUILD" == "" || "$SKIP_BUILD" == "false" ]]; then
  forge install
fi
cd "$IPC_FOLDER"

# Deploy IPC contracts
rpc_url=$(if $local_deploy; then echo "http://localhost:${ANVIL_HOST_PORT}"; else echo "https://calibration.filfox.info/rpc/v1"; fi)
pk=$(cat "${IPC_CONFIG_FOLDER}"/validator_0.sk)
if [[ -z "${PARENT_GATEWAY_ADDRESS+x}" || -z "${PARENT_REGISTRY_ADDRESS+x}" ]]; then
  echo "$DASHES Deploying new IPC contracts..."
  cd "${IPC_FOLDER}"/contracts
  rm -rf deployments/localnet

  if ! $local_deploy ; then
    deploy_contracts_output=$(PRIVATE_KEY="${pk}" RPC_URL="${rpc_url}" make deploy-stack NETWORK=calibrationnet)
  else
    deploy_contracts_output=$(PRIVATE_KEY="${pk}" RPC_URL="${rpc_url}" make deploy-stack NETWORK=localnet)
  fi

  echo "$DASHES deploy contracts output $DASHES"
  echo ""
  echo "$deploy_contracts_output"
  echo ""

  PARENT_GATEWAY_ADDRESS=$(echo "$deploy_contracts_output" | grep 'GatewayDiamond' | awk 'NR==2 {printf "%s", $NF}')
  PARENT_REGISTRY_ADDRESS=$(echo "$deploy_contracts_output" | grep 'SubnetRegistryDiamond' | awk 'NR==2 {printf "%s", $NF}')

  if [ $local_deploy == true ]; then
    cd "${IPC_FOLDER}/hoku-contracts"
    # use the same account validator 0th account to deploy supply source token
    deploy_supply_source_token_out="$(forge script script/Hoku.s.sol --private-key "${pk}" --rpc-url "${rpc_url}" --tc DeployScript --sig 'run(string)' local --broadcast --timeout 120 -vv)"
    echo "$DASHES deploy supply source token output $DASHES"
    echo ""
    echo "$deploy_supply_source_token_out"
    echo ""
    # note: this is consistently going to be
    # 0x4A679253410272dd5232B3Ff7cF5dbB88f295319 for localnet
    SUPPLY_SOURCE_ADDRESS=$(echo "$deploy_supply_source_token_out" | sed -n 's/.*contract Hoku *\([^ ]*\).*/\1/p')

    # use the same account validator 0th account to deploy validator rewarder
    forge clean
    deploy_validator_rewarder_token_out="$(forge script script/ValidatorRewarder.s.sol:DeployScript --private-key "${pk}" --rpc-url "${rpc_url}" --sig "run(address)" "${SUPPLY_SOURCE_ADDRESS}" --broadcast --timeout 120 -vv)"
    echo "$DASHES deploy validator rewarder token output $DASHES"
    echo ""

    echo "$deploy_validator_rewarder_token_out"
    echo ""
    VALIDATOR_REWARDER_ADDRESS=$(echo "$deploy_validator_rewarder_token_out" | sed -n 's/.*contract ValidatorRewarder *\([^ ]*\).*/\1/p')

    # fund the all anvil accounts with 10100 HOKU (note the extra 100 HOKU)
    token_amount="10100000000000000000000"
    # Get the role hashes
    MINTER_ROLE=$(cast keccak "MINTER_ROLE")
    # Grant minter role to the ValidatorRewarder contract
    cast send --private-key "${pk}" --rpc-url "${rpc_url}" --timeout 120 "${SUPPLY_SOURCE_ADDRESS}" \
      "grantRole(bytes32,address)" "${MINTER_ROLE}" "${VALIDATOR_REWARDER_ADDRESS}"
    echo "Granted minter role to ValidatorRewarder at ${VALIDATOR_REWARDER_ADDRESS}"

    for i in {0..9}
    do
      addr=$(jq .["$i"].address < "${IPC_CONFIG_FOLDER}"/evm_keystore.json | tr -d '"')
      cast send --private-key "$pk" --rpc-url "$rpc_url" --timeout 120 "$SUPPLY_SOURCE_ADDRESS" "mint(address,uint256)" "$addr" "$token_amount"
    done
    echo "Funded accounts with HOKU on anvil rootnet"
  fi
  cd "$IPC_FOLDER"
fi
echo "Parent gateway address: $PARENT_GATEWAY_ADDRESS"
echo "Parent registry address: $PARENT_REGISTRY_ADDRESS"
echo "Parent supply source address: $SUPPLY_SOURCE_ADDRESS"
echo "Parent validator rewarder address: $VALIDATOR_REWARDER_ADDRESS"

# use the same account validator 0th account to deploy validator gater
cd "${IPC_FOLDER}/hoku-contracts"
forge clean && forge build # Note: required to avoid upgradeable safety validation errors with `ValidatorGater.sol`
gas_mult=$(if $local_deploy; then echo 130; else echo 100000; fi)
deploy_validator_gater_token_out="$(forge script script/ValidatorGater.s.sol --private-key "${pk}" --rpc-url "${rpc_url}" --tc DeployScript --sig 'run()' --broadcast --timeout 120 -g "${gas_mult}" -vv)"

echo "$DASHES deploy validator gater output $DASHES"
echo ""
echo "$deploy_validator_gater_token_out"
echo ""
# note: this is consistently going to be
# 0xf5059a5D33d5853360D16C683c16e67980206f36 for localnet
VALIDATOR_GATER_ADDRESS=$(echo "$deploy_validator_gater_token_out" | sed -n 's/.*contract ValidatorGater *\([^ ]*\).*/\1/p')
echo "Parent validator gater address: $VALIDATOR_GATER_ADDRESS"
cd "$IPC_FOLDER"

# Approve each validator to stake
for i in {0..2}
do
  # Approve power min 1 HOKU max 10 HOKU
  cast send --private-key "$pk" --rpc-url "$rpc_url" --timeout 120 "$VALIDATOR_GATER_ADDRESS" "approve(address,uint256,uint256)" "${wallet_addresses[i]}" 1000000000000000000 100000000000000000000
done
echo "Approved validators to stake"

# Use the parent gateway and registry address to update IPC config file
toml set "${IPC_CONFIG_FOLDER}"/config.toml 'subnets[0].config.gateway_addr' "$PARENT_GATEWAY_ADDRESS" > /tmp/config.toml.1
toml set /tmp/config.toml.1 'subnets[0].config.registry_addr' "$PARENT_REGISTRY_ADDRESS" > /tmp/config.toml.2
cp /tmp/config.toml.2 "${IPC_CONFIG_FOLDER}"/config.toml

# Create a subnet
echo "$DASHES Creating a child subnet..."
root_id=$(toml get "${IPC_CONFIG_FOLDER}"/config.toml 'subnets[0].id' | tr -d '"')
echo "Using root: $root_id"
bottomup_check_period=600
if [[ $local_deploy = true ]]; then
  bottomup_check_period=10 # ~15 seconds on localnet
fi
create_subnet_output=$(ipc-cli subnet create \
    --from "$default_wallet_address" \
    --parent "$root_id" \
    --min-validators 2 \
    --min-validator-stake 1 \
    --bottomup-check-period "${bottomup_check_period}" \
    --active-validators-limit 4 \
    --permission-mode collateral \
    --supply-source-kind erc20 \
    --supply-source-address "$SUPPLY_SOURCE_ADDRESS" \
    --validator-rewarder "$VALIDATOR_REWARDER_ADDRESS" \
    --validator-gater "$VALIDATOR_GATER_ADDRESS" \
    --collateral-source-kind erc20 \
    --collateral-source-address "$SUPPLY_SOURCE_ADDRESS" \
    2>&1)

echo "$DASHES create subnet output $DASHES"
echo
echo "$create_subnet_output"
echo

subnet_id=$(echo "$create_subnet_output" | sed -n 's/.*with id: *\([^ ]*\).*/\1/p')
subnet_root=$(echo "$subnet_id" | awk -F'[/]' '{print $2}' | sed 's/[^0-9]//g')
subnet_f4_addr=${subnet_id##*/}
subnet_eth_addr=$(ipc-cli util f4-to-eth-addr --addr "$subnet_f4_addr" | sed -n 's/.*\(0x[0-9a-fA-F]\{40\}\).*/\1/p')
echo "Created new subnet id: $subnet_id ($subnet_eth_addr)"

subnet_struct="($subnet_root, [$subnet_eth_addr])"
cast send --private-key "$pk" --rpc-url "$rpc_url" --timeout 120 "$VALIDATOR_GATER_ADDRESS" "setSubnet((uint64,address[]))" "$subnet_struct"
echo "Set validator gater subnet ID"

cast send --private-key "$pk" --rpc-url "$rpc_url" --timeout 120 "$VALIDATOR_REWARDER_ADDRESS" "setSubnet((uint64,address[]), uint256)" "$subnet_struct" "$bottomup_check_period"
echo "Set validator rewarder subnet ID"

# Use the new subnet ID to update IPC config file
toml set "${IPC_CONFIG_FOLDER}"/config.toml 'subnets[1].id' "$subnet_id" > /tmp/config.toml.3
cp /tmp/config.toml.3 "${IPC_CONFIG_FOLDER}"/config.toml

if ! $local_deploy ; then
  # Force a wait to make sure the subnet is confirmed as created in the parent contracts
  echo "Wait for deployment..."
  sleep 30
  echo "Finished waiting"
fi

echo "$DASHES Join subnet for validators $DASHES"
for i in {0..2}
do
  echo "Joining subnet ${subnet_id} for validator ${wallet_addresses[i]}"
  # Approve subnet contract to lock up to 10 HOKU from collateral contract (which is also the supply source contract)
  vpk=$(cat "${IPC_CONFIG_FOLDER}"/validator_"$i".sk)
  cast send --private-key "$vpk" --rpc-url "$rpc_url" --timeout 120 "$SUPPLY_SOURCE_ADDRESS" "approve(address,uint256)" "$subnet_eth_addr" 10000000000000000000
  # Join and stake 10 HOKU
  ipc-cli subnet join --from "${wallet_addresses[i]}" --subnet "$subnet_id" --collateral 10
done

if [[ -z ${SKIP_BUILD+x} || "$SKIP_BUILD" == "" || "$SKIP_BUILD" == "false" ]]; then
  echo "$DASHES Building new fendermint docker image $DASHES"
  # Rebuild fendermint docker
  cd "${IPC_FOLDER}"/fendermint
  make clean
  make docker-build
fi

# Start the bootstrap validator node
echo "$DASHES Start the first validator node as bootstrap"
cd "${IPC_FOLDER}"

bootstrap_output=$(cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=validator-0 \
    -e PRIVATE_KEY_PATH="${IPC_CONFIG_FOLDER}"/validator_0.sk \
    -e SUBNET_ID="${subnet_id}" \
    -e CHAIN_ID="${CHAIN_ID}" \
    -e PARENT_ENDPOINT="${PARENT_ENDPOINT}" \
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
    -e IROH_CONFIG_FOLDER="${IPC_FOLDER}/infra/iroh/" \
    -e PARENT_HTTP_AUTH_TOKEN="${PARENT_HTTP_AUTH_TOKEN}" \
    -e PARENT_AUTH_FLAG="${PARENT_AUTH_FLAG}" \
    -e PARENT_REGISTRY="${PARENT_REGISTRY_ADDRESS}" \
    -e PARENT_GATEWAY="${PARENT_GATEWAY_ADDRESS}" \
    -e FM_PULL_SKIP=1 \
    -e FM_LOG_LEVEL="${FM_LOG_LEVEL}" \
    -e FM_LOG_DOMAINS="${FM_LOG_DOMAINS}" \
    child-validator 2>&1)
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
cd "${IPC_FOLDER}"
for i in {1..2}
do
  cargo make --makefile infra/fendermint/Makefile.toml \
      -e NODE_NAME=validator-"${i}" \
      -e PRIVATE_KEY_PATH="${IPC_CONFIG_FOLDER}"/validator_"${i}".sk \
      -e SUBNET_ID="${subnet_id}" \
      -e CHAIN_ID="${CHAIN_ID}" \
      -e PARENT_ENDPOINT="${PARENT_ENDPOINT}" \
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
      -e IROH_CONFIG_FOLDER="${IPC_FOLDER}/infra/iroh/" \
      -e RESOLVER_BOOTSTRAPS="${bootstrap_resolver_endpoint}" \
      -e BOOTSTRAPS="${bootstrap_node_endpoint}" \
      -e PARENT_HTTP_AUTH_TOKEN="${PARENT_HTTP_AUTH_TOKEN}" \
      -e PARENT_AUTH_FLAG="${PARENT_AUTH_FLAG}" \
      -e PARENT_REGISTRY="${PARENT_REGISTRY_ADDRESS}" \
      -e PARENT_GATEWAY="${PARENT_GATEWAY_ADDRESS}" \
      -e FM_PULL_SKIP=1 \
      -e FM_LOG_LEVEL="${FM_LOG_LEVEL}" \
      -e FM_LOG_DOMAINS="${FM_LOG_DOMAINS}" \
      child-validator
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
echo
echo "$DASHES Test Object API endpoints of validator nodes"
for i in {0..2}
do
  curl --location http://localhost:"${OBJECTS_HOST_PORTS[i]}"/health
done

# Test Prometheus endpoints
echo "$DASHES Test Prometheus endpoints of validator nodes"
curl -s -o /dev/null -w "%{http_code}" --location http://localhost:"${PROMETHEUS_HOST_PORT}"/graph
echo
for i in {0..2}
do
  curl -s -o /dev/null -w "%{http_code}" --location http://localhost:"${FENDERMINT_METRICS_HOST_PORTS[i]}"/metrics
  echo
done

# Start relayer
# note: this command mutates the order of keys in the evm_keystore.json file. to
# keep the accounts consistent for usage (e.g., logging accounts, using
# validator keys, etc.), we temporarily copy the file and then restore it.
echo "$DASHES Start relayer process (in the background)"
if [[ $local_deploy = true ]]; then
  toml set "${IPC_CONFIG_FOLDER}"/config.toml 'subnets[0].config.provider_http' 'http://anvil:8545' > /tmp/config.toml.4
  toml set /tmp/config.toml.4 'subnets[1].config.provider_http' 'http://validator-0-ethapi:8545' > "${IPC_CONFIG_FOLDER}"/relayer.config.toml
else
  toml set "${IPC_CONFIG_FOLDER}"/config.toml 'subnets[1].config.provider_http' 'http://validator-0-ethapi:8545' > "${IPC_CONFIG_FOLDER}"/relayer.config.toml
fi
temp_evm_keystore=$(jq . "${IPC_CONFIG_FOLDER}"/evm_keystore.json)
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=relayer \
    -e SUBNET_ID="$subnet_id" \
    -e RELAYER_SUBMITTER="${default_wallet_address}" \
    -e RELAYER_METRICS_HOST_PORT="${RELAYER_METRICS_HOST_PORT}" \
    relayer-start
sleep 3 # briefly wait for the relayer to start
echo "$temp_evm_keystore" > "${IPC_CONFIG_FOLDER}"/evm_keystore.json

# move localnet funds to subnet
BUCKET_MANAGER_ADDRESS=""
CREDIT_MANAGER_ADDRESS=""
if [[ $local_deploy = true ]]; then
  echo "$DASHES Move account funds into subnet"
  # move 10000 HOKU to subnet (i.e., leave 100 HOKU on rootnet for
  # testing purposes)
  # note: see comment above about why we're using 10**18 due 
  # to `ipc-cli` & `hoku` CLI's atto assumption
  token_amount="10000000000000000000000"
  for i in {0..9}
  do
    addr=$(jq .["$i"].address < "${IPC_CONFIG_FOLDER}"/evm_keystore.json | tr -d '"')
    ipc-cli cross-msg fund-with-token --subnet "${subnet_id}" --from "${addr}" --approve "${token_amount}"
  done
  echo "Waiting for deposits to process..."
  # TODO: this takes ~2 minutes for the topdown messages to propagate. need to reduce this for 
  # localnet (i.e., ideally is seconds, not minutes)
  while true; do
    # validate accounts have subnet balance (the last account will be final deposit tx)
    addr=$(jq .[9].address < "${IPC_CONFIG_FOLDER}"/evm_keystore.json | tr -d '"')
    balance=$(cast balance --rpc-url http://localhost:"${ETHAPI_HOST_PORTS[0]}" --ether "${addr}" | awk '{printf "%.0f", $1}')
    if [[ $balance != 0 ]]; then
      break
    fi
    sleep 5
  done
  echo "Deposited HOKU for test accounts"
  # buy 5000 credits if the hoku CLI is installed
  if [[ -n $(which hoku) ]]; then
    echo "Buying credits for test accounts..."
    credit_amount="5000"
    for i in {0..9}
    do
      private_key=$(jq .["$i"].private_key < "${IPC_CONFIG_FOLDER}"/evm_keystore.json | tr -d '"')
      HOKU_PRIVATE_KEY="${private_key}" HOKU_NETWORK=localnet hoku account credit buy "${credit_amount}"
    done
    echo "Bought subnet credits for test accounts"
  else
    echo "Hoku CLI not installed...skipping credit funding"
  fi
  echo
  
  # Deploy the bucket and credit manager contracts
  # Note: due to validators also submitting checkpoints, it's impossible to get around nonce issues
  # with validator accounts. So, we deploy the bucket and credit manager contracts via the last evm
  # account. Ideally, we would use validator 0 and somehow avoid nonce clashes.
  deployer_pk=$(jq .[9].private_key < "${IPC_CONFIG_FOLDER}"/evm_keystore.json | tr -d '"')
  cd "${IPC_FOLDER}/hoku-contracts"
  echo "$DASHES deploy bucket and credit manager output $DASHES"
  deploy_blob_manager_token_out="$(forge script script/BlobManager.s.sol --private-key "${deployer_pk}" --rpc-url http://localhost:"${ETHAPI_HOST_PORTS[0]}" --tc DeployScript --sig 'run()' --broadcast --timeout 120 -g 100000 -vv)"
  deploy_bucket_manager_token_out="$(forge script script/BucketManager.s.sol --private-key "${deployer_pk}" --rpc-url http://localhost:"${ETHAPI_HOST_PORTS[0]}" --tc DeployScript --sig 'run()' --broadcast --timeout 120 -g 100000 -vv)"
  deploy_credit_manager_token_out="$(forge script script/CreditManager.s.sol --private-key "${deployer_pk}" --rpc-url http://localhost:"${ETHAPI_HOST_PORTS[0]}" --tc DeployScript --sig 'run()' --broadcast --timeout 120 -g 100000 -vv)"
  echo ""
  echo "$deploy_bucket_manager_token_out"
  echo "$deploy_credit_manager_token_out"
  echo ""
  # note: these are consistently going to be 0xe1Aa25618fA0c7A1CFDab5d6B456af611873b629 and
  # 0xf7Cd8fa9b94DB2Aa972023b379c7f72c65E4De9D, respectively, for localnet
  BLOB_MANAGER_ADDRESS=$(echo "$deploy_blob_manager_token_out" | sed -n 's/.*contract BlobManager *\([^ ]*\).*/\1/p')
  BUCKET_MANAGER_ADDRESS=$(echo "$deploy_bucket_manager_token_out" | sed -n 's/.*contract BucketManager *\([^ ]*\).*/\1/p')
  CREDIT_MANAGER_ADDRESS=$(echo "$deploy_credit_manager_token_out" | sed -n 's/.*contract CreditManager *\([^ ]*\).*/\1/p')
  echo "Blob manager address: ${BLOB_MANAGER_ADDRESS}"
  echo "Bucket manager address: ${BUCKET_MANAGER_ADDRESS}"
  echo "Credit manager address: ${CREDIT_MANAGER_ADDRESS}"
  echo
  echo "${DASHES} Subnet setup complete ${DASHES}"
  echo
  cd "$IPC_FOLDER"
fi

# Print a summary of the deployment
cat << EOF
#############################
#                           #
# Hoku deployment ready! 🚀 #
#                           #
#############################
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

Contracts:
Parent gateway:            ${PARENT_GATEWAY_ADDRESS}
Parent registry:           ${PARENT_REGISTRY_ADDRESS}
Parent supply source:      ${SUPPLY_SOURCE_ADDRESS}
Parent validator gater:    ${VALIDATOR_GATER_ADDRESS}
Parent validator rewarder: ${VALIDATOR_REWARDER_ADDRESS}
Subnet gateway:            0x77aa40b105843728088c0132e43fc44348881da8
Subnet registry:           0x74539671a1d2f1c8f200826baba665179f53a1b7
EOF

if [[ $local_deploy = true ]]; then
  echo "Subnet blob manager:    ${BLOB_MANAGER_ADDRESS}"
  echo "Subnet bucket manager:  ${BUCKET_MANAGER_ADDRESS}"
  echo "Subnet credit manager:  ${CREDIT_MANAGER_ADDRESS}"
  echo
  echo "Account balances:"
  addr=$(jq .[3].address < "${IPC_CONFIG_FOLDER}"/evm_keystore.json | tr -d '"')
  parent_native=$(cast balance --rpc-url http://localhost:"${ANVIL_HOST_PORT}" --ether "${addr}" | awk '{printf "%.2f", $1}')
  parent_hoku=$(cast balance --rpc-url http://localhost:"${ANVIL_HOST_PORT}" --erc20 "${SUPPLY_SOURCE_ADDRESS}" "${addr}" | awk '{printf "%.0f", $1 / 1000000000000000000}')
  subnet_native=$(cast balance --rpc-url http://localhost:"${ETHAPI_HOST_PORTS[0]}" --ether "${addr}" | awk '{printf "%.2f", $1}')
  echo "Parent native: ${parent_native%.*} ETH"
  echo "Parent HOKU:   ${parent_hoku%.*} HOKU"
  echo "Subnet native: ${subnet_native%.*} HOKU"
  if [[ -n $(which hoku) ]]; then
    credit_balance=$(HOKU_NETWORK=localnet hoku account info --address "${addr}" | jq '.credit.credit_free' | tr -d '"')
    echo "Subnet credits: ${credit_balance}"
  fi
  echo
  echo "Accounts:"
  for i in {0..9}
  do
    addr=$(jq .["$i"].address < "${IPC_CONFIG_FOLDER}"/evm_keystore.json | tr -d '"')
    # validator accounts should *not* be used by devs to avoid nonce race conditions
    type="available"
    if [[ $i -eq 0 || $i -eq 1 || $i -eq 2 ]]; then
      type="reserved"
    fi
    echo "($i) $addr ($type)"
  done
  echo
  echo "Private keys:"
  for i in {0..9}
  do
    private_key=$(jq .["$i"].private_key < "${IPC_CONFIG_FOLDER}"/evm_keystore.json | tr -d '"')
    echo "($i) $private_key"
  done
  echo
fi

echo "Done"
exit 0
