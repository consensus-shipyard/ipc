---
description: This page provides an in-depth explanation of IPC components
---

# IPC Architecture

<figure><img src="../../.gitbook/assets/image.png" alt=""><figcaption><p>IPC Architecture</p></figcaption></figure>

### Lotus as the Rootnet

Lotus, the reference implementation for Filecoin, serves as the rootnet for IPC subnets. The novelty of IPC is that these subnets run in parallel with unique consensus protocols. &#x20;

Operators and validators of a subnet run a full node for both the parent and the subnet. Requiring the nodes of children to run the nodes of parents is a security parameter to ensure [checkpointed](../../key-concepts/checkpointing.md) states of the subnet are appropriately stored at the right time in the parent.&#x20;

Subnet users running nodes of the child and the parent repeats recursively as the tree of subnets is built.

### Gateway smart contracts

A gateway is a smart contract in each subnet that implements the hierarchical consensus logic for that subnet and spawns new child subnets by implementing a new subnet [actor](https://docs.filecoin.io/basics/the-blockchain/actors) interface. A subnet is able to interact with its parent when it is registered in the subnet actor interface of the parent.

Gateway actors enforce some level of security for account balances as well.  As an example, gateway actors ensure it is not possible to withdraw more native tokens from subnet accounts to parent accounts than were used to fund the subnet to begin with.

#### IPC Actors

IPC relies on two actors, the _IPC Subnet Actor (ISA)_ and the _IPC Gateway Actor (IGA)_, which are instantiated in each subnet and provide convenience and governance functions.

The IGA is an actor that contains all IPC-related information and logic associated with a subnet that needs to be replicated within the subnet. The ISA is the IGA’s parent-side counterpart; that is, it is deployed to a subnet’s parent and contains all data and logic associated with the particular child subnet.

### Tendermint & ABCI++

[Tendermint Core](https://tendermint.com/) is a byzantine fault tolerant (BFT) consensus engine for blockchains. For IPC, Tendermint Core enables [state machine replication](https://en.wikipedia.org/wiki/State\_machine\_replication) in each subnet.&#x20;

An [ABCI++](https://members.delphidigital.io/learn/abci) interface is implemented in order to handle the IPC ledger logic, the transaction handling, using the [Filecoin Virtual Machine](https://docs.filecoin.io/smart-contracts/fundamentals/the-fvm) (or Ethereum-compatible FVM). The ABCI can pass [checkpointed](../../key-concepts/checkpointing.md) headers to the parent and use the ledger to gather relevant signatures.&#x20;

An ABCI++ application can contact the [IPLD](https://docs.filecoin.io/basics/project-and-community/related-projects#ipld) [resolver & store](ipld-resolver.md) to read and write data so that it is IPLD addressable.&#x20;

### Filecoin Virtual Machine (FVM)

The [FVM](https://docs.filecoin.io/smart-contracts/fundamentals/the-fvm) enables on-chain programmability and is built as a polyglot VM. It is currently compatible with Filecoin and Ethereum and has plans to support more chains in its [roadmap](https://fvm.filecoin.io/).&#x20;

FVM is included as a transaction execution later in the subnet, allowing use cases enabled by smart contracts to be built on top of subnets.

You may want to build an dApp on FVM that requires a transaction throughput higher than Filecoin provides. You can deploy a subnet that has a higher transaction settling frequency, and deploy the dApp on the subnet with FVM.

### Relayer

The role of relayers is to pass messages between parent and child subnets. They have to follow both the parent and the child consensus, subscribe to events, re-package the messages in the appropriate formats and resend them. Relayers can be incentivized. Both parent and child subnets can have an entirely different block structure and consensus, and only the relayers understand both, by being purposefully constructed to act between certain combinations.





