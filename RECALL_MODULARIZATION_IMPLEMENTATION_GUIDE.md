# Recall Storage Modularization - Implementation Guide

**Purpose:** Step-by-step guide to make recall storage an optional compile-time feature.

**Estimated Total Time:** 2-3 weeks
**Difficulty:** Medium
**Risk Level:** Low-Medium (well-contained changes)

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Phase 1: Feature Flag Architecture](#phase-1-feature-flag-architecture)
3. [Phase 2: Gate Core Components](#phase-2-gate-core-components)
4. [Phase 3: Gate Integration Points](#phase-3-gate-integration-points)
5. [Phase 4: Testing & Validation](#phase-4-testing--validation)
6. [Phase 5: CI/CD Updates](#phase-5-cicd-updates)
7. [Troubleshooting](#troubleshooting)

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

## Phase 1: Feature Flag Architecture

**Goal:** Set up feature flags without changing any code
**Time Estimate:** 1-2 days
**Risk:** Low

### Step 1.1: Update Root Cargo.toml

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

# Full recall storage support
recall-storage = [
    "recall-core",
    "recall-actors",
    "recall-http-api",
]

# Core recall runtime
recall-core = []

# On-chain actors
recall-actors = ["recall-core"]

# HTTP Objects API
recall-http-api = ["recall-core"]
```

**Note:** We'll populate these feature arrays in subsequent steps.

### Step 1.2: Make Recall Dependencies Optional

**File:** `/Cargo.toml` (workspace.dependencies section)

Update recall-related dependencies:

```toml
[workspace.dependencies]
# ... existing dependencies ...

# Recall/Iroh dependencies (make optional)
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

### Step 1.3: Update Workspace Members

**File:** `/Cargo.toml` (workspace.members section)

Mark recall members as optional:

```toml
[workspace.members]
# ... existing members ...

# Recall storage (optional via feature flags)
# Keep in members list, but we'll make them conditional via features
"recall/kernel",
"recall/kernel/ops",
"recall/syscalls",
"recall/executor",
"recall/iroh_manager",
"recall/ipld",
"recall/actor_sdk",

# Recall actors (optional)
"fendermint/actors/adm",
"fendermint/actors/adm_types",
"fendermint/actors/blobs",
"fendermint/actors/blobs/shared",
"fendermint/actors/blobs/testing",
"fendermint/actors/blob_reader",
"fendermint/actors/bucket",
"fendermint/actors/timehub",
"fendermint/actors/recall_config",
"fendermint/actors/recall_config/shared",

# Recall contracts (optional)
"recall-contracts/crates/facade",

# Note: ipc-decentralized-storage stays as optional workspace member
# It can be built independently
]
```

### Step 1.4: Test Build Without Changes

```bash
# Should still build normally
cargo build --workspace
cargo test --workspace

# Verify feature flag syntax
cargo build --features recall-storage
```

**Expected Result:** Everything builds exactly as before.

---

## Phase 2: Gate Core Components

**Goal:** Make recall modules optional via feature flags
**Time Estimate:** 2-3 days
**Risk:** Low-Medium

### Step 2.1: Gate Recall Core Modules

For each crate in `recall/`:

#### File: `recall/kernel/Cargo.toml`

```toml
[package]
name = "recall_kernel"
# ... existing config ...

[features]
# No default features
default = []

[dependencies]
recall_kernel_ops = { path = "../kernel/ops" }
recall_syscalls = { path = "../syscalls" }
# ... rest of dependencies ...
```

#### File: `recall/executor/Cargo.toml`

```toml
[package]
name = "recall_executor"
# ... existing config ...

[dependencies]
recall_kernel = { path = "../kernel" }
# ... rest of dependencies ...
```

**Repeat for:**
- `recall/syscalls/Cargo.toml`
- `recall/ipld/Cargo.toml`
- `recall/iroh_manager/Cargo.toml`
- `recall/actor_sdk/Cargo.toml`

### Step 2.2: Gate Recall Actors

For each actor in `fendermint/actors/`:

#### File: `fendermint/actors/blobs/Cargo.toml`

```toml
[package]
name = "fendermint_actor_blobs"
# ... existing config ...

[features]
default = []

[dependencies]
fendermint_actor_blobs_shared = { path = "./shared" }
# ... rest of dependencies ...
```

#### File: `fendermint/actors/blob_reader/Cargo.toml`

```toml
[package]
name = "fendermint_actor_blob_reader"
# ... existing config ...

[features]
default = []

[dependencies]
fendermint_actor_blobs_shared = { path = "../blobs/shared" }
# ... rest of dependencies ...
```

**Repeat for:**
- `fendermint/actors/recall_config/Cargo.toml`
- `fendermint/actors/bucket/Cargo.toml`
- `fendermint/actors/timehub/Cargo.toml`
- `fendermint/actors/adm/Cargo.toml`

### Step 2.3: Update fendermint/app/Cargo.toml

**File:** `fendermint/app/Cargo.toml`

```toml
[package]
name = "fendermint_app"
# ... existing config ...

[features]
default = []
recall-storage = [
    "dep:warp",
    "dep:uuid",
    "dep:mime_guess",
    "dep:urlencoding",
    "dep:entangler",
    "dep:entangler_storage",
    "dep:iroh_manager",
    "dep:iroh",
    "dep:iroh-blobs",
    "dep:fendermint_actor_bucket",
    "dep:fendermint_actor_blobs_shared",
    "dep:fendermint_vm_iroh_resolver",
]

[dependencies]
# ... existing dependencies ...

# Objects/Recall HTTP API dependencies (now optional)
warp = { workspace = true, optional = true }
uuid = { workspace = true, optional = true }
mime_guess = { workspace = true, optional = true }
urlencoding = { workspace = true, optional = true }
entangler = { workspace = true, optional = true }
entangler_storage = { workspace = true, optional = true }
iroh_manager = { path = "../../recall/iroh_manager", optional = true }
iroh = { workspace = true, optional = true }
iroh-blobs = { workspace = true, optional = true }
fendermint_actor_bucket = { path = "../actors/bucket", optional = true }
fendermint_actor_blobs_shared = { path = "../actors/blobs/shared", optional = true }
fendermint_vm_iroh_resolver = { path = "../vm/iroh_resolver", optional = true }
```

### Step 2.4: Update fendermint/vm/interpreter/Cargo.toml

**File:** `fendermint/vm/interpreter/Cargo.toml`

```toml
[package]
name = "fendermint_vm_interpreter"
# ... existing config ...

[features]
default = []
recall-storage = [
    "dep:recall_executor",
    "dep:recall_kernel",
    "dep:fendermint_actor_adm",
    "dep:fendermint_actor_blobs",
    "dep:fendermint_actor_blobs_shared",
    "dep:fendermint_actor_blob_reader",
    "dep:fendermint_actor_recall_config",
    "dep:fendermint_actor_recall_config_shared",
    "dep:fendermint_vm_iroh_resolver",
    "dep:iroh",
    "dep:iroh-blobs",
]

[dependencies]
# ... existing dependencies ...

# Recall dependencies (now optional)
fendermint_actor_adm = { path = "../../actors/adm", optional = true }
fendermint_actor_blobs = { path = "../../actors/blobs", optional = true }
fendermint_actor_blobs_shared = { path = "../../actors/blobs/shared", optional = true }
fendermint_actor_blob_reader = { path = "../../actors/blob_reader", optional = true }
fendermint_actor_recall_config = { path = "../../actors/recall_config", optional = true }
fendermint_actor_recall_config_shared = { path = "../../actors/recall_config/shared", optional = true }
recall_executor = { path = "../../../recall/executor", optional = true }
recall_kernel = { path = "../../../recall/kernel", optional = true }
fendermint_vm_iroh_resolver = { path = "../iroh_resolver", optional = true }
iroh = { workspace = true, optional = true }
iroh-blobs = { workspace = true, optional = true }
```

### Step 2.5: Test Compilation

```bash
# Test without recall (should fail - expected at this stage)
cargo build --workspace

# Test with recall
cargo build --workspace --features recall-storage

# Test individual crates
cargo build -p fendermint_app
cargo build -p fendermint_app --features recall-storage
```

---

## Phase 3: Gate Integration Points

**Goal:** Add conditional compilation directives to code
**Time Estimate:** 3-5 days
**Risk:** Medium

### Step 3.1: Gate Message Type Extensions

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

    // Recall-specific variants
    #[cfg(feature = "recall-storage")]
    #[serde(rename = "read_request_pending")]
    ReadRequestPending(ReadRequest),

    #[cfg(feature = "recall-storage")]
    #[serde(rename = "read_request_closed")]
    ReadRequestClosed(ReadRequest),
}

// Add conditional import
#[cfg(feature = "recall-storage")]
pub use crate::read_request::ReadRequest;

// Create new module (gated)
#[cfg(feature = "recall-storage")]
pub mod read_request {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ReadRequest {
        pub id: Hash,
        // ... fields ...
    }
}
```

### Step 3.2: Gate Message Handlers

**File:** `fendermint/vm/interpreter/src/fvm/interpreter.rs`

At the top of the file:

```rust
// Conditional imports
#[cfg(feature = "recall-storage")]
use crate::fvm::recall_env::ReadRequestPool;
#[cfg(feature = "recall-storage")]
use crate::fvm::recall_helpers::{
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

                // Recall handlers (gated)
                #[cfg(feature = "recall-storage")]
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

                #[cfg(feature = "recall-storage")]
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

### Step 3.3: Gate Genesis Initialization

**File:** `fendermint/vm/interpreter/src/genesis.rs`

Add conditional imports:

```rust
#[cfg(feature = "recall-storage")]
use fendermint_vm_actor_interface::{adm, blob_reader, blobs, recall_config};
```

In the genesis builder:

```rust
impl<'a> GenesisBuilder<'a> {
    pub fn build(&mut self) -> Result<()> {
        // ... existing actor initialization ...

        // Recall actors (conditional)
        #[cfg(feature = "recall-storage")]
        self.initialize_recall_actors()?;

        Ok(())
    }

    #[cfg(feature = "recall-storage")]
    fn initialize_recall_actors(&mut self) -> Result<()> {
        // ADM actor
        let mut machine_codes = std::collections::HashMap::new();
        for machine_name in &["bucket", "timehub"] {
            if let Some(cid) = self.state.custom_actor_manifest.code_by_name(machine_name) {
                let kind = fendermint_actor_adm::Kind::from_str(machine_name)?;
                machine_codes.insert(kind, *cid);
            }
        }
        let adm_state = fendermint_actor_adm::State::new(
            self.state.store(),
            machine_codes,
            fendermint_actor_adm::PermissionModeParams::Unrestricted,
        )?;
        self.state.create_custom_actor(
            fendermint_vm_actor_interface::adm::ADM_ACTOR_NAME,
            adm::ADM_ACTOR_ID,
            &adm_state,
            TokenAmount::zero(),
            None,
        )?;

        // Recall config actor
        let recall_config_state = fendermint_actor_recall_config::State {
            admin: None,
            config: fendermint_actor_recall_config_shared::RecallConfig::default(),
        };
        self.state.create_custom_actor(
            fendermint_actor_recall_config::ACTOR_NAME,
            recall_config::RECALL_CONFIG_ACTOR_ID,
            &recall_config_state,
            TokenAmount::zero(),
            None,
        )?;

        // Blobs actor (with delegated address)
        let blobs_state = fendermint_actor_blobs::State::new(&self.state.store())?;
        let blobs_eth_addr = init::builtin_actor_eth_addr(blobs::BLOBS_ACTOR_ID);
        let blobs_f4_addr = fvm_shared::address::Address::from(blobs_eth_addr);
        self.state.create_custom_actor(
            fendermint_actor_blobs::BLOBS_ACTOR_NAME,
            blobs::BLOBS_ACTOR_ID,
            &blobs_state,
            TokenAmount::zero(),
            Some(blobs_f4_addr),
        )?;

        // Blob reader actor
        self.state.create_custom_actor(
            fendermint_actor_blob_reader::BLOB_READER_ACTOR_NAME,
            blob_reader::BLOB_READER_ACTOR_ID,
            &fendermint_actor_blob_reader::State::new(&self.state.store())?,
            TokenAmount::zero(),
            None,
        )?;

        Ok(())
    }
}
```

### Step 3.4: Gate Objects HTTP Command

**File:** `fendermint/app/src/cmd/mod.rs`

```rust
pub mod genesis;
pub mod key;
pub mod materialize;
pub mod run;
pub mod rpc;

// Objects command (conditional)
#[cfg(feature = "recall-storage")]
pub mod objects;

#[derive(Debug, Parser)]
pub enum Commands {
    Genesis(genesis::GenesisCmd),
    Key(key::KeyCmd),
    Materialize(materialize::MaterializeCmd),
    Run(run::RunCmd),
    Rpc(rpc::RpcCmd),

    #[cfg(feature = "recall-storage")]
    #[command(about = "Run Objects HTTP API for blob storage")]
    Objects(objects::ObjectsCmd),
}

impl Commands {
    pub async fn exec(self, ...) -> anyhow::Result<()> {
        match self {
            Commands::Genesis(cmd) => cmd.exec(...).await,
            Commands::Key(cmd) => cmd.exec(...),
            Commands::Materialize(cmd) => cmd.exec(...).await,
            Commands::Run(cmd) => cmd.exec(...).await,
            Commands::Rpc(cmd) => cmd.exec(...).await,

            #[cfg(feature = "recall-storage")]
            Commands::Objects(cmd) => cmd.exec(...).await,
        }
    }
}
```

### Step 3.5: Gate Vote Tally Extensions

**File:** `fendermint/vm/topdown/src/voting.rs`

```rust
use std::collections::{HashMap, HashSet};

#[cfg(feature = "recall-storage")]
use iroh_blobs::Hash as BlobHash;

pub struct VoteTally<V> {
    // Existing fields...

    #[cfg(feature = "recall-storage")]
    blob_votes: HashMap<BlobHash, HashSet<V>>,
}

impl<V: ValidatorKey> VoteTally<V> {
    // Existing methods...

    #[cfg(feature = "recall-storage")]
    pub fn add_blob_vote(&mut self, validator: V, hash: BlobHash) {
        self.blob_votes
            .entry(hash)
            .or_insert_with(HashSet::new)
            .insert(validator);
    }

    #[cfg(feature = "recall-storage")]
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

### Step 3.6: Gate Iroh Resolver Integration

**File:** `ipld/resolver/src/client.rs`

```rust
#[cfg(feature = "recall-storage")]
use iroh::{NodeAddr};
#[cfg(feature = "recall-storage")]
use iroh_blobs::Hash;

// Existing Resolver trait...

#[cfg(feature = "recall-storage")]
#[async_trait]
pub trait ResolverIroh {
    async fn resolve_iroh(
        &self,
        hash: Hash,
        size: u64,
        node_addr: NodeAddr,
    ) -> anyhow::Result<ResolveResult>;
}

#[cfg(feature = "recall-storage")]
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

    #[cfg(feature = "recall-storage")]
    iroh_manager: Option<IrohManager>,
}

impl<S, V> Service<S, V> {
    pub async fn new(config: Config) -> Result<Self> {
        // Existing initialization...

        #[cfg(feature = "recall-storage")]
        let iroh_manager = if let Some(iroh_config) = config.iroh {
            Some(IrohManager::new(iroh_config).await?)
        } else {
            None
        };

        Ok(Self {
            // ... existing fields ...
            #[cfg(feature = "recall-storage")]
            iroh_manager,
        })
    }

    async fn handle_request(&mut self, req: Request<V>) {
        match req {
            // Existing handlers...

            #[cfg(feature = "recall-storage")]
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

### Step 3.7: Test Compilation

```bash
# Test without recall - should now compile!
cargo build --workspace

# Test with recall
cargo build --workspace --features recall-storage

# Test individual components
cargo build -p fendermint_app
cargo build -p fendermint_app --features recall-storage
cargo build -p fendermint_vm_interpreter
cargo build -p fendermint_vm_interpreter --features recall-storage
```

---

## Phase 4: Testing & Validation

**Goal:** Ensure both configurations work correctly
**Time Estimate:** 5-7 days
**Risk:** Medium-High

### Step 4.1: Unit Tests

Add conditional test gating where needed:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Tests that work without recall
    #[test]
    fn test_standard_functionality() {
        // ...
    }

    // Tests that require recall
    #[cfg(feature = "recall-storage")]
    #[test]
    fn test_blob_operations() {
        // ...
    }
}
```

### Step 4.2: Run Test Suites

```bash
# Test without recall
cargo test --workspace

# Test with recall
cargo test --workspace --features recall-storage

# Test specific crates
cargo test -p fendermint_vm_interpreter
cargo test -p fendermint_vm_interpreter --features recall-storage

# Test all feature combinations (comprehensive)
cargo test --workspace --all-features
cargo test --workspace --no-default-features
```

### Step 4.3: Integration Tests

Create test script:

```bash
#!/bin/bash
# test_all_configurations.sh

set -e

echo "Testing default configuration (no recall)..."
cargo build --release
cargo test --release

echo "Testing with recall-core..."
cargo build --release --features recall-core
cargo test --release --features recall-core

echo "Testing with recall-storage..."
cargo build --release --features recall-storage
cargo test --release --features recall-storage

echo "Testing standalone storage services..."
cd ipc-decentralized-storage
cargo build --release
cargo test --release
cd ..

echo "All configurations passed!"
```

### Step 4.4: Verify Binary Sizes

```bash
# Build both variants
cargo build --release
ls -lh target/release/fendermint
# Note the size

cargo build --release --features recall-storage
ls -lh target/release/fendermint
# Compare with previous size

# Expected difference: ~15-20MB
```

### Step 4.5: Smoke Tests

#### Without Recall:
```bash
# Genesis should work
fendermint genesis --genesis-file genesis.json ...

# Run should work
fendermint run ...

# RPC should work
fendermint rpc ...

# Objects command should not exist
fendermint objects --help  # Should fail
```

#### With Recall:
```bash
# Build with recall
cargo build --release --features recall-storage

# All standard commands should work
fendermint genesis --genesis-file genesis.json ...
fendermint run ...

# Objects command should exist
fendermint objects --help  # Should succeed
fendermint objects run --iroh-path ./data/iroh ...

# Standalone services
./target/release/gateway --listen 0.0.0.0:8080
./target/release/node --iroh-path ./data ...
```

---

## Phase 5: CI/CD Updates

**Goal:** Update CI to test both configurations
**Time Estimate:** 2-3 days
**Risk:** Low

### Step 5.1: Update GitHub Actions

**File:** `.github/workflows/ci.yml`

```yaml
name: CI

on: [push, pull_request]

jobs:
  test-default:
    name: Test Default Configuration (no recall)
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

  test-recall-storage:
    name: Test with Recall Storage
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
          key: ${{ runner.os }}-cargo-recall-${{ hashFiles('**/Cargo.lock') }}

      - name: Build with recall
        run: cargo build --workspace --release --features recall-storage

      - name: Test with recall
        run: cargo test --workspace --release --features recall-storage

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
        working-directory: ipc-decentralized-storage
        run: cargo build --release --bin gateway

      - name: Build node
        working-directory: ipc-decentralized-storage
        run: cargo build --release --bin node

      - name: Test standalone services
        working-directory: ipc-decentralized-storage
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

      - name: Clippy with recall
        run: cargo clippy --workspace --features recall-storage -- -D warnings

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

### Step 5.2: Add Feature Matrix Testing (Optional)

For comprehensive testing, add matrix strategy:

```yaml
  test-feature-matrix:
    name: Test Feature Combinations
    runs-on: ubuntu-latest
    strategy:
      matrix:
        features:
          - ""
          - "recall-core"
          - "recall-actors"
          - "recall-http-api"
          - "recall-storage"
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

### Step 5.3: Update Documentation

Create or update `docs/building.md`:

```markdown
# Building IPC

## Default Build (Without Recall Storage)

```bash
cargo build --release
```

This builds the standard IPC node without recall storage support.
Binary size: ~50MB

## Build with Recall Storage

```bash
cargo build --release --features recall-storage
```

This includes full recall storage support with:
- Blob storage actors
- HTTP Objects API
- Iroh P2P integration
- Erasure coding

Binary size: ~70MB

## Build Options

### Minimal Build
```bash
cargo build --release --no-default-features
```

### With Core Recall (no HTTP API)
```bash
cargo build --release --features recall-core
```

### With Actors Only
```bash
cargo build --release --features recall-actors
```

## Standalone Storage Services

```bash
cd ipc-decentralized-storage
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
This occurs when deserializing messages compiled without recall support.
Add migration logic:
```rust
#[serde(rename_all = "snake_case")]
pub enum IpcMessage {
    #[cfg(feature = "recall-storage")]
    ReadRequestPending(ReadRequest),

    // For compatibility
    #[cfg(not(feature = "recall-storage"))]
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
#[cfg(all(test, feature = "recall-storage"))]
mod recall_tests {
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
Reserve actor IDs even when recall is disabled:
```rust
// In genesis initialization
const RESERVED_ACTOR_IDS: &[ActorID] = &[
    90,  // ADM (recall)
    99,  // Blobs (recall)
    100, // RecallConfig (recall)
    101, // BlobReader (recall)
];

// Don't create actors with these IDs when recall is disabled
```

---

## Verification Checklist

Before merging:

- [ ] Default build compiles without errors
- [ ] Recall-enabled build compiles without errors
- [ ] All tests pass in default configuration
- [ ] All tests pass with recall enabled
- [ ] Binary size differences are acceptable
- [ ] CI passes for both configurations
- [ ] Documentation is updated
- [ ] Feature flags are documented
- [ ] Migration guide is created
- [ ] Breaking changes are documented (if any)

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

✅ **Phase 1 Complete:**
- Feature flags defined in workspace Cargo.toml
- Dependencies marked as optional
- Builds still work exactly as before

✅ **Phase 2 Complete:**
- All recall crates have feature flags
- fendermint/app and fendermint/vm/interpreter updated
- Both configurations compile

✅ **Phase 3 Complete:**
- All integration points gated with `#[cfg(feature = "recall-storage")]`
- Default build works without recall
- Recall-enabled build works with all features

✅ **Phase 4 Complete:**
- All tests pass in both configurations
- Binary sizes verified
- Smoke tests pass

✅ **Phase 5 Complete:**
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
1. More granular feature flags (e.g., `recall-actors-blobs` separate from `recall-actors-bucket`)
2. Dynamic loading of recall modules (advanced)
3. Runtime configuration instead of compile-time (requires architectural changes)

---

**Implementation Guide Version:** 1.0
**Created:** December 4, 2024
**Last Updated:** December 4, 2024
