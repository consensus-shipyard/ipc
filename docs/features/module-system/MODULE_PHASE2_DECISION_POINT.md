# Phase 2 - Decision Point

**Date:** December 4, 2025
**Current Errors:** 68 (fluctuating due to cascading changes)
**Status:** ⚠️ Refactor Complexity Higher Than Expected

---

## Situation

We've successfully completed **Phase 1** (module framework - 100%) and made solid progress on **Phase 2** (~40%). However, the refactor is proving more complex than initially estimated due to:

### Challenges

1. **Cascading Dependencies**: Each type change creates errors in callers
2. **Multiple Update Paths Required**: Not just interpreter, but also:
   - `genesis.rs` (outside fvm/)
   - `app/` layer (not started)
   - `abci/` layer (not started)
   - Test files

3. **Struct with Many Fields**: `FvmGenesisState`, `UpgradeScheduler`, etc. have complex initialization

4. **Type Propagation**: `M` needs to propagate through entire call chain

---

## Options Forward

### Option 1: Continue Current Approach ⏰ Est: 6-10 hours

**Pros:**
- Clean architecture
- Zero runtime overhead
- Follows original design

**Cons:**
- Time intensive
- High risk of introducing subtle bugs
- Touches 30+ files

**Next Steps:**
1. Finish interpreter package (current: 68 errors)
2. Fix genesis.rs callsites
3. Update app layer
4. Update abci layer
5. Add type aliases
6. Remove #[cfg] directives

### Option 2: Simplified Approach - Type Aliases First ⏰ Est: 2-3 hours

Create convenience type aliases **now** to minimize changes:

```rust
// Add to fendermint/vm/interpreter/src/lib.rs
#[cfg(feature = "storage-node")]
pub type DefaultModule = storage_node_module::StorageNodeModule;

#[cfg(not(feature = "storage-node"))]
pub type DefaultModule = fendermint_module::NoOpModuleBundle;

// Use concrete type aliases everywhere
pub type DefaultFvmExecState<DB> = FvmExecState<DB, DefaultModule>;
pub type DefaultFvmMessagesInterpreter<DB> = FvmMessagesInterpreter<DB, DefaultModule>;
pub type DefaultFvmGenesisState<DB> = FvmGenesisState<DB, DefaultModule>;
```

**Then:**
- Most code uses `DefaultFvmExecState<DB>` (still feature-gated)
- Only top-level app needs to know about modules
- Fewer files to change

**Pros:**
- Faster completion
- Less invasive
- Still achieves modularity goal

**Cons:**
- Less flexible (need recompile to change module)
- Type aliases hide the generic nature

### Option 3: Hybrid Approach ⏰ Est: 4-6 hours

1. **Create type aliases** for internal use
2. **Keep generics** at the public API boundary
3. **App layer** stays generic for true modularity
4. **Internal code** uses type aliases for simplicity

**Example:**
```rust
// Public API - fully generic
pub trait MessagesInterpreter<DB, M: ModuleBundle> { ... }

// Internal convenience
type FvmExecState<DB> = fvm::state::FvmExecState<DB, DefaultModule>;
type FvmMessagesInterpreter<DB> = fvm::interpreter::FvmMessagesInterpreter<DB, DefaultModule>;
```

### Option 4: Pause and Commit Phase 1 ⏰ Est: 30 min

**Checkpoint current progress:**
- Phase 1 is production-ready
- Phase 2 core types done (valuable even incomplete)
- Return to Phase 2 in fresh session

**Pros:**
- Preserve excellent Phase 1 work
- Clear stopping point
- Can rethink approach

**Cons:**
- Doesn't finish Phase 2
- Branch won't compile

---

## Recommendation

Given complexity,I recommend **Option 3 (Hybrid)**:

### Why Hybrid?

1. **Best of both worlds**:
   - Generic at API boundary (app can choose module)
   - Type aliases internally (less churn)

2. **Incremental path**:
   - Can finish in one session
   - Less risky than full generic propagation

3. **Still meets goals**:
   - Module system works
   - Compile-time selection
   - Clean architecture

### Implementation

```rust
// 1. Create module selection (NEW FILE: fendermint/vm/interpreter/src/fvm/module_selection.rs)
#[cfg(feature = "storage-node")]
pub type SelectedModule = storage_node_module::StorageNodeModule;

#[cfg(not(feature = "storage-node"))]
pub type SelectedModule = fendermint_module::NoOpModuleBundle;

// 2. Create type aliases for internal use
pub type FvmExecState<DB> = fvm::state::FvmExecState<DB, SelectedModule>;
pub type FvmMessagesInterpreter<DB> = fvm::interpreter::FvmMessagesInterpreter<DB, SelectedModule>;

// 3. Keep public API generic
#[async_trait]
pub trait MessagesInterpreter<DB, M: ModuleBundle> {
    // ... stays generic
}

// 4. Implement for the selected module
impl<DB> MessagesInterpreter<DB, SelectedModule> for FvmMessagesInterpreter<DB> {
    // ... concrete implementation
}
```

This way:
- ✅ Module framework works (Phase 1 success)
- ✅ Compile-time selection (#[cfg])
- ✅ Less code churn (~10 files instead of 30+)
- ✅ Can finish in this session
- ✅ Can still remove #[cfg] later by making app generic

---

## Your Decision

Which option would you prefer?

1. **Continue** full generic approach (6-10 hours)
2. **Simplify** with type aliases everywhere (2-3 hours)
3. **Hybrid** - generics at boundaries, aliases internally (4-6 hours) ⭐
4. **Pause** - commit Phase 1, revisit Phase 2 (30 min)

Let me know and I'll proceed accordingly!
