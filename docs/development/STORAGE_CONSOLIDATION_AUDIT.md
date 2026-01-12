# Storage Node Consolidation - Phase 1 Audit

**Date**: December 16, 2025
**Branch**: `storage-consolidation` (from PR #1474)
**Status**: ✅ Audit Complete

---

## Executive Summary

Audited the `recall-migration` branch to understand the current structure of storage-related code.
Found storage code scattered across 4 main locations with a total of **8 storage actors** and **6 core modules**.

---

## Current Structure

### 1. Core Storage Modules (`recall/`)

Located at: `/recall/`

```
recall/
├── actor_sdk/          # SDK for actor development
│   ├── Cargo.toml
│   └── src/
│       ├── caller.rs
│       ├── constants.rs
│       ├── evm.rs
│       ├── lib.rs
│       ├── storage.rs
│       └── util.rs
│
├── executor/           # RecallExecutor (FVM integration)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── outputs.rs
│
├── ipld/               # IPLD data structures (AMT, HAMT)
│   ├── Cargo.toml
│   └── src/
│       ├── amt/ (core.rs, vec.rs)
│       ├── hamt/ (core.rs, map.rs)
│       ├── hash_algorithm.rs
│       └── lib.rs
│
├── iroh_manager/       # Iroh network management
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── manager.rs
│       └── node.rs
│
├── kernel/             # Kernel operations
│   ├── Cargo.toml
│   ├── ops/           # Kernel ops subcrate
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   └── src/lib.rs
│
├── syscalls/           # System calls
│   ├── Cargo.toml
│   └── src/lib.rs
│
└── Makefile
```

**Total**: 6 core modules (7 if counting kernel/ops separately)

---

### 2. Storage Actors (`fendermint/actors/`)

Located at: `/fendermint/actors/`

```
fendermint/actors/
│
├── adm/                # Autonomous Data Management actor
│   ├── Cargo.toml
│   └── src/
│       ├── ext.rs
│       ├── lib.rs
│       ├── sol_facade.rs
│       └── state.rs
│
├── adm_types/          # ADM type definitions
│   ├── Cargo.toml
│   └── src/lib.rs
│
├── machine/            # Machine base trait
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── sol_facade.rs
│
├── blobs/              # Main storage actor ⭐
│   ├── Cargo.toml
│   ├── shared/        # Shared types subcrate
│   │   ├── Cargo.toml
│   │   └── src/ (accounts, blobs, credit, etc.)
│   ├── testing/       # Test utilities subcrate
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   └── src/
│       ├── actor/ (admin, metrics, system, user)
│       ├── sol_facade/ (blobs, credit, gas)
│       ├── state/ (accounts, blobs, credit)
│       └── (lib.rs, caller.rs, etc.)
│
├── blob_reader/        # Read-only blob access
│   ├── Cargo.toml
│   └── src/
│       ├── actor.rs
│       ├── lib.rs
│       ├── shared.rs
│       ├── sol_facade.rs
│       └── state.rs
│
├── bucket/             # S3-like object storage
│   ├── Cargo.toml
│   └── src/
│       ├── actor.rs
│       ├── lib.rs
│       ├── shared.rs
│       ├── sol_facade.rs
│       └── state.rs
│
├── timehub/            # Timestamping service
│   ├── Cargo.toml
│   └── src/
│       ├── actor.rs
│       ├── lib.rs
│       ├── shared.rs
│       └── sol_facade.rs
│
└── recall_config/      # Network configuration
    ├── Cargo.toml
    ├── shared/        # Shared types subcrate
    │   ├── Cargo.toml
    │   └── src/lib.rs
    └── src/
        ├── lib.rs
        └── sol_facade.rs
```

**Total**: 8 actors (11 crates including subcrates)

---

### 3. VM Integration Code (`fendermint/vm/`)

Located at: `/fendermint/vm/`

```
fendermint/vm/
│
├── iroh_resolver/      # IPLD resolver with Iroh
│   ├── Cargo.toml
│   └── src/
│       ├── iroh.rs     (10KB - main Iroh integration)
│       ├── lib.rs      (165 bytes - re-exports)
│       ├── observe.rs  (5KB - observation/monitoring)
│       └── pool.rs     (11KB - request pooling)
│
├── interpreter/src/fvm/
│   ├── recall_helpers.rs   # Storage helper functions
│   └── recall_env.rs       # Storage environment (pools, etc.)
│
└── actor_interface/src/
    ├── blob_reader.rs      # Blob reader interface
    ├── blobs.rs           # Blobs interface
    └── recall_config.rs   # Config interface
```

**Integration Files**:
- 2 files in `interpreter/src/fvm/`
- 1 directory (`iroh_resolver/`) with 4 files
- 3 files in `actor_interface/src/`

---

### 4. Solidity Facades (`recall-contracts/`)

Located at: `/recall-contracts/crates/facade/`

```
recall-contracts/
└── crates/
    └── facade/
        ├── Cargo.toml
        ├── forge/          # Foundry code generation
        └── src/
            ├── blobreader_facade/
            ├── blobs_facade/
            ├── bucket_facade/
            ├── config_facade/
            ├── credit_facade/
            ├── gas_facade/
            ├── machine_facade/
            └── timehub_facade/
```

**Purpose**: Vendored locally, upgraded to FVM 4.7 (from 4.3)

---

### 5. Other Storage-Related Code

#### `ipc-decentralized-storage/`

Located at: `/ipc-decentralized-storage/`

```
ipc-decentralized-storage/
├── Cargo.toml
└── src/
    ├── bin/
    │   ├── gateway.rs
    │   └── node.rs
    ├── gateway.rs
    ├── lib.rs
    └── node/ (mod.rs, resolver.rs, rpc.rs, store.rs)
```

**Status**: ❓ **Unclear** if this should be consolidated or kept separate.
**Note**: Appears to be a separate binary for storage gateway/node. May not be part of core storage-node.

#### `fendermint/app/` Integration

Files found with storage references:
- `fendermint/app/settings/src/resolver.rs`
- `fendermint/app/options/src/lib.rs`
- `fendermint/app/options/src/objects.rs`
- `fendermint/app/src/cmd/genesis.rs`
- `fendermint/app/src/cmd/objects.rs`
- `fendermint/app/src/ipc.rs`
- `fendermint/app/src/store.rs`
- `fendermint/app/src/service/node.rs`
- `fendermint/app/src/app.rs`

**Status**: Need to review these for storage initialization and service startup code.

---

## Workspace Configuration

From `Cargo.toml` (lines 48-71):

```toml
# recall actors
"fendermint/actors/adm_types",
"fendermint/actors/adm",
"fendermint/actors/machine",
"fendermint/actors/blobs",
"fendermint/actors/blobs/shared",
"fendermint/actors/blobs/testing",
"fendermint/actors/blob_reader",
"fendermint/actors/bucket",
"fendermint/actors/timehub",
"fendermint/actors/recall_config",
"fendermint/actors/recall_config/shared",

# recall storage (netwatch patched for socket2 0.5 compatibility!)
"recall/kernel",
"recall/kernel/ops",
"recall/syscalls",
"recall/executor",
"recall/iroh_manager",
"recall/ipld",
"recall/actor_sdk",

# recall contracts (vendored locally, FVM 4.7 upgrade)
"recall-contracts/crates/facade",
```

**Total Workspace Members**: 20 storage-related crates

---

## Summary Statistics

| Category | Count | Location |
|----------|-------|----------|
| **Core Modules** | 7 | `recall/` |
| **Storage Actors** | 8 (11 crates) | `fendermint/actors/` |
| **VM Integration Files** | 6 files | `fendermint/vm/` |
| **Solidity Facades** | 1 crate | `recall-contracts/` |
| **App Integration** | 9 files | `fendermint/app/` |
| **Total Workspace Crates** | 20+ | - |

---

## Consolidation Target Structure

Based on audit, proposed consolidated structure in `storage-node/`:

```
storage-node/
├── actors/                      # All 8 actors + subcrates
│   ├── adm/
│   ├── adm_types/
│   ├── machine/
│   ├── blobs/ (with shared/, testing/)
│   ├── blob_reader/
│   ├── bucket/
│   ├── timehub/
│   └── recall_config/ (with shared/)
│
├── core/                        # Core modules from recall/
│   ├── executor/
│   ├── kernel/ (with ops/)
│   ├── syscalls/
│   ├── sdk/                    # Renamed from actor_sdk
│   └── ipld/
│
├── iroh/                        # Iroh-related code
│   ├── manager/                # From recall/iroh_manager/
│   └── resolver/               # From fendermint/vm/iroh_resolver/
│
├── integration/                 # NEW: Integration layer
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs              # Public API
│       ├── genesis.rs          # Genesis initialization
│       ├── message_handler.rs  # IPC message handling
│       ├── storage_env.rs      # From recall_env.rs
│       ├── storage_helpers.rs  # From recall_helpers.rs
│       └── actor_interface.rs  # From actor_interface/*.rs
│
└── contracts/                   # Vendored Solidity facades
    └── facade/                 # From recall-contracts/
```

---

## Migration Complexity Assessment

### Easy (Low Risk)
- ✅ Move `recall/` → `storage-node/core/`
- ✅ Move `recall-contracts/` → `storage-node/contracts/`
- ✅ Move actors from `fendermint/actors/` → `storage-node/actors/`

### Medium (Moderate Risk)
- ⚠️ Move `fendermint/vm/iroh_resolver/` → `storage-node/iroh/resolver/`
- ⚠️ Extract `recall_helpers.rs` and `recall_env.rs` → `storage-node/integration/`
- ⚠️ Extract actor interfaces → `storage-node/integration/`

### Complex (High Risk)
- 🔴 Update all Cargo.toml path dependencies (20+ crates)
- 🔴 Update import paths throughout codebase
- 🔴 Add `#[cfg(feature = "storage-node")]` to integration points
- 🔴 Refactor `fendermint/app/` service initialization

---

## Integration Points Identified

### 1. Message Handling
**File**: `fendermint/vm/interpreter/src/fvm/interpreter.rs`
**Need**: Add `#[cfg(feature = "storage-node")]` to ReadRequest handlers

### 2. Genesis
**File**: `fendermint/vm/interpreter/src/genesis.rs` or similar
**Need**: Conditional actor initialization

### 3. Actor Bundle
**File**: `fendermint/actors/src/lib.rs` or custom bundle
**Need**: Conditional actor registration

### 4. Service Startup
**File**: `fendermint/app/src/service/node.rs`
**Need**: Conditional storage service initialization

### 5. Configuration
**Files**: `fendermint/app/settings/`, `fendermint/app/options/`
**Need**: Feature-gated storage settings

---

## Decisions to Make

### Question 1: `ipc-decentralized-storage/`
**Should this be included in consolidation or kept separate?**

**Recommendation**: Keep separate (it's a different binary/service)

### Question 2: Folder naming
**Use `storage-node/` or `recall/` for consolidated directory?**

**Recommendation**: Use `storage-node/` (more descriptive, matches current branch work)

### Question 3: Core module organization
**Keep flat or group into `storage-node/core/`?**

**Recommendation**: Group into `core/` for better organization

---

## Next Steps

**Phase 2**: Review and finalize consolidation structure
**Phase 3**: Create integration layer skeleton
**Phase 4**: Begin systematic migration
**Phase 5**: Update all Cargo.toml files
**Phase 6**: Add feature flags to integration points
**Phase 7**: Test compilation with/without feature

---

## Notes

- ⚠️ **netwatch patch**: Current branch has patched netwatch for socket2 0.5 compatibility
- ⚠️ **FVM version**: Upgraded from 4.3 to 4.7
- ✅ **Actors**: All 8 storage actors are present and appear complete
- ✅ **Core modules**: All core modules from original Recall are present
- ⚠️ **Integration**: Scattered across multiple files, needs consolidation

---

**Audit Complete** ✅
Ready for Phase 2: Design Refinement

