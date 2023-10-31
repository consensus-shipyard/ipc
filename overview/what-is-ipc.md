---
description: An introduction
---

# What is IPC

### Introduction

Interplanetary Consensus (IPC) is a framework that enables on-demand horizontal scalability of networks, by deploying "subnets" running different consensus algorithms depending on the application's requirements.

### Subnets

#### Root, parent and child networks

Since a given subnet can have any number of subnets, but each subnet only has one parent network, a graphical representation of IPC subnets form a tree. To learn more about trees in graph theory, see [this article](https://discrete.openmathbooks.org/dmoi3/sec\_trees.html).&#x20;

In our graphical representation of IPC, we first choose blockchain, such as Filecoin or Ethereum, that will serve as the [root](https://mathworld.wolfram.com/RootedTree.html) (we call the chosen blockchain the rootnet). [Child](https://www.gatevidyalay.com/tag/child-node/) subnets are then spawned from the rootnet, making the rootnet the parent to each of those child subnets.&#x20;

#### Features&#x20;

Subnets have the following features:&#x20;

* Users can spawn new subnets to scale from a root network. They can also be temporarily deployed and closed down, to increase network throughput as needed.
* Subnets run on their own consensus, cryptoeconomic rules and agreement algorithm
* Each subnet keeps its own state and validates transactions in parallel. Subnets periodically [checkpoint](../key-concepts/ipc-architecture/checkpointing.md#checkpointing) by copying their state and storing it on their parent network, enabling a number of security features. &#x20;
* Subnets are firewalled from the parent network&#x20;
* Subnets can interact with each other through cross-net messages.
* Validators are required to commit funds before operating a subnet

### Chain compatibility

While IPC subnets can resemble other L2 platforms, such as [Optimistic rollups](https://ethereum.org/en/developers/docs/scaling/optimistic-rollups/), [Zero-knowledge rollups](https://ethereum.org/en/developers/docs/scaling/zk-rollups/) or a [sidechain](https://ethereum.org/en/developers/docs/scaling/sidechains/) with a native communication bridge, IPC subnets are specially designed to be compatible with multiple networks.

IPC is currently fully compatible with the [Filecoin network](https://docs.filecoin.io/basics/what-is-filecoin) and [EVM-compatible networks](https://chainlist.org/). Compatibility with more chains is in the [roadmap](https://ipc.space/#roadmap).&#x20;

