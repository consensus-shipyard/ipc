# Recall Storage: Vote Tally Mechanism

## Overview

The Recall storage layer (Basin network) uses a **weighted vote tally system** to achieve Byzantine Fault Tolerant (BFT) consensus on blob storage across the validator network. This document explains how validators vote on blob resolution and how the system determines when a blob has been successfully stored.

## Table of Contents

- [Core Concepts](#core-concepts)
- [Vote Tally Architecture](#vote-tally-architecture)
- [Voting Process](#voting-process)
- [Quorum Calculation](#quorum-calculation)
- [Vote Tallying Algorithm](#vote-tallying-algorithm)
- [Finalization Process](#finalization-process)
- [Security Guarantees](#security-guarantees)

---

## Core Concepts

### Validator Power

Each validator in the network has a **voting weight** (also called "power") that corresponds to their stake in the network. Validators with higher stakes have proportionally more voting power when determining consensus.

```rust
pub type Weight = u64;

/// Current validator weights. These are the ones who will vote on the blocks,
/// so these are the weights that need to form a quorum.
power_table: TVar<im::HashMap<K, Weight>>,
```

### Vote Types

When a validator attempts to download and verify a blob, it casts one of two vote types:

- **Success Vote (`true`)**: The validator successfully downloaded and verified the blob from the source node
- **Failure Vote (`false`)**: The validator failed to download or verify the blob

### Quorum Threshold

The system requires a **supermajority** to finalize any decision. The quorum threshold is calculated as:

```
quorum_threshold = (total_voting_weight × 2 / 3) + 1
```

This matches CometBFT's Byzantine Fault Tolerant consensus model and ensures the system can tolerate up to 1/3 of validators being malicious or offline.

---

## Vote Tally Architecture

The `VoteTally` structure maintains the state needed for consensus:

```rust
pub struct VoteTally<K = ValidatorKey, V = BlockHash, O = Blob> {
    /// Current validator weights for voting
    power_table: TVar<im::HashMap<K, Weight>>,

    /// Index votes received by blob
    /// Maps: Blob -> Validator -> Vote (true=resolved, false=failed)
    blob_votes: TVar<im::HashMap<O, im::HashMap<K, bool>>>,

    /// Pause flag to prevent vote additions during quorum calculation
    pause_blob_votes: TVar<bool>,
}
```

### Key Features

1. **Weighted Voting**: Each validator's vote is weighted by their stake
2. **Equivocation Prevention**: Validators cannot change a "resolved" vote to "failed"
3. **Concurrent Tallying**: Uses Software Transactional Memory (STM) for thread-safe operations
4. **Efficient Lookup**: Indexed by blob hash for fast quorum checks

---

## Voting Process

### 1. Blob Resolution Attempt

When a validator picks up a blob from the "added" or "pending" queue, it attempts to download it from the specified source node:

```rust
match client.resolve_iroh(task.hash(), size, source.id.into()).await {
    Ok(Ok(())) => {
        // Successfully downloaded and verified
        tracing::debug!(hash = %task.hash(), "iroh blob resolved");
        atomically(|| task.set_resolved()).await;

        // Cast success vote
        if add_own_vote(
            task.hash(),
            client,
            vote_tally,
            key,
            subnet_id,
            true,  // resolved = true
            to_vote,
        ).await {
            emit(BlobsFinalityVotingSuccess {
                blob_hash: Some(task.hash().to_string()),
            });
        }
    }
    Err(e) | Ok(Err(e)) => {
        // Failed to download or verify
        // Retry or cast failure vote after exhausting attempts
    }
}
```

### 2. Vote Recording

Each validator's vote is recorded with validation checks:

```rust
pub fn add_blob_vote(
    &self,
    validator_key: K,
    blob: O,
    resolved: bool,
) -> StmResult<bool, Error<K, O>> {
    // Check if voting is paused during quorum calculation
    if *self.pause_blob_votes.read()? {
        retry()?;
    }

    // Verify validator has voting power
    if !self.has_power(&validator_key)? {
        return abort(Error::UnpoweredValidator(validator_key));
    }

    let mut votes = self.blob_votes.read_clone()?;
    let votes_for_blob = votes.entry(blob).or_default();

    // Prevent equivocation: can't change "resolved" to "failed"
    if let Some(existing_vote) = votes_for_blob.get(&validator_key) {
        if *existing_vote {
            return Ok(false); // Ignore later votes
        }
    }

    votes_for_blob.insert(validator_key, resolved);
    self.blob_votes.write(votes)?;

    Ok(true)
}
```

### 3. Vote Propagation

After recording their own vote, validators gossip it to peers via the P2P network:

```rust
let vote = to_vote(vote_hash, resolved);
match VoteRecord::signed(&key, subnet_id, vote) {
    Ok(vote) => {
        let validator_key = ValidatorKey::from(key.public());

        // Add to local tally
        atomically_or_err(|| {
            vote_tally.add_blob_vote(
                validator_key.clone(),
                vote_hash.as_bytes().to_vec(),
                resolved,
            )
        }).await;

        // Broadcast to peers
        if let Err(e) = client.publish_vote(vote) {
            tracing::error!(error = e.to_string(), "failed to publish vote");
            return false;
        }
    }
}
```

---

## Quorum Calculation

### Standard Quorum (With Power Table)

For subnets with a parent chain that provides validator power information:

```rust
pub fn quorum_threshold(&self) -> Stm<Weight> {
    let total_weight: Weight = self.power_table.read().map(|pt| pt.values().sum())?;

    // Require 2/3 + 1 of total voting power
    Ok(total_weight * 2 / 3 + 1)
}
```

**Example:**
- Total validator power: 100
- Quorum threshold: (100 × 2 / 3) + 1 = 67

This means at least 67 units of voting power must agree for consensus.

### Development Mode (Empty Power Table)

For standalone/testing subnets without a parent chain:

```rust
let quorum_threshold = if power_table.is_empty() {
    1 as Weight  // At least one vote required
} else {
    self.quorum_threshold()?
};
```

---

## Vote Tallying Algorithm

The system separately tallies votes for "resolved" and "failed" outcomes:

```rust
pub fn find_blob_quorum(&self, blob: &O) -> Stm<(bool, bool)> {
    self.pause_blob_votes.write(false)?;

    let votes = self.blob_votes.read()?;
    let power_table = self.power_table.read()?;
    let quorum_threshold = if power_table.is_empty() {
        1 as Weight
    } else {
        self.quorum_threshold()?
    };

    let mut resolved_weight = 0;
    let mut failed_weight = 0;
    let mut voters = im::HashSet::new();

    let Some(votes_for_blob) = votes.get(blob) else {
        return Ok((false, false)); // No votes yet
    };

    // Sum weighted votes
    for (validator_key, resolved) in votes_for_blob {
        if voters.insert(validator_key.clone()).is_none() {
            // Get validator's current power (may be 0 if removed)
            let power = if power_table.is_empty() {
                1
            } else {
                power_table.get(validator_key).cloned().unwrap_or_default()
            };

            tracing::debug!("voter; key={}, power={}", validator_key.to_string(), power);

            if *resolved {
                resolved_weight += power;
            } else {
                failed_weight += power;
            }
        }
    }

    tracing::debug!(
        resolved_weight,
        failed_weight,
        quorum_threshold,
        "blob quorum; votes={}",
        votes_for_blob.len()
    );

    // Check if either outcome reached quorum
    if resolved_weight >= quorum_threshold {
        Ok((true, true))   // Quorum reached: RESOLVED
    } else if failed_weight >= quorum_threshold {
        Ok((true, false))  // Quorum reached: FAILED
    } else {
        Ok((false, false)) // No quorum yet
    }
}
```

### Return Values

The function returns a tuple `(bool, bool)`:

| Return Value | Meaning |
|--------------|---------|
| `(true, true)` | Quorum reached, blob **successfully stored** |
| `(true, false)` | Quorum reached, blob **failed to store** |
| `(false, false)` | No quorum reached yet, **keep waiting** |

---

## Finalization Process

### Proposing Finalization

When a validator believes a blob has reached quorum, they can propose finalization in a block:

```rust
ChainMessage::Ipc(IpcMessage::BlobFinalized(blob)) => {
    // 1. Check if already finalized on-chain
    let (is_blob_finalized, status) =
        with_state_transaction(&mut state, |state| {
            is_blob_finalized(state, blob.subscriber, blob.hash, blob.id.clone())
        })?;

    if is_blob_finalized {
        tracing::warn!(hash = %blob.hash, "blob already finalized (status={:?})", status);
    }

    // 2. Verify global quorum exists
    let (is_globally_finalized, succeeded) = atomically(|| {
        chain_env
            .parent_finality_votes
            .find_blob_quorum(&blob.hash.as_bytes().to_vec())
    }).await;

    if !is_globally_finalized {
        tracing::warn!(hash = %blob.hash, "not globally finalized; rejecting");
        return Ok(false);
    }

    // 3. Verify outcome matches proposal
    if blob.succeeded != succeeded {
        tracing::warn!(
            hash = %blob.hash,
            quorum = ?succeeded,
            message = ?blob.succeeded,
            "finalization mismatch; rejecting"
        );
        return Ok(false);
    }

    // 4. Accept proposal for inclusion in block
    // ...
}
```

### On-Chain State Update

Once finalized, the blob's status is updated in the Blobs Actor:

- **If succeeded**: Status changes to `BlobStatus::Resolved`
- **If failed**: Status changes to `BlobStatus::Failed`

The blob is then removed from the pending queues and recorded in the permanent state.

---

## Security Guarantees

### Byzantine Fault Tolerance

The 2/3+1 quorum threshold provides BFT guarantees:

- **Safety**: Can tolerate up to 1/3 Byzantine (malicious or faulty) validators
- **Liveness**: Can make progress as long as 2/3+ validators are online and honest

### Equivocation Prevention

The vote recording logic prevents validators from equivocating:

```rust
if let Some(existing_vote) = votes_for_blob.get(&validator_key) {
    if *existing_vote {
        // A vote for "resolved" was already made, ignore later votes
        return Ok(false);
    }
}
```

Once a validator votes "resolved", they cannot later vote "failed" for the same blob.

### Sybil Resistance

Votes are weighted by stake, preventing Sybil attacks where an attacker creates many low-power validators. An attacker would need to control 1/3+ of the total stake to disrupt consensus.

### Network Partition Tolerance

If the network partitions:
- No partition can finalize blobs without 2/3+ of total voting power
- Once the partition heals, validators with the minority view will accept the majority chain

---

## Vote Tally Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│ 1. Blob Added to Network                                    │
│    - Client uploads to their Iroh node                      │
│    - Registers with Blobs Actor (on-chain)                  │
│    - Blob enters "added" queue                              │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. Validators Pick Up Blob                                  │
│    - Fetch from "added" queue                               │
│    - Move to "pending" status                               │
│    - Begin download attempt from source node                │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. Each Validator Casts Weighted Vote                       │
│    ┌─────────────────┐        ┌─────────────────┐          │
│    │ Download Success│   OR   │ Download Failed │          │
│    │ Vote: true      │        │ Vote: false     │          │
│    │ Weight: stake   │        │ Weight: stake   │          │
│    └─────────────────┘        └─────────────────┘          │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 4. Votes Gossiped to Peers                                  │
│    - P2P network propagates signed votes                    │
│    - Each validator updates their local tally               │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 5. Vote Tally Accumulation                                  │
│    resolved_weight = Σ(power of validators voting success)  │
│    failed_weight   = Σ(power of validators voting failed)   │
│    quorum_threshold = (total_power × 2/3) + 1              │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 6. Quorum Check                                             │
│    ┌──────────────────────┐                                 │
│    │ resolved_weight      │─ YES ──> Blob RESOLVED ✓        │
│    │ >= quorum_threshold? │                                 │
│    └──────────────────────┘                                 │
│    ┌──────────────────────┐                                 │
│    │ failed_weight        │─ YES ──> Blob FAILED ✗          │
│    │ >= quorum_threshold? │                                 │
│    └──────────────────────┘                                 │
│            │                                                 │
│           NO ──> Keep waiting for more votes                │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 7. Finalization Proposal                                    │
│    - Validator proposes BlobFinalized message               │
│    - Other validators verify quorum exists                  │
│    - If consensus, include in block                         │
└────────────────────┬────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────┐
│ 8. On-Chain State Update                                    │
│    - Blob status updated in Blobs Actor                     │
│    - Removed from pending queue                             │
│    - Subscription confirmed for subscriber                  │
└─────────────────────────────────────────────────────────────┘
```

---

## Example Scenario

### Network Setup

```
Validator A: Power = 40
Validator B: Power = 35
Validator C: Power = 25
─────────────────────────
Total Power = 100
Quorum Threshold = (100 × 2/3) + 1 = 67
```

### Vote Progression for Blob `0xABCD...`

**Time T1:**
```
Validator A: ✓ resolved (weight: 40)
─────────────────────────
resolved_weight = 40
failed_weight   = 0
Status: No quorum yet (40 < 67)
```

**Time T2:**
```
Validator A: ✓ resolved (weight: 40)
Validator B: ✓ resolved (weight: 35)
─────────────────────────
resolved_weight = 75
failed_weight   = 0
Status: QUORUM REACHED - RESOLVED ✓
```

At T2, the blob can be finalized as successfully stored since `resolved_weight (75) >= quorum_threshold (67)`.

### Alternative: Failure Scenario

**Time T1:**
```
Validator A: ✗ failed (weight: 40)
Validator C: ✗ failed (weight: 25)
─────────────────────────
resolved_weight = 0
failed_weight   = 65
Status: No quorum yet (65 < 67)
```

**Time T2:**
```
Validator A: ✗ failed (weight: 40)
Validator B: ✓ resolved (weight: 35)
Validator C: ✗ failed (weight: 25)
─────────────────────────
resolved_weight = 35
failed_weight   = 65
Status: No quorum yet (neither reached 67)
```

In this scenario, no quorum is reached and the system waits for more validators to vote.

---

## Implementation Notes

### Concurrency Control

The system uses Software Transactional Memory (STM) for thread-safe operations:

```rust
// Atomic vote addition
let res = atomically_or_err(|| {
    vote_tally.add_blob_vote(
        validator_key.clone(),
        vote_hash.as_bytes().to_vec(),
        resolved,
    )
}).await;
```

### Pause Mechanism

During quorum calculation, vote additions can be paused to prevent race conditions:

```rust
pub fn pause_blob_votes_until_find_quorum(&self) -> Stm<()> {
    self.pause_blob_votes.write(true)
}
```

The `find_blob_quorum` function automatically re-enables voting when complete.

### Vote Cleanup

Once a blob is finalized on-chain, votes are cleared to free memory:

```rust
pub fn clear_blob(&self, blob: O) -> Stm<()> {
    self.blob_votes.update_mut(|votes| {
        votes.remove(&blob);
    })?;
    Ok(())
}
```

---

## Metrics and Observability

The system emits metrics for monitoring vote tally behavior:

```rust
// Vote success/failure counters
BLOBS_FINALITY_VOTING_SUCCESS
    .with_label_values(&[blob_hash])
    .inc();

BLOBS_FINALITY_VOTING_FAILURE
    .with_label_values(&[blob_hash])
    .inc();

// Pending blob gauges
BLOBS_FINALITY_PENDING_BLOBS.set(pending_count as i64);
BLOBS_FINALITY_PENDING_BYTES.set(pending_bytes as i64);
```

These metrics help operators monitor:
- Vote distribution across blobs
- Time to reach quorum
- Failed vs. successful resolutions
- Queue sizes and backlogs

---

## Related Documentation

- [CometBFT Consensus](https://github.com/cometbft/cometbft) - The underlying BFT consensus algorithm
- [Iroh P2P Network](https://iroh.computer/) - The peer-to-peer blob transfer layer
- IPC Subnet Architecture - Parent-child chain relationship and validator power propagation
- Recall Storage Architecture - Overall system design

---

## Conclusion

The vote tally mechanism provides a robust, Byzantine Fault Tolerant method for achieving consensus on blob storage across the Recall network. By combining weighted voting, supermajority quorums, and equivocation prevention, the system ensures that blobs are only marked as "stored" when a sufficient majority of validators (by stake) have successfully downloaded and verified them.

This design tolerates network partitions, validator failures, and up to 1/3 malicious actors while maintaining safety and liveness properties essential for a decentralized storage network.

