# IPC Quick Start: zero-to-subnet (on Calibration)

>💡 Background and detailed are available in the [README](/README.md).

Ready to test the waters with your first subnet? This guide will deploy a subnet with three local validators orchestrated by `ipc-cli`. This subnet will be anchored to the public [Calibration testnet](https://docs.filecoin.io/networks/calibration/details/). This will be a minimal example and may not work on all systems. The full documentation provides more details on each step.

Several steps in this guide involve running long-lived processes. In each of these cases, the guide advises starting a new *session*. Depending on your set-up, you may do this using tools like `screen` or `tmux`, or, if using a graphical environment, by opening a new terminal tab, pane, or window.

<!-- >💡A video walkthrough of this guide is current being prepared. We still encourage you to try it for yourself! -->

<!-- >💡If you're only looking to connect to an existing subnet, please see the [README](deploying-hierarchy.md) instead. -->

## Step 0: Prepare your system

We assume a Ubuntu Linux instance when discussing prerequisites, but annotate steps with system-specificity and links to detailed multi-OS instructions. Exact procedures will vary for other systems, so please follow the links if running something different. Details on IPC-specific requirements can also be found in the [README](/README.md).

* Install basic dependencies [Ubuntu/Debian] ([details](https://lotus.filecoin.io/lotus/install/prerequisites/#supported-platforms))
```bash
sudo apt update && sudo apt install build-essential libssl-dev mesa-opencl-icd ocl-icd-opencl-dev gcc git bzr jq pkg-config curl clang hwloc libhwloc-dev wget ca-certificates gnupg -y 
```

* Install Rust [Linux] ([details](https://www.rust-lang.org/tools/install))
```bash
curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"
```

* Install Docker Engine [Ubuntu] ([details](https://docs.docker.com/engine/install/))
```bash
sudo install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg
sudo chmod a+r /etc/apt/keyrings/docker.gpg
echo \
  "deb [arch="$(dpkg --print-architecture)" signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
  "$(. /etc/os-release && echo "$VERSION_CODENAME")" stable" | \
  sudo tee /etc/apt/sources.list.d/docker.list > /dev/null
sudo apt-get update && sudo apt-get install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin -y
sudo usermod -aG docker $USER && newgrp docker
```


## Step 1: Build the IPC stack

Next, we'll download and build the different components (mainly, `ipc-cli` and Fendermint).

* Pick a folder where to build the IPC stack.
* Download and compile the `ipc-cli`.
```bash
git clone https://github.com/consensus-shipyard/ipc.git
(cd ipc && make build && make install-infra)
```

## Step 2: Initialise your config

* Initialise the config
```bash
./bin/ipc-cli config init
```

This should have populated an default config file with all the parameters required to connect to calibration at `~/.ipc/config.toml`. Feel free to update this configuration to fit your needs. You may need to replace the content of the config to reflect the address of the up-to-date contracts in Calibration.


* You can run `nano ~/.ipc/config.toml` to double-check that the config file has been populated with the following content:
```toml
keystore_path = "~/.ipc"

[[subnets]]
id = "/r314159"

[subnets.config]
gateway_addr = "0x56948d2CFaa2EF355B8C08Ac925202db212146D1"
network_type = "fevm"
provider_http = "https://api.calibration.node.glif.io/rpc/v1"
registry_addr = "0x6A4884D2B6A597792dC68014D4B7C117cca5668e"

# Subnet template - uncomment and adjust before using
# [[subnets]]
# id = "/r314159/<SUBNET_ID>"

# [subnets.config]
# gateway_addr = "t064"
# jsonrpc_api_http = "http://127.0.0.1:1251/rpc/v1"
# auth_token = "<AUTH_TOKEN_1>"
# network_type = "fvm"
```

## Step 3: Set up your wallets

You'll need to create a set of wallets to spawn and interact of the subnet. Please make a note of the addresses as you go along, it may make your life easier.

* Create the three different wallets
```bash
./bin/ipc-cli wallet new -w evm
./bin/ipc-cli wallet new -w evm
./bin/ipc-cli wallet new -w evm
```

* You can optionally set one of the wallets as your default so you don't have to use the `--from` flag explicitly in some of the commands:
```bash
./bin/ipc-cli wallet set-default --address <DEFAULT_ETH_ADDR> -w evm
```

<!-- * Convert the 0x addresses to f4 addresses for later usage (OWNER_1_F4, OWNER_2_F4, and OWNER_3_F4)  -->
<!-- ```bash -->
<!-- ./ipc-agent/bin/ipc-agent util eth-to-f4-addr --addr <OWNER_1> -->
<!-- ./ipc-agent/bin/ipc-agent util eth-to-f4-addr --addr <OWNER_2> -->
<!-- ./ipc-agent/bin/ipc-agent util eth-to-f4-addr --addr <OWNER_3> -->
<!-- ``` -->

* Go to the [Calibration faucet](https://faucet.calibration.fildev.network/) and get some funds sent to each of your addresses 

>💡 In case you'd like to import an EVM account into Metamask, you can use export the private key using `./bin/ipc-cli wallet export -w evm -a <ADDRESS>`. More information is available in the [EVM IPC agent support docs](./usage.md#key-management).

>💡 Note that you may hit faucet rate limits. In that case, wait a few minutes or continue with the guide and come back to this before step 9. Alternatively, you can send funds from your primary wallet to your owner wallets.


## Step 4: Create a child subnet

* The next step is to create a subnet under `/r314159` in calibration. Remember to set a default wallet or explicitly specifying the wallet from which you want to perform the action with the `--from` flag.
```bash
./bin/ipc-cli subnet create --parent /r314159 --min-validators 3 --min-validator-stake 1 --bottomup-check-period 30
```

* Make a note of the address of the subnet you created (`/r314159/<SUBNET_ID>`)

## Step 5: Join the subnet

Before we deploy the infrastructure for the subnet, we will have to bootstrap the subnet and join from our validators, putting some initial collateral into the subnet. For this, we need to send a `join` command from each of our validators from their validator owner addresses providing their corresponding public key.

* Get the public key for all of your wallets and note it down. This is the public key that each of your validators will use to sign blocks in the subnet.
```bash
./bin/ipc-cli wallet pub-key -w evm --address <WALLET_ADDR1>
./bin/ipc-cli wallet pub-key -w evm --address <WALLET_ADDR2>
./bin/ipc-cli wallet pub-key -w evm --address <WALLET_ADDR3>
```

* Join the subnet with each validator
```bash
./bin/ipc-cli subnet join --from=<WALLET_ADDR1> --subnet=/r314159/<SUBNET_ID> --collateral=10 --public-key=<PUBKEY_WALLET1>
./bin/ipc-cli subnet join --from=<WALLET_ADDR2> --subnet=/r314159/<SUBNET_ID> --collateral=10 --public-key=<PUBKEY_WALLET2>
./bin/ipc-cli subnet join --from=<WALLET_ADDR3> --subnet=/r314159/<SUBNET_ID> --collateral=10 --public-key=<PUBKEY_WALLET3>
```

## Step 6: Deploy the infrastructure
> Work in progress
> - Deploy bootstrap
> - Deploy Fendermint nodes

<!-- We can deploy the subnet nodes. Note that each node should be importing a different worker wallet key for their validator, and should be exposing different ports. If these ports are unavailable in your system, please pick different ones. -->

<!-- * Deploy and run a container for each validator, importing the corresponding wallet keys -->
<!-- ```bash -->
<!-- ./ipc-agent/bin/ipc-infra/run-subnet-docker.sh 1251 1351 /r314159/<SUBNET_ID> ~/.ipc-agent/worker-wallet1.key -->
<!-- ./ipc-agent/bin/ipc-infra/run-subnet-docker.sh 1252 1352 /r314159/<SUBNET_ID> ~/.ipc-agent/worker-wallet2.key -->
<!-- ./ipc-agent/bin/ipc-infra/run-subnet-docker.sh 1253 1353 /r314159/<SUBNET_ID> ~/.ipc-agent/worker-wallet3.key -->
<!-- ``` -->

<!-- * If the deployment is successful, each of these nodes should return the following output at the end of their logs. Save the information for the next step. -->
<!-- ``` -->
<!-- >>> Subnet /r314159/<SUBNET_ID> daemon running in container: <CONTAINER_ID_#> (friendly name: <CONTAINER_NAME_#>) -->
<!-- >>> Token to /r314159/<SUBNET_ID> daemon: <AUTH_TOKEN_#> -->
<!-- >>> Default wallet: <WORKER_#> -->
<!-- >>> Subnet validator info: -->
<!-- <VALIDATOR_ADDR_#> -->
<!-- >>> API listening in host port <PORT_#> -->
<!-- >>> Validator listening in host port <VALIDATOR_PORT_#> -->
<!-- ``` -->

## Step 7: Update the IPC Agent configuration

* Edit the `ipc-cli` configuration `config.toml`
```bash
nano ~/.ipc/config.toml
```

* Append the new subnet to the configuration
```toml
[[subnets]]
id = "/r314159/<SUBNET_ID>"
network_name = "andromeda"

[subnets.config]
gateway_addr = "t064"
accounts = ["<WORKER_1>", "<WORKER_2>", "<WORKER_3>"]
jsonrpc_api_http = "http://127.0.0.1:1251/rpc/v1"
auth_token = "<AUTH_TOKEN_1>"
network_type = "fvm"
```
## Step 10: Start validating!

We have everything in place now to start validating. Run the following script for each of the validators [**each in a new session**], passing the container names:
```bash
./ipc-agent/bin/ipc-infra/mine-subnet.sh <CONTAINER_NAME_1> 
./ipc-agent/bin/ipc-infra/mine-subnet.sh <CONTAINER_NAME_2> 
./ipc-agent/bin/ipc-infra/mine-subnet.sh <CONTAINER_NAME_3> 
```

>💡 When starting mining and reloading the config to include the new subnet, you can sometimes get errors in the agent logs saying that the checkpoint manager couldn't be spawned successfully because the on-chain ID of the validator couldn't be change. This is because the subnet hasn't been fully initialized yet. You can `./ipc-agent/bin/ipc-agent config reload` to re-spawn the checkpoint manager and fix the error.


## Step 11: Interact with your the ETH RPC
> WIP: Connect Metamask and Eth tooling.

## Step 11: What now?
> WIP: Update
* Proceed to the [usage](usage.md) guide to learn how you can test your new subnet.
* If something went wrong, please have a look at the [README](https://github.com/consensus-shipyard/ipc-agent). If it doesn't help, please join us in #ipc-help. In either case, let us know your experience!
* Please note that to repeat this guide or spawn a new subnet, you may need to change the parameters or reset your system.
