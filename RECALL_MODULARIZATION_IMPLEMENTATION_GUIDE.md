# Storage Node Modularization - Implementation Guide

**Purpose:** Step-by-step guide to make storage-node an optional compile-time feature with complete renaming from "recall/basin" to "storage-node".

**Estimated Total Time:** 3-4 weeks (includes renaming)
**Difficulty:** Medium
**Risk Level:** Low-Medium (well-contained changes, breaking change acceptable)

---

## Table of Contents

0. [Phase 0: Renaming Strategy](#phase-0-renaming-strategy)
1. [Prerequisites](#prerequisites)
2. [Phase 1: Directory and Crate Renaming](#phase-1-directory-and-crate-renaming)
3. [Phase 2: Feature Flag Architecture](#phase-2-feature-flag-architecture)
4. [Phase 3: Gate Core Components](#phase-3-gate-core-components)
5. [Phase 4: Gate Integration Points](#phase-4-gate-integration-points)
6. [Phase 5: Testing & Validation](#phase-5-testing--validation)
7. [Phase 6: CI/CD Updates](#phase-6-cicd-updates)
8. [Troubleshooting](#troubleshooting)

---

## Phase 0: Renaming Strategy

**Goal:** Define comprehensive renaming from "recall/basin" to "storage-node"
**Time Estimate:** N/A (planning phase)
**Risk:** None

### Renaming Map

#### Directory Structure
- `recall/` → `storage-node/`
- `ipc-decentralized-storage/` → `storage-services/`
- `recall-contracts/` → `storage-node-contracts/`
- `fendermint/actors/adm/` → `fendermint/actors/storage_adm/`
- `fendermint/actors/blobs/` → `fendermint/actors/storage_blobs/`
- `fendermint/actors/blob_reader/` → `fendermint/actors/storage_blob_reader/`
- `fendermint/actors/bucket/` → `fendermint/actors/storage_bucket/`
- `fendermint/actors/timehub/` → `fendermint/actors/storage_timehub/`
- `fendermint/actors/recall_config/` → `fendermint/actors/storage_config/`

#### Crate Names (in Cargo.toml `name` field)
- `recall_kernel` → `storage_node_kernel`
- `recall_kernel_ops` → `storage_node_kernel_ops`
- `recall_syscalls` → `storage_node_syscalls`
- `recall_executor` → `storage_node_executor`
- `recall_ipld` → `storage_node_ipld`
- `iroh_manager` → `storage_node_iroh_manager`
- `recall_actor_sdk` → `storage_node_actor_sdk`
- `ipc-decentralized-storage` → `storage-services`
- `fendermint_actor_adm` → `fendermint_actor_storage_adm`
- `fendermint_actor_adm_types` → `fendermint_actor_storage_adm_types`
- `fendermint_actor_blobs` → `fendermint_actor_storage_blobs`
- `fendermint_actor_blobs_shared` → `fendermint_actor_storage_blobs_shared`
- `fendermint_actor_blobs_testing` → `fendermint_actor_storage_blobs_testing`
- `fendermint_actor_blob_reader` → `fendermint_actor_storage_blob_reader`
- `fendermint_actor_bucket` → `fendermint_actor_storage_bucket`
- `fendermint_actor_timehub` → `fendermint_actor_storage_timehub`
- `fendermint_actor_recall_config` → `fendermint_actor_storage_config`
- `fendermint_actor_recall_config_shared` → `fendermint_actor_storage_config_shared`

#### Feature Flags
- `recall-storage` → `storage-node`
- `recall-core` → `storage-node-core`
- `recall-actors` → `storage-node-actors`
- `recall-http-api` → `storage-node-http-api`

#### Module Names (in code)
- `use recall_kernel` → `use storage_node_kernel`
- `use recall_executor` → `use storage_node_executor`
- `mod recall_env` → `mod storage_env`
- `mod recall_helpers` → `mod storage_helpers`
- `pub mod objects` → `pub mod storage_node` (CLI command)

#### Type/Struct Names to Consider
- `ReadRequestPool` → keep as-is (internal implementation detail)
- `RecallConfig` → `StorageConfig`
- `IrohManager` → keep as-is (it's about Iroh, not recall)
- Message types like `ReadRequestPending` → keep as-is (internal)

#### On-Chain Actor Names (KEEP AS-IS for compatibility)
- `BLOBS_ACTOR_NAME = "blobs"` - DO NOT CHANGE
- `ADM_ACTOR_NAME = "adm"` - DO NOT CHANGE
- `BUCKET_ACTOR_NAME = "bucket"` - DO NOT CHANGE
- Actor IDs (90, 99, 100, 101) - DO NOT CHANGE

#### Documentation Files
- `RECALL_*.md` → `STORAGE_NODE_*.md`
- `docs/ipc/recall-*.md` → `docs/ipc/storage-node-*.md`

#### CLI Commands
- `fendermint objects` → `fendermint storage-node`
- Subcommands remain the same (run, etc.)

### What NOT to Rename
1. **Actor IDs and on-chain names** - maintain chain compatibility
2. **Iroh-specific types** - `IrohManager`, `iroh_blobs::Hash`, etc.
3. **Internal implementation details** that don't leak to public API
4. **Third-party dependency names** - `iroh`, `warp`, etc.

---

## Prerequisites

### Required Knowledge
- Rust feature flags and conditional compilation
- Cargo workspace management
- IPC architecture basics
- Git branching strategy

### Tools Required
- Rust toolchain (matching project version)
- Git
- Text editor with Rust support
- CI/CD access (for final phase)

### Recommended Reading
- [Cargo Features Documentation](https://doc.rust-lang.org/cargo/reference/features.html)
- [Conditional Compilation in Rust](https://doc.rust-lang.org/reference/conditional-compilation.html)
- `RECALL_STORAGE_MODULARIZATION_ANALYSIS.md` (this repo)

---

## Phase 1: Directory and Crate Renaming

**Goal:** Rename all directories, crates, and update imports
**Time Estimate:** 2-3 days
**Risk:** Medium (many file moves and import updates)

### Step 1.1: Rename Core Directories

**Commands:**

```bash
# Rename main storage-node directory
git mv recall storage-node

# Rename standalone services
git mv ipc-decentralized-storage storage-services

# Rename contracts
git mv recall-contracts storage-node-contracts

# Rename actor directories
git mv fendermint/actors/adm fendermint/actors/storage_adm
git mv fendermint/actors/blobs fendermint/actors/storage_blobs
git mv fendermint/actors/blob_reader fendermint/actors/storage_blob_reader
git mv fendermint/actors/bucket fendermint/actors/storage_bucket
git mv fendermint/actors/timehub fendermint/actors/storage_timehub
git mv fendermint/actors/recall_config fendermint/actors/storage_config

# Rename VM modules
git mv fendermint/vm/iroh_resolver fendermint/vm/storage_resolver
```

### Step 1.2: Update Crate Names in Cargo.toml Files

Update each `Cargo.toml` file's `[package] name` field:

**Files to update:**
- `storage-node/kernel/Cargo.toml`: `recall_kernel` → `storage_node_kernel`
- `storage-node/kernel/ops/Cargo.toml`: `recall_kernel_ops` → `storage_node_kernel_ops`
- `storage-node/syscalls/Cargo.toml`: `recall_syscalls` → `storage_node_syscalls`
- `storage-node/executor/Cargo.toml`: `recall_executor` → `storage_node_executor`
- `storage-node/ipld/Cargo.toml`: `recall_ipld` → `storage_node_ipld`
- `storage-node/iroh_manager/Cargo.toml`: `iroh_manager` → `storage_node_iroh_manager`
- `storage-node/actor_sdk/Cargo.toml`: `recall_actor_sdk` → `storage_node_actor_sdk`
- `storage-services/Cargo.toml`: `ipc-decentralized-storage` → `storage-services`
- All actor `Cargo.toml` files: add `storage_` prefix

### Step 1.3: Update Workspace Members in Root Cargo.toml

**File:** `/Cargo.toml`

Update the `[workspace.members]` section:

```toml
[workspace.members]
# ... existing members ...

# Storage node components (formerly recall)
"storage-node/kernel",
"storage-node/kernel/ops",
"storage-node/syscalls",
"storage-node/executor",
"storage-node/iroh_manager",
"storage-node/ipld",
"storage-node/actor_sdk",

# Storage node actors (formerly recall actors)
"fendermint/actors/storage_adm",
"fendermint/actors/storage_adm/types",
"fendermint/actors/storage_blobs",
"fendermint/actors/storage_blobs/shared",
"fendermint/actors/storage_blobs/testing",
"fendermint/actors/storage_blob_reader",
"fendermint/actors/storage_bucket",
"fendermint/actors/storage_timehub",
"fendermint/actors/storage_config",
"fendermint/actors/storage_config/shared",

# Storage node contracts (formerly recall-contracts)
"storage-node-contracts/crates/facade",

# Standalone storage services (formerly ipc-decentralized-storage)
"storage-services",

# ... other members ...
]
```

### Step 1.4: Global Import Updates

Use find-and-replace across the workspace for import statements:

**Search and replace patterns:**
- `use recall_kernel` → `use storage_node_kernel`
- `use recall_executor` → `use storage_node_executor`
- `use recall_syscalls` → `use storage_node_syscalls`
- `use recall_ipld` → `use storage_node_ipld`
- `use recall_actor_sdk` → `use storage_node_actor_sdk`
- `use iroh_manager` → `use storage_node_iroh_manager`
- `path = "../recall/` → `path = "../storage-node/`
- `path = "../../recall/` → `path = "../../storage-node/`
- `path = "../../../recall/` → `path = "../../../storage-node/`
- `fendermint_actor_adm` → `fendermint_actor_storage_adm`
- `fendermint_actor_blobs` → `fendermint_actor_storage_blobs`
- `fendermint_actor_blob_reader` → `fendermint_actor_storage_blob_reader`
- `fendermint_actor_bucket` → `fendermint_actor_storage_bucket`
- `fendermint_actor_timehub` → `fendermint_actor_storage_timehub`
- `fendermint_actor_recall_config` → `fendermint_actor_storage_config`
- `fendermint_vm_iroh_resolver` → `fendermint_vm_storage_resolver`

### Step 1.5: Update Type Names

**Search and replace for public types:**
- `RecallConfig` → `StorageConfig`
- `recall_config::` → `storage_config::`
- `pub mod recall_env` → `pub mod storage_env`
- `pub mod recall_helpers` → `pub mod storage_helpers`

### Step 1.6: Test Compilation After Renaming

```bash
# Should compile with new names
cargo check --workspace

# Fix any remaining import errors manually
# Look for errors about missing crates or modules
```

**Expected Result:** All references updated, workspace compiles with new names.

---

## Phase 2: Feature Flag Architecture

**Goal:** Set up feature flags for the renamed components
**Time Estimate:** 1-2 days
**Risk:** Low

### Step 2.1: Update Root Cargo.toml

**File:** `/Cargo.toml`

Add feature definitions to the workspace:

```toml
[workspace]
# ... existing workspace config ...

# Add this section at the end of the file
[workspace.metadata.docs.rs]
all-features = true

[features]
default = []

# Full storage node support
storage-node = [
    "storage-node-core",
    "storage-node-actors",
    "storage-node-http-api",
]

# Core storage node runtime
storage-node-core = []

# On-chain actors
storage-node-actors = ["storage-node-core"]

# HTTP Objects API
storage-node-http-api = ["storage-node-core"]
```

**Note:** We'll populate these feature arrays in subsequent steps.

### Step 2.2: Make Storage Node Dependencies Optional

**File:** `/Cargo.toml` (workspace.dependencies section)

Update storage-node-related dependencies:

```toml
[workspace.dependencies]
# ... existing dependencies ...

# Storage node/Iroh dependencies (make optional)
ambassador = { version = "0.3.5", optional = true }
iroh = { version = "0.35", optional = true }
iroh-base = { version = "0.35", optional = true }
iroh-blobs = { version = "0.35", features = ["rpc"], optional = true }
iroh-relay = { version = "0.35", optional = true }
iroh-quinn = { version = "0.13", optional = true }
n0-future = { version = "0.1.2", optional = true }
quic-rpc = { version = "0.20", features = ["quinn-transport"], optional = true }

# HTTP API dependencies (make optional)
warp = { version = "0.3", optional = true }
uuid = { version = "1.0", features = ["v4"], optional = true }
mime_guess = { version = "2.0", optional = true }
urlencoding = { version = "2.1", optional = true }
entangler = { version = "0.1", optional = true }
entangler_storage = { version = "0.1", optional = true }
```

### Step 2.3: Test Build Without Changes

```bash
# Should still build normally after renaming
cargo build --workspace
cargo test --workspace

# Verify feature flag syntax
cargo build --features storage-node
```

**Expected Result:** Everything builds with new names.

---

## Phase 3: Gate Core Components

**Goal:** Make storage-node modules optional via feature flags
**Time Estimate:** 2-3 days
**Risk:** Low-Medium

### Step 3.1: Gate Storage Node Core Modules

For each crate in `storage-node/`:

#### File: `storage-node/kernel/Cargo.toml`

```toml
[package]
name = "storage_node_kernel"
# ... existing config ...

[features]
# No default features
default = []

[dependencies]
storage_node_kernel_ops = { path = "../kernel/ops" }
storage_node_syscalls = { path = "../syscalls" }
# ... rest of dependencies ...
```

#### File: `storage-node/executor/Cargo.toml`

```toml
[package]
name = "storage_node_executor"
# ... existing config ...

[dependencies]
storage_node_kernel = { path = "../kernel" }
# ... rest of dependencies ...
```

**Repeat for:**
- `storage-node/syscalls/Cargo.toml`
- `storage-node/ipld/Cargo.toml`
- `storage-node/iroh_manager/Cargo.toml`
- `storage-node/actor_sdk/Cargo.toml`

### Step 3.2: Gate Storage Node Actors

For each actor in `fendermint/actors/storage_*`:

#### File: `fendermint/actors/storage_blobs/Cargo.toml`

```toml
[package]
name = "fendermint_actor_storage_blobs"
# ... existing config ...

[features]
default = []

[dependencies]
fendermint_actor_storage_blobs_shared = { path = "./shared" }
# ... rest of dependencies ...
```

#### File: `fendermint/actors/storage_blob_reader/Cargo.toml`

```toml
[package]
name = "fendermint_actor_storage_blob_reader"
# ... existing config ...

[features]
default = []

[dependencies]
fendermint_actor_storage_blobs_shared = { path = "../storage_blobs/shared" }
# ... rest of dependencies ...
```

**Repeat for:**
- `fendermint/actors/storage_config/Cargo.toml`
- `fendermint/actors/storage_bucket/Cargo.toml`
- `fendermint/actors/storage_timehub/Cargo.toml`
- `fendermint/actors/storage_adm/Cargo.toml`

### Step 3.3: Update fendermint/app/Cargo.toml

**File:** `fendermint/app/Cargo.toml`

```toml
[package]
name = "fendermint_app"
# ... existing config ...

[features]
default = []
storage-node = [
    "dep:warp",
    "dep:uuid",
    "dep:mime_guess",
    "dep:urlencoding",
    "dep:entangler",
    "dep:entangler_storage",
    "dep:storage_node_iroh_manager",
    "dep:iroh",
    "dep:iroh-blobs",
    "dep:fendermint_actor_storage_bucket",
    "dep:fendermint_actor_storage_blobs_shared",
    "dep:fendermint_vm_storage_resolver",
]

[dependencies]
# ... existing dependencies ...

# Storage node HTTP API dependencies (now optional)
warp = { workspace = true, optional = true }
uuid = { workspace = true, optional = true }
mime_guess = { workspace = true, optional = true }
urlencoding = { workspace = true, optional = true }
entangler = { workspace = true, optional = true }
entangler_storage = { workspace = true, optional = true }
storage_node_iroh_manager = { path = "../../storage-node/iroh_manager", optional = true }
iroh = { workspace = true, optional = true }
iroh-blobs = { workspace = true, optional = true }
fendermint_actor_storage_bucket = { path = "../actors/storage_bucket", optional = true }
fendermint_actor_storage_blobs_shared = { path = "../actors/storage_blobs/shared", optional = true }
fendermint_vm_storage_resolver = { path = "../vm/storage_resolver", optional = true }
```

### Step 3.4: Update fendermint/vm/interpreter/Cargo.toml

**File:** `fendermint/vm/interpreter/Cargo.toml`

```toml
[package]
name = "fendermint_vm_interpreter"
# ... existing config ...

[features]
default = []
storage-node = [
    "dep:storage_node_executor",
    "dep:storage_node_kernel",
    "dep:fendermint_actor_storage_adm",
    "dep:fendermint_actor_storage_blobs",
    "dep:fendermint_actor_storage_blobs_shared",
    "dep:fendermint_actor_storage_blob_reader",
    "dep:fendermint_actor_storage_config",
    "dep:fendermint_actor_storage_config_shared",
    "dep:fendermint_vm_storage_resolver",
    "dep:iroh",
    "dep:iroh-blobs",
]

[dependencies]
# ... existing dependencies ...

# Storage node dependencies (now optional)
fendermint_actor_storage_adm = { path = "../../actors/storage_adm", optional = true }
fendermint_actor_storage_blobs = { path = "../../actors/storage_blobs", optional = true }
fendermint_actor_storage_blobs_shared = { path = "../../actors/storage_blobs/shared", optional = true }
fendermint_actor_storage_blob_reader = { path = "../../actors/storage_blob_reader", optional = true }
fendermint_actor_storage_config = { path = "../../actors/storage_config", optional = true }
fendermint_actor_storage_config_shared = { path = "../../actors/storage_config/shared", optional = true }
storage_node_executor = { path = "../../../storage-node/executor", optional = true }
storage_node_kernel = { path = "../../../storage-node/kernel", optional = true }
fendermint_vm_storage_resolver = { path = "../storage_resolver", optional = true }
iroh = { workspace = true, optional = true }
iroh-blobs = { workspace = true, optional = true }
```

### Step 3.5: Test Compilation

```bash
# Test without storage-node (should fail - expected at this stage)
cargo build --workspace

# Test with storage-node
cargo build --workspace --features storage-node

# Test individual crates
cargo build -p fendermint_app
cargo build -p fendermint_app --features storage-node
```

---

## Phase 4: Gate Integration Points

**Goal:** Add conditional compilation directives to code
**Time Estimate:** 3-5 days
**Risk:** Medium

### Step 4.1: Gate Message Type Extensions

**File:** `fendermint/vm/message/src/ipc.rs`

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IpcMessage {
    // Existing variants
    BottomUpExec(BottomUpCheckpoint),
    TopDownExec(TopDownExec),
    // ... other variants ...

    // Storage node-specific variants
    #[cfg(feature = "storage-node")]
    #[serde(rename = "read_request_pending")]
    ReadRequestPending(ReadRequest),

    #[cfg(feature = "storage-node")]
    #[serde(rename = "read_request_closed")]
    ReadRequestClosed(ReadRequest),
}

// Add conditional import
#[cfg(feature = "storage-node")]
pub use crate::read_request::ReadRequest;

// Create new module (gated)
#[cfg(feature = "storage-node")]
pub mod read_request {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ReadRequest {
        pub id: Hash,
        // ... fields ...
    }
}
```

### Step 4.2: Gate Message Handlers

**File:** `fendermint/vm/interpreter/src/fvm/interpreter.rs`

At the top of the file:

```rust
// Conditional imports
#[cfg(feature = "storage-node")]
use crate::fvm::storage_env::ReadRequestPool;
#[cfg(feature = "storage-node")]
use crate::fvm::storage_helpers::{
    close_read_request, read_request_callback, set_read_request_pending,
};
```

In the message handling code:

```rust
impl<DB, TC> ChainMessageInterpreter<...> for FvmMessageInterpreter<...> {
    async fn apply(&self, msg: ChainMessage) -> Result<ApplyMessageResponse> {
        match msg {
            ChainMessage::Ipc(ipc_msg) => match ipc_msg {
                // Existing handlers...

                // Storage node handlers (gated)
                #[cfg(feature = "storage-node")]
                IpcMessage::ReadRequestPending(read_request) => {
                    let ret = set_read_request_pending(state, read_request.id)?;
                    tracing::debug!(
                        request_id = %read_request.id,
                        "chain interpreter has set read request to pending"
                    );
                    Ok(ApplyMessageResponse {
                        applied_message: ret.into(),
                        domain_hash: None,
                    })
                }

                #[cfg(feature = "storage-node")]
                IpcMessage::ReadRequestClosed(read_request) => {
                    read_request_callback(state, &read_request)?;
                    let ret = close_read_request(state, read_request.id)?;
                    tracing::debug!(
                        hash = %read_request.id,
                        "chain interpreter has closed read request"
                    );
                    Ok(ApplyMessageResponse {
                        applied_message: ret.into(),
                        domain_hash: None,
                    })
                }
            },

            // Other message types...
        }
    }
}
```

### Step 4.3: Gate Genesis Initialization

**File:** `fendermint/vm/interpreter/src/genesis.rs`

Add conditional imports:

```rust
#[cfg(feature = "storage-node")]
use fendermint_vm_actor_interface::{storage_adm, storage_blob_reader, storage_blobs, storage_config};
```

In the genesis builder:

```rust
impl<'a> GenesisBuilder<'a> {
    pub fn build(&mut self) -> Result<()> {
        // ... existing actor initialization ...

        // Storage node actors (conditional)
        #[cfg(feature = "storage-node")]
        self.initialize_storage_actors()?;

        Ok(())
    }

    #[cfg(feature = "storage-node")]
    fn initialize_storage_actors(&mut self) -> Result<()> {
        // ADM actor
        let mut machine_codes = std::collections::HashMap::new();
        for machine_name in &["bucket", "timehub"] {
            if let Some(cid) = self.state.custom_actor_manifest.code_by_name(machine_name) {
                let kind = fendermint_actor_storage_adm::Kind::from_str(machine_name)?;
                machine_codes.insert(kind, *cid);
            }
        }
        let adm_state = fendermint_actor_storage_adm::State::new(
            self.state.store(),
            machine_codes,
            fendermint_actor_storage_adm::PermissionModeParams::Unrestricted,
        )?;
        self.state.create_custom_actor(
            fendermint_vm_actor_interface::storage_adm::ADM_ACTOR_NAME,
            storage_adm::ADM_ACTOR_ID,
            &adm_state,
            TokenAmount::zero(),
            None,
        )?;

        // Storage config actor
        let storage_config_state = fendermint_actor_storage_config::State {
            admin: None,
            config: fendermint_actor_storage_config_shared::StorageConfig::default(),
        };
        self.state.create_custom_actor(
            fendermint_actor_storage_config::ACTOR_NAME,
            storage_config::STORAGE_CONFIG_ACTOR_ID,
            &storage_config_state,
            TokenAmount::zero(),
            None,
        )?;

        // Blobs actor (with delegated address)
        let blobs_state = fendermint_actor_storage_blobs::State::new(&self.state.store())?;
        let blobs_eth_addr = init::builtin_actor_eth_addr(storage_blobs::BLOBS_ACTOR_ID);
        let blobs_f4_addr = fvm_shared::address::Address::from(blobs_eth_addr);
        self.state.create_custom_actor(
            fendermint_actor_storage_blobs::BLOBS_ACTOR_NAME,
            storage_blobs::BLOBS_ACTOR_ID,
            &blobs_state,
            TokenAmount::zero(),
            Some(blobs_f4_addr),
        )?;

        // Blob reader actor
        self.state.create_custom_actor(
            fendermint_actor_storage_blob_reader::BLOB_READER_ACTOR_NAME,
            storage_blob_reader::BLOB_READER_ACTOR_ID,
            &fendermint_actor_storage_blob_reader::State::new(&self.state.store())?,
            TokenAmount::zero(),
            None,
        )?;

        Ok(())
    }
}
```

### Step 4.4: Gate Storage Node HTTP Command

**File:** `fendermint/app/src/cmd/mod.rs`

```rust
pub mod genesis;
pub mod key;
pub mod materialize;
pub mod run;
pub mod rpc;

// Storage node command (conditional)
#[cfg(feature = "storage-node")]
pub mod storage_node;

#[derive(Debug, Parser)]
pub enum Commands {
    Genesis(genesis::GenesisCmd),
    Key(key::KeyCmd),
    Materialize(materialize::MaterializeCmd),
    Run(run::RunCmd),
    Rpc(rpc::RpcCmd),

    #[cfg(feature = "storage-node")]
    #[command(about = "Run storage node HTTP API for blob storage")]
    StorageNode(storage_node::StorageNodeCmd),
}

impl Commands {
    pub async fn exec(self, ...) -> anyhow::Result<()> {
        match self {
            Commands::Genesis(cmd) => cmd.exec(...).await,
            Commands::Key(cmd) => cmd.exec(...),
            Commands::Materialize(cmd) => cmd.exec(...).await,
            Commands::Run(cmd) => cmd.exec(...).await,
            Commands::Rpc(cmd) => cmd.exec(...).await,

            #[cfg(feature = "storage-node")]
            Commands::StorageNode(cmd) => cmd.exec(...).await,
        }
    }
}
```

### Step 4.5: Gate Vote Tally Extensions

**File:** `fendermint/vm/topdown/src/voting.rs`

```rust
use std::collections::{HashMap, HashSet};

#[cfg(feature = "storage-node")]
use iroh_blobs::Hash as BlobHash;

pub struct VoteTally<V> {
    // Existing fields...

    #[cfg(feature = "storage-node")]
    blob_votes: HashMap<BlobHash, HashSet<V>>,
}

impl<V: ValidatorKey> VoteTally<V> {
    // Existing methods...

    #[cfg(feature = "storage-node")]
    pub fn add_blob_vote(&mut self, validator: V, hash: BlobHash) {
        self.blob_votes
            .entry(hash)
            .or_insert_with(HashSet::new)
            .insert(validator);
    }

    #[cfg(feature = "storage-node")]
    pub fn find_blob_quorum(&self) -> Option<BlobHash> {
        let threshold = self.power_table.threshold();

        for (hash, validators) in &self.blob_votes {
            let power: u64 = validators
                .iter()
                .filter_map(|v| self.power_table.get_power(v))
                .sum();

            if power >= threshold {
                return Some(*hash);
            }
        }

        None
    }
}
```

### Step 4.6: Gate Storage Resolver Integration

**File:** `ipld/resolver/src/client.rs`

```rust
#[cfg(feature = "storage-node")]
use iroh::{NodeAddr};
#[cfg(feature = "storage-node")]
use iroh_blobs::Hash;

// Existing Resolver trait...

#[cfg(feature = "storage-node")]
#[async_trait]
pub trait ResolverIroh {
    async fn resolve_iroh(
        &self,
        hash: Hash,
        size: u64,
        node_addr: NodeAddr,
    ) -> anyhow::Result<ResolveResult>;
}

#[cfg(feature = "storage-node")]
#[async_trait]
impl<V> ResolverIroh for Client<V>
where
    V: Sync + Send + 'static,
{
    async fn resolve_iroh(
        &self,
        hash: Hash,
        size: u64,
        node_addr: NodeAddr,
    ) -> anyhow::Result<ResolveResult> {
        let (tx, rx) = oneshot::channel();
        let req = Request::ResolveIroh(hash, size, node_addr, tx);
        self.send_request(req)?;
        let res = rx.await?;
        Ok(res)
    }
}
```

**File:** `ipld/resolver/src/service.rs`

```rust
pub struct Service<S, V> {
    // Existing fields...

    #[cfg(feature = "storage-node")]
    iroh_manager: Option<IrohManager>,
}

impl<S, V> Service<S, V> {
    pub async fn new(config: Config) -> Result<Self> {
        // Existing initialization...

        #[cfg(feature = "storage-node")]
        let iroh_manager = if let Some(iroh_config) = config.iroh {
            Some(IrohManager::new(iroh_config).await?)
        } else {
            None
        };

        Ok(Self {
            // ... existing fields ...
            #[cfg(feature = "storage-node")]
            iroh_manager,
        })
    }

    async fn handle_request(&mut self, req: Request<V>) {
        match req {
            // Existing handlers...

            #[cfg(feature = "storage-node")]
            Request::ResolveIroh(hash, size, node_addr, tx) => {
                let result = if let Some(ref manager) = self.iroh_manager {
                    manager.download_blob(hash, size, node_addr).await
                } else {
                    Err(anyhow!("Iroh not enabled"))
                };
                let _ = tx.send(result);
            }
        }
    }
}
```

### Step 4.7: Test Compilation

```bash
# Test without storage-node - should now compile!
cargo build --workspace

# Test with storage-node
cargo build --workspace --features storage-node

# Test individual components
cargo build -p fendermint_app
cargo build -p fendermint_app --features storage-node
cargo build -p fendermint_vm_interpreter
cargo build -p fendermint_vm_interpreter --features storage-node
```

---

## Phase 5: Testing & Validation

**Goal:** Ensure both configurations work correctly
**Time Estimate:** 5-7 days
**Risk:** Medium-High

### Step 5.1: Unit Tests

Add conditional test gating where needed:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Tests that work without storage-node
    #[test]
    fn test_standard_functionality() {
        // ...
    }

    // Tests that require storage-node
    #[cfg(feature = "storage-node")]
    #[test]
    fn test_blob_operations() {
        // ...
    }
}
```

### Step 5.2: Run Test Suites

```bash
# Test without storage-node
cargo test --workspace

# Test with storage-node
cargo test --workspace --features storage-node

# Test specific crates
cargo test -p fendermint_vm_interpreter
cargo test -p fendermint_vm_interpreter --features storage-node

# Test all feature combinations (comprehensive)
cargo test --workspace --all-features
cargo test --workspace --no-default-features
```

### Step 5.3: Integration Tests

Create test script:

```bash
#!/bin/bash
# test_all_configurations.sh

set -e

echo "Testing default configuration (no storage-node)..."
cargo build --release
cargo test --release

echo "Testing with storage-node-core..."
cargo build --release --features storage-node-core
cargo test --release --features storage-node-core

echo "Testing with storage-node..."
cargo build --release --features storage-node
cargo test --release --features storage-node

echo "Testing standalone storage services..."
cd storage-services
cargo build --release
cargo test --release
cd ..

echo "All configurations passed!"
```

### Step 5.4: Verify Binary Sizes

```bash
# Build both variants
cargo build --release
ls -lh target/release/fendermint
# Note the size

cargo build --release --features storage-node
ls -lh target/release/fendermint
# Compare with previous size

# Expected difference: ~15-20MB
```

### Step 5.5: Smoke Tests

#### Without Storage Node:
```bash
# Genesis should work
fendermint genesis --genesis-file genesis.json ...

# Run should work
fendermint run ...

# RPC should work
fendermint rpc ...

# Storage node command should not exist
fendermint storage-node --help  # Should fail
```

#### With Storage Node:
```bash
# Build with storage-node
cargo build --release --features storage-node

# All standard commands should work
fendermint genesis --genesis-file genesis.json ...
fendermint run ...

# Storage node command should exist
fendermint storage-node --help  # Should succeed
fendermint storage-node run --iroh-path ./data/iroh ...

# Standalone services
./target/release/gateway --listen 0.0.0.0:8080
./target/release/node --iroh-path ./data ...
```

---

## Phase 6: CI/CD Updates

**Goal:** Update CI to test both configurations
**Time Estimate:** 2-3 days
**Risk:** Low

### Step 6.1: Update GitHub Actions

**File:** `.github/workflows/ci.yml`

```yaml
name: CI

on: [push, pull_request]

jobs:
  test-default:
    name: Test Default Configuration (no storage-node)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Cache cargo
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-default-${{ hashFiles('**/Cargo.lock') }}

      - name: Build default
        run: cargo build --workspace --release

      - name: Test default
        run: cargo test --workspace --release

      - name: Check binary size
        run: |
          ls -lh target/release/fendermint
          du -h target/release/fendermint

  test-storage-node:
    name: Test with Storage Node
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Cache cargo
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-storage-node-${{ hashFiles('**/Cargo.lock') }}

      - name: Build with storage-node
        run: cargo build --workspace --release --features storage-node

      - name: Test with storage-node
        run: cargo test --workspace --release --features storage-node

      - name: Check binary size
        run: |
          ls -lh target/release/fendermint
          du -h target/release/fendermint

  test-standalone-storage:
    name: Test Standalone Storage Services
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build gateway
        working-directory: storage-services
        run: cargo build --release --bin gateway

      - name: Build node
        working-directory: storage-services
        run: cargo build --release --bin node

      - name: Test standalone services
        working-directory: storage-services
        run: cargo test --release

  clippy:
    name: Clippy (both configurations)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: clippy

      - name: Clippy default
        run: cargo clippy --workspace -- -D warnings

      - name: Clippy with storage-node
        run: cargo clippy --workspace --features storage-node -- -D warnings

  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt

      - name: Check formatting
        run: cargo fmt --all -- --check
```

### Step 6.2: Add Feature Matrix Testing (Optional)

For comprehensive testing, add matrix strategy:

```yaml
  test-feature-matrix:
    name: Test Feature Combinations
    runs-on: ubuntu-latest
    strategy:
      matrix:
        features:
          - ""
          - "storage-node-core"
          - "storage-node-actors"
          - "storage-node-http-api"
          - "storage-node"
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1

      - name: Build with features
        run: |
          if [ -z "${{ matrix.features }}" ]; then
            cargo build --workspace
          else
            cargo build --workspace --features ${{ matrix.features }}
          fi

      - name: Test with features
        run: |
          if [ -z "${{ matrix.features }}" ]; then
            cargo test --workspace
          else
            cargo test --workspace --features ${{ matrix.features }}
          fi
```

### Step 6.3: Update Documentation

Create or update `docs/building.md`:

```markdown
# Building IPC

## Default Build (Without Storage Node)

```bash
cargo build --release
```

This builds the standard IPC node without storage node support.
Binary size: ~50MB

## Build with Storage Node

```bash
cargo build --release --features storage-node
```

This includes full storage node support with:
- Blob storage actors
- HTTP Storage Node API
- Iroh P2P integration
- Erasure coding

Binary size: ~70MB

## Build Options

### Minimal Build
```bash
cargo build --release --no-default-features
```

### With Core Storage Node (no HTTP API)
```bash
cargo build --release --features storage-node-core
```

### With Actors Only
```bash
cargo build --release --features storage-node-actors
```

## Standalone Storage Services

```bash
cd storage-services
cargo build --release
```

Produces:
- `gateway` - HTTP gateway for blob operations
- `node` - Storage node with chain integration
```

---

## Troubleshooting

### Common Issues

#### Issue 1: Conditional Compilation Errors

**Symptom:**
```
error: cannot find type `ReadRequest` in this scope
```

**Solution:**
Ensure imports are also gated:
```rust
#[cfg(feature = "recall-storage")]
use crate::read_request::ReadRequest;
```

#### Issue 2: Feature Dependency Errors

**Symptom:**
```
error: feature `recall-storage` includes `dep:warp` which is not defined
```

**Solution:**
Ensure dependency is marked as optional in `[dependencies]`:
```toml
warp = { workspace = true, optional = true }
```

#### Issue 3: Serialization Issues with Gated Enums

**Symptom:**
```
error: unknown variant `read_request_pending`
```

**Solution:**
This occurs when deserializing messages compiled without storage-node support.
Add migration logic:
```rust
#[serde(rename_all = "snake_case")]
pub enum IpcMessage {
    #[cfg(feature = "storage-node")]
    ReadRequestPending(ReadRequest),

    // For compatibility
    #[cfg(not(feature = "storage-node"))]
    #[serde(other)]
    Unknown,
}
```

#### Issue 4: Test Failures in Gated Code

**Symptom:**
```
test result: FAILED. 0 passed; 5 failed
```

**Solution:**
Ensure tests are properly gated:
```rust
#[cfg(all(test, feature = "storage-node"))]
mod storage_tests {
    #[test]
    fn test_blob_operations() { ... }
}
```

#### Issue 5: Actor ID Conflicts

**Symptom:**
```
error: actor ID 99 already exists
```

**Solution:**
Reserve actor IDs even when storage-node is disabled:
```rust
// In genesis initialization
const RESERVED_ACTOR_IDS: &[ActorID] = &[
    90,  // ADM (storage)
    99,  // Blobs (storage)
    100, // StorageConfig (storage)
    101, // BlobReader (storage)
];

// Don't create actors with these IDs when storage-node is disabled
```

---

## Verification Checklist

Before merging:

- [ ] All directories renamed successfully (recall → storage-node, etc.)
- [ ] All crate names updated in Cargo.toml files
- [ ] All imports updated across workspace
- [ ] Default build compiles without errors
- [ ] Storage-node-enabled build compiles without errors
- [ ] All tests pass in default configuration
- [ ] All tests pass with storage-node enabled
- [ ] Binary size differences are acceptable
- [ ] CI passes for both configurations
- [ ] Documentation is updated
- [ ] Feature flags are documented
- [ ] Migration guide is created
- [ ] Breaking changes are documented

---

## Rollback Plan

If issues are encountered:

1. **Revert Cargo.toml changes**
   ```bash
   git checkout HEAD -- Cargo.toml */Cargo.toml
   ```

2. **Revert code changes**
   ```bash
   git checkout HEAD -- fendermint/vm/interpreter/src/
   git checkout HEAD -- fendermint/vm/message/src/
   git checkout HEAD -- fendermint/app/src/cmd/
   ```

3. **Rebuild and test**
   ```bash
   cargo clean
   cargo build --workspace
   cargo test --workspace
   ```

---

## Success Criteria

✅ **Phase 0 Complete:**
- Renaming strategy documented and reviewed

✅ **Phase 1 Complete:**
- All directories renamed (recall → storage-node, etc.)
- All crate names updated in Cargo.toml
- All imports updated across workspace
- Workspace compiles with new names

✅ **Phase 2 Complete:**
- Feature flags defined in workspace Cargo.toml
- Dependencies marked as optional
- Builds still work as before

✅ **Phase 3 Complete:**
- All storage-node crates have feature flags
- fendermint/app and fendermint/vm/interpreter updated
- Both configurations compile

✅ **Phase 4 Complete:**
- All integration points gated with `#[cfg(feature = "storage-node")]`
- Default build works without storage-node
- Storage-node-enabled build works with all features

✅ **Phase 5 Complete:**
- All tests pass in both configurations
- Binary sizes verified
- Smoke tests pass

✅ **Phase 6 Complete:**
- CI updated to test both configurations
- Documentation updated
- Team reviewed and approved

---

## Post-Implementation

### Monitoring

After merge, monitor:
1. CI build times (should be faster for default configuration)
2. Binary sizes in releases
3. User feedback on build options
4. Feature adoption rates

### Future Improvements

Consider:
1. More granular feature flags (e.g., `storage-node-actors-blobs` separate from `storage-node-actors-bucket`)
2. Dynamic loading of storage node modules (advanced)
3. Runtime configuration instead of compile-time (requires architectural changes)

---

**Implementation Guide Version:** 2.0 (with renaming)
**Created:** December 4, 2024
**Last Updated:** December 4, 2024
**Major Changes:**
- Added Phase 0: Renaming Strategy
- Complete recall/basin → storage-node renaming throughout
- Updated all feature flags to use storage-node naming
- Renumbered phases to accommodate renaming phase
