# Local Testnets

Prerequisites:
```bash
make build docker-build
```

## Single node deployment

To run IPC in the local rootnet just perform the following :
```bash
cargo make --makefile ./infra/Makefile.toml testnode

```

It will create three docker containers (cometbft, fendermint, and eth-api).

To stop run the following:
```bash
cargo make --makefile ./infra/Makefile.toml testnode-down
```

## Local 4-nodes deployment
To run IPC in the local rootnet with 4 nodes perform the following command :
```bash
cargo make --makefile ./infra/Makefile.toml testnet

```

To stop the network:
```bash
cargo make --makefile ./infra/Makefile.toml testnet-down
```

The testnet contains four logical nodes. Each node consists of cometbft, fendermint, and ethapi containers.
The Docker internal network is `192.167.10.0/24`.

ETH-API is accessible on the following interfaces on the Docker internal network:
- `192.167.10.10:8545` or `ethapi-node0:8545`
- `192.167.10.11:8545` or `ethapi-node1:8545`
- `192.167.10.12:8545` or `ethapi-node2:8545`
- `192.167.10.13:8545` or `ethapi-node3:8545`

and on the following interfaces from the host machine:
- `127.0.0.1:8545`
- `127.0.0.1:8546`
- `127.0.0.1:8547`
- `127.0.0.1:8548`

## Deployment process

The deployment process is as follows:
- Remove all docker containers, files, networks, etc. from the previous deployment
- Create all necessary directories
- Initialize CometBFT testnet by creating `config` and `data` directories using `cometbft` tools
- Read cometbft nodes private keys,derive node IDs and store in `config.toml` for each node
- Create the `genesis` file for Fendermint
- Share the genesis among all Fendermint nodes
- Run Fendermint application in 4 containers
- Run CometBFT in 4 containers
- Run Eth API in 4 containers