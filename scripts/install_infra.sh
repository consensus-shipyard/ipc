#!/bin/bash
#
# Installs and builds all the infrastructure required
# to run Fendermint-based subnets.

set -e

PWD=$(pwd)
infra_path="$PWD/bin/ipc-infra"
git_repo_url="https://github.com/consensus-shipyard/fendermint.git"

if ! command -v cargo-make &> /dev/null
then
    echo "[*] 'cargo make' not found. Installing..."
    cargo install cargo-make
else
    echo "[*] 'cargo make' is already installed."
fi

build_infra() {
    cd $PWD

    echo "[*] Updating infra scripts..."
    cp -r $infra_path/fendermint/infra/* $infra_path
}

# Function to display help message
function show_help() {
    echo "Usage: ./scripts/install-infra.sh [options]"
    echo "Options:"
    echo "  -h      Show help message"
    echo "  -f      Force the build of the infra"
}

# Main script logic
if [ "$1" == "-h" ]; then
    show_help
    exit 0
fi

# Check if infra path exists
if [ ! -d "$infra_path" ]; then
    echo "[*] Infra directory doesn't exist, creating infra path"
    mkdir -p $infra_path
fi


if [[ "$#" -gt 0 && "$1" == "-f" ]]; then
    echo "[*] -f is set. Force pulling the Fendermint repo again, and pulling latest scripts"
    rm -rf $infra_path/fendermint
fi


# Check if fendermint exists
if [ ! -d "$infra_path/fendermint" ]; then
    cd "$infra_path"
    echo "[*] Fendermint directory doesn't exist, cloning code"
    git clone https://github.com/consensus-shipyard/fendermint.git fendermint
    build_infra
    exit 0
else
    echo "[*] Infra scripts already installed."
fi
