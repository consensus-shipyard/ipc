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
    echo "[*] Building fendermint..."
    make build docker-build
    cd $PWD

    echo "[*] Updating infra scripts..."
    cp -r $infra_path/fendermint/infra/* $infra_path
    # TODO: This will no longer be necessary once https://github.com/consensus-shipyard/fendermint/pull/329
    # is merged
    mkdir -p ./target/release
    mv $infra_path/fendermint/target/release/fendermint $PWD/target/release
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


# Check if fendermint exists
if [ ! -d "$infra_path/fendermint" ]; then
    cd "$infra_path"
    echo "[*] Fendermint directory doesn't, cloning code"
    git clone https://github.com/consensus-shipyard/fendermint.git fendermint
    echo "[*] Building infrastructure assets"
    cd "fendermint"
    build_infra
    exit 0
else
    echo "[*] Fendermint code already pulled"
fi

image_name="fendermint"
cd "$infra_path/fendermint"


if [[ "$#" -gt 0 && "$1" == "-f" ]]; then
    echo "[*] -f is set. Forcing new build"
    build_infra
    exit 0
fi

if docker inspect "$image_name" &> /dev/null ; then
    echo "[*] Docker image '$image_name' already exists."
else
    build_infra
    exit 0
fi

# Perform a Git pull to update the repository
git_output=$(git pull)

# Check if there are changes
git fetch
if [[ "$(git rev-list HEAD...origin/master --count)" -gt 0  ]]; then
    echo "[*] No changes in the repository."
else
    build_infra
    exit 0
fi

# Check if there are changes
if [ -n "$(git status --porcelain)" ]; then
    build_infra
else
    echo "[*] No changes detected in the repository. Doing nothing!"
fi
