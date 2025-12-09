# Phase 2 Progress: Code Migration to Plugin

**Status:** IN PROGRESS - Moving storage code from fendermint to plugin
**Current:** Phase 2.1 ✅ Complete

---

## ✅ Phase 2.1: Storage Resolver Module - COMPLETE

### What Was Moved
- **Module:** `fendermint/vm/storage_resolver/` → `plugins/storage-node/src/resolver/`
- **Files:**
  - `iroh.rs` (295 lines)
  - `pool.rs` (430 lines)
  - `observe.rs` (173 lines)
- **Total:** ~900 lines of code

### Changes Made

1. **Copied module to plugin** ✅
   - Created `plugins/storage-node/src/resolver/`
   - Added `mod.rs` with public exports
   - Fixed imports from `crate::` to `super::`

2. **Added dependencies to plugin** ✅
   ```toml
   hex, im, libp2p, prometheus
   ipc-api, ipc_ipld_resolver, ipc-observability
   fendermint_vm_topdown
   ```

3. **Updated imports in fendermint** ✅
   - `fendermint/app/src/service/node.rs` now uses `ipc_plugin_storage_node::resolver::`
   - `fendermint/vm/interpreter/src/fvm/storage_env.rs` updated temporarily

4. **Removed old module** ✅
   - Deleted `fendermint/vm/storage_resolver/` directory
   - Removed from `fendermint/app/Cargo.toml` dependencies

5. **Compilation Status** ✅
   - Plugin compiles successfully
   - App compiles with `--features plugin-storage-node`
   - All references updated

---

## 🎯 Next: Phase 2.2 - storage_helpers.rs (Complex)

**Challenge:** 381 lines tightly coupled to `FvmExecState`

### Analysis
```rust
// Current: storage_helpers.rs in fendermint/vm/interpreter/src/fvm/
// Functions like:
- get_added_blobs(state: &mut FvmExecState, ...)
- get_pending_blobs(state: &mut FvmExecState, ...)
- set_read_request_pending(state: &mut FvmExecState, ...)
- read_request_callback(state: &mut FvmExecState, ...)
- close_read_request(state: &mut FvmExecState, ...)
```

### Options for Phase 2.2

**Option A:** Create Plugin State Access Trait
```rust
// In fendermint/module/src/
pub trait PluginStateAccess {
    fn execute_implicit_message(&mut self, msg: Message) -> Result<FvmApplyRet>;
    // ... other methods
}
```

**Option B:** Keep helpers in fendermint, export via plugin-accessible API
- Helpers stay in `fendermint/vm/interpreter/src/fvm/`
- Plugin gets access through trait methods
- Less code movement, cleaner boundaries

**Option C:** Move helpers to plugin, make them generic over state trait
- More complex refactoring
- Better long-term separation
- Requires more trait design

**Recommendation:** Start with Option B (pragmatic), can evolve to A/C later

---

## Phase 2.3: storage_env.rs - Ready to Move

**Status:** Easy move, no complex coupling

- **File:** `fendermint/vm/interpreter/src/fvm/storage_env.rs` (71 lines)
- **Purpose:** Type definitions for `BlobPool` and `ReadRequestPool`
- **Dependencies:** Uses `ipc_plugin_storage_node::resolver::pool` types
- **Plan:** Simple file move, already references plugin types

---

## Phase 2.4: Topdown Storage Types

**Files to update:**
- `fendermint/vm/topdown/src/lib.rs`
  - `IPCBlobFinality` struct
  - `IPCReadRequestClosed` struct
- `fendermint/app/src/ipc.rs`
  - `AppVote::BlobFinality` variant
  - `AppVote::ReadRequestClosed` variant

**Strategy:**
- Make topdown finality types generic or extensible
- Plugin provides concrete implementations
- Or: Keep minimal types in topdown, plugin extends

---

## Compilation Status After Phase 2.1

| Package | Status | Notes |
|---------|--------|-------|
| `ipc_plugin_storage_node` | ✅ Compiles | With resolver module |
| `fendermint_vm_interpreter` | ✅ Compiles | Updated import |
| `fendermint_app` | ✅ Compiles | Uses plugin's resolver |
| Full workspace | ✅ Compiles | All packages build |

---

## Impact Summary

### Before Phase 2.1:
```
fendermint/vm/storage_resolver/  (~900 lines)
├── Used by fendermint/app/
└── Separate crate in fendermint

plugins/storage-node/
├── Basic structure
└── No resolver functionality
```

### After Phase 2.1:
```
fendermint/vm/storage_resolver/  [DELETED]

plugins/storage-node/src/resolver/  (~900 lines) ✅
├── All Iroh resolution logic
├── Self-contained module
└── Used by fendermint/app/ via plugin

fendermint/app/
└── Imports from ipc_plugin_storage_node::resolver
```

---

## Key Learnings

1. **Module moves are straightforward** when well-isolated
2. **Import updates need care** (`crate::` → `super::`)
3. **Dependencies follow the code** (moved to plugin Cargo.toml)
4. **Compilation validates migration** - no runtime needed yet

---

## Next Steps

### Immediate (Phase 2.3):
- Move `storage_env.rs` to plugin (simple, 71 lines)
- Update remaining imports
- Test compilation

### After 2.3 (Phase 2.2):
- Design approach for `storage_helpers.rs`
- Decide on Option A/B/C above
- Implement chosen strategy

---

##

 Progress Tracking

- ✅ Phase 1: API Extensions Complete
- 🔄 Phase 2: Code Migration (30% complete)
  - ✅ Phase 2.1: storage_resolver moved
  - ⏳ Phase 2.2: storage_helpers (design needed)
  - ⏳ Phase 2.3: storage_env (ready to move)
  - ⏳ Phase 2.4: topdown types
- ⏳ Phase 3: Feature flag removal
- ⏳ Phase 4: Dependency cleanup
- ⏳ Phase 5: Testing

**Overall Progress: ~30% Complete**

---

## Commands to Verify Phase 2.1

```bash
# Verify old module is gone
ls fendermint/vm/storage_resolver  # Should error: No such file

# Verify plugin has resolver
ls plugins/storage-node/src/resolver/  # Should show iroh.rs, pool.rs, observe.rs

# Verify compilation
cargo check -p ipc_plugin_storage_node  # Should pass ✅
cargo check -p fendermint_app --features plugin-storage-node  # Should pass ✅
```

All checks pass! ✅
