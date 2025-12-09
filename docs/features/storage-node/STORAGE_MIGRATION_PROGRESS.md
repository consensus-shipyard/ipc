# Storage Plugin Migration - Progress Report

## Status: IN PROGRESS - Phase 1 (API Extension)

### ✅ Completed Tasks

#### Phase 0: Assessment & Planning
- ✅ Moved all storage actors from `fendermint/actors/` to `storage-node/actors/`
  - `machine/`, `storage_adm/`, `storage_adm_types/`
  - `storage_blobs/` (with shared/ and testing/)
  - `storage_blob_reader/`, `storage_bucket/`, `storage_config/`, `storage_timehub/`
- ✅ Updated workspace Cargo.toml
- ✅ Created comprehensive audit documents:
  - `STORAGE_PLUGIN_MIGRATION_PLAN.md` (400+ lines)
  - `STORAGE_DEPENDENCIES_MAP.md` (200+ lines)
  - `ARCHITECTURE_DECISION_NEEDED.md`
- ✅ Decision made: **Full Extraction (Option B)**

#### Phase 1.1: Actor Interface Migration
- ✅ Created `plugins/storage-node/src/actor_interface/`
- ✅ Moved 5 storage actor interface files:
  - `adm.rs` (77 lines - full interface)
  - `blob_reader.rs` (4 lines)
  - `blobs.rs` (4 lines)
  - `bucket.rs` (5 lines)
  - `recall_config.rs` (4 lines)
- ✅ Removed from `fendermint/vm/actor_interface/src/`
- ✅ Plugin compiles with actor interfaces
- ✅ Updated imports in genesis.rs to be conditional

#### Phase 1.2: GenesisState Trait Extension
- ✅ Added `create_custom_actor()` method to `GenesisState` trait
- ✅ Added serde dependency to fendermint_module
- 🔄 Implementing trait for `FvmGenesisState` (in progress)

---

### 🔄 Current Work

**Issue:** Implementing `GenesisState` trait for `FvmGenesisState<DB>`

**Blockers:**
1. Send/Sync trait bounds on generic DB parameter
2. `circ_supply` not tracked in `FvmGenesisState` (used workaround)
3. Conditional compilation of storage actor interfaces

**Next Steps:**
1. Fix Send/Sync bounds for trait implementation
2. Complete GenesisState impl for FvmGenesisState
3. Test that plugin can call create_custom_actor

---

### 📋 Remaining Work

#### Phase 1.3-1.4: Additional API Extensions
- [ ] Design FvmExecState plugin access pattern
- [ ] Design ServiceContext for plugin resources
- [ ] Add message handling hooks

#### Phase 2: Code Migration
- [ ] Move `fendermint/vm/storage_resolver/` → `plugins/storage-node/src/resolver/`
- [ ] Move `storage_helpers.rs` logic to plugin (381 lines!)
- [ ] Move `storage_env.rs` to plugin (71 lines)
- [ ] Move topdown storage types to plugin

#### Phase 3: Feature Flag Removal
- [ ] Remove `#[cfg(feature = "storage-node")]` from interpreter (3 locations)
- [ ] Remove `#[cfg(feature = "storage-node")]` from node.rs (4 locations)
- [ ] Remove `#[cfg(feature = "storage-node")]` from genesis.rs (1 location)
- [ ] Update genesis to call plugin's GenesisModule

#### Phase 4: Dependency Cleanup
- [ ] Remove storage actor deps from fendermint/vm/interpreter/Cargo.toml
- [ ] Remove storage deps from fendermint/app/Cargo.toml
- [ ] Remove storage-node features from app/settings/options
- [ ] Move all storage deps to plugins/storage-node/Cargo.toml

#### Phase 5: RPC & Testing
- [ ] Update RPC to use plugin interfaces
- [ ] Update CLI commands
- [ ] Test storage-node with plugin enabled
- [ ] Test fendermint compiles without plugin
- [ ] Comprehensive integration testing

---

## Files Modified So Far

### Plugin Files Created/Modified:
- `plugins/storage-node/src/actor_interface/` (NEW)
  - `mod.rs`, `adm.rs`, `blob_reader.rs`, `blobs.rs`, `bucket.rs`, `recall_config.rs`
- `plugins/storage-node/src/helpers/`
  - `genesis.rs` (placeholder impl)
  - `message_handler.rs` (placeholder impl)
- `plugins/storage-node/src/lib.rs` (updated)
- `plugins/storage-node/Cargo.toml` (updated dependencies)

### Fendermint Files Modified:
- `fendermint/module/src/genesis.rs` (trait extended)
- `fendermint/module/Cargo.toml` (added serde)
- `fendermint/vm/interpreter/src/genesis.rs` (conditional imports)
- `fendermint/vm/interpreter/src/fvm/state/genesis.rs` (trait impl in progress)
- `fendermint/vm/actor_interface/src/lib.rs` (removed storage modules)

### Files Deleted:
- `fendermint/vm/actor_interface/src/adm.rs`
- `fendermint/vm/actor_interface/src/blob_reader.rs`
- `fendermint/vm/actor_interface/src/blobs.rs`
- `fendermint/vm/actor_interface/src/bucket.rs`
- `fendermint/vm/actor_interface/src/recall_config.rs`

---

## Key Challenges Encountered

### 1. Actor Interface Dependencies
**Issue:** Storage actor interfaces were in core fendermint
**Solution:** Moved to plugin with macro support ✅

### 2. GenesisState Trait Limitations
**Issue:** Original trait didn't support custom actor creation
**Solution:** Extended trait with `create_custom_actor()` ✅

### 3. Circular Supply Tracking
**Issue:** `FvmGenesisState` doesn't track `circ_supply`
**Workaround:** Used thread_local for stub implementation 🔄

### 4. Send/Sync Bounds
**Issue:** Generic `DB` parameter doesn't guarantee Send+Sync
**Status:** Working on resolution 🔄

---

## Compilation Status

| Package | Status | Notes |
|---------|--------|-------|
| `ipc_plugin_storage_node` | ✅ Compiles | With actor_interface modules |
| `fendermint_module` | ✅ Compiles | With extended GenesisState trait |
| `fendermint_vm_interpreter` | ⚠️  Errors | GenesisState impl issues |
| `fendermint_app` | ❓ Not tested | Depends on interpreter |

---

## Effort Tracking

**Time Invested:** ~4-5 hours
**Estimated Remaining:** 10-15 hours (full extraction is 2-3 weeks total)

**Progress:** ~20% complete

---

## Next Session Priorities

1. **Fix GenesisState implementation** (highest priority)
   - Resolve Send/Sync bounds
   - Test plugin can create custom actors

2. **Move storage_resolver module**
   - Self-contained, lower coupling
   - Good next step after genesis works

3. **Design message handling hooks**
   - Critical for removing feature flags
   - Needs careful API design

---

## Notes

- The full extraction is ambitious but achievable
- Module system APIs are being extended as needed
- Plugin architecture is proving flexible
- Main complexity is in the deep coupling to FvmExecState (storage_helpers.rs)

---

## Success Criteria Progress

- ✅ Actors isolated in storage-node/actors
- 🔄 Plugin can initialize actors in genesis (in progress)
- ⏳ Plugin can handle storage messages
- ⏳ No `#[cfg(feature = "storage-node")]` in fendermint
- ⏳ Fendermint compiles without plugin
- ⏳ All tests pass

**Target:** True plugin modularity with zero compile-time coupling
