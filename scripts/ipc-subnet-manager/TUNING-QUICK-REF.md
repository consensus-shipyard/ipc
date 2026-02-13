# Performance Tuning Quick Reference

## 🎯 Current Status

| Setting | Original | Current | With Advanced Tuning |
|---------|----------|---------|----------------------|
| **Block Time** | 2.5s | 0.65s | 0.35-0.50s |
| **Blocks/Min** | 24 | 90 | 120-180 |
| **Parent Finality** | Every ~25 blocks | Every ~20 blocks | Every ~10 blocks |

## ⚡ Quick Actions

### Apply Advanced Tuning NOW
```bash
cd /Users/philip/github/ipc/scripts/ipc-subnet-manager
./apply-advanced-tuning.sh
```

### Monitor Performance
```bash
# Watch blocks (look for 0.3-0.5s average)
./ipc-manager watch-blocks

# Watch parent finality (look for faster progression)
./ipc-manager watch-finality

# Full health check
./ipc-manager info
```

### Revert If Needed
```bash
# SSH to each validator and restore backups:
ssh philip@<validator-ip>
sudo su - ipc
cd ~/.ipc-node/cometbft/config
cp config.toml.before-advanced-tuning config.toml
cd ~/.ipc-node/fendermint/config
cp default.toml.before-advanced-tuning default.toml

# Then restart
./ipc-manager restart --yes
```

---

## 🔧 Manual Tuning Options

### Speed Presets

#### Conservative (Stable)
```yaml
timeout_commit: "300ms"
timeout_propose: "1s"
timeout_prevote: "500ms"
timeout_precommit: "500ms"
```
**Result:** 0.6-0.8s blocks, ~75-100/min

#### Aggressive (Current Config)
```yaml
timeout_commit: "100ms"
timeout_propose: "500ms"
timeout_prevote: "200ms"
timeout_precommit: "200ms"
```
**Result:** 0.35-0.50s blocks, ~120-180/min

#### Extreme (Risk of instability)
```yaml
timeout_commit: "50ms"
timeout_propose: "200ms"
timeout_prevote: "100ms"
timeout_precommit: "100ms"
```
**Result:** 0.15-0.30s blocks, ~200-400/min
**Warning:** May cause consensus failures!

---

## 📊 What Each Parameter Does

### Block Production Speed
| Parameter | What it controls | Recommended Value |
|-----------|-----------------|-------------------|
| `timeout_commit` | ⏱️ Time between blocks | 100ms-300ms |
| `timeout_propose` | 📤 Wait for proposal | 500ms-1s |
| `timeout_prevote` | 🗳️ Wait for prevotes | 200ms-500ms |
| `timeout_precommit` | ✅ Wait for precommits | 200ms-500ms |

### Cross-Chain Speed
| Parameter | What it controls | Recommended Value |
|-----------|-----------------|-------------------|
| `polling_interval` | 🔄 Check parent chain | 5-10s |
| `chain_head_delay` | ⏳ Process parent blocks | 5-10 blocks |
| `vote_timeout` | ⏰ Vote timeout | 30-60s |

### Network Performance
| Parameter | What it controls | Recommended Value |
|-----------|-----------------|-------------------|
| `send_rate` | 📤 Upload bandwidth | 10-20 MB/s |
| `recv_rate` | 📥 Download bandwidth | 10-20 MB/s |
| `max_packet_msg_payload_size` | 📦 Packet size | 10240 bytes |

---

## 🎮 Tuning Strategy

### Step 1: Test Current (100ms + old settings)
```bash
./ipc-manager watch-blocks
# Look for: ~0.65s average block time
```

### Step 2: Apply Advanced Tuning
```bash
./apply-advanced-tuning.sh
```

### Step 3: Monitor for 10 minutes
```bash
# Watch for issues
./ipc-manager watch-blocks
# Target: 0.35-0.50s average

# Check parent finality
./ipc-manager watch-finality
# Target: Advances every ~10 blocks
```

### Step 4: Adjust if needed

**If blocks are too slow (>0.6s):**
- Reduce timeout_commit to 50ms
- Reduce timeout_propose to 300ms

**If consensus fails frequently:**
- Increase timeout_prevote to 500ms
- Increase timeout_precommit to 500ms
- Increase timeout_propose to 1s

**If parent finality stalls:**
- Increase polling_interval to 10s
- Increase vote_timeout to 60s
- Check parent RPC is accessible

---

## 🚦 Performance Indicators

### Healthy Performance
✅ Block time: 0.3-0.6s
✅ No "stalled" warnings
✅ Parent finality advancing smoothly
✅ No timeout errors in logs

### Warning Signs
⚠️ Block time: >1s
⚠️ Frequent "stalled" status
⚠️ Parent finality not advancing
⚠️ "timeout" or "failed round" in logs

### Critical Issues
🔴 Block production stopped
🔴 Consensus failures
🔴 Parent finality stuck
🔴 Validators disconnecting

---

## 📈 Expected Results Timeline

### Immediately (0-2 minutes)
- Nodes restart
- Block production resumes
- May see initial instability

### Short term (2-10 minutes)
- Block times stabilize at new speed
- Parent finality catches up
- Network synchronizes

### Long term (10+ minutes)
- Consistent performance
- Faster cross-chain messaging
- Lower latency for users

---

## 🛟 Troubleshooting

### Blocks too slow
```bash
# Check if timeouts are being applied
ssh philip@34.73.187.192 "sudo su - ipc -c 'grep timeout_commit ~/.ipc-node/cometbft/config/config.toml'"
```

### Consensus failures
```bash
# Check logs for "entering new round"
ssh philip@34.73.187.192 "sudo su - ipc -c 'grep \"entering new round\" ~/.ipc-node/logs/*.log | tail -20'"

# If frequent, increase timeouts
```

### Parent finality stuck
```bash
# Check if polling parent
ssh philip@34.73.187.192 "sudo su - ipc -c 'grep -i \"parent finality\" ~/.ipc-node/logs/*.log | tail -20'"

# Check parent RPC is accessible
curl -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' \
  https://api.calibration.node.glif.io/rpc/v1
```

---

## 📚 Additional Resources

- **Full Guide:** [ADVANCED-TUNING-GUIDE.md](./ADVANCED-TUNING-GUIDE.md)
- **CometBFT Docs:** https://docs.cometbft.com/v0.37/core/configuration
- **IPC Docs:** https://docs.ipc.space/

---

## 🎯 Recommended Path

1. ✅ **You're here:** Config updated with advanced settings
2. ⏭️ **Next:** Run `./apply-advanced-tuning.sh`
3. 📊 **Then:** Monitor with `watch-blocks` for 10 minutes
4. 🎉 **Finally:** Enjoy 3-5x faster blockchain!

