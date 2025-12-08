# Storage-Node Dependencies in Fendermint

## Visual Dependency Map

```
fendermint/
├── app/
│   ├── src/
│   │   ├── service/node.rs          ⚠️  4x #[cfg(feature = "storage-node")]
│   │   │   ├── BlobPool             →   plugins/storage-node
│   │   │   ├── ReadRequestPool      →   plugins/storage-node
│   │   │   └── IrohResolver         →   plugins/storage-node
│   │   └── ipc.rs                   ⚠️  AppVote::BlobFinality/ReadRequestClosed
│   └── Cargo.toml                   ⚠️  storage deps, plugin-storage-node feature
│
├── vm/
│   ├── interpreter/
│   │   ├── src/
│   │   │   ├── fvm/
│   │   │   │   ├── interpreter.rs   ⚠️  3x #[cfg(feature = "storage-node")]
│   │   │   │   ├── storage_helpers.rs  →  plugins/storage-node (381 lines!)
│   │   │   │   └── storage_env.rs   →   plugins/storage-node (71 lines)
│   │   │   └── genesis.rs           ⚠️  1x #[cfg(feature = "storage-node")]
│   │   └── Cargo.toml               ⚠️  6 optional storage actor deps
│   │
│   ├── storage_resolver/            →   plugins/storage-node/src/resolver/
│   │   ├── pool.rs
│   │   ├── iroh.rs
│   │   ├── observe.rs
│   │   └── lib.rs
│   │
│   ├── topdown/
│   │   └── src/lib.rs               ⚠️  IPCBlobFinality, IPCReadRequestClosed
│   │
│   └── message/
│       └── Cargo.toml               ⚠️  depends on storage_blobs_shared
│
├── rpc/
│   ├── src/
│   │   ├── query.rs                 ⚠️  imports storage_bucket
│   │   ├── response.rs              ⚠️  imports storage_bucket
│   │   └── message.rs               ⚠️  imports storage_blobs_shared
│   └── Cargo.toml                   ⚠️  2 storage actor deps
│
└── actors/                          ✅  CLEANED (actors moved out!)

storage-node/
├── actors/                          ✅  NEW LOCATION
│   ├── machine/
│   ├── storage_adm/
│   ├── storage_adm_types/
│   ├── storage_blob_reader/
│   ├── storage_blobs/
│   ├── storage_bucket/
│   ├── storage_config/
│   └── storage_timehub/
├── executor/
├── ipld/
└── [other storage components]

plugins/
└── storage-node/                    🚧 WORK IN PROGRESS
    ├── src/
    │   ├── lib.rs                   ✅  Basic structure
    │   └── helpers/
    │       ├── genesis.rs           ✅  Placeholder
    │       └── message_handler.rs   ✅  Placeholder
    └── Cargo.toml                   ✅  Dependencies set up
```

## Feature Flag Locations

### 🔴 Critical: Message Handling
**File:** `fendermint/vm/interpreter/src/fvm/interpreter.rs`
```rust
Line 11:  #[cfg(feature = "storage-node")]
Line 529: #[cfg(feature = "storage-node")] IpcMessage::ReadRequestPending
Line 544: #[cfg(feature = "storage-node")] IpcMessage::ReadRequestClosed
```

### 🔴 Critical: Service Initialization
**File:** `fendermint/app/src/service/node.rs`
```rust
Line 13:  #[cfg(feature = "storage-node")] use BlobPool, ReadRequestPool
Line 17:  #[cfg(feature = "storage-node")] use IrohResolver
Line 27:  #[cfg(feature = "storage-node")] use IPCBlobFinality, IPCReadRequestClosed
Line 136: #[cfg(feature = "storage-node")] let blob_pool
Line 138: #[cfg(feature = "storage-node")] let read_request_pool
Line 191: #[cfg(feature = "storage-node")] spawn Iroh resolvers
```

### 🟡 Medium: Genesis
**File:** `fendermint/vm/interpreter/src/genesis.rs`
```rust
Line 406: #[cfg(feature = "storage-node")] initialize storage actors
```

## Dependency Types

### Type 1: Direct Code (needs feature flag removal)
- ✅ = Moved to plugin
- ⚠️  = Still in fendermint core
- 🚧 = Partially moved

| Component | Status | Lines | Location |
|-----------|--------|-------|----------|
| storage_helpers.rs | ⚠️ | 381 | fendermint/vm/interpreter/src/fvm/ |
| storage_env.rs | ⚠️ | 71 | fendermint/vm/interpreter/src/fvm/ |
| storage_resolver/ | ⚠️ | ~500 | fendermint/vm/storage_resolver/ |
| Genesis init | 🚧 | 43 | fendermint/vm/interpreter/src/genesis.rs |
| Message handling | 🚧 | 37 | fendermint/vm/interpreter/src/fvm/interpreter.rs |
| Service init | ⚠️ | 89 | fendermint/app/src/service/node.rs |

### Type 2: Type Definitions (needs abstraction)
- `IPCBlobFinality` - in `fendermint/vm/topdown/src/lib.rs`
- `IPCReadRequestClosed` - in `fendermint/vm/topdown/src/lib.rs`
- `AppVote` variants - in `fendermint/app/src/ipc.rs`
- `BlobPool`, `ReadRequestPool` - in `fendermint/vm/interpreter/src/fvm/storage_env.rs`

### Type 3: Actor Dependencies (✅ DONE)
- ✅ All storage actors moved to `storage-node/actors/`
- ✅ Workspace updated
- ⚠️ Still referenced in Cargo.toml as optional deps

### Type 4: Shared Types (decision needed)
- `storage_blobs_shared` - Used by RPC, message, and core
- `storage_bucket` - Used by RPC
- **Decision:** Keep as shared library or move to plugin?

## Compilation Dependencies

### With `--features plugin-storage-node`:
```
fendermint → plugin-storage-node → storage-node/actors/
                                 → storage-node/executor/
                                 → fendermint (circular!)
```

### Without `--features plugin-storage-node`:
```
Currently: Fails to compile (feature flags guard missing code)
Goal: Compiles successfully, no storage code
```

## Migration Complexity Score

| Area | Complexity | Reason |
|------|-----------|--------|
| Actor movement | ✅ Easy (DONE) | No runtime dependencies |
| Genesis init | 🟡 Medium | Needs GenesisState API extension |
| Message handling | 🔴 Hard | Deeply coupled to FvmExecState |
| Service init | 🔴 Hard | Requires service context API |
| Storage helpers | 🔴 Very Hard | 381 lines, tight FvmExecState coupling |
| Storage resolver | 🟡 Medium | Self-contained but needs topdown types |
| Type abstractions | 🔴 Hard | Affects voting, finality, IPC core |
| RPC integration | 🟡 Medium | Shared type strategy needed |

## Next Actions

### Immediate (to unblock):
1. ✅ Document current state (this file)
2. 📋 Decide on architecture approach:
   - **Pragmatic Hybrid:** Keep some integration code in fendermint behind feature flags
   - **Full Extraction:** Extend APIs, move everything to plugin
3. 📋 Get stakeholder input on effort vs. value

### Short-term (if going full extraction):
1. Design and implement `GenesisState::create_custom_actor`
2. Design plugin state access patterns
3. Design service module resource sharing
4. Create generic finality types in topdown

### Long-term:
1. Implement all plugin module traits
2. Move storage_resolver to plugin
3. Remove all feature flags
4. Test thoroughly

## Effort Estimate

- **Pragmatic Hybrid:** 2-3 days (document, minor cleanups)
- **Full Extraction:** 2-3 weeks (see detailed plan)

## Key Questions

1. **Is full extraction worth 2-3 weeks of work?**
   - Actors are already isolated ✅
   - Code still has compile-time coupling ⚠️
   - Runtime isolation could be achieved more cheaply

2. **What's the real goal?**
   - Zero compile-time dependencies? → Full extraction needed
   - Runtime modularity? → Already mostly achieved
   - Easy maintenance? → Actor isolation sufficient

3. **What breaks if we just remove feature flags?**
   - Genesis: Storage actors won't be initialized
   - Messages: ReadRequest messages won't be handled
   - Services: Iroh resolvers won't start
   - All these need plugin hooks to work
