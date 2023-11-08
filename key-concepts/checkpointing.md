---
description: >-
  Subnets exploit parents for security against smart contract catastrophic
  failures and rollbacks.
---

# Checkpointing

## Introduction

Interplanetary Consensus subnets are organized hierarchically in a tree. Parent networks can have any number of subnets, but each subnet can only have one parent.  Storing state on a parent is exploited for security by a subnet through periodic checkpointing of a subnet’s state onto the parent. &#x20;

Checkpointing allows for subnets to stop periodically, copy all data relevant state data to the parent, and then continue executing.  If there is a catastrophic failure, the subnet can re-start from the checkpointed state found in the parent, rather than from genesis.  \[[LINK](https://en.wikipedia.org/wiki/Application\_checkpointing)]  Members must agree on picking up from an older version of a subnet’s state. &#x20;

Checkpointing is done recursively from the leaf (lowest possible) subnet all the way to the parent rootnet, ensuring child subnets benefit from the security of their ancestor subnets.  A checkpointed subnet history cannot be reverted as long as a parent operates as expected.

The function that enables checkpointing is `ISA.Checkpoint(snapshot, PoF)`, where snapshot is the state of the subnet and PoF is the proof of finality.

Checkpoints from subnets are periodically submitted to the parent subnet, carrying:

* bottom-up messages
* the next highest configuration number adopted form the validator changesets observed on the parent
* a multi-sig from the current validator set
* the identity of the checkpointed block height

The high level steps are implemented in the [checkpoint](https://github.com/consensus-shipyard/fendermint/blob/main/fendermint/vm/interpreter/src/fvm/checkpoint.rs) module, which calls various methods on the [Gateway actor](https://github.com/consensus-shipyard/ipc-solidity-actors/tree/dev/src/gateway), but the end-to-end flow also relies on a working [IPC Agent](https://github.com/consensus-shipyard/ipc/) and potentially the [IPLD Resolver](https://github.com/consensus-shipyard/ipc-ipld-resolver).

The following diagram illustrates the sequence of events in detail:

[![Checkpointing](https://github.com/consensus-shipyard/fendermint/raw/main/docs/diagrams/checkpointing.png)](https://github.com/consensus-shipyard/fendermint/blob/main/docs/diagrams/checkpointing.png)

The above scenario assumes that the parent subnet is running Lotus, where we are restricted to using Solidity actors, and therefore the relayers include all bottom-up messages in their transaction, which creates redundancy but makes the messages trivially available for execution.

If both the parent and the child were Fendermint nodes, we'd have the option to use the IPLD Resolver to only include the CID of the messages in the relayed checkpoint messages, and let Fendermint make sure the data is available before proposing it for execution.

The most typical use case would be the propagation of checkpoints from child subnets to the parent subnet.

## Checkpointing in Practice

One possible conceptual model of checkpointing is depicted by the following Entity Relationship diagram:

[![Checkpoint Schema](https://github.com/consensus-shipyard/ipc-ipld-resolver/raw/main/docs/diagrams/checkpoint\_schema.png)](https://github.com/consensus-shipyard/ipc-ipld-resolver/blob/main/docs/diagrams/checkpoint\_schema.png)

It shows that the Subnet Actor in the parent subnet governs the power of validators in the child subnet by proposing _Configurations_, which the child subnet is free to adopt in its _Epochs_ when the time is right, communicating back the next adopted config via _Checkpoints_.

At the end of an epoch, the validators in the child subnet produce a checkpoint over some contents, notably the cross-messages they want to propagate towards the parent subnet. Through the cross-messages, the checkpoint indirectly points to individual messages that users or actors wanted to send.

Once enough signatures are collected to form a Quorum Certificate over the checkpoint (the specific rules are in the jurisdiction of the Subnet Actor), the checkpoint is submitted to the parent ledger.

However, the submitted checkpoint does not contain the raw messages, only the meta-data. The content needs to be resolved using the IPC Resolver, as indicated by the dotted line.

## Checkpoint Submission and Resolution

The following sequence diagram shows one possible way how checkpoints can be submitted from the child to the parent subnet.

It depicts two validators: one only participating on the parent subnet, and the other on the child subnet; the latter has to also run at least a full node on the parent subnet.&#x20;

The diagram shows that at the end of the epoch the child subnet validators produce a Quorum Certificate over the checkpoint, which are then submitted to the parent subnet.

After that, the parent subnet nodes resolve the messages referenced by the checkpoint, which is then communicated to some of its child-subnet peers.

[![Checkpoint Submission](https://github.com/consensus-shipyard/ipc-ipld-resolver/raw/main/docs/diagrams/checkpoint\_submission.png)](https://github.com/consensus-shipyard/ipc-ipld-resolver/blob/main/docs/diagrams/checkpoint\_submission.png)

This is just a high level view of what happens during message resolution.  In the next article, we will delve deeper into the internals of the IPLD Resolver.

### How fees work?
