#!/usr/bin/bash

set -euxo pipefail

DASHES='------'
IPC_FOLDER=${HOME}/ipc
IPC_CLI=${IPC_FOLDER}/target/release/ipc-cli
IPC_CONFIG_FOLDER=${HOME}/.ipc
LINKED_TOKEN_FOLDER=${HOME}/ipc/extras/linked-token
DOT_ENV_FILE=${HOME}/ipc/extras/linked-token/.env
DOT_ENV_TEMPLATE=${HOME}/ipc/extras/linked-token/.env.template

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

# Step 1: Checkout code repo
#echo "${DASHES} Checking out code repo"
#if ! $local_deploy ; then
#  echo "$DASHES Preparing ipc repo..."
#  cd $HOME
#  if ! ls $IPC_FOLDER ; then
#    git clone --recurse-submodules -j8 https://github.com/consensus-shipyard/ipc.git
#  fi
#  cd ${IPC_FOLDER}/contracts
#  git fetch
#  git stash
#  git checkout $head_ref
#  git pull --rebase origin $head_ref
#  git submodule sync
#  git submodule update --init --recursive
#fi

# Step 2: Prepare wallet address
echo "$DASHES Prepare wallet address"
for i in {0..3}
do
  addr=$(cat ${IPC_CONFIG_FOLDER}/evm_keystore.json | jq .[$i].address | tr -d '"')
  private_key=$(cat ${IPC_CONFIG_FOLDER}/evm_keystore.json | jq .[$i].private_key | tr -d '"')
  if [ $addr = 'default-key' ]; then
    default_private_key=$private_key
    echo "Default private key: $default_private_key"
  else
    wallet_addresses+=($addr)
    private_keys+=($private_key)
    echo "Wallet $i address: $addr, private key: $private_key"
  fi
done

for i in {0..2}
do
  if [ ${private_keys[i]} = $default_private_key ]; then
    default_wallet_address=${wallet_addresses[i]}
    echo "Default wallet address: $default_wallet_address}"
  fi
done

# Step 2: Configure the dot env file
echo "${DASHES} Configuring .env file for linked token deployment"
cp $DOT_ENV_TEMPLATE $DOT_ENV_FILE
echo "export PRIVATE_KEY=$default_private_key" >> $DOT_ENV_FILE