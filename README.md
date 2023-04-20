# FileCoin IPC FEVM Actors

This repository includes the reference implementation of all the actors (i.e. smart contracts) responsible for the operation of the IPC (i.e. Inter-Planetary Consensus) protocol. These actors are written in Solidity and target FileCoin’s FEVM. 

The project accommodates the following main contracts

- `Gateway.sol`: Implementation of the IPC gateway.
- `SubnetActor.sol`: Reference implementation of an IPC subnet actor.

# Building & Testing

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

# Loading the bytecode into Lotus

```bash
TODO
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