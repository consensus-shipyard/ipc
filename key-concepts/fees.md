---
description: Fees are paid for a range of different transactions with IPC.
---

# Fees

## Establishing a Subnet

There are a number of fees that are paid when a subnet is established:&#x20;

* At the time of subnet creation, a minimum collateral requirement is set by the subnet creator.&#x20;
* A standard fee for the transaction on the parent network will be paid for the transaction that establishes the subnet.&#x20;
* When a participant or validator (other than the subnet creator) joins the subnet, initial funds for their participation in the subnet should be moved from their respective account in the parent by using the `join` command.  This also enables the signing of checkpoint transactions.&#x20;

## Cross-net Transactions

There are a number of fees that are paid when transactions happen across subnets: &#x20;

* Although a transaction from a parent wallet to a subnet wallet to fund an subnet address is currently free, update M2.5 will enabling optional fees for users to prioritize their funding transactions. &#x20;
* There are no rewards for validators (fees paid by wallets) for the execution of transactions within or between subnets.
* Users in a child-subnet pay a minimum fee for their transaction to be included in the next checkpoint, as determined at the child subnet's construction. &#x20;

## Checkpointing

There are a number of fees that are paid during checkpointing:&#x20;

* When a subnet checkpoints its state to a parent, this is the equivalent of a transaction on the parent.  The usual transaction fees of the parent are paid to accomplish this.&#x20;
* In order for a subnet to be considered _anchored_ to the parent, relayers must have sufficient funds in their respective wallets in the parent to be able to pay for a checkpointed transaction.&#x20;
* When a cross-net transaction is included in a subnet's checkpoint to a parent, the fees for that transaction are distributed as a reward equally among all the relayers that have submitted an instance of that checkpoint.  &#x20;
* Relayers are allowed to submit a checkpoint and eligible for rewards from the commitment of the first checkpoint in, e.g. epoch \`h\`, to the first submission of a checkpoint of epoch \`h+1\`. From this point on, no new valid submissions for checkpoint \`h\` will be accepted.

## Closing a Subnet

The conditions for closing a subnet are as follows:&#x20;

* A child subnet cannot be killed untill its circulating supply is zero, which can be achieved when all users send their funds back to a parent.&#x20;
* If all validators leave a subnet even when their are still users of the subnet, the users will have to either run their own validator or wait for a validator to return to the subnet.&#x20;
* If a bug causes the subnet to fail, there is no way to recover funds in the subnet without a valid checkpoint signed by the latest validator committee.&#x20;
