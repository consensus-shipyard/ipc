#!/bin/bash

# IPC Quick Start Script
# See also https://github.com/consensus-shipyard/ipc/blob/main/docs/ipc/quickstart-calibration.md

# Exit on any error
set -e

# Print commands as we execute
#set -x

PREFIX='----'

# Step 1: Prepare system for building and running IPC

# Step 1.1: Install build dependencies
echo "${PREFIX} Installing build dependencies..."
sudo apt update && sudo apt install build-essential libssl-dev mesa-opencl-icd ocl-icd-opencl-dev gcc git bzr jq pkg-config curl clang hwloc libhwloc-dev wget ca-certificates gnupg -y

# Step 1.2: Install rust + cargo
echo "${PREFIX} Check rustc & cargo..."
if which cargo ; then
  echo "${PREFIX} rustc & cargo already installed."
else
  echo "${PREFIX} Need to install rustc & cargo"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
fi

# Step 1.3: Install Foundry
echo "${PREFIX} Check foundry..."
if which foundryup ; then
  echo "${PREFIX} foundry is already installed."
else
  echo "${PREFIX} Need to install foundry"
  curl -L https://foundry.paradigm.xyz | bash
  foundryup
fi

# Make sure we re-read the latest env
source ${HOME}/.bashrc

