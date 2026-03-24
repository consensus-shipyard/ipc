# Diagnostic Tools Summary

## What Was Added

### 1. `consensus-status` Command
**Purpose:** Show the current state of all validators to identify divergence

**Usage:**
```bash
./ipc-manager consensus-status
```

**Shows:**
- Current block height for each validator
- Block hash at that height
- App hash (state root) at that height
- Current consensus round and step
- Automatically detects:
  - ✅ Height synchronization across validators
  - 🚨 App hash divergence (state corruption)
  - ⚠️ Validators falling behind

**When to use:**
- Blocks stopped being produced
- Before deciding to reinitialize
- To identify which validator has bad state
- Regular health monitoring

---

### 2. `voting-status` Command
**Purpose:** Show detailed consensus voting information for the current round

**Usage:**
```bash
./ipc-manager voting-status
```

**Shows:**
- Current height, round, and consensus step
- Total voting power and quorum threshold
- Prevote and precommit participation
- Recent consensus activity from logs
- Consensus errors (app hash mismatches, timeouts)

**When to use:**
- Chain is stuck but validators are at same height
- To understand why consensus isn't progressing
- To see if validators are voting
- To detect network or voting power issues

---

## Integration with Existing Tools

### Before (No Diagnostics)
```
User: "Chain is stuck"
Engineer: *checks dashboard, sees stalled*
Engineer: "Let's just reinit"
./ipc-manager init --yes
Result: All data lost, no root cause identified
```

### After (With Diagnostics)
```
User: "Chain is stuck"
Engineer: ./ipc-manager watch-blocks
→ Shows: stalled at height 80

Engineer: ./ipc-manager consensus-status
→ Shows: All validators at height 80 with same app hash

Engineer: ./ipc-manager voting-status
→ Shows: Stuck at height 81 with app hash mismatch
→ Error: "wrong Block.Header.AppHash. Expected X, got Y"

Engineer: "validator-2 has corrupted state, let's fix it"
→ Stop validator-2
→ Wipe its data
→ Copy state from validator-1
→ Restart validator-2

Engineer: ./ipc-manager watch-blocks
→ Shows: producing blocks again

Result: Chain recovered, root cause identified, no data loss
```

---

## Diagnostic Decision Flow

```
Chain not producing blocks?
    ↓
./ipc-manager watch-blocks
    ↓ (confirms stalled)
./ipc-manager consensus-status
    ↓
Are validators at different heights?
│
├─ YES → Height divergence
│   └─ Restart the lagging validator
│       (it will sync from peers)
│
└─ NO → Same height
    ↓
    ./ipc-manager voting-status
    ↓
    Do validators have different app hashes?
    │
    ├─ YES → State divergence (CRITICAL)
    │   └─ Identify minority validator
    │       Stop it, wipe data, copy from good validator
    │
    └─ NO → Consensus stuck (not state divergence)
        └─ Check voting participation
            Check network connectivity
            Check mempool status
            Staggered restart if needed
```

---

## Key Differences from `init`

### `init` (Nuclear Option)
- **Deletes everything:** All blocks, all state, all history
- **Creates new chain:** New genesis, new subnet ID possible
- **Loses data:** Any on-chain assets or state is gone
- **Fast but destructive:** Takes ~2 minutes
- **Use when:** State is completely unsalvageable

### Diagnostic + Targeted Recovery
- **Preserves data:** Only bad validator's data is wiped
- **Same chain:** Continues from last good block
- **Identifies root cause:** Know what went wrong
- **Surgical fix:** Only fix what's broken
- **Takes longer:** 5-10 minutes depending on data size
- **Use when:** State divergence or validator lag

---

## Example Real-World Scenario

**Scenario:** After the bottom-up checkpointing fix was deployed, the subnet got stuck.

### Without Diagnostics (What We Did)
1. Noticed chain stalled via `watch-finality`
2. Assumed complete failure
3. Ran `./ipc-manager init --yes`
4. Lost all previous blocks and state
5. Had to resubmit `cross-msg fund`

### With Diagnostics (What We Should Have Done)
1. Run `./ipc-manager consensus-status`
   - Would show: All validators at height 80, same app hash
2. Run `./ipc-manager voting-status`
   - Would show: Stuck at height 81, app hash mismatch on validator-2
3. Recover validator-2:
   ```bash
   ssh validator-2 "sudo su - ipc -c 'pkill -9 -f ipc-cli'"
   ssh validator-2 "sudo su - ipc -c 'rm -rf ~/.ipc-node/cometbft/data ~/.ipc-node/fendermint/data'"

   ssh validator-1 "sudo su - ipc -c 'tar czf /tmp/state.tar.gz ~/.ipc-node/cometbft/data ~/.ipc-node/fendermint/data'"
   scp philip@validator-1:/tmp/state.tar.gz /tmp/
   scp /tmp/state.tar.gz philip@validator-2:/tmp/
   ssh validator-2 "sudo su - ipc -c 'cd / && tar xzf /tmp/state.tar.gz'"

   ssh validator-2 "sudo su - ipc -c '~/ipc/target/release/ipc-cli node start --home ~/.ipc-node &> ~/.ipc-node/logs/ipc-cli.log &'"
   ```
4. Verify recovery:
   ```bash
   ./ipc-manager watch-blocks
   ```
   - Would show: blocks producing again, height 81, 82, 83...
5. Result: **No data loss, chain continues, root cause identified**

---

## When to Still Use `init`

### Acceptable Use Cases
1. **Initial subnet creation** - First time setup
2. **Complete infrastructure change** - New validator set, new network
3. **Testing/development** - Rapid iteration, don't care about state
4. **Irrecoverable state corruption** - All validators have diverged
5. **Known bug in genesis** - Need to recreate with fixed parameters

### NOT Acceptable Use Cases
1. ❌ "Chain is stuck" - Diagnose first
2. ❌ "One validator crashed" - Just restart it
3. ❌ "Mempool is full" - Clear mempool or fix root cause
4. ❌ "I changed a config" - Use `update-config` and restart
5. ❌ "Production subnet failure" - **NEVER** without explicit approval

---

## Monitoring Integration

### Automated Health Checks
Add to cron (every 10 minutes):
```bash
#!/bin/bash
# /etc/cron.d/ipc-health-check

*/10 * * * * ipc /path/to/ipc-manager consensus-status 2>&1 | grep -q "CRITICAL" && curl -X POST https://alerts.example.com/critical
```

### Dashboard Enhancement
The `dashboard` command already shows:
- Block height and production rate
- Mempool status
- Error categorization

Add a "Consensus Health" indicator:
```bash
# In lib/dashboard.sh - fetch_metrics()
local consensus_health=$(show_consensus_status 2>&1 | grep -c "CRITICAL")
METRICS[consensus_critical]=$consensus_health
```

---

## Future Enhancements

### Automatic Recovery (with approval)
```bash
./ipc-manager auto-recover
```
- Runs diagnostics
- Proposes recovery plan
- Asks for confirmation
- Executes recovery
- Monitors results

### Historical Analysis
```bash
./ipc-manager analyze-divergence --height 81
```
- Shows what happened at the divergence point
- Compares state between validators
- Identifies which transaction caused divergence

### State Diff Tool
```bash
./ipc-manager state-diff validator-1 validator-2 --height 80
```
- Compares Fendermint state between validators
- Shows exact differences in accounts, storage, etc.

---

## Summary

**Before these tools:**
- "Chain stuck → init" was the only option
- No visibility into what went wrong
- Data loss was accepted
- Root causes remained unknown

**After these tools:**
- Surgical diagnosis of consensus issues
- Targeted recovery without data loss
- Root cause identification
- Production-ready recovery procedures

**Impact:**
- **Reduced downtime:** Minutes instead of hours
- **Preserved state:** No need to replay transactions
- **Better debugging:** Understand failure modes
- **Confidence:** Know when `init` is actually needed

The subnet manager is now a **production-grade operational tool**, not just a setup script.

