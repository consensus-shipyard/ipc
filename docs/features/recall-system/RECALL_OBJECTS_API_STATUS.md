# Recall Objects HTTP API - Port Status

## ✅ What's Been Ported

### Core Infrastructure
- ✅ `fendermint/app/src/cmd/objects.rs` - Full 1264-line HTTP API (blob upload/download)
- ✅ `fendermint/app/options/src/objects.rs` - CLI options for objects command
- ✅ `fendermint/app/settings/src/objects.rs` - Configuration settings
- ✅ `fendermint/vm/iroh_resolver/` - Iroh blob resolution module (3 files)
- ✅ Command registration in `fendermint/app/src/cmd/mod.rs`
- ✅ All workspace dependencies added (warp, uuid, mime_guess, urlencoding)

### HTTP API Endpoints

**From `ipc-recall` branch:**
```rust
POST   /v1/objects              - Upload blob with chunking & entanglement
GET    /v1/objects/{hash}/{path} - Download blob
HEAD   /v1/objects/{hash}/{path} - Get blob metadata
GET    /v1/node                  - Get node address
GET    /health                   - Health check
```

### Features Included
- ✅ File chunking (1024-byte chunks)
- ✅ Erasure coding (α=3, s=5)
- ✅ Iroh P2P integration
- ✅ Entanglement for fault tolerance
- ✅ Multipart form upload
- ✅ Range request support
- ✅ Prometheus metrics
- ✅ MIME type detection

## ⚠️ Compilation Blockers

### 1. API Incompatibilities in `iroh_resolver`

**File:** `fendermint/vm/iroh_resolver/src/iroh.rs`

**Errors:**
```rust
// vote_tally API changed
vote_tally.add_blob_vote(...)  // Method signature differs from main

// Client API doesn't exist
client.resolve_iroh(...)        // Method doesn't exist in main branch
client.close_read_request(...)  // Method doesn't exist in main branch
```

**Root Cause:** The `ipc-recall` branch has evolved `vote_tally` and IPLD resolver APIs that differ from `main`.

### 2. Bucket Actor Dependencies

**File:** `fendermint/app/src/cmd/objects.rs`
```rust
use fendermint_actor_bucket::{GetParams, Object}; // Commented out
```

**Issue:** Bucket actor depends on `machine` actor which depends on `fil_actor_adm` (not available in main).

## 🔧 Solutions Required

### Option 1: API Compatibility Layer (Recommended)

Create adapter functions to bridge API differences:

```rust
// In fendermint/vm/iroh_resolver/src/compat.rs
pub fn add_blob_vote_compat(
    vote_tally: &VoteTally,
    validator: Vec<u8>,
    blob: Vec<u8>,
    resolved: bool
) -> Result<bool, Error> {
    // Map to main branch's API
    vote_tally.add_vote(/* adapted params */)
}
```

### Option 2: Stub Implementation

Comment out iroh_resolver usage temporarily:

```rust
// In objects.rs
let iroh_resolver_node = connect_rpc(iroh_resolver_rpc_addr).await?;
// TODO: Re-enable once APIs are aligned
// let result = resolve_with_iroh(&client, &iroh_resolver_node, params).await?;
```

### Option 3: Port Missing APIs from `ipc-recall`

Update `fendermint/vm/topdown/src/voting.rs` to add:
- `add_blob_vote()` method
- Blob-specific vote tally logic

Update `ipld/resolver` to add:
- `resolve_iroh()` method
- `close_read_request()` method

## 📋 Remaining Work Checklist

### High Priority
- [ ] Fix `vote_tally.add_blob_vote()` API incompatibility
- [ ] Fix `client.resolve_iroh()` missing method
- [ ] Fix `client.close_read_request()` missing method
- [ ] Test objects HTTP server startup
- [ ] Test blob upload endpoint
- [ ] Test blob download endpoint

### Medium Priority
- [ ] Port/stub bucket actor support
- [ ] Add configuration defaults
- [ ] Create end-to-end test
- [ ] Update documentation

### Low Priority
- [ ] Port ADM actor for bucket support
- [ ] Optimize chunking performance
- [ ] Add more comprehensive error handling

## 🚀 Quick Start (Once Fixed)

```bash
# Build with objects support
cd /Users/philip/github/ipc
cargo build --release -p fendermint_app

# Start objects HTTP API
./target/release/fendermint objects run \
  --tendermint-url http://localhost:26657 \
  --iroh-path ~/.iroh \
  --iroh-resolver-rpc-addr 127.0.0.1:4402 \
  --iroh-v4-addr 0.0.0.0:11204 \
  --iroh-v6-addr [::]:11205

# Upload a file
curl -X POST http://localhost:8080/v1/objects \
  -F "file=@test.txt"

# Download a file
curl http://localhost:8080/v1/objects/{hash}/test.txt
```

## 📁 Files Modified/Added

```
Modified:
- Cargo.toml (added warp, uuid, mime_guess, urlencoding)
- fendermint/app/Cargo.toml (added objects dependencies)
- fendermint/app/options/src/lib.rs (registered objects module)
- fendermint/app/settings/src/lib.rs (registered objects settings)
- fendermint/app/src/cmd/mod.rs (registered objects command)

Added:
- fendermint/app/src/cmd/objects.rs (1264 lines - full HTTP API)
- fendermint/app/options/src/objects.rs (47 lines)
- fendermint/app/settings/src/objects.rs (18 lines)
- fendermint/vm/iroh_resolver/Cargo.toml
- fendermint/vm/iroh_resolver/src/lib.rs
- fendermint/vm/iroh_resolver/src/iroh.rs
```

## 💡 Recommendation

**For now:** Commit what we have as "WIP: port objects HTTP API from ipc-recall"

**Next steps:**
1. Align vote_tally APIs between branches
2. Port missing IPLD resolver methods
3. Test end-to-end blob upload/download
4. Full integration testing

This preserves all the work done while clearly documenting what needs to be finished.

---

**Status:** ⏳ 90% complete - API compatibility work needed
**Effort:** ~2-4 hours to finish API compatibility layer
**Value:** Complete blob upload/download functionality for Recall storage

