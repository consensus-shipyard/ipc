# Phase 2 - Hybrid Approach Implementation

**Date:** December 4, 2025
**Strategy:** Type aliases with generic foundations
**Status:** 🔄 Implementing

---

## Strategy

Instead of making **every file** generic over `M`, we:

1. ✅ Keep core types generic (`FvmExecState<DB, M>`, `FvmMessagesInterpreter<DB, M>`)
2. ✅ Create feature-gated module selection
3. 🔄 Add type aliases for internal convenience
4. 🔄 Revert unnecessary generic propagation
5. 🔄 Wire up at app boundary

---

## Implementation Steps

### Step 1: Module Selection ✅
Created `fendermint/vm/interpreter/src/fvm/default_module.rs`:
```rust
#[cfg(feature = "storage-node")]
pub type SelectedModule = storage_node_module::StorageNodeModule;

#[cfg(not(feature = "storage-node"))]
pub type SelectedModule = fendermint_module::NoOpModuleBundle;
```

### Step 2: Revert Over-Generic Files 🔄

Files that DON'T need `M` generic (use type alias instead):
- `state/genesis.rs` - Use DefaultModule internally
- `upgrades.rs` - Use DefaultModule
- `topdown.rs` - Use DefaultModule
- `end_block_hook.rs` - Use DefaultModule
- `storage_helpers.rs` - Use DefaultModule (cfg-gated anyway)
- `activity/` - Use DefaultModule

Files that SHOULD stay generic:
- `state/exec.rs` ✅ (core type)
- `interpreter.rs` ✅ (core type)
- `executions.rs` ✅ (used by core)
- `lib.rs` trait ✅ (public API)

### Step 3: Create Internal Type Aliases 🔄

Add to `fendermint/vm/interpreter/src/fvm/mod.rs`:
```rust
use default_module::DefaultModule;

// Convenient type aliases for internal use
pub type DefaultFvmExecState<DB> = state::FvmExecState<DB, DefaultModule>;
pub type DefaultFvmMessagesInterpreter<DB> = interpreter::FvmMessagesInterpreter<DB, DefaultModule>;
pub type DefaultFvmGenesisState<DB> = state::genesis::FvmGenesisState<DB, DefaultModule>;
```

### Step 4: Update Files to Use Aliases 🔄

Instead of adding `M` everywhere, use the type aliases:

```rust
// Before (what we were trying):
fn my_function<DB, M>(state: &mut FvmExecState<DB, M>)
where
    M: ModuleBundle
{ ... }

// After (hybrid):
fn my_function<DB>(state: &mut DefaultFvmExecState<DB>)
where
    DB: Blockstore
{ ... }
```

### Step 5: Wire at App Boundary 🔄

Only the app layer needs to:
1. Create module instance
2. Pass to interpreter constructor
3. Initialize services

---

## Benefits

✅ Less code churn (~10 files vs 30+)
✅ Faster implementation
✅ Still achieves modularity
✅ Can enhance later if needed
✅ Cleaner internal APIs

---

## Current Action

Reverting unnecessary changes and applying type alias pattern...
