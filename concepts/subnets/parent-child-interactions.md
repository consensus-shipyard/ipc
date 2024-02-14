# 🟡 Parent-child interactions

## Checkpointing

### Fees

There are a number of fees that are paid during checkpointing:&#x20;

* When a subnet checkpoints its state to a parent, this is the equivalent of a transaction on the parent.  The usual transaction fees of the parent are paid to accomplish this.&#x20;
* In order for a subnet to be considered _anchored_ to the parent, relayers must have sufficient funds in their respective wallets in the parent to be able to pay for a checkpointed transaction.&#x20;
* When a cross-net transaction is included in a subnet's checkpoint to a parent, the fees for that transaction are distributed as a reward equally among all the relayers that have submitted an instance of that checkpoint.  &#x20;
* Relayers are allowed to submit a checkpoint and eligible for rewards from the commitment of the first checkpoint in, e.g. epoch \`h\`, to the first submission of a checkpoint of epoch \`h+1\`. From this point on, no new valid submissions for checkpoint \`h\` will be accepted.

## Parent finality
