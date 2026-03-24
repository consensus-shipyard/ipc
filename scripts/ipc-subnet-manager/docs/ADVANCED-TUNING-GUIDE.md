# Advanced Performance Tuning Guide

## Current Configuration (After Optimization)

Your subnet is now configured with aggressive performance settings. Here's what each parameter does:

## ⚡ Consensus Timeouts

### Core Timeouts
These control how long validators wait at each consensus step:

| Parameter | Value | Default | Impact |
|-----------|-------|---------|--------|
| `timeout_commit` | **100ms** | 5s | ⏱️ Time between blocks |
| `timeout_propose` | **500ms** | 3s | 📤 Time to wait for block proposal |
| `timeout_prevote` | **200ms** | 1s | 🗳️ Time to wait for prevote messages |
| `timeout_precommit` | **200ms** | 1s | ✅ Time to wait for precommit messages |

**Expected Impact:** Block time could drop to **0.3-0.5s** (from current 0.65s)

### Timeout Deltas (Round Increases)
If consensus fails in a round, timeouts increase by these amounts:

| Parameter | Value | Default | Why it matters |
|-----------|-------|---------|----------------|
| `timeout_propose_delta` | **100ms** | 500ms | Slower recovery, but acceptable |
| `timeout_prevote_delta` | **50ms** | 500ms | Faster retry on failed prevotes |
| `timeout_precommit_delta` | **50ms** | 500ms | Faster retry on failed precommits |

**Impact:** Failed rounds recover faster (but less tolerant of persistent issues)

---

## 📦 Block Production

| Parameter | Value | Why |
|-----------|-------|-----|
| `create_empty_blocks` | **true** | Consistent timing, faster finality |
| `create_empty_blocks_interval` | **0s** | Produce immediately after timeout_commit |

**Expected:** Steady block production even with no transactions

---

## 🌐 Network Performance

### P2P Bandwidth
| Parameter | Value | Default | Impact |
|-----------|-------|---------|--------|
| `send_rate` | **20 MB/s** | 5 MB/s | 4x faster block propagation |
| `recv_rate` | **20 MB/s** | 5 MB/s | 4x faster vote collection |
| `max_packet_msg_payload_size` | **10 KB** | 1 KB | 10x larger packets = fewer round trips |

**Expected:** Faster consensus with less network overhead

---

## 🔗 IPC Cross-Chain Settings

### Parent Finality
| Parameter | Value | Default | Impact |
|-----------|-------|---------|--------|
| `vote_interval` | **1 block** | 1 | Vote on every block |
| `vote_timeout` | **30s** | 60s | Faster timeout on stalled voting |
| `chain_head_delay` | **5 blocks** | 10 | Process parent blocks sooner |
| `proposal_delay` | **5 blocks** | 10 | Propose parent finality faster |
| `polling_interval` | **5s** | 10s | Check parent chain 2x more often |

**Expected Impact:**
- **Before:** Parent finality every ~15-25 blocks (~10-20 seconds)
- **After:** Parent finality every ~8-15 blocks (~5-10 seconds)
- **Cross-msg processing:** 2x faster top-down message delivery

### Retry Behavior
| Parameter | Value | Default | Impact |
|-----------|-------|---------|--------|
| `exponential_back_off` | **3** | 5 | Faster retries (3s, 9s, 27s) |
| `exponential_retry_limit` | **3** | 5 | Give up faster if parent unreachable |
| `parent_http_timeout` | **30s** | 60s | Faster RPC timeout detection |

---

## 📊 Expected Performance

### Block Production
| Metric | Current (100ms + old deltas) | With Advanced Tuning | Improvement |
|--------|------------------------------|----------------------|-------------|
| Average Block Time | 0.65s | **0.35-0.50s** | **35-50% faster** |
| Blocks/Second | ~1.5 | **2-3** | **2x** |
| Blocks/Minute | ~92 | **120-180** | **30-95% more** |

### Cross-Chain
| Metric | Current | Optimized | Improvement |
|--------|---------|-----------|-------------|
| Parent Finality Frequency | Every ~20 blocks | Every ~10 blocks | **2x faster** |
| Cross-msg Latency | ~15-25 seconds | ~8-12 seconds | **40-60% faster** |

---

## 🚀 Applying Advanced Tuning

### Option 1: On Next `init` (Recommended)
All these settings are now in your config and will be applied on next `./ipc-manager init`:

```bash
cd /Users/philip/github/ipc/scripts/ipc-subnet-manager
./ipc-manager init
```

### Option 2: Apply to Existing Nodes (Manual)
If you want to apply **RIGHT NOW** without re-initializing:

```bash
# Apply consensus timeout changes
cd /Users/philip/github/ipc/scripts/ipc-subnet-manager
./apply-advanced-tuning.sh
```

This will:
1. Update all CometBFT `config.toml` files
2. Update all Fendermint `default.toml` files
3. Restart nodes to apply changes

---

## ⚠️ Risks & Trade-offs

### Aggressive Consensus Timeouts
**Risk:** Less tolerant of network hiccups
- If validator-to-validator latency spikes >200ms, consensus could fail
- Failed rounds will recover (with timeout deltas), but could cause brief stalls

**Mitigation:**
- Your validators have <1ms latency ✅
- Timeout deltas will increase timeouts if needed ✅
- Monitor with: `./ipc-manager watch-blocks`

### Faster Parent Finality Polling
**Risk:** More RPC load on parent chain
- Polling every 5s instead of 10s = 2x more requests

**Mitigation:**
- Calibration RPC can handle it ✅
- Uses exponential backoff on errors ✅

### Reduced Retry Limits
**Risk:** Give up faster if parent chain issues
- Only 3 retries instead of 5

**Mitigation:**
- Faster timeout means issues detected sooner ✅
- Can manually trigger retry if needed ✅

---

## 🔍 Monitoring

After applying, monitor performance:

```bash
# Watch block production
./ipc-manager watch-blocks

# Watch parent finality
./ipc-manager watch-finality

# Full health check
./ipc-manager info
```

### What to Look For

✅ **Good Signs:**
- Block time consistently 0.3-0.5s
- No "stalled" status in watch-blocks
- Parent finality advancing smoothly
- No timeout errors in logs

⚠️ **Warning Signs:**
- Frequent round failures (check logs for "entering new round")
- Parent finality stalling
- Block production pauses >2 seconds

---

## 🎯 Recommended Next Steps

1. **Apply the tuning** (Option 1 or 2 above)
2. **Monitor for 5-10 minutes** with `watch-blocks`
3. **Check parent finality** with `watch-finality`
4. **Run full health check** with `info`

If you see issues:
- Increase timeout_propose back to 1s
- Increase timeout_prevote/precommit back to 500ms
- Increase polling_interval back to 10s

---

## 🏆 Ultimate Performance Limits

With your <1ms inter-validator latency, the theoretical limits are:

| Metric | Current Config | Theoretical Max |
|--------|---------------|-----------------|
| Block Time | 0.35-0.50s | ~0.15-0.25s |
| Blocks/Second | 2-3 | 4-6 |

To reach theoretical max, you'd need:
- `timeout_commit: "50ms"`
- `timeout_propose: "200ms"`
- `timeout_prevote: "100ms"`
- `timeout_precommit: "100ms"`

**But this is extremely aggressive and not recommended for production!**

---

## 📚 References

- [CometBFT Configuration](https://docs.cometbft.com/v0.37/core/configuration)
- [Consensus Parameters](https://docs.cometbft.com/v0.37/core/consensus)
- [IPC Documentation](https://docs.ipc.space/)

