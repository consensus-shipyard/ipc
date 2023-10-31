---
description: >-
  Communication between subnets allows interoperability without the need to
  establish custom bridges.
---

# Interoperability

### Trees

Because IPC subnets are arranged in hierarchical trees, communication between subnets in the same tree can be done with actors and checkpointing. &#x20;

As an example, a parent chain and a subnet can run transactions in parallel, and then communicate updated states to each other, updating relevant account balances on each chain as needed.&#x20;

If a user deploys and arranges their subnets with the intention that they will communicate with each other, then the user would be able to achieve communication between the subnets without the need of a cross-chain bridge. Cross-chain bridges are high risk for loss of funds, such as the $650 Million exploit of the Axie Infinity Ronin bridge.  \[[LINK](https://www.protocol.com/bulletins/axie-infinity-ronin-hack)]

### IPC Actors

Communication between subnets on IPC is done by two [actors](https://docs.filecoin.io/basics/the-blockchain/actors) instantiated in each subnet--the _IPC Subnet Actor (ISA)_ and the _IPC Gateway Actor (IGA)_.

The IGA is an actor that contains all IPC-related information and logic associated with a subnet that needs to be replicated within the subnet.  The ISA is the IGA’s parent-side counterpart; that is, it is deployed to a subnet’s parent and contains all data and logic associated with the particular child subnet.

Primitives enabling cross-subnet communication include those for transfering funds between accounts in different subnets, saving checkpoints of a child's state in a parent chain, and submitting transactions on one subnet on inputs from smart contracts on another subnet.

