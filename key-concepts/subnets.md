---
description: IPC achieves scalability through spawning new subnets.
---

# Subnets

## Definition

A subnet is a new subsystem that a user can spawn from a parent subnet in a permissionless and on-demand way, depending on scalability requirements. Subnets have separate consensus algorithms and cryptoeconomic rules from their parent subnet. Subnets are firewalled from the parent network&#x20;

## Hierarchy trees

Subnets begin with a chosen "rootnet". Rootnets refer to a layer 1 blockchain, such as Filecoin or Ethereum. Child subnets are spawned from the rootnet and the rootnet becomes the parent subnet.&#x20;

Each subnet can have any number of child subnets, while each child subnet only has one parent subnet. Subnets can scale infinitely, to layer 2 and beyond. A single hierarchy tree begins at the chosen rootnet.&#x20;

<figure><img src="../.gitbook/assets/hierarchy tree 2.png" alt=""><figcaption><p>Single hierarchy tree of subnets</p></figcaption></figure>

Subnets within a single hierarchy tree have native communication protocols and are able to transfer assets and state without a custom bridge.

## Checkpointing state

To ensure security of a subnet, child subnets within a single hierarchy treee, periodically save snapshots of state to their parent subnet. This includes critical information (e.g. subnet state, membership information etc.). This is know as checkpointing and you can read more [here](checkpointing.md).&#x20;

## Subnet usage

* **Deploying and operating a subnet.** This is for developers who want to operate their own private or public subnet, as they can be temporarily deployed and closed down, to increase network throughput as needed. See the quickstart tutorial [here](../quickstarts/deploy-a-subnet.md).
* **Deploying smart contracts to an existing subnet.** This is for developers who want to build dApps on top of an existings subnets. See the quickstart tutorial [here](../quickstarts/deploy-smart-contracts-to-mycelium.md).
* For more on how fees work with IPC subnets, see [here](fees.md).
