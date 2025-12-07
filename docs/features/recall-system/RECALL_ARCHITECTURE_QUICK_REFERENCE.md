# Recall Storage - Quick Architecture Reference

## Component Map

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           OPTIONAL BOUNDARIES                            │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│ LAYER 1: Standalone Binaries (100% Optional)                            │
│ ├─ ipc-decentralized-storage/                                           │
│ │  ├─ bin/gateway.rs          → HTTP gateway for blob operations        │
│ │  └─ bin/node.rs             → Storage node with chain integration     │
│ └─ These can be built independently without fendermint                  │
└─────────────────────────────────────────────────────────────────────────┘
                                    ↕
┌─────────────────────────────────────────────────────────────────────────┐
│ LAYER 2: Application Commands (100% Optional)                           │
│ ├─ fendermint/app/cmd/objects.rs    → 1,455 lines                       │
│ │  └─ HTTP API for blob upload/download with erasure coding             │
│ ├─ fendermint/app/options/objects.rs → CLI options                      │
│ └─ fendermint/app/settings/objects.rs → Configuration                   │
└─────────────────────────────────────────────────────────────────────────┘
                                    ↕
┌─────────────────────────────────────────────────────────────────────────┐
│ LAYER 3: FVM Actors (100% Optional - except actor IDs)                  │
│ ├─ fendermint/actors/blobs/          → ~8,000 lines                     │
│ │  └─ Main blob storage with credit system, subscriptions, expiry       │
│ ├─ fendermint/actors/blob_reader/    → ~800 lines                       │
│ │  └─ Read-only blob access for unprivileged operations                 │
│ ├─ fendermint/actors/recall_config/  → ~800 lines                       │
│ │  └─ Network configuration (capacity, TTL, credit rates)               │
│ ├─ fendermint/actors/bucket/         → ~2,700 lines                     │
│ │  └─ S3-like object storage with versioning                            │
│ ├─ fendermint/actors/timehub/        → ~1,300 lines                     │
│ │  └─ Timestamping and scheduling service                               │
│ └─ fendermint/actors/adm/            → ~900 lines                       │
│    └─ Address/machine lifecycle manager                                 │
└─────────────────────────────────────────────────────────────────────────┘
                                    ↕
┌─────────────────────────────────────────────────────────────────────────┐
│ LAYER 4: VM Integration (PARTIALLY Optional - requires careful gating) │
│ ├─ fendermint/vm/interpreter/                                           │
│ │  ├─ fvm/interpreter.rs             → Handle ReadRequest messages      │
│ │  ├─ fvm/recall_env.rs       [NEW]  → Read request pool               │
│ │  ├─ fvm/recall_helpers.rs   [NEW]  → Blob operation helpers          │
│ │  ├─ genesis.rs                     → Initialize recall actors         │
│ │  └─ fvm/state/exec.rs              → Optional recall executor         │
│ ├─ fendermint/vm/topdown/                                               │
│ │  └─ voting.rs                      → Add blob vote tally (~200 lines) │
│ ├─ fendermint/vm/message/                                               │
│ │  └─ ipc.rs                         → ReadRequest message types        │
│ └─ fendermint/vm/iroh_resolver/ [NEW] → ~900 lines (100% optional)     │
│    ├─ iroh.rs                        → Blob resolution with voting      │
│    ├─ pool.rs                        → Connection pooling               │
│    └─ observe.rs                     → Metrics                          │
└─────────────────────────────────────────────────────────────────────────┘
                                    ↕
┌─────────────────────────────────────────────────────────────────────────┐
│ LAYER 5: Core Runtime (100% Optional)                                   │
│ ├─ recall/executor/                  → Custom executor with gas         │
│ ├─ recall/kernel/                    → Custom FVM kernel                │
│ ├─ recall/syscalls/                  → Blob syscalls                    │
│ ├─ recall/actor_sdk/                 → Actor SDK with EVM               │
│ ├─ recall/ipld/                      → Custom IPLD structures           │
│ └─ recall/iroh_manager/              → Iroh P2P management              │
└─────────────────────────────────────────────────────────────────────────┘
                                    ↕
┌─────────────────────────────────────────────────────────────────────────┐
│ LAYER 6: Solidity Facades (100% Optional)                               │
│ └─ recall-contracts/crates/facade/   → ~18,000 lines (auto-generated)  │
│    └─ EVM event bindings for Solidity integration                       │
└─────────────────────────────────────────────────────────────────────────┘
                                    ↕
┌─────────────────────────────────────────────────────────────────────────┐
│ LAYER 7: Infrastructure Changes (PARTIALLY Optional)                    │
│ ├─ ipld/resolver/                    → Iroh integration (~400 lines)    │
│ │  ├─ client.rs                      → ResolverIroh trait               │
│ │  ├─ service.rs                     → Iroh download logic              │
│ │  └─ behaviour/mod.rs               → Config errors                    │
│ └─ patches/netwatch/                 → macOS socket2 compatibility      │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## File Count by Category

| Category | New Files | Modified Files | Total Lines | Optional? |
|----------|-----------|----------------|-------------|-----------|
| **Recall Core** (`recall/`) | 25 | 0 | ~5,000 | ✅ 100% |
| **Recall Actors** | 88 | 0 | ~15,000 | ✅ 100% |
| **Recall Contracts** | 22 | 0 | ~18,000 | ✅ 100% |
| **Standalone Services** | 7 | 0 | ~2,300 | ✅ 100% |
| **VM Interpreter** | 3 | 4 | ~600 | ⚠️ ~70% |
| **Fendermint App** | 3 | 5 | ~1,500 | ✅ 95% |
| **IPLD Resolver** | 0 | 5 | ~400 | ⚠️ ~80% |
| **VM Topdown** | 0 | 2 | ~200 | ⚠️ ~60% |
| **Documentation** | 86 | 0 | ~24,000 | N/A |
| **Total** | **234** | **16** | **~67,000** | **~85%** |

---

## Integration Touchpoints (What Needs Gating)

### Critical Integration Points (Must Gate)

#### 1. Message Type Enum (fendermint/vm/message/src/ipc.rs)
```rust
pub enum IpcMessage {
    // Existing variants...

    #[cfg(feature = "recall-storage")]
    ReadRequestPending(ReadRequest),

    #[cfg(feature = "recall-storage")]
    ReadRequestClosed(ReadRequest),
}
```
**Risk:** Medium - Affects message serialization
**Lines:** ~50

#### 2. Message Handlers (fendermint/vm/interpreter/src/fvm/interpreter.rs)
```rust
match msg {
    #[cfg(feature = "recall-storage")]
    IpcMessage::ReadRequestPending(req) => {
        set_read_request_pending(state, req.id)?;
    }

    #[cfg(feature = "recall-storage")]
    IpcMessage::ReadRequestClosed(req) => {
        read_request_callback(state, &req)?;
        close_read_request(state, req.id)?;
    }

    // Existing handlers...
}
```
**Risk:** Low - Contained in match arm
**Lines:** ~100

#### 3. Genesis Initialization (fendermint/vm/interpreter/src/genesis.rs)
```rust
#[cfg(feature = "recall-storage")]
fn initialize_recall_actors(state: &mut GenesisBuilder) -> Result<()> {
    // Create ADM actor
    state.create_custom_actor(ADM_ACTOR_NAME, ADM_ACTOR_ID, ...)?;

    // Create recall_config actor
    state.create_custom_actor(RECALL_CONFIG_ACTOR_NAME, ...)?;

    // Create blobs actor (with delegated address)
    state.create_custom_actor(BLOBS_ACTOR_NAME, BLOBS_ACTOR_ID, ...)?;

    // Create blob_reader actor
    state.create_custom_actor(BLOB_READER_ACTOR_NAME, ...)?;

    Ok(())
}
```
**Risk:** Low - Self-contained function
**Lines:** ~150

### Optional Integration Points (Can Gate)

#### 4. HTTP Objects Command (fendermint/app/src/cmd/mod.rs)
```rust
pub enum Commands {
    #[cfg(feature = "recall-storage")]
    Objects(objects::ObjectsCmd),

    // Existing commands...
}
```
**Risk:** Very Low - Completely independent
**Lines:** ~1,500 (in objects.rs)

#### 5. Blob Voting (fendermint/vm/topdown/src/voting.rs)
```rust
impl VoteTally {
    #[cfg(feature = "recall-storage")]
    pub fn add_blob_vote(&mut self, validator: ValidatorKey, hash: Hash) {
        // BFT consensus logic for blob availability
    }

    #[cfg(feature = "recall-storage")]
    pub fn find_blob_quorum(&self) -> Option<Hash> {
        // Find blobs with 2/3+ validator votes
    }
}
```
**Risk:** Low - Extension methods
**Lines:** ~200

#### 6. Iroh Resolver (ipld/resolver/src/client.rs)
```rust
#[cfg(feature = "recall-storage")]
#[async_trait]
pub trait ResolverIroh {
    async fn resolve_iroh(
        &self,
        hash: Hash,
        size: u64,
        node_addr: NodeAddr,
    ) -> Result<ResolveResult>;
}
```
**Risk:** Low - Trait-based extension
**Lines:** ~400

---

## Dependency Tree

```
┌─── DEFAULT IPC (no recall) ───┐
│                                │
│  fendermint                    │
│  ├─ fvm (standard)             │
│  ├─ ipc-api                    │
│  ├─ ipld/resolver (basic)      │
│  └─ actors (standard)          │
│                                │
└────────────────────────────────┘

┌─── WITH recall-storage ───────┐
│                                │
│  fendermint                    │
│  ├─ fvm (standard)             │
│  ├─ recall_executor ─┐         │
│  ├─ recall_kernel    │         │
│  ├─ recall_syscalls  │         │
│  │                   │         │
│  ├─ ipc-api          │         │
│  ├─ ipld/resolver ───┤         │
│  │  └─ iroh         │         │
│  │     iroh-blobs    │         │
│  │                   │         │
│  ├─ actors (std)     │         │
│  └─ actors (recall) ─┘         │
│     ├─ blobs                   │
│     ├─ blob_reader             │
│     ├─ recall_config           │
│     ├─ bucket                  │
│     ├─ timehub                 │
│     └─ adm                     │
│                                │
│  ipc-decentralized-storage     │
│  ├─ gateway (binary)           │
│  └─ node (binary)              │
│                                │
└────────────────────────────────┘
```

---

## Feature Flag Hierarchy

```toml
[features]
default = []

# Full recall support (everything)
recall-storage = [
    "recall-core",
    "recall-actors",
    "recall-http-api",
]

# Core runtime (kernel, executor, syscalls)
recall-core = [
    "dep:recall_kernel",
    "dep:recall_syscalls",
    "dep:recall_executor",
    "dep:recall_ipld",
    "dep:iroh",
    "dep:iroh-blobs",
]

# On-chain actors
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
    "dep:entangler",
]
```

---

## Build Time Comparison

| Configuration | Build Time | Binary Size | Dependencies |
|---------------|------------|-------------|--------------|
| **Default (no recall)** | Baseline | ~50 MB | Standard |
| **+ recall-core** | +20-30s | ~60 MB | +Iroh |
| **+ recall-actors** | +30-45s | ~65 MB | +Actors |
| **+ recall-http-api** | +40-60s | ~70 MB | +Warp |
| **Full recall-storage** | +45-60s | ~70 MB | Everything |

---

## Testing Matrix

| Configuration | Unit Tests | Integration Tests | E2E Tests |
|---------------|------------|-------------------|-----------|
| Default | ✅ All pass | ✅ All pass | ✅ All pass |
| recall-core | ✅ + Recall runtime | ✅ + Actor tests | ⚠️ Limited |
| recall-actors | ✅ + Actor tests | ✅ + Chain tests | ⚠️ Limited |
| recall-http-api | ✅ + API tests | ✅ + HTTP tests | ✅ Full |
| recall-storage | ✅ All tests | ✅ All tests | ✅ All tests |

---

## Risk Assessment

### Low Risk (Easy to Make Optional)
- ✅ Standalone binaries (`ipc-decentralized-storage`)
- ✅ HTTP Objects API (`fendermint/app/cmd/objects.rs`)
- ✅ All recall actors
- ✅ Recall core runtime (`recall/` directory)
- ✅ Iroh resolver module

### Medium Risk (Requires Careful Gating)
- ⚠️ Message type extensions (serialization concerns)
- ⚠️ Genesis initialization (actor ID allocation)
- ⚠️ Vote tally extensions (consensus impact)

### High Risk (Consider Keeping Always Compiled)
- ❌ None - all recall features can be made optional

---

## Migration Checklist

### Phase 1: Setup (1-2 days)
- [ ] Add feature flags to workspace Cargo.toml
- [ ] Make all recall dependencies `optional = true`
- [ ] Define feature hierarchy (recall-core, recall-actors, etc.)
- [ ] Test that default build still works

### Phase 2: Core Integration (3-5 days)
- [ ] Gate message types with `#[cfg(feature = "recall-storage")]`
- [ ] Gate message handlers in interpreter
- [ ] Gate genesis initialization
- [ ] Gate HTTP objects command
- [ ] Test both configurations build successfully

### Phase 3: Actor Integration (2-3 days)
- [ ] Verify all actors compile with feature flag
- [ ] Gate actor interface exports
- [ ] Update genesis to conditionally create actors
- [ ] Test actor creation and calls

### Phase 4: Infrastructure (2-3 days)
- [ ] Gate Iroh integration in IPLD resolver
- [ ] Gate blob voting in vote tally
- [ ] Gate recall executor usage
- [ ] Test P2P functionality

### Phase 5: Testing (5-7 days)
- [ ] Run full test suite without recall
- [ ] Run full test suite with recall
- [ ] Test all feature combinations
- [ ] Verify binary sizes
- [ ] Benchmark build times

### Phase 6: Documentation & CI (2-3 days)
- [ ] Update build documentation
- [ ] Update CI to test both configurations
- [ ] Create migration guide
- [ ] Document feature flags

---

## Command Examples

### Build Commands
```bash
# Default (no recall)
cargo build --release

# With recall core
cargo build --release --features recall-core

# With recall actors
cargo build --release --features recall-actors

# Full recall
cargo build --release --features recall-storage

# Standalone storage services
cd ipc-decentralized-storage && cargo build --release
```

### Test Commands
```bash
# Test default
cargo test

# Test with recall
cargo test --features recall-storage

# Test specific feature
cargo test --features recall-core

# Test all combinations (CI)
cargo test --all-features
```

### Run Commands
```bash
# Fendermint without recall (default)
fendermint run

# Fendermint with recall HTTP API (if compiled with recall-storage)
fendermint objects run --iroh-path ./data/iroh

# Standalone storage node
cd ipc-decentralized-storage
./target/release/node --iroh-path ./data --rpc-url http://localhost:26657

# Standalone gateway
./target/release/gateway --listen 0.0.0.0:8080
```

---

**Quick Reference Version:** 1.0
**Created:** December 4, 2024
**For Full Details:** See `RECALL_STORAGE_MODULARIZATION_ANALYSIS.md`
