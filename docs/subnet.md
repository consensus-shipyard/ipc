# Deploying IPC subnet infrastructure

>💡 For background and setup information, make sure to start with the [README](/README.md).

To spawn a new subnet, our IPC agent should be connected to the parent subnet (or rootnet) from which we plan to deploy a new subnet. Please refer to the [README](/README.md) for information on how to run or connect to a rootnet. This instructions will assume the deployment of a subnet from `/root`, but the steps are equivalent for any other parent subnet. 

We provide instructions for running both a [simple single-validator subnet](#running-a-simple-subnet-with-a-single-validator) and a more useful [multi-validator subnet](#running-a-subnet-with-several-validators). The two sets mostly overlap.

## Preliminaries

### Exporting wallet keys

In order to run a validator in a subnet, we'll need a set of keys to handle that validator. To export the validator key from a wallet that may live in another network into a file (like the wallet address we are using in the rootnet), we can use the following Lotus command:

*Example*:
```bash
$ ./eudico wallet export --lotus-json <address-to-export> > <output file>

# Example execution
$ ./eudico wallet export --lotus-json t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq > ~/.ipc-agent/wallet.key
```

If your daemon is running on a docker container, you can get the container id or name (provided also in the output of the infra scripts), and run the following command above inside a container outputting the exported private key into a file locally:
```bash
$ docker exec -it <container-id> eudico wallet export --lotus-json <adress-to-export> > ~/.ipc-agent/wallet.key

# Example execution
$ docker exec -it ipc_root_1234 eudico wallet export --lotus-json t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq > ~/.ipc-agent/wallet.key
```

### Importing wallet keys

Depending on whether the subnet is running inside a docker container or not, you may need to import keys into a node. You may use the following commands to import a wallet to a subnet node: 

```bash
# Bare: Import directly into eudico
$ ./eudico wallet import --lotus-json <wallet-key-file-path>

# Example execution
$ ./eudico wallet import --lotus-json ~/.ipc-agent/wallet.key
```

```bash
# Docker: Copy the wallet key into the container and import into eudico
$ docker cp <wallet-key-path> <container-id>:<target-file-in-container> && docker exec -it <container-id> eudico wallet import --format=json-lotus <target-file-in-container>

# Example execution
$ docker cp ~/.ipc-agent/wallet.key ipc_root_t01002_1250:/input.key && docker exec -it ipc_root_t01002_1250 eudico wallet import --format=json-lotus input.key
```

## Running a simple subnet with a single validator

This section provides instructions for spawning a simple subnet with a single validator. If you'd like to spawn a subnet with multiple validators in a Docker setup, read and understand this section first but then follow the steps under [the multi-validator section below](#running-a-subnet-with-several-validators).

### Spawning a subnet actor

To run a subnet the first thing is to configure and create the subnet actor that will govern the subnet's operation.

```bash
$ ./bin/ipc-agent subnet create --parent <parent> --name <name> --min-validator-stake <min_validator_stake> --min-validators <min-validators> --bottomup-check-period <bottomup-check-period> --topdown-check-period <topdown-check-period>

# Example execution
$ ./bin/ipc-agent subnet create --parent /root --name test --min-validator-stake 1 --min-validators 0 --bottomup-check-period 30 --topdown-check-period 30
[2023-03-21T09:32:58Z INFO  ipc_agent::cli::commands::manager::create] created subnet actor with id: /root/t01002
```
This command deploys a subnet actor for a new subnet from the `root`, with a human-readable name `test`, that requires at least `1` validator to join the subnet to be able to mine new blocks, and with a checkpointing period (both bottom-up and top-down) of `30` blocks. We can see that the output of this command is the ID of the new subnet.

### Exporting your wallet

We will need to export the wallet key from our root node so that we can import them to our validators. Depending on how you are running your rootnet node you'll have to make a call to the docker container, or your nodes API. More information about exporting keys from your node can be found under [this section](#Exporting-wallet-keys). Make sure that the wallet holds enough funds to meet the subnet collateral requirements.

### Deploying a subnet node

Before joining a new subnet, our node for that subnet must  be initialised. For the deployment of subnet daemons we also provide a convenient infra script:
```bash
$ ./bin/ipc-infra/run-subnet-docker.sh <lotus-api-port> <validator-libp2p-port> <subnet-id> <absolute-path-validator-key>

# Example execution
$ ./bin/ipc-infra/run-subnet-docker.sh 1250 1350 /root/t01002 ~/.ipc-agent/wallet.key
(...)
>>> Subnet /root/t01002 daemon running in container: 22312347b743f1e95e50a31c1f47736580c9a84819f41cb4ed3d80161a0d750f (friendly name: ipc_root_t01002_1239)
>>> Token to /root/t01002 daemon: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJyZWFkIiwid3JpdGUiLCJzaWduIiwiYWRtaW4iXX0.TnoDqZJ1fqdkr_oCHFEXvdwU6kYR7Va_ALyEuoPnksA
>>> Default wallet: t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq
>>> Subnet validator info:
/dns/host.docker.internal/tcp/1349/p2p/12D3KooWN5hbWkCxwvrX9xYxMwFbWm2Jpa1o4qhwifmSw3Fb
>>> API listening in host port 1250
>>> Validator listening in host port 1350
```
> 💡 Beware: This script doesn't support the use of relative paths for the wallet path.

The end of the log of the execution of this script provides a bit more of information than the previous one as it is implemented to be used for production deployments: API and auth tokens for the daemon, default validator wallet used, the multiaddress where the validator is listening, etc. To configure our IPC agent with this subnet daemon, we need to once again update our IPC agent with the relevant information. In this case, for the Example execution above we need to add the following section to the end of our config file.

*Example*:
```toml
[[subnets]]
id = "/root/t01002"
gateway_addr = "t064"
network_name = "test"
jsonrpc_api_http = "http://127.0.0.1:1250/rpc/v1"
auth_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJyZWFkIiwid3JpdGUiLCJzaWduIiwiYWRtaW4iXX0.TnoDqZJ1fqdkr_oCHFEXvdwU6kYR7Va_ALyEuoPnksA"
accounts = ["t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq"]
```
> 💡 Remember to run `./bin/ipc-agent config reload` for changes in the config of the agent to be picked up by the daemon.

### Joining a subnet

With the daemon for the subnet deployed, we can join the subnet:
```bash
$ ./bin/ipc-agent subnet join --subnet <subnet-id> --collateral <collateral_amount> --validator-net-addr <libp2p-add-validator>

# Example execution
$ ./bin/ipc-agent subnet join --subnet /root/t01002 --collateral 2 --validator-net-addr /dns/host.docker.internal/tcp/1349/p2p/12D3KooWN5hbWkCxwvrX9xYxMwFbWm2Jpa1o4qhwifmSw3Fb
```
This command specifies the subnet to join, the amount of collateral to provide and the validator net address used by other validators to dial them. We can pick up this information from the execution of the script above or running `eudico mir validator config validator-addr` from your deployment. Bear in mind that the multiaddress provided for the validator needs to be accessible publicly by other validators.

### Mining in a subnet

With our subnet daemon deployed, and having joined the network, as the minimum number of validators we set for our subnet is 0, we can start mining and creating new blocks in the subnet. Doing so is a simple as running the following script using as an argument the container of our subnet node: 
```bash
$  ./bin/ipc-infra/mine-subnet.sh <node-container-id>

# Example execution
$  ./bin/ipc-infra/mine-subnet.sh 84711d67cf162e30747c4525d69728c4dea8c6b4b35cd89f6d0947fee14bf908
```

The mining process is currently run in the foreground in interactive mode. Consider using `nohup ./bin/ipc-infra/mine-subnet.sh` or tmux to run the process in the background and redirect the logs to some file.

## Running a subnet with several validators

In this section, we will deploy a subnet where the IPC agent is responsible for handling more than one validator in the subnet. We are going to deploy a subnet with 3 validators. The first thing we'll need to do is create a new wallet for every validator we want to run. We can do this directly through the agent with the following command (3x):
```bash
$ ./bin/ipc-agent wallet new --key-type secp256k1 --subnet /root
```

We also need to provide with some funds our wallets so they can put collateral to join the subnet. According to the rootnet you are connected to, you may need to get some funds from the faucet, or send some from your main wallet. Funds can be sent from your main wallet also through the agent with (3x, adjusting `target-wallet` for each): 
```bash
$ ./bin/ipc-agent subnet send-value --subnet /root --to <target-wallet> <amount_FIL>
```

With this, we can already create the subnet with `/root` as its parent. We are going to set the `--min-validators 2` so no new blocks can be created without this number of validators in the subnet.
```bash
./bin/ipc-agent subnet create --parent /root --name test --min-validator-stake 1 --min-validators 2 --bottomup-check-period 30 --topdown-check-period 30
```
### Deploying the infrastructure

In order to deploy the 3 validators for the subnet, we will have to first export the keys from our root node so we can import them to our validators. Depending on how you are running your rootnet node you'll have to make a call to the docker container, or your nodes API. More information about exporting keys from your node can be found under [this section](#Exporting-wallet-keys).

With the keys conveniently exported, we can deploy the subnet nodes using the `infra-scripts`. The following code snippet showcases the deployment of five Example nodes. Note that each node should be importing a different wallet key for their validator, and should be exposing different ports for their API and validators.

*Example*:
```bash
$ ./bin/ipc-infra/run-subnet-docker.sh 1251 1351 /root/t01002 ~/.ipc-agent/wallet1.key
$ ./bin/ipc-infra/run-subnet-docker.sh 1252 1352 /root/t01002 ~/.ipc-agent/wallet2.key
$ ./bin/ipc-infra/run-subnet-docker.sh 1253 1353 /root/t01002 ~/.ipc-agent/wallet3.key
```
If the deployment is successful, each of these nodes should return the following output at the end of their logs. Note down this information somewhere as we will need it to conveniently join our validators to the subnet.

*Example*:
```
>>> Subnet /root/t01002 daemon running in container: 91d2af80534665a8d9a20127e480c16136d352a79563e74ee3c5497d50b9eda8 (friendly name: ipc_root_t01002_1240)
>>> Token to /root/t01002 daemon: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJyZWFkIiwid3JpdGUiLCJzaWduIiwiYWRtaW4iXX0.JTiumQwFIutkTb0gUC5JWTATs-lUvDaopEDE0ewgzLk
>>> Default wallet: t1ivy6mo2ofxw4fdmft22nel66w63fb7cuyslm4cy
>>> Subnet subnet validator info:
/dns/host.docker.internal/tcp/1359/p2p/12D3KooWEJXcSPw6Yv4jDk52xvp2rdeG3J6jCPX9AgBJE2mRCVoR
>>> API listening in host port 1251
>>> Validator listening in host port 1351
```

### Configuring the agent
To configure the agent for its use with all the validators, we need to connect to the RPC API of one of the validators, and import all of the wallets of the validators in that node, so the agent is able through the same API to act on behalf of any validator. More information about importing keys can be found in [this section](#Importing-wallet-keys).

Here's an example of the configuration connecting to the RPC of the first validator, and configuring all the wallets for the validators in the subnet.
```toml
[[subnets]]
id = "/root/t01002"
gateway_addr = "t064"
network_name = "test"
jsonrpc_api_http = "http://127.0.0.1:1240/rpc/v1"
auth_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJBbGxvdyI6WyJyZWFkIiwid3JpdGUiLCJzaWduIiwiYWRtaW4iXX0.JTiumQwFIutkTb0gUC5JWTATs-lUvDaopEDE0ewgzLk"
accounts = ["t1ivy6mo2ofxw4fdmft22nel66w63fb7cuyslm4cy", "t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq", "t1nv5jrdxk4ljzndaecfjgmu35k6iz54pkufktvua"]
```
Remember to run `./bin/ipc-agent config reload` for your agent to pick up the latest changes for the config.

### Joining the subnet
All the infrastructure for the subnet is now deployed, and we can join our validators to the subnet. For this, we need to send a `join` command from each of our validators from their validator wallet addresses providing the validators multiaddress. We can get the validator multiaddress from the output of the script we ran to deploy the infrastructure.

This is the command that needs to be executed for every validator to join the subnet:
```bash
$ ./bin/ipc-agent subnet join --from <validator-wallet> --subnet /root/t01002 --collateral <amount-collateral> --validator-net-addr <validator-addr>

# Example execution
$ ./bin/ipc-agent subnet join --from t1ivy6mo2ofxw4fdmft22nel66w63fb7cuyslm4cy --subnet /root/t01002 --collateral 2 --validator-net-addr /dns/host.docker.internal/tcp/1359/p2p/12D3KooWEJXcSPw6Yv4jDk52xvp2rdeG3J6jCPX9AgBJE2mRCVoR
```
Remember doing the above step for the 3 validators.

### Mining in subnet
We have everything in place now to start mining. Mining is as simple as running the following script for each of the validators, passing the container id/name:
```bash
$  ./bin/ipc-infra/mine-subnet.sh <node-container-id>
```

The mining process is currently run in the foreground in interactive mode. Consider using `nohup ./bin/ipc-infra/mine-subnet.sh` or screen to run the process in the background and redirect the logs to some file as handling the mining process of the three validators in the foreground may be quite cumbersome.

