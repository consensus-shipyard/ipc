---
description: >-
  Communication between subnets allows interoperability without the need to
  establish custom bridges.
---

# Interoperability

### IPC Actors

Communication between subnets on IPC is done by two [actors](https://docs.filecoin.io/basics/the-blockchain/actors) instantiated in each subnet--the _IPC Subnet Actor (ISA)_ and the _IPC Gateway Actor (IGA)_.

The IGA is an actor that contains all IPC-related information and logic associated with a subnet that needs to be replicated within the subnet.  The ISA is the IGA’s parent-side counterpart; that is, it is deployed to a subnet’s parent and contains all data and logic associated with the particular child subnet.

Primitives enabling cross-subnet communication include those for transfering funds between accounts in different subnets, saving checkpoints of a child's state in a parent chain, and submitting transactions on one subnet on inputs from smart contracts on another subnet.

### API Gateways
