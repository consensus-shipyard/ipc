# Deploy a subnet

Ready to test the waters with your first subnet? This guide will deploy a subnet with three local validators orchestrated by `ipc-cli`. This subnet will be anchored to the public [Calibration testnet](https://docs.filecoin.io/networks/calibration/details/). This will be a minimal example and may not work on all systems. The full documentation provides more details on each step.

Several steps in this guide involve running long-lived processes. In each of these cases, the guide advises starting a new _session_. Depending on your set-up, you may do this using tools like `screen` or `tmux`, or, if using a graphical environment, by opening a new terminal tab, pane, or window.

## Step 1: Prepare your system

### Install the basic requirements for IPC:

#### **For Linux:**

* Install system packages: `sudo apt update && sudo apt install build-essential clang cmake pkg-config libssl-dev protobuf-compiler git curl mesa-opencl-icd ocl-icd-opencl-dev gcc bzr jq hwloc libhwloc-dev wget ca-certificates gnupg -y`.
* Install Rust. See [instructions](https://www.rust-lang.org/tools/install).
* Install cargo-make: `cargo install --force cargo-make`.
* Install Docker. See [instructions](https://docs.docker.com/engine/install/ubuntu/).
* Install Foundry. See [instructions](https://book.getfoundry.sh/getting-started/installation).

#### For MacOS:

* Install Xcode from App Store or terminal: `xcode-select --install`
* Install Homebrew: https://brew.sh/
* Install dependencies: `brew install jq`
* Install Rust: https://www.rust-lang.org/tools/install (if you have homebrew installed rust, you may need to uninstall that if you get errors in the build)
* Install Cargo make: `cargo install --force cargo-make`
* Install docker: https://docs.docker.com/desktop/install/mac-install/
* Install foundry: https://book.getfoundry.sh/getting-started/installation

### Building:

```
# make sure that rust has the wasm32 target
rustup target add wasm32-unknown-unknown

# LINUX ONLY: add your user to the docker group
sudo usermod -aG docker $USER && newgrp docker

# clone this repo and build
git clone https://github.com/consensus-shipyard/ipc.git
cd ipc/contracts
make gen
cd ..
cargo build --release

# building will generate the following binaries
./target/release/ipc-cli --version
./target/release/fendermint --version
```

### Step 2: Initialise your config

* Initialise the config

```
alias ipc-cli="cargo run -q -p ipc-cli --release --"
```

```
ipc-cli config init
```

This should have populated a default config file with all the parameters required to connect to calibration at `~/.ipc/config.toml`. Feel free to update this configuration to fit your needs.

The IPC stack is changing rapidly. In order to make sure you use the latest contracts deployed on Filecoin Calibration:

* Run `nano ~/.ipc/config.toml` to see your configuration

```
keystore_path = "~/.ipc"

# Filecoin Calibration
[[subnets]]
id = "/r314159"

[subnets.config]
network_type = "fevm"
provider_http = "https://api.calibration.node.glif.io/rpc/v1"
gateway_addr = "0x5cF14D2Af9BBd5456Ea532639f1DB355B9BaCBf8"
registry_addr = "0x7308C4A503a12521215718cbCa98F951E9aAB9B5"

# Subnet template - uncomment and adjust before using
# [[subnets]]
# id = "/r314159/<SUBNET_ID>"

# [subnets.config]
# network_type = "fevm"
# provider_http = "https://<RPC_ADDR>/"
# gateway_addr = "0x77aa40b105843728088c0132e43fc44348881da8"
# registry_addr = "0x74539671a1d2f1c8f200826baba665179f53a1b7"
```

*   **Replace** the `gateway_addr` and `registry_addr` with the following values. Click on the badges below to take you to the source to copy and paste them or go to [this link](https://github.com/consensus-shipyard/ipc/blob/cd/contracts/deployments/r314159.json).

    [![Gateway Address](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fraw.githubusercontent.com%2Fconsensus-shipyard%2Fipc%2Fcd%2Fcontracts%2Fdeployments%2Fr314159.json\&query=%24.gateway\_addr\&label=Gateway%20Address)](https://github.com/consensus-shipyard/ipc/blob/cd/contracts/deployments/r314159.json)

    [![Registry Address](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fraw.githubusercontent.com%2Fconsensus-shipyard%2Fipc%2Fcd%2Fcontracts%2Fdeployments%2Fr314159.json\&query=%24.registry\_addr\&label=Registry%20Address)](https://github.com/consensus-shipyard/ipc/blob/cd/contracts/deployments/r314159.json)

### Step 3: Set up your wallets

You'll need to create a set of wallets to spawn and interact of the subnet. Please make a note of the addresses as you go along, it may make your life easier.

* Create the three different wallets

```
ipc-cli wallet new --wallet-type evm
ipc-cli wallet new --wallet-type evm
ipc-cli wallet new --wallet-type evm
```

* You can optionally set one of the wallets as your default so you don't have to use the `--from` flag explicitly in some of the commands:

```
ipc-cli wallet set-default --address <DEFAULT_ETH_ADDR> --wallet-type evm
```

* Go to the [Calibration faucet](https://faucet.calibration.fildev.network/) and get some funds sent to each of your addresses

> 💡 In case you'd like to import an EVM account into Metamask, you can use export the private key using `ipc-cli wallet export --wallet-type evm --address <ADDRESS>`. More information is available in the [EVM IPC agent support docs](https://github.com/consensus-shipyard/ipc/blob/main/docs/ipc/usage.md#key-management).

> 💡 Note that you may hit faucet rate limits. In that case, wait a few minutes or continue with the guide and come back to this before step 9. Alternatively, you can send funds from your primary wallet to your owner wallets.

### Step 4: Create a child subnet

* The next step is to create a subnet under `/r314159` in calibration. Remember to set a default wallet or explicitly specifying the wallet from which you want to perform the action with the `--from` flag.

```
ipc-cli subnet create --parent /r314159 --min-validator-stake 1 --min-validators 3 --bottomup-check-period 30 --from <PLEASE PUT ACCOUNT ADDRESS> --permission-mode collateral --supply-source-kind native
```

This will output your subnet ID, which you will use below.

* Make a note of the address of the subnet you created.

### Step 5: Join the subnet

Before we deploy the infrastructure for the subnet, we will have to bootstrap the subnet and join from our validators, putting some initial collateral into the subnet and giving our validator address some initial balance in the subnet. For this, we need to send a `join` command from each of our validators from their validator owner addresses providing their corresponding public key.

* Get the public key for all of your wallets and note it down. This is the public key that each of your validators will use to sign blocks in the subnet.

```
ipc-cli wallet pub-key --wallet-type evm --address <PLEASE PUT ADDRESS 1>
ipc-cli wallet pub-key --wallet-type evm --address <PLEASE PUT ADDRESS 2>
ipc-cli wallet pub-key --wallet-type evm --address <PLEASE PUT ADDRESS 3>
```

* Join the subnet with each validator

```
ipc-cli subnet join --from=<PLEASE PUT ADDRESS 1> --subnet=<PLEASE PUT SUBNET ID> --collateral=10 --public-key=<PLEASE PUT PUBLIC KEY RELATED TO ADDRESS 1> --initial-balance 1
ipc-cli subnet join --from=<PLEASE PUT ADDRESS 2> --subnet=<PLEASE PUT SUBNET ID> --collateral=10 --public-key=<PLEASE PUT PUBLIC KEY RELATED TO ADDRESS 2> --initial-balance 1
ipc-cli subnet join --from=<PLEASE PUT ADDRESS 3> --subnet=<PLEASE PUT SUBNET ID> --collateral=10 --public-key=<PLEASE PUT PUBLIC KEY RELATED TO ADDRESS 3> --initial-balance 1
```

### Step 6: Deploy the infrastructure

First we need to export the validator private keys for all or wallets into separate files.

```
ipc-cli wallet export --wallet-type evm --address <PLEASE PUT ADDRESS 1> --hex > ~/.ipc/validator_1.sk
ipc-cli wallet export --wallet-type evm --address <PLEASE PUT ADDRESS 2> --hex > ~/.ipc/validator_2.sk
ipc-cli wallet export --wallet-type evm --address <PLEASE PUT ADDRESS 3> --hex > ~/.ipc/validator_3.sk
```

Let's start our first validator and make it be the one the others will bootstrap from.

```
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=validator-1 \
    -e PRIVATE_KEY_PATH=<PLEASE PUT FULL PATH TO validator_1.sk> \
    -e SUBNET_ID=<PLEASE PUT SUBNET ID> \
    -e CMT_P2P_HOST_PORT=26656 \
    -e CMT_RPC_HOST_PORT=26657 \
    -e ETHAPI_HOST_PORT=8545 \
    -e RESOLVER_HOST_PORT=26655 \
    -e PARENT_GATEWAY=<PLEASE PUT GATEWAY_ADDR> \
    -e PARENT_REGISTRY=<PLEASE PUT REGISTRY_ADD> \
    -e FM_PULL_SKIP=1 \
    child-validator
```

Note:

* Use full path to PRIVATE\_KEY\_PATH, don't path with "\~"
* Do not change values of any port from the ones provided unless you have to

You'll need the final component of the `IPLD Resolver Multiaddress` (the `peer ID`) and the `CometBFT node ID` for the next nodes we'll start.

Let's start the second validator:

```
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=validator-2 \
    -e PRIVATE_KEY_PATH=<PLEASE PUT FULL PATH TO validator_2.sk> \
    -e SUBNET_ID=<PLEASE PUT SUBNET ID> \
    -e CMT_P2P_HOST_PORT=26756 \
    -e CMT_RPC_HOST_PORT=26757 \
    -e ETHAPI_HOST_PORT=8645 \
    -e RESOLVER_HOST_PORT=26755 \
    -e BOOTSTRAPS=<PLEASE PUT COMETBFT NODE ID of VALIDATOR-1>@validator-1-cometbft:26656 \
    -e RESOLVER_BOOTSTRAPS=/dns/validator-1-fendermint/tcp/26655/p2p/<PLEASE PUT PEER_ID of VALIDATOR-1> \
    -e PARENT_GATEWAY=<PLEASE PUT GATEWAY_ADDR> \
    -e PARENT_REGISTRY=<PLEASE PUT REGISTRY_ADD> \
    child-validator
```

Notes:

* Do not change values of any port from the ones provided unless you have to

And the third:

```
cargo make --makefile infra/fendermint/Makefile.toml \
    -e NODE_NAME=validator-3 \
    -e PRIVATE_KEY_PATH=<PLEASE PUT FULL PATH TO validator_3.sk> \
    -e SUBNET_ID=<PLEASE PUT SUBNET ID> \
    -e CMT_P2P_HOST_PORT=26856 \
    -e CMT_RPC_HOST_PORT=26857 \
    -e ETHAPI_HOST_PORT=8745 \
    -e RESOLVER_HOST_PORT=26855 \
    -e BOOTSTRAPS=<PLEASE PUT COMETBFT NODE ID of VALIDATOR-1>@validator-1-cometbft:26656 \
    -e RESOLVER_BOOTSTRAPS=/dns/validator-1-fendermint/tcp/26655/p2p/<PLEASE PUT PEER_ID of VALIDATOR-1> \
    -e PARENT_GATEWAY=<PLEASE PUT GATEWAY_ADDR> \
    -e PARENT_REGISTRY=<PLEASE PUT REGISTRY_ADD> \
    child-validator
```

Notes:

* Do not change values of any port from the ones provided unless you have to

### Step 7: Interact with your subnet using the IPC CLI

* Make sure `~/.ipc/config.toml` contains the configuration of your subnet in the "Subnet template" section. Uncomment the section and populate the corresponding fields

```
# Subnet template - uncomment and adjust before using
[[subnets]]
id = <PUT YOUR SUBNET ID>

[subnets.config]
network_type = "fevm"
provider_http = "http://localhost:8545/"
gateway_addr = "0x77aa40b105843728088c0132e43fc44348881da8"
registry_addr = "0x74539671a1d2f1c8f200826baba665179f53a1b7"
```

> 💡 The ETH addresses for `gateway_addr` and `registry_addr` used when they are deployed in genesis in a child subnet by Fendermint are `0x77aa40b105843728088c0132e43fc44348881da8` and `0x74539671a1d2f1c8f200826baba665179f53a1b7`, respectively, so no need to change them.

* Fetch the balances of your wallets using  the following command. The result should show the initial balance that you have included for your validators address in genesis:

```
ipc-cli wallet balances --wallet-type evm --subnet=<SUBNET_ID>
```

### Step 8: Run a relayer

IPC relies on the role of a specific type of peer on the network called the relayers that are responsible for submitting bottom-up checkpoints that have been finalized in a child subnet to its parent. This process is key for the commitment of child subnet checkpoints in the parent, and the execution of bottom-up cross-net messages. Without relayers, cross-net messages will only flow from top levels of the hierarchy to the bottom, but not the other way around.


* Run the relayer process passing the 0x address of the submitter account:

```
ipc-cli checkpoint relayer --subnet <SUBNET_ID> --submitter <RELAYER_ADDR>
```
