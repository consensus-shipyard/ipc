---
description: >-
  Subnets exploit parents for security against smart contract catastrophic
  failures and rollbacks.
---

# Checkpointing

## Checkpointing

Interplanetary Consensus subnets are organized hierarchically in a tree.  Parent networks can have any number of subnets, but each subnet can only have one parent.  Storing state on a parent is exploited for security by a subnet through periodic checkpointing of a subnet’s state onto the parent. &#x20;

Checkpointing allows for subnets to stop periodically, copy all data relevant state data to the parent, and then continue executing.  If there is a catastrophic failure, the subnet can re-start from the checkpointed state found in the parent, rather than from genesis.  \[[LINK](https://en.wikipedia.org/wiki/Application\_checkpointing)]  Members must agree on picking up from an older version of a subnet’s state. &#x20;

Checkpointing is done recursively from the leaf (lowest possible) subnet all the way to the parent rootnet, ensuring child subnets benefit from the security of their ancestor subnets.  A checkpointed subnet history cannot be reverted as long as a parent operates as expected.

The function that enables checkpointing is `ISA.Checkpoint(snapshot, PoF)`, where snapshot is the state of the subnet and PoF is the proof of finality.

In the next article, we will discuss how Proof of Finality ensures a checkpointed state will never be rolled back and how to update the state of a subnet.&#x20;
