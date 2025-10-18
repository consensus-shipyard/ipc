# Parent Finality 16-Hour Lookback Issue

## Problem Summary

IPC subnets that are more than 16 hours old **cannot establish parent finality** when using the Glif Calibration testnet RPC endpoint (`https://api.calibration.node.glif.io/rpc/v1`). This makes parent finality and top-down message processing (including `cross-msg fund`) completely non-functional.

## Root Cause

### The Technical Chain of Events

1. **Subnet Genesis is Fixed**: When a subnet is created on the parent chain, it records a `genesis_epoch` (the parent block height at subnet creation time).

2. **Parent Finality Initialization**: When subnet nodes start, the parent finality polling syncer calls:
   ```
   query_starting_finality() → get_genesis_epoch() → get_block_hash(genesis_epoch)
   ```

3. **16-Hour RPC Restriction**: The Glif Calibration RPC endpoint returns:
   ```
   ERROR: bad tipset height: lookbacks of more than 16h40m0s are disallowed
   ```

4. **Fatal Failure**: The `launch_polling_syncer()` function returns an error and **never retries**. Parent finality is permanently broken.

### Code Reference

From `fendermint/vm/topdown/src/sync/mod.rs`:

```rust
async fn query_starting_finality<T, P>(
    query: &Arc<T>,
    parent_client: &Arc<P>,
) -> anyhow::Result<IPCParentFinality>
{
    // ...
    if finality.height == 0 {
        let genesis_epoch = parent_client.get_genesis_epoch().await?;  // ✓ This succeeds
        let r = parent_client.get_block_hash(genesis_epoch).await?;     // ✗ THIS FAILS if >16h old

        finality = IPCParentFinality {
            height: genesis_epoch,
            block_hash: r.block_hash,
        };
    }
    return Ok(finality);
}

pub async fn launch_polling_syncer<T, C, P>(...) -> anyhow::Result<()> {
    let finality = query_starting_finality(&query, &parent_client).await?;  // ✗ Error propagates up
    // ... rest of initialization never happens
}
```

From `fendermint/app/src/service/node.rs`:

```rust
if let Err(e) = launch_polling_syncer(...).await {
    tracing::error!(error = ?e, "cannot launch polling syncer");  // Logged once
    return;  // ✗ Function exits, no retry
}
```

## Impact

### Affected Scenarios
- ✗ Any subnet >16 hours old on Calibration testnet using Glif RPC
- ✗ Subnets that restart nodes after >16 hours of operation
- ✗ Development/testing subnets that are paused and resumed later
- ✗ Production subnets during multi-day outages

### Broken Functionality
- ❌ Parent finality cannot progress beyond genesis (height 0)
- ❌ No parent finality votes are exchanged
- ❌ Top-down messages never execute (`cross-msg fund`, `cross-msg release`)
- ❌ Parent chain state changes don't propagate to child subnet
- ❌ Cross-chain transfers are impossible

## Current Workarounds

### Option 1: Create a New Subnet
**Pros:**
- Guarantees a genesis epoch within the 16-hour window
- Works immediately

**Cons:**
- Loses all subnet state and history
- Requires redeploying contracts
- Not viable for production subnets

### Option 2: Use a Different RPC Endpoint
**Requirements:**
- Find a Calibration RPC endpoint without the 16-hour restriction
- Update `~/.ipc/config.toml` and `node-init.yml` configurations

**Challenges:**
- Glif is the primary/official Calibration endpoint
- Alternative endpoints may have other limitations
- No guarantee of long-term availability

### Option 3: Run Your Own Lotus Node
**Pros:**
- Full control over lookback restrictions
- No external dependencies

**Cons:**
- Significant infrastructure cost
- Requires Lotus node maintenance
- Sync time for historical data

## Proposed Solutions

### Solution 1: Retry with Incremental Catchup (Short-term Fix)

**Approach:**
Instead of querying the genesis epoch directly, use an incremental catchup strategy:

```rust
async fn query_starting_finality_with_fallback<T, P>(
    query: &Arc<T>,
    parent_client: &Arc<P>,
    max_lookback_hours: u64,
) -> anyhow::Result<IPCParentFinality>
{
    // Try to get committed finality from subnet state
    if let Some(finality) = query.get_latest_committed_finality()? {
        if finality.height > 0 {
            return Ok(finality);  // Use existing finality if available
        }
    }

    // Genesis case: try to get genesis epoch
    let genesis_epoch = parent_client.get_genesis_epoch().await?;

    // Try to get block hash for genesis epoch
    match parent_client.get_block_hash(genesis_epoch).await {
        Ok(r) => {
            // Success - genesis is within lookback window
            return Ok(IPCParentFinality {
                height: genesis_epoch,
                block_hash: r.block_hash,
            });
        }
        Err(e) if is_lookback_error(&e) => {
            // Genesis is too old, use current parent chain head instead
            tracing::warn!(
                genesis_epoch,
                error = e.to_string(),
                "genesis epoch outside lookback window, starting from current parent chain head"
            );

            let current_height = parent_client.get_chain_head_height().await?;
            let current_block = parent_client.get_block_hash(current_height).await?;

            return Ok(IPCParentFinality {
                height: current_height,
                block_hash: current_block.block_hash,
            });
        }
        Err(e) => return Err(e),
    }
}

fn is_lookback_error(err: &anyhow::Error) -> bool {
    let err_str = err.to_string().to_lowercase();
    err_str.contains("lookback") && err_str.contains("disallowed")
}
```

**Pros:**
- ✅ Works with 16-hour restriction
- ✅ Allows subnet to catch up from current height
- ✅ No infrastructure changes needed
- ✅ Backward compatible (still tries genesis first)

**Cons:**
- ⚠️ Loses historical parent finality data (gap from genesis to current)
- ⚠️ Top-down messages submitted before the gap will never execute
- ⚠️ May confuse users about missing historical data

**Implementation:**
- File: `fendermint/vm/topdown/src/sync/mod.rs`
- Function: `query_starting_finality()`
- Add fallback logic to handle lookback errors
- Add configuration option: `max_parent_lookback_hours`

### Solution 2: Persistent Parent Finality Checkpoints (Medium-term Fix)

**Approach:**
Store parent finality checkpoints in subnet state and use the most recent valid checkpoint:

```rust
struct ParentFinalityCheckpoint {
    height: BlockHeight,
    block_hash: BlockHash,
    timestamp: u64,
    checkpoint_hash: Hash,
}

impl SubnetState {
    fn get_latest_valid_checkpoint(&self, max_age_hours: u64) -> Option<ParentFinalityCheckpoint> {
        let now = current_timestamp();
        self.parent_finality_checkpoints
            .iter()
            .filter(|cp| now - cp.timestamp < max_age_hours * 3600)
            .max_by_key(|cp| cp.height)
    }

    fn store_checkpoint(&mut self, checkpoint: ParentFinalityCheckpoint) {
        self.parent_finality_checkpoints.push(checkpoint);
        // Keep only last 100 checkpoints
        if self.parent_finality_checkpoints.len() > 100 {
            self.parent_finality_checkpoints.drain(0..50);
        }
    }
}
```

**Workflow:**
1. Every N blocks (e.g., 100), store the current parent finality as a checkpoint
2. On startup, query the latest checkpoint within the lookback window
3. Resume parent finality sync from that checkpoint
4. If no valid checkpoint exists, fall back to Solution 1

**Pros:**
- ✅ Minimal data loss (only up to N blocks)
- ✅ Works across restarts
- ✅ Automatic recovery from outages
- ✅ No external dependencies

**Cons:**
- ⚠️ Requires state migration for existing subnets
- ⚠️ Adds storage overhead for checkpoints
- ⚠️ Checkpoint interval must be < lookback window

**Implementation:**
- File: `fendermint/vm/topdown/src/checkpoint.rs` (new)
- Update: `fendermint/vm/interpreter/src/fvm/state/mod.rs`
- Add checkpoint storage to subnet state
- Add checkpoint creation every N blocks
- Update `query_starting_finality()` to use checkpoints

### Solution 3: Multi-Tier Parent Syncing (Long-term Fix)

**Approach:**
Implement a tiered syncing strategy that combines multiple data sources:

```
Tier 1: Subnet State (immediate, always available)
  └─> Latest committed finality from local state

Tier 2: Peer Gossip (fast, depends on peer availability)
  └─> Request recent parent finality from peers

Tier 3: Parent Chain Current State (medium, restricted by lookback)
  └─> Query current parent chain head (always works)

Tier 4: Archive Node (slow, optional, no restrictions)
  └─> Full historical data from dedicated archive endpoint
```

**Syncing Logic:**
```rust
async fn initialize_parent_syncing(&self) -> Result<IPCParentFinality> {
    // Tier 1: Try local state
    if let Some(finality) = self.get_local_finality() {
        if self.is_recent(finality.height) {
            return Ok(finality);
        }
    }

    // Tier 2: Try peers
    if let Ok(finality) = self.request_finality_from_peers().await {
        if self.validate_peer_finality(&finality) {
            return Ok(finality);
        }
    }

    // Tier 3: Use current parent chain head (always works)
    let current = self.get_parent_chain_head().await?;

    // Tier 4: Backfill from archive if configured
    if let Some(archive_endpoint) = &self.config.archive_endpoint {
        tokio::spawn(self.backfill_from_archive(archive_endpoint, current.height));
    }

    Ok(current)
}
```

**Configuration:**
```toml
[ipc.topdown]
# Existing config
parent_http_endpoint = "https://api.calibration.node.glif.io/rpc/v1"

# New: Optional archive endpoint for historical data
parent_archive_endpoint = "https://archive.node.example.com/rpc/v1"

# New: Enable peer finality exchange
enable_peer_finality_exchange = true

# New: Maximum lookback supported by primary endpoint (in blocks)
max_lookback_blocks = 28800  # ~16 hours at 2s/block
```

**Pros:**
- ✅ Robust across all failure scenarios
- ✅ Gracefully degrades when sources unavailable
- ✅ Enables peer-to-peer recovery
- ✅ Optional archive support for full history
- ✅ No forced data loss

**Cons:**
- ⚠️ Complex implementation
- ⚠️ Requires peer finality exchange protocol
- ⚠️ Archive node infrastructure is optional but beneficial

**Implementation:**
- File: `fendermint/vm/topdown/src/sync/tiered.rs` (new)
- Update: `fendermint/vm/topdown/src/sync/mod.rs`
- File: `fendermint/vm/resolver/src/peer_finality.rs` (new)
- Add peer finality request/response messages
- Add archive endpoint configuration
- Implement tiered fallback logic

### Solution 4: Dynamic Genesis Epoch Adjustment

**Approach:**
Allow subnets to "fast-forward" their parent finality genesis under specific conditions:

```rust
struct GenesisAdjustmentProposal {
    new_genesis_height: BlockHeight,
    new_genesis_hash: BlockHash,
    reason: AdjustmentReason,
    proposer: ValidatorId,
    signatures: Vec<ValidatorSignature>,
}

enum AdjustmentReason {
    LookbackRestriction,
    ParentReorg,
    ManualIntervention,
}

impl ParentFinalityManager {
    async fn propose_genesis_adjustment(&mut self, reason: AdjustmentReason) -> Result<()> {
        // Only allow if current genesis is unreachable
        if self.can_reach_genesis() {
            return Err("Genesis is reachable, adjustment not needed");
        }

        // Require 2/3+ validator approval
        let current_height = self.parent_client.get_chain_head_height().await?;
        let proposal = GenesisAdjustmentProposal {
            new_genesis_height: current_height,
            new_genesis_hash: self.parent_client.get_block_hash(current_height).await?.block_hash,
            reason,
            proposer: self.validator_id,
            signatures: vec![],
        };

        // Broadcast to validators for voting
        self.broadcast_adjustment_proposal(proposal).await?;
        Ok(())
    }

    fn apply_genesis_adjustment(&mut self, proposal: GenesisAdjustmentProposal) -> Result<()> {
        // Verify 2/3+ signatures
        if !self.verify_quorum(&proposal.signatures) {
            return Err("Insufficient validator approval");
        }

        // Update genesis in state
        self.state.update_parent_genesis(
            proposal.new_genesis_height,
            proposal.new_genesis_hash,
        )?;

        tracing::info!(
            old_genesis = self.genesis_epoch,
            new_genesis = proposal.new_genesis_height,
            reason = ?proposal.reason,
            "applied genesis epoch adjustment"
        );

        Ok(())
    }
}
```

**Governance:**
- Requires 2/3+ validator signatures
- Can only be applied when genesis is unreachable
- Logged and auditable
- Optional manual approval mode for high-security subnets

**Pros:**
- ✅ Preserves subnet continuity
- ✅ Democratic validator decision
- ✅ Works for any lookback restriction
- ✅ Handles parent chain reorgs

**Cons:**
- ⚠️ Requires consensus mechanism
- ⚠️ Could be abused if majority collude
- ⚠️ Loses historical parent finality data
- ⚠️ Complex governance logic

**Implementation:**
- File: `fendermint/vm/topdown/src/governance.rs` (new)
- Add genesis adjustment proposal/voting
- Integrate with voting mechanism
- Add governance event logging

## Recommended Implementation Plan

### Phase 1: Immediate (Week 1-2)
**Goal:** Unblock current deployments

1. Implement **Solution 1** (Retry with Incremental Catchup)
   - Quick to implement (~2-3 days)
   - Solves immediate problem
   - Document the data gap implications

2. Add configuration option:
   ```toml
   [ipc.topdown]
   fallback_to_current_on_genesis_error = true
   ```

3. Update documentation:
   - Explain the 16-hour restriction
   - Document when data gaps occur
   - Provide workarounds for production

### Phase 2: Short-term (Month 1)
**Goal:** Minimize data loss

1. Implement **Solution 2** (Persistent Checkpoints)
   - Checkpoint every 100 blocks
   - Store in subnet state
   - Automatic recovery on restart

2. Add monitoring:
   - Alert when parent finality lags significantly
   - Track checkpoint age
   - Monitor lookback violations

### Phase 3: Medium-term (Month 2-3)
**Goal:** Robust multi-source syncing

1. Implement **Solution 3** (Multi-Tier Syncing)
   - Add peer finality exchange
   - Support optional archive endpoints
   - Tiered fallback logic

2. Configuration improvements:
   - Multiple parent RPC endpoints
   - Automatic endpoint failover
   - Health checks for endpoints

### Phase 4: Long-term (Month 4-6)
**Goal:** Complete resilience and governance

1. Implement **Solution 4** (Genesis Adjustment)
   - Validator voting mechanism
   - Governance framework
   - Audit logging

2. Testing & Documentation:
   - Test all failure scenarios
   - Update IPC specification
   - Provide migration guides

## Testing Strategy

### Test Cases

1. **Fresh Subnet (<16h old)**
   - ✅ Should use genesis epoch directly
   - ✅ Parent finality works normally

2. **Old Subnet (>16h old)**
   - ✅ Should fallback to current parent height
   - ✅ Parent finality resumes from current
   - ✅ Log warning about data gap

3. **Subnet Restart After Outage**
   - ✅ Should use latest checkpoint
   - ✅ Minimal data loss (< checkpoint interval)

4. **RPC Endpoint Failure**
   - ✅ Should try alternative endpoints
   - ✅ Should request finality from peers
   - ✅ Graceful degradation

5. **Parent Chain Reorg**
   - ✅ Detect and handle reorg
   - ✅ Revalidate recent finality
   - ✅ Recover automatically

### Integration Tests

```rust
#[tokio::test]
async fn test_genesis_outside_lookback_window() {
    let mut parent_mock = MockParentClient::new();

    // Genesis epoch is 24 hours old
    parent_mock.expect_get_genesis_epoch()
        .returning(|| Ok(43200));  // 24h * 3600s / 2s per block

    // get_block_hash for genesis returns lookback error
    parent_mock.expect_get_block_hash()
        .with(eq(43200))
        .returning(|_| Err(anyhow!("bad tipset height: lookbacks of more than 16h40m0s are disallowed")));

    // Current chain head is available
    parent_mock.expect_get_chain_head_height()
        .returning(|| Ok(50000));

    parent_mock.expect_get_block_hash()
        .with(eq(50000))
        .returning(|_| Ok(BlockHashResult {
            block_hash: vec![1, 2, 3],
            parent_block_hash: vec![0, 1, 2],
        }));

    // Should fall back to current height
    let finality = query_starting_finality_with_fallback(&query, &parent_mock, 16).await?;
    assert_eq!(finality.height, 50000);
}
```

## Documentation Updates

### User Documentation
- **`docs/ipc/troubleshooting.md`**:
  - Add section on 16-hour lookback issue
  - Explain when it occurs
  - Provide resolution steps

- **`docs/ipc/parent-finality.md`**:
  - Document parent finality architecture
  - Explain initialization process
  - Describe fallback mechanisms

### Developer Documentation
- **`fendermint/vm/topdown/README.md`**:
  - Document syncing tiers
  - Explain checkpoint system
  - API reference for parent finality

### Configuration Guide
- **`docs/ipc/configuration.md`**:
  - Document all topdown configuration options
  - Explain RPC endpoint selection
  - Best practices for production

## Metrics & Monitoring

### Key Metrics to Add

```rust
// Parent finality metrics
metrics::gauge!("ipc.parent_finality.height").set(finality.height as f64);
metrics::gauge!("ipc.parent_finality.lag_blocks").set(lag as f64);
metrics::counter!("ipc.parent_finality.lookback_errors").increment(1);
metrics::counter!("ipc.parent_finality.fallback_to_current").increment(1);
metrics::counter!("ipc.parent_finality.checkpoint_created").increment(1);

// Syncing metrics
metrics::histogram!("ipc.parent_sync.duration_ms").record(duration.as_millis() as f64);
metrics::gauge!("ipc.parent_sync.last_success_timestamp").set(timestamp as f64);
metrics::counter!("ipc.parent_sync.rpc_errors").increment(1);
```

### Alerting Rules

```yaml
alerts:
  - name: ParentFinalityStalled
    condition: ipc_parent_finality_lag_blocks > 1000
    severity: critical
    message: "Parent finality is lagging by {{ $value }} blocks"

  - name: ParentSyncErrors
    condition: rate(ipc_parent_sync_rpc_errors[5m]) > 0.1
    severity: warning
    message: "Parent RPC errors: {{ $value }}/s"

  - name: LookbackRestictionHit
    condition: ipc_parent_finality_lookback_errors > 0
    severity: info
    message: "Subnet hit RPC lookback restriction, using fallback"
```

## Alternative Approaches (Considered but Not Recommended)

### 1. Increase Lookback Window on RPC
**Why Not:** Requires infrastructure changes outside IPC's control. Glif operates the Calibration RPC and may have reasons for the 16-hour limit.

### 2. Disable Parent Finality
**Why Not:** Breaks core IPC functionality. Top-down messages are essential for cross-chain communication.

### 3. Pre-fetch and Cache All Parent Blocks
**Why Not:** Requires massive storage and doesn't solve the initial sync problem for new nodes.

### 4. Trust First Responding Peer
**Why Not:** Security risk. Malicious peer could provide fake parent finality data.

## Conclusion

The 16-hour lookback restriction is a critical blocker for IPC subnet operation on Calibration testnet. The recommended approach is a **phased implementation**:

1. **Immediate**: Fallback to current parent height (Solution 1)
2. **Short-term**: Add persistent checkpoints (Solution 2)
3. **Medium-term**: Implement multi-tier syncing (Solution 3)
4. **Long-term**: Add governance for genesis adjustment (Solution 4)

This provides immediate relief while building toward a robust, production-ready solution.

## References

- **Affected Code**: `fendermint/vm/topdown/src/sync/mod.rs`
- **RPC Error**: Glif Calibration endpoint 16-hour lookback restriction
- **Related Issue**: Subnet initialization and restart failures
- **Impact**: Complete loss of parent finality and top-down message functionality

---

**Document Version**: 1.0
**Date**: October 17, 2025
**Author**: AI Assistant (via troubleshooting session)
**Status**: Proposed Solutions - Awaiting Implementation

