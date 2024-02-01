#!/bin/bash

# IPC Quick Start Script
# See also https://github.com/consensus-shipyard/ipc/blob/main/docs/ipc/quickstart-calibration.md

# Exit on any error
set -e

# Print commands as we execute
set -x

PREFIX='------'
IPC_FOLDER=${HOME}/ipc
IPC_CLI=${IPC_FOLDER}/target/release/ipc-cli
IPC_CONFIG_FOLDER=${HOME}/.ipc

wallet_addresses=()

get_wallet_addresses() {
  for i in {0..2}
  do
    addr=$(cat ${IPC_CONFIG_FOLDER}/evm_keystore.json | jq .[$i].address | tr -d '"')
    wallet_addresses+=($addr)
    echo "Wallet $i address: $addr"
  done
}

# Step 1: Prepare system for building and running IPC

# Step 1.1: Install build dependencies
#echo "${PREFIX} Installing build dependencies..."
#sudo apt update && sudo apt install build-essential libssl-dev mesa-opencl-icd ocl-icd-opencl-dev gcc git bzr jq pkg-config curl clang hwloc libhwloc-dev wget ca-certificates gnupg -y

# Step 1.2: Install rust + cargo
echo "$PREFIX Check rustc & cargo..."
if which cargo ; then
  echo "$PREFIX rustc & cargo already installed."
else
  echo "$PREFIX Need to install rustc & cargo"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
fi

# Step 1.3: Install Foundry
echo "$PREFIX Check foundry..."
if which foundryup ; then
  echo "$PREFIX foundry is already installed."
else
  echo "$PREFIX Need to install foundry"
  curl -L https://foundry.paradigm.xyz | bash
  foundryup
fi

# Make sure we re-read the latest env
source ${HOME}/.bashrc

# Step 2: Prepare code repo and build ipc-cli
echo "$PREFIX Preparing ipc repo..."
cd $HOME
if ! ls $IPC_FOLDER ; then
  git clone https://github.com/consensus-shipyard/ipc.git
fi

#echo "$PREFIX Building ipc contracts..."
#cd ${IPC_FOLDER}/contracts
#make build

#echo "$PREFIX Building ipc-cli..."
#cd ${IPC_FOLDER}/ipc
#make build

# Step 3: Prepare wallet
#echo "$PREFIX Creating 3 address in wallet..."
#for i in {1..3}
#do
#    addr=$($IPC_CLI wallet new -w evm | tr -d '"')
#    wallet_addresses+=($addr)
#    echo "Wallet $i address: $addr"
#done

# Step 3: Prepare wallet
echo "$PREFIX Using 3 address in wallet..."
get_wallet_addresses

default_wallet_address=${wallet_addresses[0]}
echo "Default wallet address: $default_wallet_address"

# Step 4: Create a subnet
#echo "$PREFIX Creating a child subnet..."
#create_subnet_output=$($IPC_CLI subnet create --parent /r314159 --min-validators 3 --min-validator-stake 1 --bottomup-check-period 30 --from $default_wallet_address --permission-mode 0 --supply-source-kind 0 2>&1)
#echo $create_subnet_output
#subnet_id=$(echo $create_subnet_output | sed 's/.*with id: \([^ ]*\).*/\1/')
#
#echo "Created subnet ID: $subnet_id"

subnet_id=/r314159/t410fqmlmt6usaeewvxdj3slk6t57ti776ycqsjp2lsa