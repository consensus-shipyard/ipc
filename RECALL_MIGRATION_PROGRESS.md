# Recall Migration Progress

## ✅ Completed Work

### 1. API Compatibility Fixes (COMPLETED)

**Blob Voting Support**
- ✅ Replaced `fendermint/vm/topdown/src/voting.rs` with full blob-aware version from `ipc-recall`
- ✅ Added `Blob` type alias to `fendermint/vm/topdown/src/lib.rs`
- ✅ Implemented `add_blob_vote()` method for blob resolution voting
- ✅ Added `find_blob_quorum()` for blob consensus detection

**Iroh Resolver Integration**
- ✅ Replaced `ipld/resolver` (lib.rs, client.rs, service.rs) with Iroh-aware versions
- ✅ Added `resolve_iroh()` trait method to `ResolverIroh` trait
- ✅ Added `close_read_request()` trait method to `ResolverIrohReadRequest` trait
- ✅ Added `bytes`, `iroh`, `iroh-blobs`, and `iroh_manager` dependencies to `ipld/resolver/Cargo.toml`
- ✅ Added `IrohClient` error variant to `ConfigError` enum
- ✅ Made `Service::new()` async to support Iroh initialization
- ✅ Added `IrohConfig` struct with v4/v6 addresses, path, and RPC address
- ✅ Updated `Config` struct to include `iroh: IrohConfig` field

**Iroh Resolver VM Module**
- ✅ Created `fendermint/vm/iroh_resolver/` module
- ✅ Ported `iroh.rs` - core Iroh blob resolution logic
- ✅ Ported `observe.rs` - observability/metrics for blob operations
- ✅ Ported `pool.rs` - connection pooling for Iroh clients
- ✅ Added module to workspace members in root `Cargo.toml`
- ✅ Added dependency to `fendermint_app/Cargo.toml`

**Objects HTTP API**
- ✅ Ported `fendermint/app/src/cmd/objects.rs` - HTTP API for blob upload/download
- ✅ Ported `fendermint/app/options/src/objects.rs` - CLI options
- ✅ Ported `fendermint/app/settings/src/objects.rs` - settings structure
- ✅ Registered `Objects` command in CLI (`fendermint/app/options/src/lib.rs`)
- ✅ Integrated objects settings (`fendermint/app/settings/src/lib.rs`)
- ✅ Added command execution logic (`fendermint/app/src/cmd/mod.rs`)
- ✅ Added all required dependencies: `warp`, `uuid`, `mime_guess`, `urlencoding`, `entangler`, `entangler_storage`, `iroh_manager`, `iroh`, `iroh-blobs`, `thiserror`, `futures-util`
- ✅ Created stub types for ADM bucket actor (`GetParams`, `HashBytes`, `ObjectMetadata`, `Object`)
- ✅ Fixed HashBytes conversion to `[u8; 32]` for Iroh Hash compatibility
- ✅ Stubbed `os_get()` function (requires ADM bucket actor)

**Settings Updates**
- ✅ Added `IrohResolverSettings` struct to `fendermint/app/settings/src/resolver.rs`
- ✅ Added `iroh_resolver_config` field to `ResolverSettings`
- ✅ Added default values for Iroh data dir and RPC address
- ✅ Updated `to_resolver_config()` to create `IrohConfig` from settings
- ✅ Made `make_resolver_service()` async and added `.await` call

## 📋 Remaining Work

### 2. Interpreter Blob Handling (TODO)

**Goal**: Integrate blob resolution into the FVM interpreter's message execution path.

**Files to Port/Modify**:
- `fendermint/vm/interpreter/src/fvm/state/iface.rs` - Add blob-specific state management
- `fendermint/vm/interpreter/src/fvm/state/exec.rs` - Integrate blob resolution in execution
- `fendermint/vm/interpreter/src/fvm/check.rs` - Add blob validation logic
- `fendermint/vm/interpreter/src/fvm/observe.rs` - Add blob metrics

**Key Changes Needed**:
1. Add blob resolution calls during message execution
2. Integrate with `fendermint_vm_iroh_resolver` for blob downloads
3. Handle blob status updates (Added → Pending → Resolved/Failed)
4. Add blob-specific error handling
5. Add metrics for blob resolution time, success/failure rates

### 3. Blob Vote Tally Chain Integration (TODO)

**Goal**: Process blob votes from validators and update blob status on-chain.

**Files to Port/Modify**:
- `fendermint/vm/interpreter/src/fvm/exec.rs` - Process blob vote messages
- `fendermint/app/src/service/node.rs` - Wire up blob voting loop
- Vote processing logic integration with `VoteTally::add_blob_vote()`

**Key Changes Needed**:
1. Create event loop to monitor blob resolution requests
2. Call `add_blob_vote()` when validators report blob resolution
3. Detect quorum via `find_blob_quorum()`
4. Update on-chain blob status when quorum is reached
5. Emit events for blob status changes

### 4. Chain Blob Processing (TODO)

**Goal**: Process blob-related transactions and maintain blob lifecycle on-chain.

**Files to Port/Modify**:
- `fendermint/vm/interpreter/src/fvm/state/exec.rs` - Add blob transaction handlers
- Blobs actor integration for blob registration, voting, resolution

**Key Changes Needed**:
1. Handle blob registration transactions
2. Process blob subscription requests
3. Track blob status transitions
4. Handle validator vote submissions
5. Update blob metadata on resolution

## 🚧 Known Limitations

### ADM Bucket Actor
- **Status**: Not available in main branch
- **Impact**:
  - `os_get()` function stubbed out
  - Bucket-based blob storage disabled
  - Object metadata limited
- **Workaround**: Created stub types (`GetParams`, `Object`, `ObjectMetadata`, `HashBytes`)
- **Resolution**: Will require porting:
  - `fendermint/actors/bucket`
  - `fendermint/actors/machine`
  - `fendermint/actors/timehub`
  - `fil_actor_adm` dependency

### Recall SOL Facade
- **Status**: Vendored locally and updated to FVM 4.7
- **Location**: `recall/sol_facade/`
- **Changes**: Updated `fvm_shared` and `fvm_ipld_encoding` to workspace versions

## 🔧 Dependencies Added

### Workspace (`Cargo.toml`)
- `bytes = "1.5.0"`
- `warp = "0.3"`
- `uuid = { version = "1.0", features = ["v4"] }`
- `mime_guess = "2.0"`
- `urlencoding = "2.1"`
- `ambassador = "0.3.5"`
- `replace_with = "0.1.7"`
- `data-encoding = "2.3.3"`
- `recall_sol_facade = { path = "recall/sol_facade" }`

### IPLD Resolver (`ipld/resolver/Cargo.toml`)
- `bytes = { workspace = true }`
- `iroh = { workspace = true }`
- `iroh-blobs = { workspace = true }`
- `iroh_manager = { path = "../../recall/iroh_manager" }`

### Fendermint App (`fendermint/app/Cargo.toml`)
- `warp = { workspace = true }`
- `uuid = { workspace = true }`
- `mime_guess = { workspace = true }`
- `urlencoding = { workspace = true }`
- `entangler = { workspace = true }`
- `entangler_storage = { workspace = true }`
- `iroh_manager = { path = "../../recall/iroh_manager" }`
- `iroh = { workspace = true }`
- `iroh-blobs = { workspace = true }`
- `thiserror = { workspace = true }`
- `futures-util = { workspace = true }`
- `fendermint_vm_iroh_resolver = { path = "../vm/iroh_resolver" }`

## 📊 Current Status

- **Core API Compatibility**: ✅ COMPLETE (100%)
- **Objects HTTP API**: ✅ COMPLETE (100%)
- **Interpreter Integration**: ⏳ TODO (0%)
- **Vote Tally Integration**: ⏳ TODO (0%)
- **Chain Processing**: ⏳ TODO (0%)

**Overall Progress**: ~40% Complete

## 🎯 Next Steps

1. **Port Interpreter Blob Handling**
   - Start with `fendermint/vm/interpreter/src/fvm/state/iface.rs`
   - Add blob resolution to state interface
   - Integrate with existing message execution flow

2. **Integrate Vote Tally**
   - Create blob voting event loop in node service
   - Wire up to `VoteTally::add_blob_vote()`
   - Add quorum detection and status updates

3. **Test End-to-End Flow**
   - Upload blob via Objects HTTP API
   - Verify blob registration on-chain
   - Test validator resolution and voting
   - Confirm quorum detection and finalization

4. **Re-enable ADM Bucket Support**
   - Port ADM actor dependencies
   - Remove stub types
   - Integrate bucket-based storage

## 📝 Testing Commands

```bash
# Build everything
cargo build -p fendermint_app

# Run single node (when ready)
cargo make --makefile infra/fendermint/Makefile.toml testnode

# Test Objects HTTP API (when ready)
# Upload
curl -X POST http://localhost:8080/upload -F "file=@test.txt"

# Download
curl http://localhost:8080/download/<blob_hash>
```

## 🔗 Related Documents

- [RECALL_OBJECTS_API_STATUS.md](./RECALL_OBJECTS_API_STATUS.md) - Objects HTTP API porting status
- [RECALL_TESTING_GUIDE.md](./RECALL_TESTING_GUIDE.md) - Testing guide for Recall functionality
- [docs/ipc/recall-migration-guide.md](./docs/ipc/recall-migration-guide.md) - Full migration guide
- [docs/ipc/recall-vote-tally.md](./docs/ipc/recall-vote-tally.md) - Vote tally mechanism documentation

