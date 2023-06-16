# FileCoin IPC FEVM Actors

**‼️ The IPC Agent, the IPC actors, and eudico haven't been audited, tested in depth, or otherwise verified. Moreover, the system is missing critical recovery functionality in case of crashes. There are multiple ways in which you may lose funds moved into an IPC subnet, and we strongly advise against deploying IPC on mainnet and/or using it with tokens with real value.**

This repository includes the reference implementation of all the actors (i.e. smart contracts) responsible for the operation of the IPC (i.e. Inter-Planetary Consensus) protocol. These actors are written in Solidity and target Filecoin’s FEVM. 

The project accommodates the following main contracts

- `Gateway.sol`: Implementation of the IPC gateway.
- `SubnetActor.sol`: Reference implementation of an IPC subnet actor.
- `SubnetRegistry.sol`: Registry contract for the seamless deployment of subnet actors.

# Deploying IPC Solidity contracts
Before deploying the contract you'll need to configure the `RPC_URL` and `PRIVATE_KEY` environmental variables to point to your network provider and the private key of the address you want to use for the deployment, respectively.

Alternatively, you can rename the `env.template` file included in the repo to `.env`, set your variables there, and run `source .env` before running the deployment scripts.

To deploy the the IPC Solidity contracts in an FEVM network, you can directly run: 
```bash
make deploy-ipc
```
The scripts run by `make` make use of hardhat under the hood. The default network for the deployment is `localnet` as configured in `hardhat.config.ts`. To deploy the contracts in some other network configured in the Hardhat config you can run: 
```bash
make deploy-ipc NETWORK=<network-name>
```

# Actors overview

## Gateway

The purpose of the Gateway is to

1. serve as a register for the subnets of a given chain, dictating the rules for adding and removing new subnets
2. route cross chain messages
    1. store messages that are traveling from upper subnets in the hierarchy down to subnets that are on same branch of its own chain (top-down messages) 
    2. prepare epoch-defined checkpoints that collect messages traveling from lower levels of the hierarchy upwards (bottom-up messages)
3. distribute rewards to SubnetActors of child subnets 

## SubnetActor

The purpose of the SubnetActor is to

1. keep track of a subnet’s parameters (name, parent, consensus, staking parameters, status etc.)
2. provide validators with the ability to join and leave the subnet
3. manage validators’ stake
4. manage the subnet’s status
5. allows validators to submit checkpoints and to commit them to the Gateway once majority is reached
6. distribute rewards, received from the Gateway, to validators

## SubnetRegistry
The purpose of the SubnetRegistry is to

1. deploy instances of the SubnetActor. Its role is to be the subnet actor factory in a subnet.

# Building & Testing with Forge

To build all contracts run

```bash
forge build
```

The build artifacts (contracts’ ABI `.json` files), can be found in the `out` directory.

To run all repo tests run

```bash
forge test
```

And to generate coverage report run

```bash
forge coverage
```
