---
description: >-
  This page covers Proof of Finality, ordering protocols, nodes, and
  transactions.
---

# Approving Transactions on IPC Subnets

## Proof of Finality

Subnets rely on a cryptographic Proof of Finality, which proves that a subnet irreversibly reached a certain state.  The proof tells a node that a state will not be rolled back and establishes a partial ordering between the states of two subnets.  When a subnet checkpoints to a parent, it provides a Proof of Finality to the parent.

Ordering protocols will determine the chronological order of states between two independent subnets.  Ordering protocols are chosen at the genesis of a subnet, and may be unique to subnets.  To learn more about proof of finality and ordering protocols, see the design reference doc. \[LINK]&#x20;

## Transactions

Note that the name of a subnet is always prefixed by the name of the parent, similar to a file system.  A subnet is endowed with value when a user uses a smart contract to deposit coins from a wallet in a parent to a wallet in the subnet.  A user of a subnet will then have funds available for use in the subnet that was once in their corresponding parent wallet. &#x20;

Each of the users of subnet runes a full node for that subnet.  By default, subnets run total-order broadcasts across the nodes for the given subnet, meaning that all participants will agree on the order of the transactions in the subnet so that a block may be approved.  However, these consensus protocols can be customized for each subnet.&#x20;

The state of a subnet can only be updated through transactions by users, and a batch of transactions can be included in a block.  Each transaction on a subnet has some transaction fee, referred to as gas, which is deducted from the user’s account on the subnet.  Insufficient funds will result in a failed execution.&#x20;

As with any blockchain, spending coins is done when the user signs with their private key.
