# `localnet` setup

Start a three-node IPC network for local development.

## Dependencies

### Linux

```shell
sudo apt update && sudo apt install build-essential libssl-dev mesa-opencl-icd ocl-icd-opencl-dev gcc git bzr jq pkg-config curl clang hwloc libhwloc-dev wget ca-certificates gnupg -y
```

### macOS

```shell
# TODO
```

### Install Rust

```shell
curl https://sh.rustup.rs -sSf | sh -s -- -y
```

### Install `cargo make`

```shell
cargo install cargo-make
```

### Install `toml-cli`

```shell
cargo install toml-cli
```

### Install Foundry

```shell
curl -L https://foundry.paradigm.xyz | bash
foundryup
```

### Install Node

```shell
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.3/install.sh | bash
source "$HOME/.bashrc" # This step will vary by OS 
nvm install --default lts/*
```

### Install Docker on Ubuntu

```shell
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
```

### Install Docker on macOS

See https://docs.docker.com/desktop/install/mac-install/.

### Source env

```shell
source ${HOME}/.bashrc # This step will vary by OS
```

# Running

All scripts are meant to be run from the IPC repo root by default.
If run from elsewhere, specify the path to your IPC repo with `IPC_FOLDER=/path/to/ipc/repo`.

All scripts use `cargo make` to start docker containers, volumes, and a docker network. Use `docker ps` to check status.

## Start

The `setup.sh` script will create new validator private keys and a `genesis.json` config, and move them to the IPC
config folder (`~/.ipc`).
You only need to run `setup.sh` once on your machine.

```shell
bash scripts/deploy_subnet_no_parent/setup.sh
```

Now you can start the three-node `localnet`. The IPC contracts take a while to compile the first time.

```shell
bash scripts/deploy_subnet_no_parent/start.sh
```

### Logs

Use `docker ps` to list the network's containers. Each validator has four containers.

Check validator 0's CometBFT logs:

```shell
docker logs validator-0-cometbft
```

Check validator 0's Fendermint logs:

```shell
docker logs validator-0-fendermint
```

You should see blocks being produced at ~1 per second.

### Usage with `adm` SDK/CLI

In a real subnet anchored to an EVM chain, users deposit funds into the subnet from the EVM chain.
This creates their account in the subnet FVM.
Since `localnet` isn't anchored to an EVM-chain (this may change in the future by using Hardhat),
all accounts must be added at genesis.
To make our lives easier, we reuse the validator keys for development.

The `start.sh` script prints the hex-encoded private key for each validator.
You can use these keys with the `adm` SDK and CLI.

Create an `.env` file:

```dotenv
export NETWORK=localnet
export PRIVATE_KEY=<insert a hex-encoded validator private key>
```

Source it:

```shell
source .env
```

You can now use `adm` as normal, e.g., `adm os create`.

## Restart

Rebuild and restart all validators. Consensus will restart from the last committed block. If you've made changes to
actors, you'll have to _stop and start_ instead of restarting because the actor code is committed to the chain during
genesis.

```shell
bash scripts/deploy_subnet_no_parent/restart.sh
```

## Stop

Stops all validators and deletes all data.

```shell
bash scripts/deploy_subnet_no_parent/stop.sh
```
