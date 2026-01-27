# Consensus Crash Issue - Root Cause & Fix

## Problem Summary

All 3 validators crashed with **CONSENSUS FAILURE** due to bottom-up checkpointing errors.

---

## Root Cause Analysis

### Timeline of Events

1. **Fendermint tried to fetch incomplete checkpoints**
   ```
   ERROR: failed to execute ABCI request: other error: failed to fetch incomplete checkpoints
   ```

2. **This caused an ABCI error response to CometBFT**

3. **CometBFT couldn't handle the error** and crashed:
   ```
   CONSENSUS FAILURE!!! err="failed to apply block; error read message: EOF"
   ```

4. **CometBFT shut down completely**, leaving only port 26658 (metrics) listening

5. **Fendermint services couldn't connect** to CometBFT:
   - ETH API: `failed to connect to Tendermint WebSocket`
   - Topdown sync: `failed to get Tendermint status`

---

## Why This Happened

The bottom-up checkpointing feature has a critical bug where:
- It tries to fetch incomplete checkpoints
- When this fails, it returns an error to CometBFT via ABCI
- CometBFT's error handling crashes with "EOF"
- This brings down the entire consensus

**This is a critical bug in IPC** - bottom-up checkpointing should not crash consensus.

---

## The Fix Applied

### Step 1: Restart Nodes
```bash
./ipc-manager restart --yes
```

### Step 2: Disable Bottom-Up Checkpointing

Added to `~/.ipc-node/fendermint/config/default.toml` on all 3 validators:

```toml
# Disable bottom-up checkpointing
[ipc.bottomup]
enabled = false
```

### Step 3: Restart Again
```bash
./ipc-manager restart --yes
```

---

## Verification

After the fix:
- ✅ All 3 validators running
- ✅ CometBFT producing blocks (height 23,440+)
- ✅ Ports 26656 (P2P) and 26657 (RPC) listening
- ✅ No "CONSENSUS FAILURE" errors
- ✅ No "failed to fetch incomplete checkpoints" errors

---

## Remaining Issue

**ETH API WebSocket Connection Problem**

Even after fixing the consensus crash, the ETH API still cannot connect to CometBFT's WebSocket:

```
WARN: failed to connect to Tendermint WebSocket; retrying in 5s...
  error="failed to create WS client to: ws://127.0.0.1:26657/websocket"
```

**Status:**
- CometBFT RPC (port 26657) is listening ✓
- CometBFT is producing blocks ✓
- ETH RPC (port 8545) is listening ✓
- But WebSocket connections are failing ✗

**Possible Causes:**
1. `max_open_connections = 3` in CometBFT RPC config might be too low
2. WebSocket endpoint might not be properly configured
3. Connection limit might be exhausted
4. There might be a CometBFT configuration issue

**Impact:**
- Consensus is working
- Blocks are being produced
- But ETH JSON-RPC queries might not work properly
- This affects the `info` command and any Ethereum tooling

---

## Upstream Issues to Report

### 1. Bottom-Up Checkpointing Crashes Consensus (CRITICAL)

**File:** `fendermint/vm/interpreter/src/fvm/bottomup.rs` (likely)
**Issue:** When fetching incomplete checkpoints fails, it causes an ABCI error that crashes CometBFT with "EOF"
**Expected:** Error should be handled gracefully without bringing down consensus
**Severity:** Critical - causes total network outage

### 2. WebSocket Connection Issues After Restart

**File:** Possibly CometBFT configuration or `fendermint/eth/api/src/client.rs`
**Issue:** ETH API cannot connect to CometBFT WebSocket even when CometBFT is running
**Impact:** ETH JSON-RPC doesn't work properly
**Severity:** High - breaks Ethereum tooling integration

---

## For Federated Subnets

**Recommendation:** Disable bottom-up checkpointing by default in federated subnets

Bottom-up checkpointing is primarily needed for:
- Moving assets from child subnet back to parent
- Cross-chain state proofs
- Decentralized subnet validation

Federated subnets typically don't need these features, so the risk/benefit ratio favors disabling it.

---

## Commands Used

### Check Node Status
```bash
ssh philip@34.73.187.192 "ps aux | grep ipc-cli"
ssh philip@34.73.187.192 "ss -tuln | grep -E '26657|26656|8545'"
```

### Check Logs for Errors
```bash
ssh philip@34.73.187.192 "sudo su - ipc -c 'tail -50 ~/.ipc-node/logs/2025-10-19.consensus.log'"
ssh philip@34.73.187.192 "sudo su - ipc -c 'grep \"13:32:5[7-8]\" ~/.ipc-node/logs/2025-10-19.app.log'"
```

### Check Block Height
```bash
ssh philip@34.73.187.192 "curl -s http://localhost:26657/status | jq -r '.result.sync_info.latest_block_height'"
```

### Disable Bottom-Up Checkpointing
```bash
ssh philip@34.73.187.192 "sudo su - ipc -c 'echo -e \"\n# Disable bottom-up checkpointing\n[ipc.bottomup]\nenabled = false\" >> ~/.ipc-node/fendermint/config/default.toml'"
```

---

## Next Steps

1. **Monitor for stability** - ensure no more consensus crashes occur
2. **Debug WebSocket issue** - figure out why ETH API can't connect
3. **Report upstream bugs** - create issues for IPC team
4. **Update subnet manager** - add option to disable bottom-up by default for federated subnets
5. **Add health check** - detect when WebSocket connections are failing

---

## Lessons Learned

1. **Bottom-up checkpointing is not production-ready** for federated subnets
2. **Error handling in ABCI layer needs improvement** - should never crash consensus
3. **WebSocket configuration is fragile** - needs better defaults and diagnostics
4. **The `info` command needs better timeout handling** - shouldn't hang indefinitely

---

## Status: PARTIALLY RESOLVED

✅ **Consensus crash fixed** - nodes producing blocks
⚠️ **WebSocket issue remains** - ETH API not fully functional
📝 **Upstream bugs identified** - need to be reported to IPC team

