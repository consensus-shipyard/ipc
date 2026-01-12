# Storage Node Consolidation Plan

**Goal**: Consolidate storage-node functionality from PR #1474 (recall-migration) into a single unified folder structure with simple `#[cfg(feature = "storage-node")]` conditionals instead of a full plugin system.

**Date Created**: December 16, 2025
**Target Branch**: New branch based on `recall-migration` PR base
**Lessons Applied**: From commits `5a515cd3d` → `0e9ccb58d` → `cf6cf5629`

---

## Overview

This plan applies the learnings from the recent plugin architecture work, but takes a simpler approach:
- **No plugin system**: Use direct feature flags instead of `ModuleBundle` traits
- **Single folder**: Consolidate all storage code into one `storage-node/` directory
- **Hard-coded conditionals**: Add `#[cfg(feature = "storage-node")]` at integration points
- **Clean separation**: Storage code lives in one place, IPC core has minimal coupling

---

## Phase 1: Branch Setup & Audit

### 1.1 Create New Branch
```bash
# Fetch PR #1474
git fetch origin pull/1474/head:recall-migration-review

# Create new working branch from it
git checkout -b storage-consolidation recall-migration-review

# Or if starting from main + PR changes:
git checkout main
git checkout -b storage-consolidation
git merge recall-migration-review --no-commit
```

### 1.2 Audit Existing Code
Map what currently exists in the recall-migration branch:

**Expected Structure** (based on PR description):
```
recall/                           # Core storage modules
├── actor_sdk/                   # SDK for actor development
├── executor/                    # RecallExecutor (FVM integration)
├── ipld/                        # IPLD data structures
├── iroh_manager/               # Iroh network management
├── kernel/                      # Kernel operations
├── syscalls/                    # System calls
└── recall-contracts/           # Vendored Solidity facades

fendermint/actors/              # Recall actors
├── blobs/                       # Main storage actor
├── blob_reader/                # Read-only access
├── bucket/                      # S3-like abstraction (disabled?)
├── machine/                     # ADM integration (disabled?)
├── timehub/                     # Time operations (disabled?)
├── recall_config/              # Network config
└── adm/                         # Autonomous Data Management

fendermint/vm/
├── interpreter/src/fvm/
│   └── recall_env.rs           # Storage environment (maybe)
├── iroh_resolver/              # IPLD resolver with Iroh
└── message/src/ipc.rs          # ReadRequest messages already defined
```

**Action Items**:
- [ ] List all `recall/` subdirectories
- [ ] List all `fendermint/actors/*` that are storage-related
- [ ] Find any scattered storage code in `fendermint/vm/`
- [ ] Check `fendermint/app/` for storage initialization code
- [ ] Document all Cargo.toml entries for storage dependencies

---

## Phase 2: Design Consolidated Structure

### 2.1 Target Folder Structure

Consolidate everything into a single `storage-node/` directory:

```
storage-node/
├── Cargo.toml                   # Workspace manifest for storage-node
│
├── actors/                      # All storage actors
│   ├── blobs/                  # Main storage actor
│   │   ├── shared/             # Shared types
│   │   ├── testing/            # Test utilities
│   │   └── src/
│   ├── blob_reader/            # Read-only access
│   ├── recall_config/          # Network configuration
│   ├── adm/                    # ADM actor (if enabled)
│   ├── bucket/                 # S3 abstraction (if enabled)
│   └── timehub/                # Time operations (if enabled)
│
├── executor/                    # RecallExecutor
│   ├── Cargo.toml
│   └── src/lib.rs
│
├── kernel/                      # Kernel + syscalls
│   ├── Cargo.toml
│   ├── ops/                    # Kernel operations
│   └── src/lib.rs
│
├── syscalls/                    # System calls
│   ├── Cargo.toml
│   └── src/lib.rs
│
├── sdk/                         # Actor SDK (renamed from actor_sdk)
│   ├── Cargo.toml
│   └── src/
│
├── ipld/                        # IPLD structures
│   ├── Cargo.toml
│   └── src/
│
├── iroh/                        # Iroh integration
│   ├── manager/                # Iroh manager
│   │   ├── Cargo.toml
│   │   └── src/
│   └── resolver/               # IPLD resolver
│       ├── Cargo.toml
│       └── src/
│
├── integration/                 # Integration helpers for IPC
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # Public API for IPC integration
│       ├── genesis.rs          # Genesis initialization
│       ├── message_handler.rs  # IPC message handling
│       ├── storage_env.rs      # Environment setup
│       └── storage_helpers.rs  # Helper functions
│
└── contracts/                   # Vendored Solidity facades
    └── facade/
```

### 2.2 Integration API Design

Create a clean API in `storage-node/integration/src/lib.rs`:

```rust
// Public API that IPC code will use
#[cfg(feature = "storage-node")]
pub mod genesis;
#[cfg(feature = "storage-node")]
pub mod message_handler;
#[cfg(feature = "storage-node")]
pub mod storage_env;
#[cfg(feature = "storage-node")]
pub mod storage_helpers;

// Re-exports for convenience
#[cfg(feature = "storage-node")]
pub use genesis::initialize_storage_actors;
#[cfg(feature = "storage-node")]
pub use message_handler::{handle_read_request_pending, handle_read_request_closed};
#[cfg(feature = "storage-node")]
pub use storage_env::{BlobPool, ReadRequestPool};
```

---

## Phase 3: Integration Points

### 3.1 Message Types (Already Done)

The `fendermint/vm/message/src/ipc.rs` already has:
```rust
pub enum IpcMessage {
    TopDownExec(ParentFinality),
    ReadRequestPending(PendingReadRequest),
    ReadRequestClosed(ClosedReadRequest),
}
```

**No changes needed** - these are always compiled in, handlers are conditional.

### 3.2 Interpreter Integration

**File**: `fendermint/vm/interpreter/src/fvm/interpreter.rs`

Add conditional imports:
```rust
#[cfg(feature = "storage-node")]
use storage_node_integration::{
    handle_read_request_pending,
    handle_read_request_closed,
};
```

Add conditional match arms:
```rust
IpcMessage::TopDownExec(p) => {
    // Always present
    self.top_down_manager.execute_topdown_msg(state, p).await?
}
#[cfg(feature = "storage-node")]
IpcMessage::ReadRequestPending(req) => {
    handle_read_request_pending(state, req)?
}
#[cfg(feature = "storage-node")]
IpcMessage::ReadRequestClosed(req) => {
    handle_read_request_closed(state, req)?
}
#[cfg(not(feature = "storage-node"))]
IpcMessage::ReadRequestPending(_) | IpcMessage::ReadRequestClosed(_) => {
    Err(anyhow!("Storage-node feature not enabled"))
}
```

### 3.3 Genesis Integration

**File**: `fendermint/vm/interpreter/src/genesis.rs`

Add conditional initialization:
```rust
#[cfg(feature = "storage-node")]
use storage_node_integration::initialize_storage_actors;

pub fn create_genesis_state<DB>(...) -> Result<...> {
    // ... existing genesis setup ...

    #[cfg(feature = "storage-node")]
    {
        tracing::info!("Initializing storage-node actors");
        initialize_storage_actors(&mut state, genesis)?;
    }

    // ... rest of genesis ...
}
```

### 3.4 Actor Bundle Registration

**File**: `fendermint/actors/Cargo.toml` or custom bundle

Add conditional actor dependencies:
```toml
[dependencies]
# ... existing actors ...

[target.'cfg(feature = "storage-node")'.dependencies]
fendermint_actor_blobs = { path = "../storage-node/actors/blobs" }
fendermint_actor_blob_reader = { path = "../storage-node/actors/blob_reader" }
fendermint_actor_recall_config = { path = "../storage-node/actors/recall_config" }
```

**File**: `fendermint/actors/src/lib.rs` or bundle file

```rust
#[cfg(feature = "storage-node")]
use fendermint_actor_blobs;
#[cfg(feature = "storage-node")]
use fendermint_actor_blob_reader;
#[cfg(feature = "storage-node")]
use fendermint_actor_recall_config;

pub fn create_actor_bundle() -> Vec<...> {
    let mut actors = vec![
        // ... core IPC actors ...
    ];

    #[cfg(feature = "storage-node")]
    {
        actors.extend(vec![
            fendermint_actor_blobs::BLOBS_ACTOR,
            fendermint_actor_blob_reader::BLOB_READER_ACTOR,
            fendermint_actor_recall_config::RECALL_CONFIG_ACTOR,
        ]);
    }

    actors
}
```

### 3.5 Service Initialization

**File**: `fendermint/app/src/service/node.rs` or similar

Add conditional service startup:
```rust
#[cfg(feature = "storage-node")]
use storage_node_integration::storage_env::{BlobPool, ReadRequestPool};

pub async fn start_node(...) -> Result<...> {
    // ... existing node setup ...

    #[cfg(feature = "storage-node")]
    let (blob_pool, read_request_pool) = {
        tracing::info!("Starting storage-node services");
        let blob_pool = Arc::new(BlobPool::new());
        let read_request_pool = Arc::new(ReadRequestPool::new());

        // Spawn background tasks for vote publishing, etc.
        spawn_storage_services(blob_pool.clone(), read_request_pool.clone());

        (blob_pool, read_request_pool)
    };

    // ... rest of node setup ...
}
```

### 3.6 Cargo Workspace Configuration

**File**: `Cargo.toml` (root)

Add storage-node to workspace:
```toml
[workspace]
members = [
    # ... existing members ...
    "storage-node/actors/blobs",
    "storage-node/actors/blob_reader",
    "storage-node/actors/recall_config",
    "storage-node/executor",
    "storage-node/kernel",
    "storage-node/syscalls",
    "storage-node/sdk",
    "storage-node/ipld",
    "storage-node/iroh/manager",
    "storage-node/iroh/resolver",
    "storage-node/integration",
]

[workspace.dependencies]
# Storage-node specific dependencies
storage_node_integration = { path = "storage-node/integration" }
storage_node_executor = { path = "storage-node/executor" }
# ... etc
```

**Feature flag definition**:
```toml
[features]
default = []
storage-node = [
    "fendermint_vm_interpreter/storage-node",
    "fendermint_app/storage-node",
]
```

### 3.7 Interpreter Cargo.toml

**File**: `fendermint/vm/interpreter/Cargo.toml`

```toml
[features]
storage-node = [
    "dep:storage_node_integration",
    "dep:storage_node_executor",
]

[dependencies]
# ... existing deps ...

# Optional storage-node dependencies
storage_node_integration = { workspace = true, optional = true }
storage_node_executor = { workspace = true, optional = true }
```

---

## Phase 4: Migration Steps

### Step-by-Step Execution Plan

#### Step 1: Create Directory Structure
```bash
mkdir -p storage-node/{actors,executor,kernel,syscalls,sdk,ipld,iroh/{manager,resolver},integration,contracts}
```

#### Step 2: Move Recall Core Modules
```bash
# Move executor
mv recall/executor storage-node/

# Move kernel + syscalls
mv recall/kernel storage-node/
mv recall/syscalls storage-node/

# Move SDK
mv recall/actor_sdk storage-node/sdk

# Move IPLD
mv recall/ipld storage-node/

# Move Iroh
mv recall/iroh_manager storage-node/iroh/manager
mv fendermint/vm/iroh_resolver storage-node/iroh/resolver

# Move contracts
mv recall/recall-contracts storage-node/contracts
```

#### Step 3: Move Actors
```bash
# Identify storage actors in fendermint/actors/
# Based on PR, likely: blobs, blob_reader, recall_config, adm, bucket, timehub

mv fendermint/actors/blobs storage-node/actors/
mv fendermint/actors/blob_reader storage-node/actors/
mv fendermint/actors/recall_config storage-node/actors/

# Optional actors (if present and enabled)
[ -d fendermint/actors/adm ] && mv fendermint/actors/adm storage-node/actors/
[ -d fendermint/actors/bucket ] && mv fendermint/actors/bucket storage-node/actors/
[ -d fendermint/actors/timehub ] && mv fendermint/actors/timehub storage-node/actors/
```

#### Step 4: Create Integration Layer
Create `storage-node/integration/` with files extracted from:
- `fendermint/vm/interpreter/src/fvm/recall_env.rs` → `storage_env.rs`
- `fendermint/vm/interpreter/src/fvm/recall_helpers.rs` → `storage_helpers.rs`
- Genesis storage logic → `genesis.rs`
- Message handling logic → `message_handler.rs`

#### Step 5: Update All Cargo.toml Files
- Update path dependencies to reflect new structure
- Add `storage-node = []` feature flags
- Make storage dependencies optional
- Update workspace members list

#### Step 6: Add Feature Guards
Add `#[cfg(feature = "storage-node")]` to:
- Interpreter message handlers
- Genesis initialization
- Service startup
- Actor bundle registration
- Import statements

#### Step 7: Update Import Paths
Search and replace old import paths:
```bash
# Find all imports from recall/ or scattered storage code
rg "use recall::" --files-with-matches
rg "use fendermint_actor_blobs" --files-with-matches
rg "use.*recall_config" --files-with-matches

# Update to new paths
# recall:: → storage_node_executor:: or storage_node_sdk::
# fendermint_vm_iroh_resolver:: → storage_node_iroh_resolver::
```

#### Step 8: Clean Up Empty Directories
```bash
# Remove old recall/ directory if now empty
[ -d recall ] && rmdir recall/ 2>/dev/null || rm -rf recall/

# Remove scattered storage files
rm -f fendermint/vm/interpreter/src/fvm/recall_env.rs
rm -f fendermint/vm/interpreter/src/fvm/recall_helpers.rs
```

---

## Phase 5: Testing & Validation

### 5.1 Compilation Tests

Test that code compiles with and without the feature:

```bash
# Test WITHOUT storage-node (default)
cargo clean
cargo check --workspace
cargo build --workspace

# Test WITH storage-node
cargo clean
cargo check --workspace --features storage-node
cargo build --workspace --features storage-node

# Test specific crates
cargo check -p fendermint_vm_interpreter
cargo check -p fendermint_vm_interpreter --features storage-node
```

### 5.2 Feature Flag Validation

Verify conditionals work:
```bash
# Should fail (no storage-node feature)
cargo test test_storage_functionality

# Should pass
cargo test test_storage_functionality --features storage-node
```

### 5.3 Integration Tests

Run full node:
```bash
# Without storage
cargo run --bin fendermint -- run

# With storage
cargo run --bin fendermint --features storage-node -- run
```

### 5.4 Actor Bundle Check

Verify actors are included/excluded correctly:
```bash
# Check actor bundle contents
cargo run --bin fendermint -- genesis --help

# With storage
cargo run --bin fendermint --features storage-node -- genesis --help
```

---

## Phase 6: Documentation

### 6.1 Update README

Create `storage-node/README.md`:
```markdown
# Storage Node

Consolidated storage functionality for IPC.

## Structure
- `actors/` - Storage actors (blobs, blob_reader, recall_config)
- `executor/` - RecallExecutor for FVM
- `kernel/` - Kernel operations and syscalls
- `sdk/` - Actor development SDK
- `ipld/` - IPLD data structures
- `iroh/` - Iroh network integration
- `integration/` - Integration API for IPC core

## Usage

Enable storage-node functionality with the feature flag:

```toml
[dependencies]
fendermint = { version = "...", features = ["storage-node"] }
```

Or via cargo:
```bash
cargo build --features storage-node
```

## Architecture

Storage-node is integrated via conditional compilation.
When disabled, IPC runs without any storage overhead.
```

### 6.2 Migration Document

Create `storage-node/MIGRATION.md` documenting:
- What was moved from where
- Why the consolidation approach was chosen
- Integration points in IPC core
- How to enable/disable storage-node

### 6.3 Integration Guide

Create `storage-node/INTEGRATION.md`:
- How to add new storage actors
- How to extend integration API
- Testing guidelines
- Feature flag best practices

---

## Phase 7: Final Verification Checklist

- [ ] All files moved to `storage-node/`
- [ ] No remnants in `recall/` or scattered locations
- [ ] All Cargo.toml paths updated
- [ ] Feature flags added to all integration points
- [ ] Compiles successfully WITHOUT `--features storage-node`
- [ ] Compiles successfully WITH `--features storage-node`
- [ ] Tests pass in both configurations
- [ ] Documentation complete
- [ ] Git history is clean and logical
- [ ] Ready for PR review

---

## Risk Mitigation

### Potential Issues

1. **Circular dependencies**: Storage-node depends on IPC, IPC depends on storage
   - **Solution**: Keep integration layer thin, use trait objects if needed

2. **Feature flag explosion**: Too many nested conditional compilation blocks
   - **Solution**: Limit conditionals to integration points only

3. **Test coverage**: Tests might not run without storage-node
   - **Solution**: Add CI job that tests both configurations

4. **Import path confusion**: Old paths might linger
   - **Solution**: Use `cargo deny` or similar to catch bad imports

### Rollback Plan

If consolidation causes issues:
1. Commit each phase separately
2. Tag working states: `storage-consolidation-phase-N`
3. Can revert to last working phase
4. Each phase should leave codebase in compilable state

---

## Success Criteria

✅ **Single folder**: All storage code in `storage-node/`
✅ **Feature gated**: Works with/without `--features storage-node`
✅ **Clean integration**: IPC core has minimal coupling
✅ **Compiles**: Both configurations build successfully
✅ **Tests pass**: Both configurations tested
✅ **Documented**: Clear guide for future maintenance

---

## Timeline Estimate

- **Phase 1** (Audit): 1-2 hours
- **Phase 2** (Design): 30 minutes
- **Phase 3** (Planning): 30 minutes
- **Phase 4** (Migration): 3-4 hours
- **Phase 5** (Testing): 1-2 hours
- **Phase 6** (Documentation): 1 hour
- **Phase 7** (Verification): 30 minutes

**Total**: 7-10 hours (can be split across multiple sessions)

---

## Next Steps

1. Review this plan
2. Switch to `recall-migration` branch or create new branch
3. Execute Phase 1: Audit existing code structure
4. Update plan based on findings
5. Execute remaining phases systematically

**Ready to proceed?** Let me know when you've switched branches!

