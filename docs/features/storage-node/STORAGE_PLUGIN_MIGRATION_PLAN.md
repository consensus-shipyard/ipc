# Storage Plugin Migration Plan
## Goal: Truly Modular Plugin System

Remove all `#[cfg(feature = "storage-node")]` from fendermint core and make storage-node a true plugin with zero compile-time coupling.

---

## Current State Analysis

### Files with storage-node feature flags:
1. **`fendermint/vm/interpreter/src/fvm/interpreter.rs`** - Message handling (3 locations)
2. **`fendermint/app/src/service/node.rs`** - Service initialization (4 locations)
3. **`fendermint/vm/interpreter/src/genesis.rs`** - Genesis initialization (1 location)

### Storage-Specific Code in Fendermint:
1. **`fendermint/vm/interpreter/src/fvm/storage_helpers.rs`** (381 lines)
   - Helper functions for blob/read request operations
   - Tightly coupled to `FvmExecState`

2. **`fendermint/vm/interpreter/src/fvm/storage_env.rs`** (71 lines)
   - Type definitions: `BlobPool`, `ReadRequestPool`
   - Pool item types for Iroh resolution

3. **`fendermint/vm/storage_resolver/`** (entire module)
   - Iroh-based resolution logic
   - Pool management
   - Observability

4. **`fendermint/vm/topdown/src/lib.rs`**
   - `IPCBlobFinality` struct
   - `IPCReadRequestClosed` struct
   - Used in voting/finality

5. **`fendermint/app/src/ipc.rs`**
   - `AppVote::BlobFinality` variant
   - `AppVote::ReadRequestClosed` variant

### Dependencies:
- `fendermint_actor_storage_*` ✅ **Already moved to `storage-node/actors/`**
- `storage_node_executor` - Used by module system
- `storage_node_iroh_manager` - Optional dependency
- `fendermint_vm_storage_resolver` - Entire module

---

## Migration Strategy

### Phase 1: Extend Module System APIs ✅ (Started)

**Status:** Plugin structure created, but APIs need extension

**What's needed:**

1. **Extend `GenesisState` trait** to support custom actor creation
   ```rust
   // In fendermint/module/src/genesis.rs
   pub trait GenesisState {
       fn create_custom_actor(
           &mut self,
           name: &str,
           id: ActorID,
           state: &impl Serialize,
           balance: TokenAmount,
           delegated_address: Option<Address>,
       ) -> Result<()>;
   }
   ```

2. **Add plugin hooks for message handling** in interpreter
   ```rust
   // In fendermint/module/src/message.rs
   pub trait MessageHandlerModule {
       async fn handle_ipc_message<S: FvmExecState>(
           &self,
           state: &mut S,
           msg: &IpcMessage,
       ) -> Result<Option<ApplyMessageResponse>>;
   }
   ```

3. **Add service resource sharing** for pools/resolvers
   ```rust
   // In fendermint/module/src/service.rs
   pub trait ServiceModule {
       fn create_shared_resources(&self) -> ModuleResources;
   }
   ```

---

### Phase 2: Move Storage Components to Plugin

#### 2.1 Move `fendermint/vm/storage_resolver/` → `plugins/storage-node/src/resolver/`

**Files to move:**
- `pool.rs` - Resolution pool management
- `iroh.rs` - Iroh resolver implementation
- `observe.rs` - Metrics/observability
- `lib.rs` - Module exports

**Why:** This is storage-specific infrastructure, not general-purpose.

#### 2.2 Move storage helper logic to plugin

**Current location:** `fendermint/vm/interpreter/src/fvm/storage_helpers.rs`

**Strategy:**
- Keep the file in fendermint temporarily (tightly coupled to FvmExecState)
- Make it accessible through a trait that the plugin can implement
- OR extend FvmExecState to expose needed methods to plugins

**Alternative:** Create a `StorageStateOps` trait that plugins can use:
```rust
pub trait StorageStateOps {
    fn execute_implicit_message(&mut self, msg: Message) -> Result<FvmApplyRet>;
    // ... other needed operations
}
```

#### 2.3 Move type definitions to plugin

**From:** `fendermint/vm/interpreter/src/fvm/storage_env.rs`
**To:** `plugins/storage-node/src/types.rs`

These are storage-specific type definitions that don't need to be in core.

#### 2.4 Move topdown types to plugin

**From:** `fendermint/vm/topdown/src/lib.rs`
- `IPCBlobFinality`
- `IPCReadRequestClosed`

**Strategy:**
- Define generic finality types in core (`GenericResourceFinality<T>`)
- Storage plugin provides concrete implementations
- Update `AppVote` to use plugin-provided types

**Alternative:** Keep minimal trait definitions in core, implementations in plugin.

---

### Phase 3: Remove Feature Flags

#### 3.1 Genesis Initialization

**Current:** `fendermint/vm/interpreter/src/genesis.rs:406-448`
```rust
#[cfg(feature = "storage-node")]
{
    // Initialize recall config actor
    // Initialize blobs actor
    // Initialize blob reader actor
}
```

**After:** Plugin's `GenesisModule::initialize_actors()` is called
```rust
// In plugins/storage-node/src/lib.rs
impl GenesisModule for StorageNodeModule {
    fn initialize_actors<S: GenesisState>(&self, state: &mut S, genesis: &Genesis) -> Result<()> {
        crate::helpers::genesis::initialize_storage_actors(state, genesis)
    }
}
```

**Remove:** Entire `#[cfg(feature = "storage-node")]` block

---

#### 3.2 Message Handling

**Current:** `fendermint/vm/interpreter/src/fvm/interpreter.rs:529-565`
```rust
#[cfg(feature = "storage-node")]
IpcMessage::ReadRequestPending(read_request) => {
    let ret = set_read_request_pending(state, read_request.id)?;
    // ...
}

#[cfg(feature = "storage-node")]
IpcMessage::ReadRequestClosed(read_request) => {
    read_request_callback(state, &read_request)?;
    // ...
}
```

**After:** Plugin handles these messages
```rust
// In plugins/storage-node/src/lib.rs
impl MessageHandlerModule for StorageNodeModule {
    async fn handle_message<S: FvmExecState>(
        &self,
        state: &mut S,
        msg: &IpcMessage,
    ) -> Result<Option<ApplyMessageResponse>> {
        match msg {
            IpcMessage::ReadRequestPending(req) => {
                // Handle via storage_helpers (made accessible to plugin)
            }
            IpcMessage::ReadRequestClosed(req) => {
                // Handle via storage_helpers
            }
            _ => Ok(None)
        }
    }
}
```

**Remove:** Both `#[cfg(feature = "storage-node")]` blocks

---

#### 3.3 Service Initialization

**Current:** `fendermint/app/src/service/node.rs:136-224`
```rust
#[cfg(feature = "storage-node")]
let blob_pool: BlobPool = ResolvePool::new();
#[cfg(feature = "storage-node")]
let read_request_pool: ReadRequestPool = ResolvePool::new();

#[cfg(feature = "storage-node")]
if let Some(ref key) = validator_keypair {
    // Create and spawn Iroh resolvers
    // Create and spawn read request resolver
}
```

**After:** Plugin's `ServiceModule::initialize_services()` handles this
```rust
// In plugins/storage-node/src/lib.rs
impl ServiceModule for StorageNodeModule {
    async fn initialize_services(
        &self,
        ctx: &ServiceContext,
    ) -> Result<Vec<tokio::task::JoinHandle<()>>> {
        // Create pools
        // Spawn Iroh resolvers
        // Return task handles
    }

    fn resources(&self) -> ModuleResources {
        // Provide blob_pool and read_request_pool to other components
    }
}
```

**Remove:** All 4 `#[cfg(feature = "storage-node")]` blocks

---

### Phase 4: Update Dependencies

#### 4.1 Move storage_resolver module

**Current:** `fendermint/vm/storage_resolver/` (separate crate)
**After:** `plugins/storage-node/src/resolver/` (part of plugin)

**Update:**
- Remove from `fendermint/vm/` workspace
- Add to plugin's internal modules
- Update all import paths

#### 4.2 Clean up Cargo.toml files

**Remove from `fendermint/vm/interpreter/Cargo.toml`:**
```toml
fendermint_actor_storage_adm = { ... }
fendermint_actor_storage_blobs = { ... }
fendermint_actor_storage_blob_reader = { ... }
fendermint_actor_storage_config = { ... }
```

**Remove from `fendermint/app/Cargo.toml`:**
```toml
fendermint_actor_storage_bucket = { ... }
fendermint_actor_storage_blobs_shared = { ... }
fendermint_vm_storage_resolver = { ... }
storage_node_iroh_manager = { ... }
```

**Remove features:**
- `plugin-storage-node` from `fendermint/app/Cargo.toml`
- `storage-node` aliases from settings/options

**All storage dependencies move to:** `plugins/storage-node/Cargo.toml`

---

### Phase 5: Update RPC and CLI

**Current issues:**
- `fendermint/rpc/` imports storage actors directly
- `fendermint/app/src/cmd/objects.rs` uses storage_bucket

**Strategy:**
- RPC should use plugin-provided interfaces
- Or: Keep minimal shared types in a `storage-node/shared` crate
- CLI commands should be plugin-provided

**Options:**

**Option A:** Shared types crate
```
storage-node/
  shared/           # Minimal shared types (like storage_blobs/shared)
  actors/           # Actor implementations
  ...
```

**Option B:** Plugin exposes RPC handlers
```rust
impl RpcModule for StorageNodeModule {
    fn rpc_handlers(&self) -> Vec<RpcHandler> {
        // Provide storage-specific RPC endpoints
    }
}
```

---

## Implementation Order

### ✅ Completed:
1. Move actor crates to `storage-node/actors/`
2. Update workspace Cargo.toml
3. Create basic plugin structure

### 🔄 In Progress:
4. Design module system API extensions

### 📋 TODO:

#### Priority 1 (Core APIs):
- [ ] Extend `GenesisState` trait with `create_custom_actor`
- [ ] Add `FvmExecState` trait or helper access for plugins
- [ ] Design `ServiceContext` for plugin service initialization
- [ ] Create plugin resource sharing mechanism

#### Priority 2 (Move Code):
- [ ] Move `storage_resolver` module to plugin
- [ ] Move `storage_env.rs` to plugin
- [ ] Move topdown types to plugin (or create generic versions)
- [ ] Update `AppVote` to be plugin-extensible

#### Priority 3 (Implement Plugin):
- [ ] Implement `GenesisModule` with actual actor initialization
- [ ] Implement `MessageHandlerModule` with storage helpers
- [ ] Implement `ServiceModule` with Iroh resolvers
- [ ] Add storage-specific CLI commands

#### Priority 4 (Remove Feature Flags):
- [ ] Remove `#[cfg(feature = "storage-node")]` from interpreter
- [ ] Remove `#[cfg(feature = "storage-node")]` from node.rs
- [ ] Remove `#[cfg(feature = "storage-node")]` from genesis.rs
- [ ] Remove optional dependencies from fendermint Cargo.toml files
- [ ] Remove `storage-node` features from app/settings/options

#### Priority 5 (Test & Document):
- [ ] Test storage-node functionality with plugin enabled
- [ ] Test that fendermint compiles without plugin
- [ ] Document plugin architecture
- [ ] Update user documentation

---

## Key Design Decisions Needed

### 1. Storage Helpers Coupling

**Question:** How to handle `storage_helpers.rs` coupling to `FvmExecState`?

**Options:**
A. Keep in fendermint, make accessible via trait
B. Extract interface that plugins can depend on
C. Refactor FvmExecState to be more plugin-friendly

**Recommendation:** Option A initially, migrate to B long-term

---

### 2. Topdown Types

**Question:** Should `IPCBlobFinality` and `IPCReadRequestClosed` stay in topdown?

**Options:**
A. Keep in topdown, conditionally compiled
B. Move to plugin, make topdown generic
C. Create abstraction layer

**Recommendation:** Option B - make voting/finality extensible

---

### 3. RPC Integration

**Question:** How should storage RPC endpoints work?

**Options:**
A. Shared types crate (minimal)
B. Plugin-provided RPC handlers
C. Keep minimal RPC in core, extend via plugin

**Recommendation:** Option A + C hybrid

---

## Success Criteria

✅ **Compilation:**
- Fendermint compiles without `--features plugin-storage-node`
- No storage-related code in fendermint core (only in plugin)
- No `#[cfg(feature = "storage-node")]` in fendermint

✅ **Functionality:**
- Storage-node works identically with plugin enabled
- All tests pass
- No regression in storage functionality

✅ **Modularity:**
- Plugin can be maintained independently
- New storage features only touch plugin code
- Other plugins can follow same pattern

---

## Estimated Effort

- **Phase 1:** 3-5 days (API design and implementation)
- **Phase 2:** 5-7 days (Code movement and refactoring)
- **Phase 3:** 2-3 days (Feature flag removal)
- **Phase 4:** 2-3 days (Dependency cleanup)
- **Phase 5:** 2-3 days (Testing and documentation)

**Total:** ~2-3 weeks of focused development

---

## Notes

- This plan achieves true modularity but requires significant module system enhancements
- The plugin system needs to be more powerful than currently designed
- Consider if this level of decoupling is worth the effort vs. pragmatic hybrid approach
- Alternative: Document current hybrid as acceptable and focus on actor isolation (already done)
