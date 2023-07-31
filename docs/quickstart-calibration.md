# IPC Quick Start: zero-to-subnet (on Calibration)

>💡 Background and detailed are available in the [README](/README.md).

Ready to test the waters with your first subnet? This guide will deploy a subnet with three local validators orchestrated by the same IPC agent. This subnet will be anchored to the public [Calibration testnet](https://docs.filecoin.io/networks/calibration/details/). This will be a minimal example and may not work on all systems. The full documentation provides more details on each step.

Several steps in this guide involve running long-lived processes. In each of these cases, the guide advises starting a new *session*. Depending on your set-up, you may do this using tools like `screen` or `tmux`, or, if using a graphical environment, by opening a new terminal tab, pane, or window.

>💡A video walkthrough of this guide is current being prepared. We still encourage you to try it for yourself!

>💡If you're only looking to connect to an existing subnet, please see the [README](deploying-hierarchy.md) instead.

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

Next, we'll download and build the different components (IPC agent, docker images, and eudico).

* Pick a folder where to build the IPC stack. In this example, we'll go with `~/ipc/`.
```bash
mkdir -p ~/ipc/ && cd ~/ipc/ 
```

* Download and compile the IPC Agent (might take a while)
```bash
git clone https://github.com/consensus-shipyard/ipc-agent.git
(cd ipc-agent && make build && make install-infra)
```

## Step 2: Initialise and start the IPC Agent

* Initialise the config
```bash
./ipc-agent/bin/ipc-agent config init
nano ~/.ipc-agent/config.toml
```

* Replace the content of `config.toml` with the text below, including the reference contract on Calibration.
```toml
[server]
json_rpc_address = "0.0.0.0:3030"

[[subnets]]
id = "/r314159"
network_name = "calibration"

[subnets.config]
accounts = []
gateway_addr = "0xDcDb352D8397EA50b1b131BBfE49288CDa198591"
network_type = "fevm"
provider_http = "https://api.calibration.node.glif.io/rpc/v1"
registry_addr = "0x28337700f4432ff140360BbBEAfE3a80AcaaD1Be"
```

* [**In a new session**] Start your IPC Agent
```bash
./ipc-agent/bin/ipc-agent daemon
```


## Step 3: Set up your owner wallets

You'll need to create a set of owner wallets. Please make a note of the addresses as you go along.

* Create the owner wallets for each validator (OWNER_1, OWNER_2, and OWNER_3) 
```bash
./ipc-agent/bin/ipc-agent wallet new -w evm
./ipc-agent/bin/ipc-agent wallet new -w evm
./ipc-agent/bin/ipc-agent wallet new -w evm
```

* Copy your new wallet addresses into `~/.ipc-agent/config.toml`
```toml
...
accounts = ["<OWNER_1>", "<OWNER_2>", "<OWNER_3>"]
...
```

* Reload the config to apply the changes.
```bash
./ipc-agent/bin/ipc-agent config reload
```

* Convert the 0x addresses to f4 addresses for later usage (OWNER_1_F4, OWNER_2_F4, and OWNER_3_F4) 
```bash
./ipc-agent/bin/ipc-agent util eth-to-f4-addr --addr <OWNER_1>
./ipc-agent/bin/ipc-agent util eth-to-f4-addr --addr <OWNER_2>
./ipc-agent/bin/ipc-agent util eth-to-f4-addr --addr <OWNER_3>
```

* Go to the [Calibration faucet](https://faucet.calibration.fildev.network/) and get some funds sent to each of your addresses 

>💡 In case you'd like to import an EVM account into Metamask, you can use export the private key using `./ipc-agent/bin/ipc-agent wallet export -w evm -a <ADDRESS>`. More information is available in the [EVM IPC agent support docs](./evm-usage.md#key-management).

>💡 Note that you may hit faucet rate limits. In that case, wait a few minutes or continue with the guide and come back to this before step 9. Alternatively, you can send funds from your primary wallet to your owner wallets.


## Step 4: Set up your validator worker wallets

Mir validators do not support the use of EVM addresses to create new blocks. Therefore, we'll need to create separate worker wallets for each validator.

* First, create a worker wallet for each validator (WORKER_1, WORKER_2, and WORKER_3) 
```bash
./ipc-agent/bin/ipc-agent wallet new -w fvm --key-type secp256k1
./ipc-agent/bin/ipc-agent wallet new -w fvm --key-type secp256k1
./ipc-agent/bin/ipc-agent wallet new -w fvm --key-type secp256k1
```

* Export each wallet by substituting their addresses below
```bash
./ipc-agent/bin/ipc-agent wallet export -w fvm --address <WORKER_1> --output ~/.ipc-agent/worker-wallet1.key
./ipc-agent/bin/ipc-agent wallet export -w fvm --address <WORKER_2> --output ~/.ipc-agent/worker-wallet2.key
./ipc-agent/bin/ipc-agent wallet export -w fvm --address <WORKER_3> --output ~/.ipc-agent/worker-wallet3.key
```


## Step 5: Create a child subnet

* The next step is to create a subnet under `/r314159` in calibration
```bash
./ipc-agent/bin/ipc-agent subnet create --parent /r314159 --name andromeda --min-validator-stake 10 --min-validators 2 --bottomup-check-period 30 --topdown-check-period 30
```

* Make a note of the address of the subnet you created (`/r314159/<SUBNET_ID>`)


## Step 6: Deploy the infrastructure

We can deploy the subnet nodes. Note that each node should be importing a different worker wallet key for their validator, and should be exposing different ports. If these ports are unavailable in your system, please pick different ones.

* Deploy and run a container for each validator, importing the corresponding wallet keys
```bash
./ipc-agent/bin/ipc-infra/run-subnet-docker.sh 1251 1351 /r314159/<SUBNET_ID> ~/.ipc-agent/worker-wallet1.key
./ipc-agent/bin/ipc-infra/run-subnet-docker.sh 1252 1352 /r314159/<SUBNET_ID> ~/.ipc-agent/worker-wallet2.key
./ipc-agent/bin/ipc-infra/run-subnet-docker.sh 1253 1353 /r314159/<SUBNET_ID> ~/.ipc-agent/worker-wallet3.key
```

* If the deployment is successful, each of these nodes should return the following output at the end of their logs. Save the information for the next step.
```
>>> Subnet /r314159/<SUBNET_ID> daemon running in container: <CONTAINER_ID_#> (friendly name: <CONTAINER_NAME_#>)
>>> Token to /r314159/<SUBNET_ID> daemon: <AUTH_TOKEN_#>
>>> Default wallet: <WORKER_#>
>>> Subnet validator info:
<VALIDATOR_ADDR_#>
>>> API listening in host port <PORT_#>
>>> Validator listening in host port <VALIDATOR_PORT_#>
```


## Step 7: Update the IPC Agent configuration

* Edit the IPC agent configuration `config.toml`
```bash
nano ~/.ipc-agent/config.toml
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

* Reload the config
```bash 
./ipc-agent/bin/ipc-agent config reload
```


## Step 8: Join the subnet 

All the infrastructure for the subnet is now deployed, and we can join our validators to the subnet. For this, we need to send a `join` command from each of our validators from their validator owner addresses providing the validators multiaddress. 

* Join the subnet with each validator
```bash
./ipc-agent/bin/ipc-agent subnet join --subnet /r314159/<SUBNET_ID> --collateral 10 --from <OWNER_1_F4> --validator-net-addr <VALIDATOR_ADDR_1> --worker-addr <WORKER_1> 
./ipc-agent/bin/ipc-agent subnet join --subnet /r314159/<SUBNET_ID> --collateral 10 --from <OWNER_2_F4> --validator-net-addr <VALIDATOR_ADDR_2> --worker-addr <WORKER_2>
./ipc-agent/bin/ipc-agent subnet join --subnet /r314159/<SUBNET_ID> --collateral 10 --from <OWNER_3_F4> --validator-net-addr <VALIDATOR_ADDR_3> --worker-addr <WORKER_3>
```

>💡 Make sure to use the f4 addresses for the owner wallets


## Step 9: Start validating! 

We have everything in place now to start validating. Run the following script for each of the validators [**each in a new session**], passing the container names:
```bash
./ipc-agent/bin/ipc-infra/mine-subnet.sh <CONTAINER_NAME_1> 
./ipc-agent/bin/ipc-infra/mine-subnet.sh <CONTAINER_NAME_2> 
./ipc-agent/bin/ipc-infra/mine-subnet.sh <CONTAINER_NAME_3> 
```

>💡 When starting mining and reloading the config to include the new subnet, you can sometimes get errors in the agent logs saying that the checkpoint manager couldn't be spawned successfully because the on-chain ID of the validator couldn't be change. This is because the subnet hasn't been fully initialized yet. You can `./ipc-agent/bin/ipc-agent config reload` to re-spawn the checkpoint manager and fix the error.


## Step 10: Deploy IPC Gateway [optional]

If you'd like to interact with your subnet using Metamask or other tooling, you should deploy a `lotus-gateway` instance for tokenless RPC access.

>💡 The instructions below assume you do not have a local `lotus` set-up. If you do, you may want to create a separate directory for `lotus-gw` and pass it as an argument to the application.

* Install Go [Linux] ([details](https://go.dev/doc/install))
```bash
curl -fsSL https://golang.org/dl/go1.19.7.linux-amd64.tar.gz | sudo tar -xz -C /usr/local
echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.bashrc && source ~/.bashrc
```

* Download and compile eudico (might take a while)
```bash
git clone --branch spacenet https://github.com/consensus-shipyard/lotus.git
(cd lotus && make spacenet && make lotus-gateway)
```

* Create the config directory and populate the configuration
```bash
mkdir -p ~/.lotus/datastore
echo '/ip4/127.0.0.1/tcp/1251/http' > ~/.lotus/api
echo '<AUTH_TOKEN_1>' > ~/.lotus/token
```

* Obtain your Chain ID
```bash
./ipc-agent/bin/ipc-agent subnet rpc --subnet <SUBNET_ID>
```

* [**In a new session**] Start your lotus-gateway
```bash
./lotus/lotus-gateway run --api-max-lookback=1600000h --api-wait-lookback-limit 2000
```

>💡 You may now use your chain ID and `http://<IP_ADDR>:2346/rpc/v1` as your RPC endpoint in EVM tooling.


## Step 11: What now?
* Proceed to the [usage](usage.md) guide to learn how you can test your new subnet.
* If something went wrong, please have a look at the [README](https://github.com/consensus-shipyard/ipc-agent). If it doesn't help, please join us in #ipc-help. In either case, let us know your experience!
* Please note that to repeat this guide or spawn a new subnet, you may need to change the parameters or reset your system.
