# Recall Storage Migration Guide: ipc-recall → main

## Executive Summary

This document outlines the requirements and steps needed to migrate the Recall storage implementation from the `ipc-recall` branch to the `main` branch.

**Branch Status:**
- `ipc-recall` is **959 commits behind** and **77 commits ahead** of `main`
- Current commit on `ipc-recall`: `567108af` (fix: non-determinism from actor debug flag)
- Current commit on `main`: `984fc4a4` (feat: add f3 cert actor)

**Migration Complexity:** High - requires significant reconciliation of architectural changes

---

## Table of Contents

1. [Critical Version Differences](#critical-version-differences)
2. [Architectural Changes on Main](#architectural-changes-on-main)
3. [Recall-Specific Components](#recall-specific-components)
4. [Migration Strategy](#migration-strategy)
5. [Step-by-Step Migration Plan](#step-by-step-migration-plan)
6. [Testing Requirements](#testing-requirements)
7. [Risk Assessment](#risk-assessment)
8. [Rollback Plan](#rollback-plan)

---

## Critical Version Differences

### FVM (Filecoin Virtual Machine)

**Current State:**
- `ipc-recall`: FVM **4.3.0**
- `main`: FVM **4.7.4** (updated in #1459)

**Impact:** HIGH
- FVM upgrade includes API changes, new features, and bug fixes
- Actor code may need updates for new FVM interfaces
- Syscalls and kernel interfaces may have changed

**Action Required:**
1. Audit all FVM-dependent code in Recall components
2. Update `recall/kernel/`, `recall/syscalls/`, `recall/executor/` for FVM 4.7.4 compatibility
3. Test actor execution with new FVM version
4. Review FVM 4.4, 4.5, 4.6, 4.7 changelogs for breaking changes

### Rust Toolchain

**Current State:**
- `ipc-recall`: Rust 1.81.0 (approximately)
- `main`: Rust 1.83.0 (updated in #1385)

**Impact:** MEDIUM
- New Rust features and lints available
- Dependency version conflicts possible
- Clippy rule changes

**Action Required:**
1. Update `rust-toolchain.toml`
2. Run `cargo clippy` and fix new warnings
3. Update dependencies for Rust 1.83.0 compatibility

### Builtin Actors

**Current State:**
- Builtin actors versions likely diverged significantly

**Impact:** HIGH
- Core actor interfaces may have changed
- Gateway, Subnet, and Registry contracts updated on main

**Action Required:**
1. Review builtin actors submodule version on main
2. Test compatibility with Recall actors
3. Update actor interfaces if needed

### Iroh (P2P Storage Layer)

**Current State:**
- `ipc-recall`: iroh 0.34.x (updated in #565)
- `main`: Unknown (may be older or removed)

**Impact:** CRITICAL
- Iroh is fundamental to Recall storage
- API changes between versions can be breaking

**Action Required:**
1. Verify iroh version compatibility requirements
2. Test iroh_manager with target version
3. Update iroh_blobs API calls if needed

---

## Architectural Changes on Main

### 1. Workspace Reorganization

**Changes:**
```diff
- contract-bindings/ (root level)
+ contracts/binding/ (moved under contracts/)

- build-rs-utils/ (removed)
- contracts-artifacts/ (removed)
```

**Impact:** MEDIUM
- Build scripts need updating
- Import paths may need changes
- Cargo workspace configuration different

**Migration Required:**
- Update `Cargo.toml` workspace members list
- Fix contract binding imports throughout Recall code
- Update build scripts in recall actors

### 2. Contract Bindings Refactoring (#1290)

**Changes:**
- Contract bindings moved to `contracts/binding/`
- Build process standardized
- Error parsing improvements

**Impact:** MEDIUM
- Any Recall code importing contract bindings needs path updates
- Blobs actor Solidity facade may need updates

**Migration Required:**
- Update import statements in:
  - `fendermint/actors/blobs/src/sol_facade/`
  - `fendermint/actors/bucket/src/sol_facade.rs`
  - `fendermint/actors/recall_config/src/sol_facade.rs`

### 3. Actors Builder Refactoring (#1300)

**Changes:**
- New actor building and bundling system
- Custom actors bundle generation updated

**Impact:** HIGH
- Recall actors need to integrate with new build system
- Custom actor manifest may need updates

**Migration Required:**
- Update `fendermint/actors/src/manifest.rs` to include Recall actors
- Ensure Recall actors are included in `custom_actors_bundle.car`
- Test actor loading and initialization

### 4. F3 Cert Actor Addition (#1438)

**Changes:**
- New F3 (Fast Finality) certificate actor added
- Genesis and actor initialization updated

**Impact:** LOW
- Doesn't directly affect Recall, but changes genesis flow

**Migration Required:**
- Ensure Recall actors initialize properly with F3 actor present
- Test genesis with all actors

### 5. Observability Refinements (#1085, #1207)

**Changes:**
- Metrics scheme migrated
- Logging levels refactored
- Tracing improvements

**Impact:** MEDIUM
- Recall observability code may need updates

**Migration Required:**
- Update metrics in `fendermint/vm/iroh_resolver/src/observe.rs`
- Update blobs actor metrics
- Verify logging works with new scheme

### 6. IPC CLI UI (#1401)

**Changes:**
- New CLI interface and commands
- Node management commands added

**Impact:** LOW (unless Recall adds CLI commands)

**Migration Required:**
- Consider adding Recall-specific CLI commands for:
  - Blob management
  - Storage statistics
  - Node diagnostics

---

## Recall-Specific Components

### Core Recall Modules (in `recall/`)

#### 1. `recall/kernel/`
**Purpose:** Custom FVM kernel with Recall-specific operations

**Files:**
- `src/lib.rs` - RecallKernel implementation
- `ops/src/lib.rs` - RecallOps trait

**Dependencies:**
- `fvm` 4.3.0 → needs upgrade to 4.7.4
- `fvm_shared`, `fvm_ipld_blockstore`

**Migration Concerns:**
- Kernel API changes in FVM 4.7.4
- Syscall linker interface updates
- Block operations compatibility

#### 2. `recall/syscalls/`
**Purpose:** Syscall implementations for blob operations

**Files:**
- `src/lib.rs` - delete_blob syscall

**Dependencies:**
- `iroh_blobs` - RPC client for blob deletion
- `iroh_manager` - connection management

**Migration Concerns:**
- Syscall signature changes in new FVM
- Iroh RPC client compatibility

#### 3. `recall/executor/`
**Purpose:** Custom executor with gas allowances for storage

**Files:**
- `src/lib.rs` - RecallExecutor implementation
- `outputs.rs` - Gas calculation logic

**Dependencies:**
- `fvm`, `fvm_shared` - needs FVM upgrade
- `fendermint_actor_blobs_shared` - gas allowance types

**Migration Concerns:**
- Executor interface changes in FVM 4.7.4
- Gas calculation compatibility
- Actor method invocation updates

#### 4. `recall/iroh_manager/`
**Purpose:** Iroh node management and blob operations

**Files:**
- `src/lib.rs` - Helper functions for hash sequences
- `src/manager.rs` - IrohManager with RPC server
- `src/node.rs` - IrohNode wrapper

**Dependencies:**
- `iroh` 0.34.x - P2P networking
- `iroh_blobs` - blob storage protocol
- `quic_rpc` - RPC transport

**Migration Concerns:**
- Iroh version compatibility (critical)
- RPC protocol changes
- Endpoint and relay configuration

#### 5. `recall/ipld/`
**Purpose:** Custom IPLD data structures (AMT, HAMT)

**Files:**
- `src/amt/` - Array Mapped Trie
- `src/hamt/` - Hash Array Mapped Trie

**Dependencies:**
- `fvm_ipld_blockstore`, `fvm_ipld_encoding`
- `fvm_shared` - actor error types

**Migration Concerns:**
- IPLD encoding compatibility
- Blockstore interface changes

#### 6. `recall/actor_sdk/`
**Purpose:** SDK for actors using Recall storage

**Files:**
- `src/lib.rs` - Public exports
- `src/caller.rs` - Actor caller utilities
- `src/evm.rs` - EVM integration
- `src/storage.rs` - Storage syscall wrapper
- `src/util.rs` - Helper functions

**Dependencies:**
- `fvm_sdk` - needs FVM upgrade

**Migration Concerns:**
- SDK API changes in new FVM
- Actor calling conventions

### Fendermint Actors (in `fendermint/actors/`)

#### 7. `fendermint/actors/blobs/`
**Purpose:** Main Blobs actor for storage management

**Structure:**
```
blobs/
├── Cargo.toml
├── shared/           # Shared types and traits
├── src/
│   ├── actor/        # Actor methods (user, admin, system)
│   ├── caller.rs     # Caller authentication
│   ├── state/        # State management
│   └── sol_facade/   # Solidity interface
└── testing/          # Test utilities
```

**Key Features:**
- Blob subscription management
- Credit and gas allowance system
- TTL and expiry tracking
- Status tracking (Added, Pending, Resolved, Failed)

**Migration Concerns:**
- Contract binding imports (sol_facade)
- Actor interface registration
- State serialization compatibility
- Integration with FVM executor

#### 8. `fendermint/actors/blob_reader/`
**Purpose:** Read-only access to blob data

**Migration Concerns:**
- Actor method registration
- Query interface compatibility

#### 9. `fendermint/actors/bucket/`
**Purpose:** S3-like bucket abstraction over blobs

**Migration Concerns:**
- Object key management
- Blob ownership model
- Solidity facade updates

#### 10. `fendermint/actors/recall_config/`
**Purpose:** Network-wide Recall configuration

**Migration Concerns:**
- Configuration parameter compatibility
- Governance integration

### VM Components

#### 11. `fendermint/vm/iroh_resolver/`
**Purpose:** Blob resolution and vote tallying

**Structure:**
```
iroh_resolver/
├── src/
│   ├── iroh.rs      # Resolution logic
│   ├── pool.rs      # Task pool management
│   ├── observe.rs   # Metrics and events
│   └── lib.rs
```

**Key Features:**
- Async blob download from Iroh nodes
- Vote casting for resolution
- Retry logic for failed downloads
- Read request handling

**Migration Concerns:**
- Vote tally integration (uses fendermint_vm_topdown)
- Metrics registration
- Task scheduling
- Iroh client compatibility

#### 12. `fendermint/vm/interpreter/` (modifications)
**Purpose:** Integration of blob resolution into chain execution

**Key Changes:**
- Blob pool management
- BlobPending and BlobFinalized message handling
- Proposal validation with vote quorum
- State transitions for blobs

**Migration Concerns:**
- ChainMessage enum additions
- Interpreter state transaction handling
- Block proposal validation
- Integration with CheckInterpreter

---

## Migration Strategy

### Approach: Incremental Integration

We recommend an **incremental integration** approach rather than a full merge:

1. **Create clean feature branch** from latest main
2. **Port components incrementally** in dependency order
3. **Test each component** before proceeding
4. **Fix compatibility issues** as they arise
5. **Validate integration** with full system tests

### Why Not Direct Merge?

❌ **Direct merge would fail because:**
- 959 commits divergence = massive conflicts
- Workspace structure completely reorganized
- FVM version incompatibility
- Build system changes throughout
- Many files moved/renamed/deleted

✅ **Incremental port advantages:**
- Control over what changes are adopted
- Easier to test each component
- Can adapt Recall code to new patterns
- Clear audit trail of changes
- Reduced risk of breaking main

---

## Step-by-Step Migration Plan

### Phase 0: Preparation (1-2 days)

**Goal:** Set up environment and understand scope

- [ ] **0.1** Create tracking branch from main: `git checkout -b recall-migration origin/main`
- [ ] **0.2** Document current test coverage on ipc-recall
- [ ] **0.3** Review FVM 4.4 → 4.7.4 changelogs
- [ ] **0.4** Review Iroh 0.34.x requirements and compatibility
- [ ] **0.5** Set up comparison testing environment
- [ ] **0.6** Create migration test plan document

### Phase 1: Core Dependencies (2-3 days)

**Goal:** Update low-level dependencies and utilities

#### Step 1.1: Update Recall IPLD Structures
```bash
# Port recall/ipld/ to new workspace
cp -r recall/ipld/ <target>/recall/ipld/
```

**Tasks:**
- [ ] Update `Cargo.toml` with FVM 4.7.4 dependencies
- [ ] Fix any IPLD API changes
- [ ] Run tests: `cargo test -p recall_ipld`
- [ ] Fix compilation errors
- [ ] Validate HAMT/AMT functionality

**Potential Issues:**
- `fvm_ipld_encoding` API changes
- `ActorError` type changes
- Blockstore interface updates

#### Step 1.2: Update Recall Kernel
```bash
cp -r recall/kernel/ <target>/recall/kernel/
```

**Tasks:**
- [ ] Update FVM dependencies to 4.7.4
- [ ] Update `RecallKernel` trait implementations
- [ ] Update syscall linker for new FVM
- [ ] Fix `block_add` operation if API changed
- [ ] Test kernel operations

**Potential Issues:**
- Kernel trait signature changes
- CallManager interface updates
- Gas charging changes

#### Step 1.3: Update Recall Syscalls
```bash
cp -r recall/syscalls/ <target>/recall/syscalls/
```

**Tasks:**
- [ ] Update FVM SDK to 4.7.4
- [ ] Verify syscall signature compatibility
- [ ] Update `delete_blob` implementation
- [ ] Test syscall registration

**Watch out for:**
- Syscall context parameter changes
- Memory access API updates

#### Step 1.4: Update Recall Actor SDK
```bash
cp -r recall/actor_sdk/ <target>/recall/actor_sdk/
```

**Tasks:**
- [ ] Update `fvm_sdk` to 4.7.4
- [ ] Fix actor calling conventions
- [ ] Update EVM integration if needed
- [ ] Test storage syscall wrapper

### Phase 2: Iroh Integration (2-3 days)

**Goal:** Ensure Iroh P2P layer works with target environment

#### Step 2.1: Verify Iroh Version
```bash
# Check if main has iroh
cd <main-workspace>
grep -r "iroh" Cargo.toml
```

**Tasks:**
- [ ] Determine if Iroh exists on main
- [ ] If not, add `iroh` and `iroh_blobs` dependencies to workspace
- [ ] Verify version compatibility (prefer 0.34.x or document upgrade needs)
- [ ] Test basic Iroh node creation

**Decision Point:**
- If main has no Iroh: Add it as new dependency
- If main has old Iroh: Determine upgrade path
- If main has newer Iroh: Update recall code

#### Step 2.2: Port Iroh Manager
```bash
cp -r recall/iroh_manager/ <target>/recall/iroh_manager/
```

**Tasks:**
- [ ] Update `Cargo.toml` dependencies
- [ ] Fix Iroh API compatibility issues
- [ ] Update relay configuration
- [ ] Test node creation and RPC server
- [ ] Validate blob upload/download

**Critical Tests:**
- [ ] Create persistent Iroh node
- [ ] Upload test blob
- [ ] Download blob from node ID
- [ ] RPC client connection
- [ ] Hash sequence operations

### Phase 3: Recall Executor (3-4 days)

**Goal:** Integrate custom executor with gas allowances

#### Step 3.1: Port Executor Code
```bash
cp -r recall/executor/ <target>/recall/executor/
```

**Tasks:**
- [ ] Update FVM dependencies
- [ ] Update `RecallExecutor` for FVM 4.7.4 API
- [ ] Fix `execute_message` signature changes
- [ ] Update gas calculation logic
- [ ] Fix `preflight_message` compatibility
- [ ] Test gas allowance system

**Key Integration Points:**
- [ ] Verify actor method invocation works
- [ ] Test gas charging with allowances
- [ ] Validate sponsor gas mechanics
- [ ] Ensure BLOBS_ACTOR integration

#### Step 3.2: Update Fendermint App Integration

**Tasks:**
- [ ] Update `fendermint/app/src/app.rs` to use RecallExecutor
- [ ] Pass IrohManager to app initialization
- [ ] Configure executor with engine pool
- [ ] Test message execution end-to-end

**Files to modify:**
- `fendermint/app/src/app.rs`
- `fendermint/app/src/cmd/run.rs`

### Phase 4: Actors (5-7 days)

**Goal:** Port and integrate all Recall actors

#### Step 4.1: Port Blobs Actor (Shared)
```bash
cp -r fendermint/actors/blobs/shared/ <target>/fendermint/actors/blobs/shared/
```

**Tasks:**
- [ ] Update `Cargo.toml`
- [ ] Fix dependency imports
- [ ] Compile shared types
- [ ] No test failures in shared

#### Step 4.2: Port Blobs Actor (Main)
```bash
cp -r fendermint/actors/blobs/src/ <target>/fendermint/actors/blobs/src/
```

**Tasks:**
- [ ] Update contract binding imports (sol_facade)
  - Fix path from `ipc_actors_abis` to new location
- [ ] Update actor registration in manifest
- [ ] Fix state serialization if needed
- [ ] Compile all actor methods
- [ ] Run actor unit tests

**Critical Files:**
- `src/actor.rs` - Main actor dispatcher
- `src/state.rs` - State management
- `src/sol_facade/blobs.rs` - Solidity interface

**Solidity Contract Updates:**
- [ ] Verify Solidity contracts exist in contracts/
- [ ] Update ABI paths if contracts moved
- [ ] Regenerate bindings if needed

#### Step 4.3: Port Bucket Actor
```bash
cp -r fendermint/actors/bucket/ <target>/fendermint/actors/bucket/
```

**Tasks:**
- [ ] Update imports
- [ ] Fix Solidity facade
- [ ] Test bucket operations

#### Step 4.4: Port Blob Reader Actor
```bash
cp -r fendermint/actors/blob_reader/ <target>/fendermint/actors/blob_reader/
```

**Tasks:**
- [ ] Update imports
- [ ] Fix query interfaces
- [ ] Test read operations

#### Step 4.5: Port Recall Config Actor
```bash
cp -r fendermint/actors/recall_config/ <target>/fendermint/actors/recall_config/
```

**Tasks:**
- [ ] Update imports
- [ ] Fix Solidity facade
- [ ] Test config read/write

#### Step 4.6: Update Actor Manifest
**File:** `fendermint/actors/src/manifest.rs`

**Tasks:**
- [ ] Add Recall actors to manifest
- [ ] Set correct actor codes (CIDs)
- [ ] Register in builtin actors list
- [ ] Update genesis initialization

**Example:**
```rust
pub const BLOBS_ACTOR_NAME: &str = "blobs";
pub const BUCKET_ACTOR_NAME: &str = "bucket";
pub const BLOB_READER_ACTOR_NAME: &str = "blob_reader";
pub const RECALL_CONFIG_ACTOR_NAME: &str = "recall_config";
```

#### Step 4.7: Update Actor Bundle Build
**File:** `fendermint/actors/build.rs`

**Tasks:**
- [ ] Ensure Recall actors included in bundle
- [ ] Test bundle generation
- [ ] Verify bundle.car contains Recall actors
- [ ] Test actor loading from bundle

### Phase 5: VM Integration (4-5 days)

**Goal:** Integrate blob resolution and vote tallying

#### Step 5.1: Port Iroh Resolver
```bash
cp -r fendermint/vm/iroh_resolver/ <target>/fendermint/vm/iroh_resolver/
```

**Tasks:**
- [ ] Update `Cargo.toml` workspace registration
- [ ] Fix import paths
- [ ] Update metrics registration (new observability scheme)
- [ ] Fix vote tally integration
- [ ] Update Iroh client usage
- [ ] Test resolution logic

**Files to update:**
- `src/iroh.rs` - Core resolution
- `src/pool.rs` - Task pool
- `src/observe.rs` - Metrics (update to new scheme)

#### Step 5.2: Update Vote Tally (if needed)
**File:** `fendermint/vm/topdown/src/voting.rs`

**Check:**
- [ ] Verify blob voting methods exist
- [ ] Ensure `VoteTally` has blob_votes field
- [ ] Test vote tallying logic

**If missing:**
- [ ] Port blob voting code from ipc-recall
- [ ] Add `add_blob_vote` and `find_blob_quorum`
- [ ] Update vote gossip protocol

#### Step 5.3: Update Chain Interpreter
**File:** `fendermint/vm/interpreter/src/chain.rs`

**Tasks:**
- [ ] Add blob pool fields to ChainEnv
- [ ] Import BlobPoolItem, PendingBlob, FinalizedBlob
- [ ] Add blob message handling in `propose()`
- [ ] Add blob message validation in `check()`
- [ ] Add blob finalization in `deliver()`
- [ ] Integrate with vote tally

**Key Sections:**
```rust
// In propose():
- Fetch added blobs from state
- Create BlobPending messages
- Fetch finalized blobs from pool
- Create BlobFinalized messages

// In check():
- Validate BlobFinalized has quorum
- Check blob not already finalized

// In deliver():
- Call blobs actor to finalize
- Remove from pool
```

#### Step 5.4: Update Message Types
**File:** `fendermint/vm/message/src/chain.rs`

**Tasks:**
- [ ] Add `ChainMessage::Ipc(IpcMessage::BlobPending(...))`
- [ ] Add `ChainMessage::Ipc(IpcMessage::BlobFinalized(...))`
- [ ] Update message serialization
- [ ] Test message encoding/decoding

**File:** `fendermint/vm/message/src/ipc.rs`

**Tasks:**
- [ ] Add `IpcMessage::BlobPending` variant
- [ ] Add `IpcMessage::BlobFinalized` variant
- [ ] Implement message type methods

#### Step 5.5: Update State Queries
**File:** `fendermint/vm/interpreter/src/fvm/state/query.rs`

**Tasks:**
- [ ] Add `get_added_blobs()` function
- [ ] Add `get_pending_blobs()` function
- [ ] Add `is_blob_finalized()` function
- [ ] Query blobs actor state correctly

### Phase 6: Genesis Integration (2-3 days)

**Goal:** Initialize Recall actors at genesis

#### Step 6.1: Update Genesis Configuration
**File:** `fendermint/vm/genesis/src/lib.rs`

**Tasks:**
- [ ] Add Recall actor initialization
- [ ] Set BLOBS_ACTOR_ID
- [ ] Configure initial credits
- [ ] Set storage capacity

#### Step 6.2: Test Genesis Creation
**Tasks:**
- [ ] Create test genesis with Recall
- [ ] Verify all actors initialized
- [ ] Check actor addresses assigned correctly
- [ ] Validate initial state

### Phase 7: Application Layer (2-3 days)

**Goal:** Integrate with fendermint application

#### Step 7.1: Update App Settings
**File:** `fendermint/app/settings/src/lib.rs`

**Tasks:**
- [ ] Add Recall configuration section
- [ ] Add blob concurrency settings
- [ ] Add Iroh node configuration
- [ ] Add resolver settings

#### Step 7.2: Update App Initialization
**File:** `fendermint/app/src/app.rs`

**Tasks:**
- [ ] Initialize IrohManager
- [ ] Start iroh resolver
- [ ] Configure blob pools
- [ ] Set up vote tally

#### Step 7.3: Add Objects API (Optional)
**File:** `fendermint/app/src/cmd/objects.rs`

**Tasks:**
- [ ] Port upload/download handlers
- [ ] Port entangler integration
- [ ] Add HTTP endpoints
- [ ] Test API functionality

### Phase 8: Contracts Integration (3-4 days)

**Goal:** Deploy and integrate Solidity contracts

#### Step 8.1: Port Solidity Contracts
**Directory:** `contracts/contracts/`

**Tasks:**
- [ ] Add Blobs.sol interface/facade
- [ ] Add Bucket.sol interface
- [ ] Add RecallConfig.sol interface
- [ ] Update contract compilation
- [ ] Generate ABI files

#### Step 8.2: Update Contract Bindings
**Directory:** `contracts/binding/`

**Tasks:**
- [ ] Update build.rs to include Recall contracts
- [ ] Generate Rust bindings
- [ ] Test binding imports in actors
- [ ] Verify error parsing

#### Step 8.3: Update Deployment Scripts
**Directory:** `contracts/tasks/`

**Tasks:**
- [ ] Add Recall actor deployment scripts (if needed)
- [ ] Update genesis task
- [ ] Test contract deployment
- [ ] Document deployment process

### Phase 9: Testing (5-7 days)

**Goal:** Comprehensive testing of integration

#### Step 9.1: Unit Tests
**Tasks:**
- [ ] Run all recall unit tests: `cargo test -p recall_*`
- [ ] Run actor tests: `cargo test -p fendermint_actor_blobs`
- [ ] Fix any failing tests
- [ ] Add new tests for integrations

#### Step 9.2: Integration Tests
**Tasks:**
- [ ] Create integration test for full upload flow
- [ ] Test blob resolution with vote tally
- [ ] Test blob finalization
- [ ] Test bucket operations
- [ ] Test credit system

**Test Scenarios:**
```rust
#[test]
async fn test_blob_upload_and_resolution() {
    // 1. Initialize network with Recall actors
    // 2. Upload blob to client's Iroh node
    // 3. Register blob with Blobs actor
    // 4. Validators fetch and vote
    // 5. Verify quorum reached
    // 6. Verify blob finalized on-chain
    // 7. Download blob from validator
}
```

#### Step 9.3: End-to-End Tests
**Tasks:**
- [ ] Deploy test subnet with Recall
- [ ] Upload real files
- [ ] Verify replication
- [ ] Test TTL expiry
- [ ] Test failure scenarios
- [ ] Test network partition recovery

#### Step 9.4: Performance Testing
**Tasks:**
- [ ] Benchmark upload throughput
- [ ] Test concurrent uploads
- [ ] Measure resolution latency
- [ ] Check memory usage
- [ ] Monitor gas consumption

### Phase 10: Documentation (2-3 days)

**Goal:** Document changes and usage

**Tasks:**
- [ ] Update main README with Recall features
- [ ] Document Recall actor APIs
- [ ] Create deployment guide
- [ ] Update CLI documentation (if added)
- [ ] Document configuration options
- [ ] Create troubleshooting guide
- [ ] Update architecture diagrams

---

## Testing Requirements

### Unit Test Coverage

**Minimum Requirements:**
- [ ] 80%+ code coverage for recall/ modules
- [ ] 90%+ coverage for critical paths (vote tally, state transitions)
- [ ] All actor methods have unit tests
- [ ] Edge cases tested (TTL expiry, vote equivocation, etc.)

### Integration Test Suites

#### 1. Blob Lifecycle Tests
```rust
- test_blob_add_and_subscribe()
- test_blob_resolution_success()
- test_blob_resolution_failure()
- test_blob_expiry()
- test_blob_overwrite()
```

#### 2. Vote Tally Tests
```rust
- test_vote_recording()
- test_quorum_calculation()
- test_equivocation_prevention()
- test_power_table_update()
```

#### 3. Credit System Tests
```rust
- test_gas_allowance_creation()
- test_gas_allowance_consumption()
- test_sponsored_transactions()
- test_allowance_expiry()
```

#### 4. Iroh Integration Tests
```rust
- test_iroh_node_initialization()
- test_blob_upload()
- test_blob_download()
- test_node_discovery()
- test_relay_connection()
```

### Regression Tests

**Must not break existing functionality:**
- [ ] IPC cross-net messaging still works
- [ ] Subnet creation/join unaffected
- [ ] Checkpoint submission works
- [ ] Gateway operations work
- [ ] All existing integration tests pass

### Performance Benchmarks

**Baseline Metrics to Maintain:**
- [ ] Block time: < 2s
- [ ] Transaction throughput: > 100 tx/s
- [ ] Memory usage: < 2GB per validator
- [ ] Sync time: < 30 min for 10k blocks

**New Recall Metrics:**
- [ ] Blob upload time: < 30s for 10MB
- [ ] Resolution time: < 60s for 10MB blob
- [ ] Vote propagation: < 5s
- [ ] Finalization latency: < 1 block after quorum

---

## Risk Assessment

### Critical Risks

#### 1. FVM API Incompatibility
**Risk Level:** 🔴 **HIGH**

**Impact:** Recall kernel/executor may not compile or work correctly

**Mitigation:**
- Thorough review of FVM 4.4→4.7 changelogs
- Create compatibility layer if needed
- Extensive testing of actor execution
- Have FVM experts review changes

**Contingency:**
- May need to stay on FVM 4.3 temporarily
- Create isolated branch for FVM upgrade
- Parallel track with stability fixes

#### 2. Iroh Version Mismatch
**Risk Level:** 🔴 **HIGH**

**Impact:** P2P blob transfer may fail completely

**Mitigation:**
- Test Iroh compatibility early (Phase 2)
- Have fallback plan for Iroh upgrade
- Maintain version compatibility matrix
- Test with real network conditions

**Contingency:**
- Bundle specific Iroh version
- Vendor Iroh dependencies if needed
- Consider alternative P2P layer

#### 3. State Serialization Breaking Changes
**Risk Level:** 🟡 **MEDIUM**

**Impact:** Cannot deserialize existing Recall state

**Mitigation:**
- Test state migrations explicitly
- Create state version detection
- Implement migration logic if needed
- Backup/restore testing

**Contingency:**
- Fresh genesis for Recall launch
- State migration scripts
- Parallel chain for testing

#### 4. Vote Tally Integration Issues
**Risk Level:** 🟡 **MEDIUM**

**Impact:** Blobs never reach quorum, network stalls

**Mitigation:**
- Extensive vote tally testing
- Simulate various validator scenarios
- Test network partition recovery
- Monitor vote metrics

**Contingency:**
- Temporary lower quorum for testing
- Manual intervention mechanisms
- Enhanced diagnostics

#### 5. Contract Binding Path Changes
**Risk Level:** 🟢 **LOW**

**Impact:** Compilation errors in Solidity facades

**Mitigation:**
- Update imports systematically
- Regenerate bindings
- Test contract interactions

**Contingency:**
- Simple find/replace for paths
- Straightforward to fix

### Migration Risks by Phase

| Phase | Risk Level | Key Concerns |
|-------|-----------|--------------|
| Phase 1: Core Dependencies | 🔴 HIGH | FVM compatibility |
| Phase 2: Iroh Integration | 🔴 HIGH | P2P functionality |
| Phase 3: Executor | 🟡 MEDIUM | Gas mechanics |
| Phase 4: Actors | 🟡 MEDIUM | State compatibility |
| Phase 5: VM Integration | 🟡 MEDIUM | Message handling |
| Phase 6: Genesis | 🟢 LOW | Initialization |
| Phase 7: Application | 🟢 LOW | Configuration |
| Phase 8: Contracts | 🟢 LOW | Path updates |
| Phase 9: Testing | 🟡 MEDIUM | Coverage gaps |
| Phase 10: Documentation | 🟢 LOW | Completeness |

---

## Rollback Plan

### Immediate Rollback (Day 1-7)
**Scenario:** Critical blocker discovered early

**Action:**
1. Abandon migration branch
2. Return to ipc-recall for continued development
3. Document blockers
4. Plan remediation

**Cost:** Minimal - early in migration

### Mid-Migration Rollback (Day 7-21)
**Scenario:** Unexpected complexity, delayed beyond timeline

**Action:**
1. Create snapshot of partial migration
2. Tag branch: `recall-migration-paused-YYYY-MM-DD`
3. Document completed phases
4. Return to ipc-recall temporarily
5. Plan revised approach

**Cost:** Moderate - partial work done

### Late Rollback (Day 21+)
**Scenario:** Integration issues found during final testing

**Action:**
1. Keep feature-flag disabled on main
2. Fix issues in migration branch
3. Retest thoroughly
4. Merge when ready

**Cost:** Higher - significant work invested

### Post-Merge Rollback
**Scenario:** Production issues after merge to main

**Action:**
1. **Immediate:** Disable Recall features via config
2. **Short-term:** Revert merge commit if critical
3. **Long-term:** Fix issues and re-enable

**Protection Mechanisms:**
- [ ] Feature flags for Recall components
- [ ] Configuration to disable Recall actors
- [ ] Separate test vs. production deployments
- [ ] Canary deployments

---

## Success Criteria

### Phase Completion Criteria

Each phase must meet these before proceeding:

✅ **All code compiles without warnings**
✅ **All unit tests pass**
✅ **No regressions in existing functionality**
✅ **Code reviewed and approved**
✅ **Documentation updated**

### Final Migration Acceptance

Migration is complete when:

- [ ] All Recall components integrated and working
- [ ] Full test suite passes (unit + integration + e2e)
- [ ] Performance benchmarks met
- [ ] Documentation complete
- [ ] Code reviewed by 2+ team members
- [ ] Production deployment plan approved
- [ ] Rollback procedures tested
- [ ] Monitoring and alerting configured

---

## Resource Requirements

### Team Composition

**Recommended Team:**
- 2-3 Senior Rust/FVM developers
- 1 Solidity developer (contracts)
- 1 DevOps engineer (deployment)
- 1 QA engineer (testing)

**Availability:**
- Full-time for 4-6 weeks
- Or part-time for 8-12 weeks

### Infrastructure

**Development:**
- [ ] Development testnet with 4-5 validators
- [ ] CI/CD pipeline for Recall branch
- [ ] Performance testing environment
- [ ] Staging environment

**Monitoring:**
- [ ] Metrics collection (Prometheus)
- [ ] Log aggregation (Loki/ELK)
- [ ] Distributed tracing
- [ ] Alerting (Alertmanager)

---

## Timeline Estimate

### Optimistic (Expert Team, No Blockers)
**4-5 weeks**

```
Week 1: Phases 0-2 (Prep, Core, Iroh)
Week 2: Phases 3-4 (Executor, Actors)
Week 3: Phases 5-6 (VM, Genesis)
Week 4: Phases 7-8 (App, Contracts)
Week 5: Phases 9-10 (Testing, Docs)
```

### Realistic (Experienced Team, Minor Issues)
**6-8 weeks**

```
Weeks 1-2: Phases 0-3
Weeks 3-4: Phases 4-5
Weeks 5-6: Phases 6-8
Weeks 7-8: Phases 9-10 + Buffer
```

### Conservative (Learning Required, Major Issues)
**10-12 weeks**

```
Weeks 1-3: Phases 0-3 + FVM learning
Weeks 4-6: Phases 4-5 + Issue resolution
Weeks 7-9: Phases 6-8
Weeks 10-12: Phases 9-10 + Hardening
```

---

## Next Steps

### Immediate Actions (This Week)

1. **Decision:** Approve migration approach
2. **Staffing:** Assign team members
3. **Setup:** Create migration branch from main
4. **Kickoff:** Phase 0 preparation tasks
5. **Communication:** Notify stakeholders

### Before Starting Phase 1

- [ ] Review this document with full team
- [ ] Set up project tracking (Jira/GitHub Projects)
- [ ] Create test environment
- [ ] Schedule daily standups
- [ ] Establish code review process
- [ ] Define success metrics
- [ ] Create risk register

### Key Decisions Needed

1. **FVM Strategy:** Stay on 4.3 temporarily or upgrade immediately?
2. **Iroh Version:** Which version to target?
3. **Genesis Approach:** Fresh genesis or state migration?
4. **Deployment:** Testnet first or devnet?
5. **Timeline:** Which estimate (optimistic/realistic/conservative)?

---

## Appendix

### A. Key Files Changed on Main (Sample)

```
High Impact:
- Cargo.toml (workspace reorganization)
- fendermint/actors/src/manifest.rs (actor registration)
- fendermint/app/src/app.rs (app initialization)
- fendermint/vm/interpreter/src/chain.rs (message handling)

Medium Impact:
- fendermint/vm/genesis/src/lib.rs (genesis flow)
- contracts/binding/build.rs (contract bindings)
- fendermint/actors/build.rs (actor bundle)

Low Impact:
- Various Cargo.toml version bumps
- CI/CD configuration
- Documentation files
```

### B. Recall Dependencies

```toml
# Core Dependencies
fvm = "4.3.0" → "4.7.4"
fvm_shared = "4.3.0" → "4.7.4"
fvm_sdk = "4.3.0" → "4.7.4"
fvm_ipld_* = "0.2" → Check main version

# Iroh Dependencies
iroh = "0.34.x"
iroh_blobs = "0.34.x"
quic_rpc = "0.14"

# Async Runtime
tokio = "1.x"
async-trait = "0.1"
futures = "0.3"

# Serialization
serde = "1.0"
fvm_ipld_encoding = "0.4"
```

### C. Useful Commands

```bash
# Check diff between branches
git diff main..ipc-recall --stat

# Find all Recall-specific files
find . -name "*blob*" -o -name "*recall*" -o -name "*iroh*"

# Count lines of Recall code
cloc recall/ fendermint/actors/blob* fendermint/vm/iroh_resolver/

# Test specific component
cargo test -p recall_kernel -- --nocapture

# Check for FVM API usage
rg "fvm::" --type rust | wc -l

# Find all actor registrations
rg "register_actor|ACTOR_ID" fendermint/actors/
```

### D. Contact Points

**For Questions:**
- FVM compatibility: Review FVM repo issues/discussions
- Iroh integration: Check Iroh documentation
- Actor patterns: Reference other actors in fendermint/actors/
- Vote tally: See fendermint/vm/topdown/src/voting.rs

---

## Conclusion

The migration of Recall storage from ipc-recall to main is a **significant undertaking** requiring 4-12 weeks depending on team experience and issues encountered. The incremental approach outlined here minimizes risk while providing clear checkpoints.

**Key Success Factors:**
1. Strong Rust/FVM expertise on the team
2. Thorough testing at each phase
3. Early identification of blockers (FVM, Iroh)
4. Clear communication and decision-making
5. Realistic timeline expectations

**Go/No-Go Decision Points:**
- ✋ **After Phase 2:** If Iroh integration blocked, pause and reassess
- ✋ **After Phase 3:** If FVM executor broken, may need FVM expert consultation
- ✋ **After Phase 5:** If VM integration issues, consider architectural changes

With proper planning and execution, Recall storage can be successfully integrated into main, bringing decentralized storage capabilities to the IPC network.

---

**Document Version:** 1.0
**Last Updated:** 2024-11-04
**Status:** Draft for Review
**Next Review:** After Phase 0 completion

