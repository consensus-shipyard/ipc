# Deploying an IPC Solidity instance on Calibration

This guide will guide you through deploying IPC over an FEVM-compatible rootnet. Specifically, after this tutorial, you'll have IPC deployed over Filecoin's [Calibration testnet](https://docs.filecoin.io/networks/calibration/details/), and you'll be able to deploy new subnets with Calibration as the rootnet.

> TODO: A video walkthrough of this guide is also [available <TODO: We need to record this!](). We still encourage you to try it for yourself!

## Step 0: Prepare your system

Before jumping right into it, we need to get our system ready for IPC. If you haven't yet tried IPC before, use [Step 0](./quickstart.md#step-0-prepare-your-system) of the [quickstart tutorial](./quickstart.md) to install all the pre-required dependencies in your system.

## Step 1: Get Calibration Funds

In order to deploy over Calibration, and be able to deploy the IPC Solidity contracts, you'll need to get some testnet funds. For that, you can follow [the following detailed guide](https://docs.filecoin.io/smart-contracts/developing-contracts/get-test-tokens/).

At a high-level, the steps that you need to follow are the following (for those already comfortable with Metamask and the Filecoin ecosystem):
- Create a new Ethereum address with Metamask (or use one of you existing addresses).
- Go to the [Calibration faucet](https://faucet.calibration.fildev.network/) and send some funds to your address.
- If you have connected your Metamask to Calibration, you should see in a few seconds the new funds in your wallet (you can follow [this guide](https://docs.filecoin.io/basics/assets/metamask-setup/) to connect your Metamask to Calibration).

>💡 We strongly advise against using an Ethereum address that you are using for mainnet for this. This could lead to unintentional mistakes that you may regret in the future.

With this, we have the funds we need to start deploying IPC.

>💡 Instead of creating you new EVM wallet using Metamask, you can also use the IPC-agent for this. You'll need to first compile your IPC agent (see step 3), run the daemon with `./ipc-agent/bin/ipc-agent daemon` and create the new wallet with `./bin/ipc-agent wallet new -w evm`. You can get the private key for your wallet by exporting the key through `./bin/ipc-agent wallet export -w evm -a <EVM_ADDR> -o <OUTPUT_FILE>`. More information available in the [EVM IPC agent support docs](./evm-usage.md#key-management).

## Step 2: Deploy IPC Solidity contracts to Calibration

In order to deploy the latest version of the IPC Solidity contracts, you'll need to pull the following repo: 
```bash
https://github.com/consensus-shipyard/ipc-solidity-actors
cd ipc-soldity-actors
``` 
Once inside the repo, you'll need to populate the `.env.template` file with the private key of the address you provided with funds in the previous step, and the endpoint of the target network you want to deploy IPC to (in our case Calibration): 
```bash
export PRIVATE_KEY=<your_private_key>
export RPC_URL=https://api.calibration.node.glif.io/rpc/v1
```
> To export your private key from Metamask you can follow [these steps](https://support.metamask.io/hc/en-us/articles/360015289632-How-to-export-an-account-s-private-key)

In your currently open terminal, you'll need to load these variables into your environment so you can deploy the contracts.
```bash
source .env.template
make deploy-ipc NETWORK=calibrationnet
```
If the deployment is successful, you should receive an output similar to this one: 
```
$ ./ops/deploy.sh localnet
[*] Deploying libraries
[*] Output libraries available in /home/workspace/pl/ipc-solidity-actors/scripts/libraries.out
[*] Populating deploy-gateway script
[*] Gateway script in /home/workspace/pl/ipc-solidity-actors/scripts/deploy-gateway.ts
[*] Gateway deployed: 
{ Gateway: '0x2bA26fe8EB6b5C8488132900F00d853ca840F2FB' }
[*] Output gateway address in /home/workspace/pl/ipc-solidity-actors/scripts/gateway.out
[*] Populating deploy-registry script
[*] Registry script in /home/workspace/pl/ipc-solidity-actors/scripts/deploy-registry.ts
No need to generate any newer typings.
Nothing to compile
No need to generate any newer typings.
Deploying contracts with account: 0x6BE1Ccf648c74800380d0520D797a170c808b624 and balance: 2999143347078092235924
registry contract deployed to: 0x221485C0948dAa7C8dC1bCbcf813684C5EC1ecED
[*] IPC actors successfully deployed
```
Keep the addresses of the gateway and the registry contracts deployed, as you will need them to configure the IPC agent in the next step.

>💡If instead of deploying IPC Solidity in Calibration, you want to test them in Spacenet or a local network, the only thing that you need to do is to configure the `RPC_URL` of your `.env` to point to the corresponding network's RPC endpoint, and `make deploy-ipc NETWORK=localnet`.

## Step 3: Configure IPC Agent.

After deploying the IPC contracts, we come out of the contracts repo (i.e. where we cloned the `ipc-agent` repo). Let's now deploy and configure our IPC agent. 
- Compile the IPC agent
```bash
cd ipc-agent
make build
cd ..
```
- We then to initialize a config template and configure it to connect to our IPC instance in Calibration.
```toml
[server]
json_rpc_address = "0.0.0.0:3030"

[[subnets]]
id = "/r314159"
network_name = "calibration"

[subnets.config]
accounts = []
gateway_addr = "0x2bA26fe8EB6b5C8488132900F00d853ca840F2FB"
network_type = "fevm"
provider_http = "https://api.calibration.node.glif.io/rpc/v1"
registry_addr = "0x221485C0948dAa7C8dC1bCbcf813684C5EC1ecED"

```

The gateway and registry addresses should be the ones we received after deploying our IPC instance (in the previous step). Finally, the `provider_http` should be our endpoint to calibration (or our rootnet).

>💡 Remember that the subnetID depends on the chainID of that network. Thus, when you are configuring your IPC agent to connect to an instance deployed over localnet or spacenet, their rootnet ID will be `/r31415926`, while the one for Calibration is `/r314159`.

- Finally, we start the IPC agent:
```bash
./ipc-agent/bin/ipc-agent daemon 
```

## Step 4: Creating or importing an EVM account
In order to interact with Calibration through the IPC agent you'll need to either create a new wallet through the agent and provide it with funds, or import the private key that you created to deploy the contracts.
* [**In a new session**] You can create a new wallet through the following command:
```bash
./ipc-agent/bin/ipc-agent wallet new -w evm
```
* Or you can import it by running the following command and passing the private key of your wallet: 
```bash
./ipc-agent/bin/ipc-agent import -w evm --private_key=<PRIVATE>
```
* Once the wallet of the IPC agent has the management key you want to use to interact with IPC, you'll have to add it to the `accounts` list of your `~/.ipc-agent/config.toml`.
```bash
accounts = [<EVM-ADDRESS>]
```
* Finally, you'll need to reload the config to apply the changes.
```bash
./ipc-agent/bin/ipc-agent config reload
```

## Step 5: Create a child subnet

* The next step is to create a subnet under `/r314159` in calibration
```bash
./ipc-agent/bin/ipc-agent subnet create --parent /r314159 --name andromeda --min-validator-stake 1 --min-validators 2 --bottomup-check-period 30 --topdown-check-period 30
```
* Make a note of the address of the subnet you created (`/r314159/<SUBNET_ID>`)


## Step 6: Create and export validator wallets

Although we set a minimum of 2 active validators in the previous, we'll deploy 3 validators to add some redundancy. These will become the worker addresses of the validators in the child subnet.

* First, we'll need to create a wallet for each validator
```bash
./ipc-agent/bin/ipc-agent wallet new -w fvm --key-type secp256k1
./ipc-agent/bin/ipc-agent wallet new -w fvm --key-type secp256k1
./ipc-agent/bin/ipc-agent wallet new -w fvm --key-type secp256k1
```
* Export each wallet (WALLET_1, WALLET_2, and WALLET_3) by substituting their addresses below
```bash
./ipc-agent/bin/ipc-agent wallet export -w fvm --address <WALLET_1> --output ~/.ipc-agent/worker-wallet1.key
./ipc-agent/bin/ipc-agent wallet export -w fvm --address <WALLET_2> --output ~/.ipc-agent/worker-wallet2.key
./ipc-agent/bin/ipc-agent wallet export -w fvm --address <WALLET_3> --output ~/.ipc-agent/worker-wallet3.key
```

>💡 Mir validators do not support the use of Ethereum addresses to create new blocks. This is why since the deployment of IPC Solidity instances, we introduced the concept of `worker_addresses`, which is the address use by validators to create new blocks. Validators have an `owner_address`, used to join a network, and they can optionally include a `worker_address`. Setting a `worker_address` is mandatory when interacting with an FEVM-based parent and not required for FVM-based.

We will also have to create two new Ethereum addresses to be used as the owner addresses to join the subnet in the FEVM-based IPC instance from calibration. You can do this directly using Metamask (or following [this guide](https://docs.filecoin.io/basics/assets/metamask-setup/)).

## Step 7: Deploy the infrastructure

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
>>> Default wallet: <WALLET_#>
>>> Subnet validator info:
<VALIDATOR_ADDR_#>
>>> API listening in host port <PORT_#>
>>> Validator listening in host port <VALIDATOR_PORT_#>
```

## Step 8: Configure the IPC agent

For ease of use in the management of the validators the IPC Agent will act on behalf of all validators.

* Edit the IPC agent configuration `config.toml`
```bash
nano ~/.ipc-agent/config.toml
```
* Append the new subnet to the configuration
```toml
[[subnets]]
id = "/r314159/t410fns7xiomya2zq5nsejjphoywktlwvkkdtgng7ceq"
network_name = "andromeda"

[subnets.config]
gateway_addr = "t064"
accounts = ["<WALLET_1>", "<WALLET_2>", "<WALLET_3>"]
jsonrpc_api_http = "http://127.0.0.1:1251/rpc/v1"
auth_token = "<AUTH_TOKEN_1>"
network_type = "fvm"
```
* Reload the config
```bash 
./ipc-agent/bin/ipc-agent config reload
```

## Step 9: Join the subnet 

All the infrastructure for the subnet is now deployed, and we can join our validators to the subnet. For this, we need to send a `join` command from each of our validators from their validator owner addresses providing the validators multiaddress. 

* Join the subnet with each validator
```bash
./ipc-agent/bin/ipc-agent subnet join --subnet /r314159/<SUBNET_ID> --collateral 1 --validator-net-addr <VALIDATOR_ADDR_1> --worker-addr <WALLET_1> 
./ipc-agent/bin/ipc-agent subnet join --subnet /r314159/<SUBNET_ID> --collateral 1 --validator-net-addr <VALIDATOR_ADDR_2> --worker-addr <WALLET_2>
./ipc-agent/bin/ipc-agent subnet join --subnet /r314159/<SUBNET_ID> --collateral 1 --validator-net-addr <VALIDATOR_ADDR_3> --worker-addr <WALLET_3>
```
>💡 We currently do not support the use of `from` for Ethereum addresses ([see issue](https://github.com/consensus-shipyard/ipc-agent/issues/244)), in order to send a `join` from different owner Ethreum accounts, you'll need to set the `private_key` from each account of a validator address in `~/.ipc-agent/config.toml` and `./ipc-agent/bin/ipc-agent config reload` before every join to use a different owner key to join the subnet for each worker key.

## Step 10: Start validating! 

We have everything in place now to start validating. Run the following script for each of the validators [**each in a new session**], passing the container names:
```bash
./ipc-agent/bin/ipc-infra/mine-subnet.sh <CONTAINER_NAME_1> 
./ipc-agent/bin/ipc-infra/mine-subnet.sh <CONTAINER_NAME_2> 
./ipc-agent/bin/ipc-infra/mine-subnet.sh <CONTAINER_NAME_3> 
```

>💡 When starting mining and reloading the config to include the new subnet, you can sometimes get errors in the agent logs saying that the checkpoint manager couldn't be spawned successfully because the on-chain ID of the validator couldn't be change. This is because the subnet hasn't been fully initialized yet. You can `./ipc-agent/bin/ipc-agent config reload` to re-spawn the checkpoint manager and fix the error.

## Step 11: Cross-net messages

We are now going to test the use of cross-net messages between the parent and the child subnet.
- We will first create a new wallet and send a some funds to it in the child subnet.
```bash
./ipc-agent/bin/ipc-agent wallet new -w fvm --key-type=secp256k1

./ipc-agent/bin/ipc-agent cross-msg fund --subnet=<SUBNET_ID> --to=<NEW_WALLET> <AMOUNT>
```
The funds will be propagated in the next top-down checkpoint, you can see the last top-down checkpoint that was committed to see if your funds have already arrived using:
```bash
./bin/ipc-agent checkpoint last-topdown --subnet=<SUBNET_ID>
```
The epoch of this command should be higher or equal to the epoch in the output of `fund`.
- To release funds from a subnet, you can run a `release` command from any balance with funds from the subnet (bear in mind that by default, the address used to perform the operation is the one set in your agen't config):
```bash
./ipc-agent/bin/ipc-agent cross-msg release --subnet=<SUBNET_ID> --to=<NEW_WALLET> <AMOUNT>
```
- To see if the funds where successfully sent to any of your subnet run after any of the above cross-net messages you can run: 
```bash
./bin/ipc-agent wallet balances --subnet=<SUBNET_ID>
```

## Step 12: What now?
* If something went wrong, please have a look at the [README](https://github.com/consensus-shipyard/ipc-agent). If it doesn't help, please join us in #ipc-help. In either case, let us know your experience!
* Please note that to repeat this guide or spawn a new subnet, you may need to change the parameters or reset your system.
