# Recall Storage Local Testing Guide

## Current Status ✅

**Migration Complete** - All Recall components are successfully integrated and compiling!

### What's Working
- ✅ All 7 Recall core modules compiling
- ✅ All 3 Recall actors compiling
- ✅ Single-node testnode running
- ✅ Recall actors added to custom actor bundle
- ✅ Genesis setup fixed for IPC main branch

###What's Needed for Full Testing
- Rebuild Docker image with new actor bundle, OR
- Port blob upload/download CLI commands from `ipc-recall` branch

---

## Quick Test (Current Setup)

We successfully started a local single-node testnet:

```bash
# Testnode is already running!
# Access points:
Eth API:         http://0.0.0.0:8545
Fendermint API:  http://localhost:26658
CometBFT API:    http://0.0.0.0:26657

# Chain ID: 3522868364964899
# Account: t1qdcs2rupwbs376pmfzjb4crh6i5h6wgczd55adi (1000 FIL)
```

### Current Limitations

The Recall actors are **compiled into the bundle** but not yet **deployed** because:
1. The Docker container is using an older image (from Aug 28)
2. New actor bundle needs to be included in Docker image

---

## Option 1: Rebuild Docker Image (Recommended for Full Testing)

This will include the new Recall actors in genesis:

```bash
# Build new Docker image with Recall actors
cd /Users/philip/github/ipc
make -C fendermint docker-build

# Stop old testnode
FM_PULL_SKIP=true cargo make --makefile ./infra/fendermint/Makefile.toml testnode-down

# Start testnode with new image
FM_PULL_SKIP=true cargo make --makefile ./infra/fendermint/Makefile.toml testnode
```

### Verify Recall Actors in Genesis

Once the new testnode is running:

```bash
# Check if Recall actors are deployed
curl http://localhost:26657/abci_query?path=%22/actor/70%22 | jq

# Actor ID 70 should be the recall_config actor
```

---

## Option 2: Port Blob CLI Commands (For Testing Without Docker)

The `ipc-recall` branch has a full HTTP API for blob upload/download in `fendermint/app/src/cmd/objects.rs`. To test locally:

### 1. Port the Objects Command

Copy from `ipc-recall` branch:
- `fendermint/app/src/cmd/objects.rs`
- `fendermint/app/options/src/objects.rs`
- `fendermint/app/settings/src/objects.rs`

### 2. Add to Command Enum

In `fendermint/app/src/cmd/mod.rs`:
```rust
pub mod objects;  // Add this

// In exec function:
Commands::Objects(args) => {
    let settings = load_settings(opts)?.objects;
    args.exec(settings).await
}
```

### 3. Test Blob Upload

```bash
# Start the objects HTTP server
./target/release/fendermint objects run \
  --tendermint-url http://localhost:26657 \
  --iroh-path ~/.iroh \
  --iroh-resolver-rpc-addr 127.0.0.1:4402 \
  --iroh-v4-addr 0.0.0.0:11204 \
  --iroh-v6-addr [::]:11205

# Upload a blob
curl -X POST http://localhost:8080/v1/objects \
  -F "file=@/path/to/test/file.txt"

# Download a blob
curl http://localhost:8080/v1/objects/{blob_hash}/{path}
```

---

## Option 3: Direct RPC Testing (Advanced)

Call Recall actors directly via fendermint RPC:

```bash
# Call recall_config actor (ID 70)
./target/release/fendermint rpc --api http://localhost:26658 \
  message --to-addr f070 \
  --method-num 2 \
  --params '{"config": {"blob_capacity": 1000000}}' \
  --value 0 \
  --sequence 0

# Call blobs actor (once deployed)
# Add blob: method 3
# Get blob: method 4
```

---

## Architecture Overview

### Recall Storage Components

**Core Modules:**
1. `recall/kernel` - Custom FVM kernel with blob syscalls
2. `recall/syscalls` - Blob operation syscalls
3. `recall/iroh_manager` - Iroh P2P node management
4. `recall/executor` - Custom executor with gas allowances
5. `recall/actor_sdk` - Actor SDK with EVM support
6. `recall/ipld` - Custom IPLD data structures

**Actors (in custom bundle):**
1. `fendermint_actor_blobs` (ID TBD) - Main blob storage
2. `fendermint_actor_blob_reader` (ID TBD) - Read-only access
3. `fendermint_actor_recall_config` (ID 70) - Network config

### How It Works

1. **Client Upload:**
   - File chunked into 1024-byte pieces
   - Erasure coded with α=3, s=5 for fault tolerance
   - Uploaded to local Iroh node
   - Metadata registered with Blobs Actor on-chain

2. **Validator Resolution:**
   - Validators monitor "added" queue
   - Download chunks from source Iroh node
   - Verify and store locally (full replication)
   - Vote on resolution success/failure

3. **Vote Tally:**
   - Weighted BFT voting (by validator stake)
   - Quorum: 2/3 + 1 of total voting power
   - Finalization updates blob status to "resolved"

---

## Testing Checklist

### Basic Testing
- [ ] Rebuild Docker image with Recall actors
- [ ] Verify actors deployed in genesis
- [ ] Check actor IDs are correct
- [ ] Query recall_config actor

### Blob Testing
- [ ] Start Iroh node
- [ ] Upload small test file (< 1MB)
- [ ] Verify blob registered on-chain
- [ ] Check blob status transitions
- [ ] Download blob and verify content

### Integration Testing
- [ ] Multi-validator setup
- [ ] Vote tally mechanism
- [ ] Blob finalization
- [ ] Credit/debit system
- [ ] Storage quota enforcement

---

## Troubleshooting

### Issue: Actors Not in Genesis
**Cause:** Docker image using old bundle
**Fix:** Rebuild Docker image (Option 1 above)

### Issue: Iroh Connection Failed
**Cause:** UDP ports blocked or relay unavailable
**Fix:** Check firewall, verify ports 11204/11205 open

### Issue: Blob Upload Timeout
**Cause:** Validator not resolving blobs
**Fix:** Check validator Iroh node running, check logs

### Issue: Vote Tally Not Reaching Quorum
**Cause:** Not enough validators voting
**Fix:** Check validator connectivity, Iroh resolution

---

## Next Steps

**For Full Integration:**
1. Port HTTP API commands from `ipc-recall` branch
2. Add Iroh node initialization to fendermint startup
3. Add blob upload/download examples to documentation
4. Create end-to-end test suite
5. Performance testing and optimization

**For Current Testing:**
1. Rebuild Docker image with new actor bundle
2. Start fresh testnode
3. Verify actors deployed
4. Test basic actor queries

---

## Files Modified for Testing

```
fendermint/actors/Cargo.toml              # Added Recall actors to bundle
infra/fendermint/scripts/genesis.toml     # Fixed genesis command
```

## Useful Commands

```bash
# Check node status
curl http://localhost:26657/status | jq

# Check latest block
curl http://localhost:26657/block | jq

# Query actor state
curl "http://localhost:26657/abci_query?path=\"/actor/70\"" | jq

# Stop testnode
FM_PULL_SKIP=true cargo make --makefile ./infra/fendermint/Makefile.toml testnode-down

# Start testnode
FM_PULL_SKIP=true cargo make --makefile ./infra/fendermint/Makefile.toml testnode

# View logs
docker logs -f ipc-node-fendermint
docker logs -f ipc-node-cometbft
```

---

**Status:** Ready for Docker rebuild and full testing! 🚀

**Branch:** `recall-migration`
**Commit:** `5e6ef3b1`
**Date:** November 4, 2024

