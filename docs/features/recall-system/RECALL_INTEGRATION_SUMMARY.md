# Recall Storage Integration - High-Level Summary

## Overview
The recall storage implementation adds **66,000 lines** across **249 files** to enable decentralized blob storage with P2P transfer via Iroh.

## What Was Added (Self-Contained)

### New Standalone Components (~80% of changes)
- **`recall/` directory** (7 crates, 5,000 lines) - Core runtime: custom FVM kernel, executor, syscalls
- **`fendermint/actors/`** (6 new actors, 15,000 lines) - blobs, blob_reader, recall_config, bucket, timehub, adm
- **`recall-contracts/`** (18,000 lines) - Auto-generated Solidity bindings
- **`ipc-decentralized-storage/`** (2,300 lines) - Standalone gateway & node binaries
- **`fendermint/vm/iroh_resolver/`** (900 lines) - Blob resolution module
- **`fendermint/app/cmd/objects.rs`** (1,455 lines) - HTTP API for blob upload/download

**These are entirely new and could be made optional.**

## What Was Modified (Integration Points)

### Critical Integrations (~20% of changes, higher maintenance burden)

1. **Message Type System** (`fendermint/vm/message/src/ipc.rs`, ~100 lines)
   - Added 2 new `IpcMessage` enum variants: `ReadRequestPending`, `ReadRequestClosed`
   - **Risk:** Affects message serialization across the network

2. **Genesis Initialization** (`fendermint/vm/interpreter/src/genesis.rs`, ~150 lines)
   - Initializes 4 new actors at chain genesis (ADM, blobs, blob_reader, recall_config)
   - Reserves actor IDs: 90, 99, 100, 101
   - **Risk:** Changes chain genesis format

3. **Message Handlers** (`fendermint/vm/interpreter/src/fvm/interpreter.rs`, ~100 lines)
   - Added handlers for new message types
   - Calls into recall helper functions
   - **Risk:** Core execution path modified

4. **Vote Tally** (`fendermint/vm/topdown/src/voting.rs`, ~200 lines)
   - Added blob voting for BFT consensus
   - New methods: `add_blob_vote()`, `find_blob_quorum()`
   - **Risk:** Consensus mechanism extended

5. **IPLD Resolver** (`ipld/resolver/`, ~400 lines)
   - Integrated Iroh P2P blob downloads
   - Made Service initialization async
   - **Risk:** Core infrastructure modified

## Invasiveness Assessment

### Low Invasiveness (Easy to Maintain/Remove)
- ✅ All new directories (`recall/`, `ipc-decentralized-storage/`, `recall-contracts/`)
- ✅ New actors (self-contained)
- ✅ HTTP Objects API (separate command)

### Medium Invasiveness (Requires Feature Flags)
- ⚠️ Genesis initialization (one function, can be gated)
- ⚠️ Message handlers (match arms, can be gated)
- ⚠️ IPLD resolver extensions (trait-based, can be optional)

### High Invasiveness (Fork Maintenance Burden)
- ❌ **None** - No deeply embedded changes that can't be made optional

## Fork Maintenance Implications

**Good News:** The integration is surprisingly clean and modular. ~85% is self-contained.

**Maintenance Burden:** The 15% that touches core code is in well-defined locations:
- 1 enum with 2 variants
- 1 genesis function
- 2 message handler match arms
- 1 vote tally extension

**Recommendation:** This can be made into an **optional feature** with 2-3 weeks of work, eliminating fork maintenance burden. See `RECALL_MODULARIZATION_IMPLEMENTATION_GUIDE.md` for details.
