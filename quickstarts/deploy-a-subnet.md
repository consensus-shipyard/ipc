# 🟡 Deploy a subnet

Before delving into this tutorial, you should have a [basic understanding of IPC](broken-reference).

Please note that you do not need to deploy your own subnet, in order to build on one. For deploying smart contracts to an existing subnet, check out [the next quickstart](broken-reference).

In this tutorial, we will guide you through the process of spinning up your own IPC subnet validator node locally. We will use a repository with a pre-built docker image to simplify deployment of the IPC tooling.&#x20;

### Prerequisites

* [Docker](https://docs.docker.com/engine/install/)

### Deploying the subnet

**Please note**, that running the commands below will result in docker downloading a 3GB image on first run. So if you are going to be running this at somewhere with poor or metered internet connectivity, please be aware.

1.  Clone this repository:

    ```
    git clone https://github.com/consensus-shipyard/ipc-dx-docker.git
    ```
2.  Navigate to the repository:

    ```
    cd ipc-dx-docker
    ```
3.  OPTIONAL: By default the subnet started will be `r0`. If you want a different subnet ID (also affects the chain ID), then you can either set the env variable `SUBNET_ID`, or edit it in the `.env` file.

    ```
    export SUBNET_ID=r42
    ```
4.  To run a single standalone IPC node testnode::

    ```
    docker compose run fendermint testnode
    ```
5.  To stop the network run::

    ```
    docker compose run fendermint testnode-down
    ```

### Metamask and Funding a Wallet

#### Setting up Metamask

A default metamask wallet will be funded and the details show to you on startup:

```
############################
#                          #
# Testnode ready! 🚀       #
#                          #
############################

Eth API:
	http://0.0.0.0:8545

Accounts:
	t1vwjol3lvimayhxxvcr2fmbtr4dm2krsta4vxmvq: 1000000000000000000000 coin units
	t410f5joupqsfnfz2g2b5cakucfkigur2synrvem5d5q: 1000000000000000000000 coin units

Private key (hex ready to import in MetaMask):
	d870269696821eca9c628fe3780e8b54a5f471d29cc3cd444c9261d4d16e7730

Note: both accounts use the same private key @ /tmp/data/.ipc/r0/keys/validator_key.sk

Chain ID:
	3522868364964899

Fendermint API:
	http://localhost:26658

CometBFT API:
	http://0.0.0.0:26657
```

You can configure metamask to connect to this local network by adding a new custom network, with the following steps:

1. Click the network at the top of Metamask
2. Click `Add a network manually` at the bottom
3.  Enter the network information below

    ```
    Network name: IPC localnode
    New RPC URL: http://127.0.0.1:8545
    Chain ID: 3522868364964899
    Currency symbol: tFIL
    ```

#### Funding a wallet

Your wallet will already be funded with 1000 tFIL, ready to use and deploy your first contract. To import the key into your Metamask wallet you will need to:

1. Click on the Metamask icon at the top of your browser
2. Click the accounts drop down at the top
3. Click `Add account or new hardware wallet`
4. Click `Import account`
5. Paste in the private key as shown in the output from starting the network

With this, we have everything in place, and our subnet should start automatically validating new blocks. While this is a locally deployed subnet, there will soon be resources to deploy a subnet, anchored to a live testnet. Stay tuned!
