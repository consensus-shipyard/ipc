# Deploy a subnet

Before delving into this tutorial, you should have a [basic understanding of IPC](../).&#x20;

Please note that you do not need to deploy your own subnet, in order to build on one. For deploying smart contracts to an existing subnet, check out [the next quickstart](deploy-smart-contract-to-mycelium-testnet.md).

In this tutorial, we will guide you through the process of spinning up your own IPC subnet validator node and anchoring it to the [Filecoin Calibration testnet](../reference/networks.md). We will use a repository with a pre-built docker image to simplify deployment of the IPC tooling. Steps include:

* Initialize and setup our config
* Fund our wallets needed to run our subnet
* Create a [child](../key-concepts/subnets.md#hierarchy-trees) subnet
* Join the subnet

### Prerequisites

This tutorial uses a repository containing a docker compose configuration and docker images pre-built of the main components needed. You will need to [have docker installed](https://docs.docker.com/engine/install/).

### Setting up our Environment

Clone the repository:

```
git clone https://github.com/consensus-shipyard/ipc-dx-docker.git
cd ipc-dx-docker
```

### Intialize your config

```
docker compose run ipc-cli config init
```

You can run `nano ~/.ipc/config.toml` to double-check that the config file has been populated with the following content:

```
keystore_path = "~/.ipc"

[[subnets]]
id = "/r314159"

[subnets.config]
network_type = "fevm"
provider_http = "https://api.calibration.node.glif.io/rpc/v1"
gateway_addr = "0x2e994B75095a39EB7C6dDA0516731B02AbcDbdb4"
registry_addr = "0x5FE1a06cA4534cB878260522a572e2254214261D"

# Subnet template - uncomment and adjust before using
# [[subnets]]
# id = "/r314159/<SUBNET_ID>"

# [subnets.config]
# network_type = "fevm"
# provider_http = "https://api.calibration.node.glif.io/rpc/v1"
# gateway_addr = "0x77aa40b105843728088c0132e43fc44348881da8"
# registry_addr = "0x74539671a1d2f1c8f200826baba665179f53a1b7"
```

### Set up your wallets

You'll need to create a set of wallets to spawn and interact with the subnet. Please make a note of the addresses as you go along, for easy reference.

* Create the three different wallets

```
docker compose run ipc-cli wallet new -w evm
docker compose run ipc-cli wallet new -w evm
docker compose run ipc-cli wallet new -w evm
```

* You can optionally set one of the wallets as your default so you don't have to use the `--from` flag explicitly in some of the commands:

```
docker compose run ipc-cli wallet set-default --address <DEFAULT_ETH_ADDR> -w evm
```

* Go to the [Calibration faucet](https://faucet.calibration.fildev.network/) and get some funds sent to each of your addresses

This tutorial uses a pre-built docker image to simplify the process of deploying your own subnet using IPC.&#x20;

### Create a child subnet <a href="#user-content-step-4-create-a-child-subnet" id="user-content-step-4-create-a-child-subnet"></a>

* The next step is to create a subnet under `/r314159` in calibration. Remember to set a default wallet or explicitly specifying the wallet from which you want to perform the action with the `--from` flag.

```
docker compose run ipc-cli subnet create --parent /r314159 --min-validators 3 --min-validator-stake 1 --bottomup-check-period 30
```

* Make a note of the address of the subnet you created.

### Join the subnet <a href="#user-content-step-5-join-the-subnet" id="user-content-step-5-join-the-subnet"></a>

Before we deploy the infrastructure for the subnet, we will have to bootstrap the subnet and join from our validators. We will put some initial collateral into the subnet and give our validator address some initial balance in the subnet. For this, we need to send a `join` command from each of our validators, from their validator owner addresses, providing their corresponding public key.

* Get the public key for all your wallets and note it down. This is the public key that each of your validators will use to sign blocks in the subnet.

```
docker compose run ipc-cli wallet pub-key -w evm --address <WALLET_ADDR1>
docker compose run ipc-cli wallet pub-key -w evm --address <WALLET_ADDR2>
docker compose run ipc-cli wallet pub-key -w evm --address <WALLET_ADDR3>
```

* Join the subnet with each validator

```
docker compose run ipc-cli subnet join --from=<WALLET_ADDR1> --subnet=/r314159/<SUBNET_ID> --collateral=10 --public-key=<PUBKEY_WALLET1> --initial-balance 1
docker compose run ipc-cli subnet join --from=<WALLET_ADDR2> --subnet=/r314159/<SUBNET_ID> --collateral=10 --public-key=<PUBKEY_WALLET2> --initial-balance 1
docker compose run ipc-cli subnet join --from=<WALLET_ADDR3> --subnet=/r314159/<SUBNET_ID> --collateral=10 --public-key=<PUBKEY_WALLET3> --initial-balance 1
```

### Deploying a bootstrap validator node

Before running our validators, at least one bootstrap needs to be deployed and advertised in the network. Bootstrap nodes allow validators discover other peers and validators in the network. In the current implementation of IPC, only validators are allowed to advertise bootstrap nodes.

* We can deploy a new bootstrap node in the subnet by running:

```
docker compose run ipc -e CMT_P2P_HOST_PORT=26650 bootstrap
```

At the end of the output, this command should return the ID of your new bootstrap node:

```
[cargo-make] INFO - Running Task: cometbft-wait
[cargo-make] INFO - Running Task: cometbft-node-id
2b23b8298dff7711819172252f9df3c84531b1d9@172.26.0.2:26650
[cargo-make] INFO - Build Done in 13.38 seconds.
```

Remember the address of your bootstrap for the next step. This address has the following format `id@ip:port`, and by default shows the public IP of your network interface. Feel free to adjust the `ip` to use a reachable IP for your deployment so other nodes can contact it (in our case our localhost IP, `127.0.0.1`).

* To advertise the endpoint to the rest of nodes in the network we need to run:

<pre><code><strong># Example of BOOTSTRAP_ENDPOINT = 2b23b8298dff7711819172252f9df3c84531b1d9@172.26.0.2:26650
</strong>docker compose run ipc-cli subnet add-bootstrap --subnet=&#x3C;SUBNET_ID> --endpoint="&#x3C;BOOTSRAP_ENDPOINT>"
</code></pre>

### Deploying the validator infrastructure <a href="#user-content-deploying-the-validator-infrastructure" id="user-content-deploying-the-validator-infrastructure"></a>

With the bootstrap node deployed and advertised to the network, we are now ready to deploy the validators that will run the subnet.

* First we need to export the private keys of our validators from the addresses that we created with our `ipc-cli wallet` to a known path so they can be picked by Fendermint to sign blocks. We can use the default repo of IPC for this, `~/.ipc`.

```
docker compose run ipc-cli wallet export -w evm -a <WALLET_ADDR1> --hex -o ~/.ipc/<PRIV_KEY_VALIDATOR_1>
docker compose run ipc-cli wallet export -w evm -a <WALLET_ADDR2> --hex -o ~/.ipc/<PRIV_KEY_VALIDATOR_2>
docker compose run ipc-cli wallet export -w evm -a <WALLET_ADDR3> --hex -o ~/.ipc/<PRIV_KEY_VALIDATOR_3>
```

* Now we have all that we need to deploy the three validators using the following command (configured for each of the validators, i.e. replace the arguments with `<..-n>` to fit that of the specific validator).

```
docker compose run ipc cargo make --makefile ./bin/ipc-infra/Makefile.toml \
    -e NODE_NAME=validator-<n> \
    -e PRIVATE_KEY_PATH=<PATH_PRIV_KEY_VALIDATOR_n> \
    -e SUBNET_ID=<SUBNET_ID> \
    -e CMT_P2P_HOST_PORT=<COMETBFT_P2P_PORT_n> -e CMT_RPC_HOST_PORT=<COMETBFT_RPC_PORT_n> \
    -e ETHAPI_HOST_PORT=<ETH_RPC_PORT_n> \
    -e BOOTSTRAPS=<BOOTSTRAP_ENDPOINT> \
    -e PARENT_REGISTRY=<PARENT_REGISTRY_CONTRACT_ADDR> \
    -e PARENT_GATEWAY=<GATEWAY_REGISTRY_CONTRACT_ADDR> \
    child-validator
```

`PARENT_REGISTRY` and `PARENT_GATEWAY` are the contract addresses of the IPC contracts in Calibration. This command also uses the calibration endpoint as default. Finally, you'll need to choose a different `NODE_NAME`, `CMT_HOST_PORT`, `ETHAPI_HOST_PORT` for each of the validators.

With this, we have everything in place, and our subnet should start automatically validating new blocks.
