# Recall Storage Node - Modularization Analysis

## Executive Summary

The recall storage node implementation adds **~66,000 lines of code** across **249 modified files** to enable decentralized blob storage with BFT consensus, erasure coding, and P2P transfer via Iroh. This analysis identifies the high-level areas modified and provides a roadmap for making the storage-node portion an optional compile-time module.

**Branch:** `recall-migration`
**Base Comparison:** `main` branch
**Total Changes:** +65,973 lines, -238 lines across 249 files

---

## 1. High-Level Architecture

### 1.1 Core Components Added

The recall implementation consists of several distinct layers:

```
┌─────────────────────────────────────────────────────────────────┐
│                    APPLICATION LAYER                             │
│  - fendermint objects command (HTTP API for blob upload/download)│
│  - ipc-decentralized-storage (standalone gateway & node binaries)│
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                    ACTOR LAYER (FVM)                             │
│  - blobs (main blob storage actor with credit system)            │
│  - blob_reader (read-only blob access)                           │
│  - recall_config (network configuration)                         │
│  - bucket (S3-like object storage)                               │
│  - timehub (timestamping service)                                │
│  - adm (Address/machine lifecycle manager)                       │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                 INTERPRETER/VM INTEGRATION                       │
│  - recall_executor (custom executor with gas allowances)         │
│  - recall_kernel (custom FVM kernel with blob syscalls)          │
│  - recall_syscalls (blob operation syscalls)                     │
│  - recall_helpers (FVM integration helpers)                      │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                    INFRASTRUCTURE LAYER                          │
│  - iroh_resolver (VM module for blob resolution & voting)        │
│  - iroh_manager (Iroh P2P node management)                       │
│  - recall_ipld (custom IPLD data structures - HAMT/AMT)          │
│  - recall_actor_sdk (actor SDK with EVM support)                 │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                      EXTERNAL DEPENDENCIES                       │
│  - Iroh v0.35 (P2P blob storage)                                 │
│  - entangler (erasure coding)                                    │
│  - netwatch (patched for socket2 0.5 compatibility)              │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Detailed Component Breakdown

### 2.1 NEW Components (Can Be Made Optional)

#### A. Recall Core Modules (`recall/` directory - 7 crates)
**Location:** `/recall/`
**Total Lines:** ~5,000 lines
**Purpose:** Core runtime components for blob storage

| Crate | Files | Purpose | Dependencies |
|-------|-------|---------|--------------|
| `recall/kernel` | 2 | Custom FVM kernel with blob syscalls | recall_kernel_ops, recall_syscalls |
| `recall/kernel/ops` | 1 | Kernel operations interface | None (minimal) |
| `recall/syscalls` | 1 | Blob operation syscalls | fvm_shared |
| `recall/executor` | 2 | Custom executor with gas allowances | recall_kernel, fvm |
| `recall/iroh_manager` | 3 | Iroh P2P node management | iroh, iroh-blobs |
| `recall/ipld` | 9 | Custom IPLD data structures (HAMT/AMT) | fvm_ipld_blockstore |
| `recall/actor_sdk` | 6 | Actor SDK with EVM support | fvm, fil_actors_runtime |

#### B. Recall Actors (`fendermint/actors/` - 6 actors)
**Location:** `/fendermint/actors/`
**Total Lines:** ~15,000 lines
**Purpose:** On-chain blob management actors

| Actor | Files | Purpose | Can Be Optional? |
|-------|-------|---------|------------------|
| `blobs` + `blobs/shared` | 40+ | Main blob storage with credit system | ✅ YES |
| `blob_reader` | 5 | Read-only blob access | ✅ YES |
| `recall_config` + `shared` | 3 | Network configuration | ✅ YES |
| `bucket` | 5 | S3-like object storage | ✅ YES |
| `timehub` | 4 | Timestamping service | ✅ YES |
| `adm` + `adm_types` | 6 | Address/machine manager | ✅ YES |

#### C. Recall Contracts (`recall-contracts/` - 1 crate)
**Location:** `/recall-contracts/crates/facade/`
**Total Lines:** ~18,000 lines (auto-generated)
**Purpose:** Solidity facade bindings for EVM integration

- Auto-generated from Solidity contracts
- Provides Rust bindings for EVM events
- FVM 4.7 compatible (upgraded from 4.3)

#### D. Standalone Storage Services (`ipc-decentralized-storage/`)
**Location:** `/ipc-decentralized-storage/`
**Total Lines:** ~2,300 lines
**Purpose:** Standalone storage gateway and node services

| Binary | Purpose | Can Be Optional? |
|--------|---------|------------------|
| `gateway` | HTTP gateway for blob upload/download | ✅ YES |
| `node` | Storage node with chain integration | ✅ YES |

**These are completely standalone and can be built as separate binaries.**

---

### 2.2 MODIFIED Components (Integration Points)

#### A. Fendermint VM Interpreter
**Location:** `/fendermint/vm/interpreter/`
**Files Modified:** 7 files
**Total Changes:** ~600 lines added

**Key Integration Points:**
1. **`fvm/interpreter.rs`** - Added handlers for `ReadRequestPending` and `ReadRequestClosed` IPC messages
2. **`fvm/recall_env.rs`** (NEW) - Read request pool for blob resolution
3. **`fvm/recall_helpers.rs`** (NEW) - Helper functions for blob operations
4. **`genesis.rs`** - Initialize recall actors at genesis (ADM, blobs, blob_reader, recall_config)
5. **`fvm/state/exec.rs`** - Optional recall executor integration

**Modularization Strategy:**
```rust
// Use conditional compilation
#[cfg(feature = "recall-storage")]
mod recall_env;
#[cfg(feature = "recall-storage")]
mod recall_helpers;

// In genesis.rs
#[cfg(feature = "recall-storage")]
fn initialize_recall_actors(state: &mut GenesisBuilder) { ... }
```

#### B. Fendermint App (CLI & HTTP API)
**Location:** `/fendermint/app/`
**Files Modified:** 8 files
**New Files:** 2 large files (~1,500 lines)

**Key Changes:**
1. **`cmd/objects.rs`** (NEW) - Complete HTTP API for blob upload/download (1,455 lines)
2. **`options/objects.rs`** (NEW) - CLI options for objects command
3. **`settings/objects.rs`** (NEW) - Settings for objects API
4. **`cmd/mod.rs`** - Register `objects` subcommand
5. **`service/node.rs`** - Added Iroh resolver initialization

**Modularization Strategy:**
```rust
// In Cargo.toml
[dependencies]
# Recall/Objects API (optional)
recall_components = { workspace = true, optional = true }

[features]
recall-storage = ["recall_components", "iroh", "iroh-blobs", ...]

// In cmd/mod.rs
#[cfg(feature = "recall-storage")]
pub mod objects;
```

#### C. VM Topdown (Voting & Consensus)
**Location:** `/fendermint/vm/topdown/`
**Files Modified:** 2 files
**Changes:** ~200 lines

**Key Changes:**
1. **`voting.rs`** - Added blob vote tally system with BFT consensus
   - `add_blob_vote()` - Record validator votes on blob availability
   - `find_blob_quorum()` - Detect when 2/3+ validators confirm blob
2. **`lib.rs`** - Export `Blob` type alias

**Modularization Strategy:**
```rust
#[cfg(feature = "recall-storage")]
pub struct BlobVote { ... }

#[cfg(feature = "recall-storage")]
impl VoteTally {
    pub fn add_blob_vote(...) { ... }
    pub fn find_blob_quorum(...) { ... }
}
```

#### D. IPLD Resolver (Iroh Integration)
**Location:** `/ipld/resolver/`
**Files Modified:** 5 files
**Changes:** ~400 lines

**Key Changes:**
1. **`client.rs`** - Added `ResolverIroh` and `ResolverIrohReadRequest` traits
2. **`service.rs`** - Integrated Iroh blob download logic
3. **`lib.rs`** - Export new Iroh-related types
4. **`behaviour/mod.rs`** - Added Iroh configuration errors

**Modularization Strategy:**
```rust
#[cfg(feature = "recall-storage")]
pub trait ResolverIroh { ... }

// Service can have optional Iroh support
pub struct Service<S, V> {
    #[cfg(feature = "recall-storage")]
    iroh_manager: Option<IrohManager>,
}
```

#### E. VM Actor Interface
**Location:** `/fendermint/vm/actor_interface/`
**New Files:** 4 files (minimal - just constants and enums)

**Key Additions:**
1. `adm.rs` - ADM actor constants
2. `blobs.rs` - Blobs actor constants
3. `blob_reader.rs` - Blob reader constants
4. `recall_config.rs` - Recall config constants

**Can be easily gated with feature flags.**

#### F. VM Message Types
**Location:** `/fendermint/vm/message/`
**Files Modified:** 1 file
**Changes:** ~100 lines

**Key Changes:**
- Added `ReadRequestPending` and `ReadRequestClosed` variants to `IpcMessage` enum

**Modularization Strategy:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcMessage {
    // ... existing variants ...

    #[cfg(feature = "recall-storage")]
    ReadRequestPending(ReadRequest),
    #[cfg(feature = "recall-storage")]
    ReadRequestClosed(ReadRequest),
}
```

#### G. Fendermint RPC
**Location:** `/fendermint/rpc/`
**Files Modified:** 3 files
**Changes:** ~100 lines

**Key Changes:**
- Added blob query endpoints
- Extended message types for blob operations

---

### 2.3 NEW Infrastructure Modules

#### Iroh Resolver VM Module
**Location:** `/fendermint/vm/iroh_resolver/`
**Files:** 4 files (~900 lines)
**Purpose:** Integrate Iroh blob resolution with FVM execution

| File | Purpose |
|------|---------|
| `iroh.rs` | Core blob resolution logic with vote submission |
| `pool.rs` | Connection pooling for Iroh nodes |
| `observe.rs` | Metrics and observability |
| `lib.rs` | Module exports |

**Can be made entirely optional with feature flag.**

---

## 3. Dependency Analysis

### 3.1 New External Dependencies

#### Critical Dependencies (Iroh P2P)
```toml
[workspace.dependencies]
# Iroh P2P stack (v0.35)
iroh = "0.35"
iroh-base = "0.35"
iroh-blobs = { version = "0.35", features = ["rpc"] }
iroh-relay = "0.35"
iroh-quinn = "0.13"
quic-rpc = { version = "0.20", features = ["quinn-transport"] }

# Recall-specific
ambassador = "0.3.5"
n0-future = "0.1.2"
```

#### HTTP/API Dependencies
```toml
# Objects HTTP API
warp = "0.3"
uuid = { version = "1.0", features = ["v4"] }
mime_guess = "2.0"
urlencoding = "2.1"
```

#### Erasure Coding
```toml
entangler = "0.1"
entangler_storage = "0.1"
```

#### Patches
```toml
[patch.crates-io]
# Required for macOS compatibility with Iroh
netwatch = { path = "patches/netwatch" }
```

### 3.2 Impact on Existing Dependencies

**No breaking changes to existing dependencies.**
All recall-related dependencies are additive.

---

## 4. Compilation Impact

### 4.1 Build Time Impact

Based on the changes:
- **+249 files** to compile
- **~66,000 lines** of new Rust code
- **~18,000 lines** of auto-generated bindings
- Estimated build time increase: **30-60 seconds** on modern hardware

### 4.2 Binary Size Impact

Estimated size increases with recall enabled:
- `fendermint` binary: **+15-20 MB**
- Iroh libraries: **~10 MB**
- Actor WebAssembly bundles: **+5 MB**

---

## 5. Runtime Integration Points

### 5.1 Genesis Initialization

**File:** `fendermint/vm/interpreter/src/genesis.rs`
**Changes:** Initialize 4 new actors at chain genesis

```rust
// Can be gated with feature flag
#[cfg(feature = "recall-storage")]
{
    // ADM actor (ID: 90)
    create_actor(ADM_ACTOR_NAME, ADM_ACTOR_ID, ...);

    // Recall config actor (ID: 100)
    create_actor(RECALL_CONFIG_ACTOR_NAME, RECALL_CONFIG_ACTOR_ID, ...);

    // Blobs actor (ID: 99) - with delegated Ethereum address
    create_actor(BLOBS_ACTOR_NAME, BLOBS_ACTOR_ID, ...);

    // Blob reader actor (ID: 101)
    create_actor(BLOB_READER_ACTOR_NAME, BLOB_READER_ACTOR_ID, ...);
}
```

### 5.2 Message Processing

**File:** `fendermint/vm/interpreter/src/fvm/interpreter.rs`

Two new IPC message types require handling:
1. `ReadRequestPending` - Mark blob read request as pending
2. `ReadRequestClosed` - Complete blob read and call callback

```rust
// Can be gated with match arms
match msg {
    #[cfg(feature = "recall-storage")]
    IpcMessage::ReadRequestPending(req) => { ... }

    #[cfg(feature = "recall-storage")]
    IpcMessage::ReadRequestClosed(req) => { ... }

    // ... existing message types
}
```

### 5.3 HTTP API Server

**File:** `fendermint/app/src/cmd/objects.rs`

Completely standalone subcommand:
```rust
#[cfg(feature = "recall-storage")]
pub mod objects;

// In main command enum
pub enum Commands {
    #[cfg(feature = "recall-storage")]
    Objects(objects::ObjectsCmd),
    // ... other commands
}
```

---

## 6. Modularization Strategy

### 6.1 Feature Flag Design

**Recommended Feature Flags:**

```toml
# In workspace Cargo.toml
[workspace.dependencies]
# Recall components (all optional)
recall_kernel = { path = "recall/kernel", optional = true }
recall_syscalls = { path = "recall/syscalls", optional = true }
recall_executor = { path = "recall/executor", optional = true }
recall_iroh_manager = { path = "recall/iroh_manager", optional = true }
recall_ipld = { path = "recall/ipld", optional = true }
recall_actor_sdk = { path = "recall/actor_sdk", optional = true }

# Recall actors (all optional)
fendermint_actor_blobs = { path = "fendermint/actors/blobs", optional = true }
fendermint_actor_blob_reader = { path = "fendermint/actors/blob_reader", optional = true }
fendermint_actor_recall_config = { path = "fendermint/actors/recall_config", optional = true }
fendermint_actor_bucket = { path = "fendermint/actors/bucket", optional = true }
fendermint_actor_timehub = { path = "fendermint/actors/timehub", optional = true }
fendermint_actor_adm = { path = "fendermint/actors/adm", optional = true }

# Iroh (optional)
iroh = { version = "0.35", optional = true }
iroh-blobs = { version = "0.35", features = ["rpc"], optional = true }

[features]
# Default: recall disabled
default = []

# Enable full recall storage support
recall-storage = [
    "recall-core",
    "recall-actors",
    "recall-http-api",
]

# Core recall runtime (kernel, executor, syscalls)
recall-core = [
    "dep:recall_kernel",
    "dep:recall_syscalls",
    "dep:recall_executor",
    "dep:recall_ipld",
    "dep:recall_iroh_manager",
    "dep:iroh",
    "dep:iroh-blobs",
]

# Recall actors (on-chain components)
recall-actors = [
    "recall-core",
    "dep:fendermint_actor_blobs",
    "dep:fendermint_actor_blob_reader",
    "dep:fendermint_actor_recall_config",
    "dep:fendermint_actor_bucket",
    "dep:fendermint_actor_timehub",
    "dep:fendermint_actor_adm",
]

# HTTP Objects API
recall-http-api = [
    "recall-core",
    "dep:warp",
    "dep:uuid",
    "dep:mime_guess",
    "dep:entangler",
]
```

### 6.2 Code Modifications Required

#### High-Priority Files (Must be Modified)

1. **`fendermint/vm/interpreter/src/fvm/interpreter.rs`**
   - Gate `ReadRequestPending` and `ReadRequestClosed` message handling
   - Add `#[cfg(feature = "recall-storage")]` around recall-specific code

2. **`fendermint/vm/interpreter/src/genesis.rs`**
   - Gate initialization of recall actors
   - Add `#[cfg(feature = "recall-storage")]` around actor creation

3. **`fendermint/vm/message/src/ipc.rs`**
   - Gate `ReadRequestPending` and `ReadRequestClosed` enum variants
   - Use `#[cfg_attr(feature = "recall-storage", ...)]`

4. **`fendermint/app/src/cmd/mod.rs`**
   - Gate `objects` subcommand registration
   - Add `#[cfg(feature = "recall-storage")]`

5. **`fendermint/vm/topdown/src/voting.rs`**
   - Gate blob voting methods
   - Keep existing voting logic, add feature flag for blob extensions

6. **`ipld/resolver/src/service.rs`**
   - Make Iroh integration optional
   - Add feature flag checks for Iroh client initialization

#### Medium-Priority Files (Should be Modified)

1. **`fendermint/app/settings/src/resolver.rs`**
   - Make `IrohResolverSettings` optional

2. **`fendermint/vm/actor_interface/src/lib.rs`**
   - Gate recall actor exports

3. **All Cargo.toml files in `fendermint/` and `recall/`**
   - Add `optional = true` to recall dependencies
   - Define feature flags

#### Low-Priority (Nice to Have)

1. **Documentation files** - Can remain as-is or be moved to `docs/recall/`
2. **Test files** - Can be gated with `#[cfg(test)]` and feature flags
3. **Examples** - Can be in separate `examples/` directory

---

## 7. Build Configuration Examples

### 7.1 Build WITHOUT Recall (Default)
```bash
# Build standard IPC without storage features
cargo build --release

# Smaller binary, faster build time
# No recall dependencies compiled
```

### 7.2 Build WITH Recall Core Only
```bash
# Build with recall runtime but no HTTP API
cargo build --release --features recall-core

# Includes: kernel, executor, syscalls, actors
# Excludes: HTTP API, standalone binaries
```

### 7.3 Build WITH Full Recall Support
```bash
# Build with all recall features
cargo build --release --features recall-storage

# Includes: everything
```

### 7.4 Build Standalone Storage Services Only
```bash
# Build just the storage gateway and node
cd ipc-decentralized-storage
cargo build --release

# Creates: gateway, node binaries
# No fendermint dependency
```

---

## 8. Testing Strategy

### 8.1 Unit Tests

All recall-specific tests should be gated:
```rust
#[cfg(all(test, feature = "recall-storage"))]
mod tests {
    // Recall-specific tests
}
```

### 8.2 Integration Tests

Create separate integration test suites:
```
tests/
  ├── recall_storage_integration.rs  (requires recall-storage feature)
  ├── standard_ipc.rs                (default, no recall)
  └── common/mod.rs
```

### 8.3 CI/CD Configuration

```yaml
# .github/workflows/ci.yml
jobs:
  test-default:
    # Test without recall
    run: cargo test

  test-with-recall:
    # Test with recall enabled
    run: cargo test --features recall-storage

  build-all-variants:
    strategy:
      matrix:
        features: ["", "recall-core", "recall-storage"]
    run: cargo build --features ${{ matrix.features }}
```

---

## 9. Migration Path

### Phase 1: Add Feature Flags (Low Risk)
1. Add feature flags to workspace `Cargo.toml`
2. Make all recall dependencies optional
3. Verify builds work with and without features
4. **Estimated Time:** 1-2 days

### Phase 2: Gate Code (Medium Risk)
1. Add `#[cfg(feature = "recall-storage")]` to integration points
2. Update message handling in interpreter
3. Update genesis initialization
4. **Estimated Time:** 3-5 days

### Phase 3: Test & Validate (High Risk)
1. Run full test suite with and without recall
2. Verify binary sizes and build times
3. Test runtime behavior
4. **Estimated Time:** 5-7 days

### Phase 4: Documentation & CI (Low Risk)
1. Update build documentation
2. Update CI/CD pipelines
3. Create migration guide for users
4. **Estimated Time:** 2-3 days

**Total Estimated Time:** 2-3 weeks

---

## 10. Key Decisions & Tradeoffs

### 10.1 What Should Be Optional?

✅ **Strongly Recommended to Make Optional:**
- All recall actors (`blobs`, `blob_reader`, `recall_config`, `bucket`, `timehub`, `adm`)
- Recall executor and kernel
- Iroh integration in IPLD resolver
- Objects HTTP API
- Standalone storage binaries

⚠️ **Consider Carefully:**
- Message type extensions (`ReadRequestPending`, `ReadRequestClosed`)
  - **Recommendation:** Make optional but requires careful serialization handling
- Vote tally extensions (blob voting)
  - **Recommendation:** Make optional, minimal impact

❌ **Should NOT Make Optional:**
- Core FVM infrastructure
- Existing IPC functionality
- Standard actor interface

### 10.2 Compilation Overhead

**With Feature Flags:**
- Default build (no recall): **No overhead**
- With recall enabled: **~30-60s additional build time**

**Without Feature Flags:**
- All builds include recall: **Always ~30-60s overhead**

### 10.3 Maintenance Burden

**With Modularization:**
- Pros:
  - Smaller default builds
  - Faster CI for non-recall changes
  - Clearer separation of concerns
  - Optional for users who don't need storage

- Cons:
  - More complex build configuration
  - Need to test multiple feature combinations
  - Risk of feature interaction bugs

**Recommendation:** Benefits outweigh costs for production use.

---

## 11. Summary

### 11.1 Scope of Changes

| Category | Files Changed | Lines Added | Can Be Optional? |
|----------|---------------|-------------|------------------|
| Recall core modules | 25 | ~5,000 | ✅ YES |
| Recall actors | 88 | ~15,000 | ✅ YES |
| Recall contracts | 22 | ~18,000 | ✅ YES |
| VM interpreter integration | 7 | ~600 | ⚠️ PARTIAL |
| Fendermint app (HTTP API) | 8 | ~1,500 | ✅ YES |
| IPLD resolver changes | 5 | ~400 | ⚠️ PARTIAL |
| VM message types | 1 | ~100 | ⚠️ PARTIAL |
| Standalone binaries | 7 | ~2,300 | ✅ YES (separate) |
| Documentation | 86 | ~24,000 | N/A |

**Total:** 249 files, ~66,000 lines

### 11.2 High-Level Areas Modified

1. **NEW: `recall/` directory** - Core runtime components (fully optional)
2. **NEW: `recall-contracts/` directory** - Solidity facades (fully optional)
3. **NEW: `ipc-decentralized-storage/` directory** - Standalone services (fully optional)
4. **NEW: `fendermint/actors/` additions** - 6 new actors (fully optional)
5. **MODIFIED: `fendermint/vm/interpreter/`** - Message handling (partially optional)
6. **MODIFIED: `fendermint/app/`** - HTTP API command (fully optional)
7. **MODIFIED: `ipld/resolver/`** - Iroh integration (partially optional)
8. **MODIFIED: `fendermint/vm/topdown/`** - Blob voting (partially optional)

### 11.3 Recommended Approach

**Make the following completely optional via feature flags:**
1. All components in `recall/` directory
2. All components in `recall-contracts/` directory
3. All components in `ipc-decentralized-storage/` directory
4. All recall actors in `fendermint/actors/`
5. Objects HTTP API in `fendermint/app/`
6. Iroh resolver in `fendermint/vm/iroh_resolver/`

**Make the following conditionally compiled:**
1. Genesis initialization of recall actors
2. Message handling for `ReadRequestPending` and `ReadRequestClosed`
3. Blob voting in vote tally
4. Iroh integration in IPLD resolver

**Keep the following always compiled:**
1. Core FVM infrastructure
2. Standard IPC functionality
3. Base message type definitions (with feature-gated variants)

---

## 12. Next Steps

1. **Review this analysis** with the team to confirm approach
2. **Create feature flag architecture** in workspace Cargo.toml
3. **Implement Phase 1** (feature flags) on a separate branch
4. **Test build configurations** to ensure both variants work
5. **Implement Phase 2** (code gating) incrementally
6. **Update CI/CD** to test both configurations
7. **Document** the feature flags for users

---

**Document Version:** 1.0
**Created:** December 4, 2024
**Branch Analyzed:** `recall-migration` vs `main`
