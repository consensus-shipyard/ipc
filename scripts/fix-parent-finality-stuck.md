# Parent Finality Stuck - Diagnosis & Solutions

## Problem

Your subnet's parent finality is stuck at epoch **3135524**, which is **~15 days old**.

The Filecoin Calibration RPC (`api.calibration.node.glif.io`) only allows lookbacks of **16h40m**, so every query to sync parent finality fails with:

```
ERROR: bad tipset height: lookbacks of more than 16h40m0s are disallowed
```

This means:
- ❌ Parent finality cannot advance
- ❌ Cross-chain fund transactions cannot be processed
- ❌ Your subnet is effectively isolated from L1

## Why This Happened

The subnet was likely down or had issues for an extended period (~15 days), causing it to fall too far behind. Now it can't catch up because the RPC won't serve that old data.

## Solutions

### Option 1: Use Different RPC Endpoint (Recommended)

Find an RPC endpoint that supports longer lookback:

1. **Run your own Lotus node** (best option):
   ```bash
   # On a server with ~2TB storage
   lotus daemon --import-snapshot
   ```
   Then update your config to point to your Lotus node.

2. **Use a different public RPC** that supports archive queries
   - Check IPC community for recommended archive nodes
   - Some providers offer archive node access

3. **Update config**:
   Edit `ipc-subnet-config.yml`:
   ```yaml
   subnet:
     parent_rpc: "http://your-archive-node:1234/rpc/v1"
   ```

### Option 2: Reset Parent Finality (DANGEROUS)

⚠️  **WARNING**: This will skip 15 days of history. Any pending cross-chain messages in that gap will be LOST!

Only do this if:
- You're certain there are NO important cross-chain messages in the gap
- This is a test subnet
- You accept losing 15 days of cross-chain message history

The process would require modifying subnet state, which is complex and risky.

### Option 3: Initialize New Subnet (Clean Slate)

If this is a test subnet and you don't mind starting over:

1. Deploy a new subnet from scratch
2. Don't let it fall behind this time
3. Monitor parent finality regularly

## Recommended Action for YOU

Since you just want to fund your faucet wallet:

1. **For now**: Fund your faucet wallet **directly on the subnet** using the IPC CLI:
   ```bash
   # Use ipc-cli to send tFIL directly on the subnet
   # (if you have a funded wallet on the subnet)
   ```

2. **For the long term**: Set up your own Lotus node or find an archive RPC endpoint

## Immediate Workaround

To test your faucet **right now** without waiting for parent finality:

1. Use the IPC CLI to send tFIL directly on the subnet (not cross-chain)
2. Or use your validator's wallet to send funds on the subnet
3. This bypasses the need for parent finality

Let me know which approach you want to take!

