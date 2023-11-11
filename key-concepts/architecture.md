---
description: Architectural components in the IPC framework.
---

# Architecture

<figure><img src="../.gitbook/assets/image.png" alt=""><figcaption><p>IPC Architecture</p></figcaption></figure>

## Validator nodes

Operators of a subnet run a full validator node for both the parent and the child subnet. Requiring the nodes of child subnets to run the nodes of parents is a security parameter to ensure [checkpointed](checkpointing.md) states of the subnet are appropriately stored, at the right time in the parent.&#x20;

The following components make up a node:

### Tendermint

[Tendermint Core](https://tendermint.com/) is a byzantine fault tolerant (BFT) consensus engine for blockchains. It acts as a generic [state machine replication](https://en.wikipedia.org/wiki/State\_machine\_replication) (SMR) in a subnet, talking to other Tendermint instances in the subnet and ensuring a consistent ledger is maintained across validator nodes. It talks to the Application in the node, via ABCI++.

### ABCI++

The [ABCI++](https://members.delphidigital.io/learn/abci) interface is implemented in order to handle the IPC ledger logic and transaction handling, using the [Filecoin Virtual Machine](https://docs.filecoin.io/smart-contracts/fundamentals/the-fvm) (or Ethereum-compatible FVM). The ABCI can pass [checkpointed](checkpointing.md) headers to the parent and use the ledger to gather relevant signatures.&#x20;

An ABCI++ application can contact the [IPLD](https://docs.filecoin.io/basics/project-and-community/related-projects#ipld) [resolver & store](../reference/ipld-resolver.md) to read and write data so that it is IPLD addressable.&#x20;

### Filecoin Virtual Machine (FVM)

The [FVM](https://docs.filecoin.io/smart-contracts/fundamentals/the-fvm) enables on-chain programmability and is built as a polyglot VM. It is currently compatible with Filecoin and Ethereum and has plans to support more chains in its [roadmap](https://fvm.filecoin.io/).&#x20;

FVM is included as a transaction execution later in the subnet, allowing use cases enabled by smart contracts to be built on top of subnets.

You may want to build an dApp on FVM that requires a transaction throughput higher than Filecoin provides. You can deploy a subnet that has a higher transaction settling frequency, and deploy the dApp on the subnet with FVM.

### IPC Actors

Communication between subnets on IPC is done by two [actors](https://docs.filecoin.io/basics/the-blockchain/actors) instantiated in each subnet--the _IPC Subnet Actor (ISA)_ and the _IPC Gateway Actor (IGA)_.

The IGA is an actor that contains all IPC-related information and logic associated with a subnet that needs to be replicated within the subnet. The ISA is the IGA’s parent-side counterpart; that is, it is deployed to a subnet’s parent and contains all data and logic associated with the particular child subnet.

Primitives enabling cross-subnet communication include those for transferring funds between accounts in different subnets, saving checkpoints of a child's state in a parent chain, and submitting transactions on one subnet on inputs from smart contracts on another subnet.

### Gateway actors

A gateway is an actor in each subnet that implements the hierarchical consensus logic for that subnet and spawns new child subnets by implementing a new subnet actor interface.&#x20;

A subnet is able to interact with its parent when it is registered in the subnet actor interface of the parent.

Gateway actors enforce some level of security for account balances as well. As an example, gateway actors ensure it is not possible to withdraw more native tokens from subnet accounts to parent accounts than were used to fund the subnet to begin with.

### Relayer

The role of relayers is to pass messages between parent and child subnets. They have to follow both the parent and the child consensus, subscribe to events, re-package the messages in the appropriate formats and resend them. Relayers can be incentivized. Both parent and child subnets can have an entirely different block structure and consensus, and only the relayers understand both, by being purposefully constructed to act between certain combinations.

### Lotus rootnet

[Lotus](https://lotus.filecoin.io/lotus/get-started/what-is-lotus/) currently serves as the rootnet for IPC subnets. It is the reference implementation for Filecoin nodes.&#x20;



