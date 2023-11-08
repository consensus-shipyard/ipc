---
description: IPC achieves scalability through subnets
---

# Subnets

## Definition

A subnet is a new subsystem that a user can spawn from a parent subnet in a permissionless and on-demand way, depending on scalability requirements. Subnets have separate consensus algorithms and cryptoeconomic rules from their parent subnet. Subnets are firewalled from the parent network&#x20;

## Hierarchy trees

Subnets begin with a chosen "rootnet". Rootnets refer to a layer 1 blockchain, such as Filecoin or Ethereum. Child subnets are spawned from the rootnet and the rootnet becomes the parent subnet.&#x20;

Each subnet can have any number of child subnets, while each child subnet only has one parent subnet. Subnets can scale infinitely, to layer 2 and beyond. A single hierarchy tree begins at the chosen rootnet.&#x20;

<figure><img src="../.gitbook/assets/hierarchy tree 2.png" alt=""><figcaption><p>Single hierarchy tree of subnets</p></figcaption></figure>

## Checkpointing state

In a hierarchy tree of subnets, child subnets periodically save snapshots of critical information (e.g. subnet state, membership information etc.), to their parent subnet.&#x20;

Checkpointing is done recursively from the leaf subnet (lowest possible) all the way to the parent rootnet, ensuring child subnets benefit from the security of their ancestor subnets.  A checkpointed subnet history cannot be reverted as long as a parent operates as expected.

Each subnet keeps its own state and validates transactions in parallel. Subnets periodically checkpoint by copying their state and storing it on their parent network, enabling a number of security features. &#x20;

Checkpointing allows for subnets to stop periodically, copy all data relevant state data to the parent, and then continue executing.&#x20;

If there is a catastrophic failure, the subnet can re-start from the checkpointed state found in the parent, rather than from genesis. Validator nodes of said subnet must agree on picking up from an older version of a subnet’s state. &#x20;

The function that enables checkpointing is `ISA.Checkpoint(snapshot, PoF)`, where snapshot is the state of the subnet and PoF is the proof of finality.

Checkpoints from subnets are periodically submitted to the parent subnet, carrying:

* bottom-up messages
* the next highest configuration number adopted form the validator changesets observed on the parent
* a multi-sig from the current validator set
* the identity of the checkpointed block height

The high level steps are implemented in the [checkpoint](https://github.com/consensus-shipyard/fendermint/blob/main/fendermint/vm/interpreter/src/fvm/checkpoint.rs) module, which calls various methods on the [Gateway actor](https://github.com/consensus-shipyard/ipc-solidity-actors/tree/dev/src/gateway), but the end-to-end flow also relies on a working [IPC Agent](https://github.com/consensus-shipyard/ipc/) and potentially the [IPLD Resolver](https://github.com/consensus-shipyard/ipc-ipld-resolver).

## Cross-subnet interaction

Subnets within a single hierarchy tree have native communication protocols and are able to transfer assets and state without a custom bridge.

## Operating a subnet

They can also be temporarily deployed and closed down, to increase network throughput as needed.

Validators are required to commit funds before operating a subnet

## Interacting with a subnet
