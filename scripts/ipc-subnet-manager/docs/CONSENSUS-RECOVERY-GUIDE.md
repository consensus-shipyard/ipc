# Consensus Recovery Guide

## When to Use This Guide

If you notice:
- Blocks stopped producing
- Parent finality stopped progressing
- Transactions not being processed
- `watch-blocks` showing `stalled` status

**DO NOT immediately run `init`!** Follow this guide first.

---

## Diagnostic Commands

### 1. Check Consensus Status
```bash
./ipc-manager consensus-status
```

**What to look for:**
- ✅ **All validators at same height** - Normal
- ⚠️ **Height difference 1-10 blocks** - Minor lag, usually OK
- 🚨 **Height difference >10 blocks** - One validator is stuck or slow
- 🚨 **Different app hashes at same height** - **STATE DIVERGENCE** (critical!)

**Example output:**
```
Validator      | Height | Block Hash          | App Hash            | Round | Step
---------------|--------|---------------------|---------------------|-------|-------------
validator-1    | 81     | B2000309938E9783... | 0171A0E40220CFBC... | 100   | RoundStepPrevote
validator-2    | 81     | B2000309938E9783... | 0171A0E40220D9F8... | 100   | RoundStepPrevote
validator-3    | 80     | A1FF0219827D8692... | 016F9E3F0110AEBF... | 0     | RoundStepNewHeight
```

☝️ This shows **state divergence** (different app hashes) and validator-3 is behind.

---

### 2. Check Voting Status
```bash
./ipc-manager voting-status
```

**What to look for:**
- ✅ **Prevote/Precommit 100%** and progressing - Normal
- ⚠️ **High round number** (>10) - Consensus struggling
- 🚨 **"wrong Block.Header.AppHash" errors** - **STATE DIVERGENCE**
- 🚨 **Low participation** (<67%) - Not enough validators voting

**Example healthy output:**
```
Current consensus: Height 150, Round 0, Step RoundStepNewHeight
Prevote participation: 3/3 validators (100%)
Precommit participation: 3/3 validators (100%)
✓ Consensus progressing normally
```

**Example stuck consensus:**
```
Current consensus: Height 81, Round 100, Step RoundStepPrevote
⚠ Consensus is in voting phase
Recent logs:
wrong Block.Header.AppHash. Expected 0171A0E4..., got 0171A0E4...
```

☝️ This means validators disagree on state and need recovery.

---

## Recovery Procedures

### Case 1: Height Divergence (No App Hash Mismatch)

One validator is behind but all have same app hash at their heights.

**Solution: Staggered Restart**
```bash
# Stop the lagging validator
ssh validator-3 "sudo su - ipc -c 'pkill -f ipc-cli'"

# Wait for it to restart (it will sync from others)
sleep 5

# Restart the validator
./ipc-manager restart --yes

# Check status again
./ipc-manager consensus-status
```

If still behind after 1-2 minutes, the validator may have disk/network issues.

---

### Case 2: App Hash Divergence (State Corruption)

Validators have **different app hashes** at the same height.

**This is CRITICAL - one or more validators have corrupted state.**

#### Step 1: Identify the bad validator
```bash
./ipc-manager consensus-status
```

Look for which validator has a different app hash from the majority.

#### Step 2: Stop the bad validator
```bash
ssh bad-validator "sudo su - ipc -c 'pkill -9 -f ipc-cli'"
```

#### Step 3: Backup its data (optional but recommended)
```bash
ssh bad-validator "sudo su - ipc -c 'cp -r ~/.ipc-node ~/.ipc-node.corrupted.$(date +%s)'"
```

#### Step 4: Wipe the bad validator's data
```bash
ssh bad-validator "sudo su - ipc -c 'rm -rf ~/.ipc-node/cometbft/data ~/.ipc-node/fendermint/data'"
```

#### Step 5: Copy state from a good validator
```bash
# From a working validator
ssh good-validator "sudo su - ipc -c 'tar czf /tmp/ipc-state.tar.gz ~/.ipc-node/cometbft/data ~/.ipc-node/fendermint/data'"

# To the bad validator
scp good-validator:/tmp/ipc-state.tar.gz /tmp/
scp /tmp/ipc-state.tar.gz bad-validator:/tmp/
ssh bad-validator "sudo su - ipc -c 'cd / && tar xzf /tmp/ipc-state.tar.gz'"
```

#### Step 6: Restart the bad validator
```bash
ssh bad-validator "sudo su - ipc -c '~/ipc/target/release/ipc-cli node start --home ~/.ipc-node &> ~/.ipc-node/logs/ipc-cli.log &'"
```

#### Step 7: Verify recovery
```bash
./ipc-manager consensus-status
./ipc-manager watch-blocks
```

---

### Case 3: Majority Stuck (No Single Bad Validator)

All validators are at the same height but can't progress (high round numbers, no state divergence).

**Possible causes:**
- Network partition (validators can't communicate)
- Insufficient voting power (need >67% to reach quorum)
- CometBFT consensus parameters too aggressive

#### Step 1: Check network connectivity
```bash
# From each validator, check if it can reach others
for ip in 34.73.187.192 34.75.205.89 35.237.175.224; do
  ssh validator-1 "ping -c 3 $ip"
done
```

#### Step 2: Check voting power
```bash
./ipc-manager info
```

Look for "Validator Status & Voting Power" section. Each validator should have >0 power.

#### Step 3: Check P2P connections
```bash
for ip in 34.73.187.192 34.75.205.89 35.237.175.224; do
  curl -s http://$ip:26657/net_info | jq '.result.n_peers'
done
```

Each should show `2` (connected to 2 other validators).

#### Step 4: Staggered restart (last resort before full reinit)
```bash
# Stop all validators (one at a time, waiting between each)
ssh validator-3 "sudo su - ipc -c 'pkill -f ipc-cli'"
sleep 10

ssh validator-2 "sudo su - ipc -c 'pkill -f ipc-cli'"
sleep 10

ssh validator-1 "sudo su - ipc -c 'pkill -f ipc-cli'"
sleep 10

# Restart all
./ipc-manager restart --yes

# Monitor
./ipc-manager watch-blocks
```

If consensus still doesn't progress after 30 seconds, **you have a deeper issue** and may need to reinitialize.

---

### Case 4: Complete Failure (Nuclear Option)

**Only use this if:**
- State divergence cannot be resolved
- All validators have different app hashes
- Network is completely partitioned
- This is a **test** subnet (not production)

```bash
./ipc-manager init --yes
```

**⚠️ WARNING:** This **deletes all subnet data** and starts a new chain with a new genesis. Any assets or state on the old chain are **lost forever**.

**For production subnets:**
1. Take full backups first
2. Investigate the root cause with the IPC team
3. Consider upgrading to a newer IPC version with bug fixes
4. Only reinit as an absolute last resort

---

## Monitoring After Recovery

After any recovery procedure, monitor for 10+ minutes:

```bash
# Terminal 1: Watch blocks
./ipc-manager watch-blocks

# Terminal 2: Watch finality
./ipc-manager watch-finality

# Terminal 3: Dashboard
./ipc-manager dashboard
```

**Healthy signs:**
- Block height increasing every 1-2 seconds
- Parent finality progressing every 10-30 seconds
- Round number staying at 0 or low (0-5)
- No app hash mismatch errors in logs
- All validators with same height (±1 block)

**Warning signs:**
- Blocks stopped for >10 seconds
- Round number climbing above 20
- App hash errors reappearing
- Height divergence increasing
- Mempool building up (>100 transactions)

If warning signs appear, re-run diagnostics:
```bash
./ipc-manager consensus-status
./ipc-manager voting-status
```

---

## Common Root Causes

### State Divergence
- **Bug in Fendermint state machine** - Non-deterministic execution
- **Disk corruption** - Validator wrote bad data
- **Manual state modification** - Someone edited files directly
- **Version mismatch** - Validators running different IPC versions

### Consensus Stalls
- **Network issues** - Firewalls, packet loss, high latency
- **Insufficient resources** - Validator out of CPU/memory/disk
- **Timeout parameters too aggressive** - `timeout_propose: 300ms` may be too fast
- **Bottom-up checkpointing bug** - Nonce errors clogging mempool

### Height Divergence
- **One validator offline** - Crashed, restarted, or slow to sync
- **Block production pause** - Mempool full or state query hang
- **Disk I/O bottleneck** - Slow writes preventing block commits

---

## Prevention

### Regular Monitoring
```bash
# Run every 10 minutes via cron
*/10 * * * * /path/to/ipc-manager consensus-status | grep -q "✗ CRITICAL" && alert-on-call
```

### Automated Alerts
Set up alerts for:
- Block production stopped for >1 minute
- Parent finality not progressing for >5 minutes
- Round number >50
- Mempool size >1000
- Height divergence >20 blocks

### Backup Strategy
```bash
# Daily backups (before they're older than 16 hours for parent finality)
0 0 * * * ssh validator-1 "sudo su - ipc -c 'tar czf /backup/ipc-node-$(date +%Y%m%d).tar.gz ~/.ipc-node/cometbft/data ~/.ipc-node/fendermint/data'"
```

### Version Control
- Keep all validators on the same IPC version
- Test upgrades on a staging subnet first
- Coordinate upgrades (don't upgrade mid-consensus round)

---

## Summary: Quick Decision Tree

```
Is consensus progressing?
├─ YES → Monitor normally
└─ NO → Run consensus-status

Are all validators at same height?
├─ NO (>10 blocks apart)
│   └─ Restart lagging validator
│       └─ Still behind? → Check disk/network/resources
│
└─ YES (same height ±1)
    └─ Run voting-status

Do all validators have same app hash?
├─ NO (app hash divergence)
│   └─ CRITICAL STATE CORRUPTION
│       ├─ Identify minority validator(s)
│       ├─ Stop bad validator(s)
│       ├─ Wipe bad validator data
│       ├─ Copy state from good validator
│       └─ Restart bad validator
│
└─ YES (same app hash)
    └─ Is round number high (>20)?
        ├─ YES → Network partition or resource issue
        │   ├─ Check P2P connectivity
        │   ├─ Check voting power (need >67%)
        │   ├─ Check mempool (full = stall)
        │   └─ Staggered restart
        │
        └─ NO → Consensus healthy, check parent finality
            └─ watch-finality
```

