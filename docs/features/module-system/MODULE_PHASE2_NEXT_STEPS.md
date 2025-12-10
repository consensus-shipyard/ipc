# Module System - Phase 2 Next Steps

**Current State:** Module Compiles ✅ | Interpreter: 31 errors | Time: 5.5 hours

---

## Clear Problem Identified

The `Deref` bounds on `ExecutorModule::Executor` are causing **systematic type inference failures** in Rust:

```rust
// This causes inference ambiguity:
type Executor: Executor<Kernel = K>
    + Deref<Target = <K::CallManager as CallManager>::Machine>;
```

**Why:** Rust's method resolution with Deref + generics + associated types = inference hell

---

## The Solution: Remove Deref Requirement

### Step 1: Update ExecutorModule Trait (5 min)

```rust
// In fendermint/module/src/executor.rs
pub trait ExecutorModule<K: Kernel> {
    type Executor: Executor<Kernel = K> + Send;
    // REMOVE: + Deref<...>
}
```

### Step 2: Verify FvmExecState Methods (10 min)

Check that existing methods still work:
```rust
// These already exist and forward correctly:
impl<DB, M> FvmExecState<DB, M> {
    pub fn block_height(&self) -> ChainEpoch {
        self.executor.context().epoch // ← calls deref implicitly
    }

    pub fn state_tree(&self) -> &StateTree<...> {
        self.executor.state_tree() // ← calls deref implicitly
    }
}
```

**They should work!** The Deref is used implicitly in the impl, not required as a trait bound.

### Step 3: Fix Remaining Errors (1-2 hours)

With Deref removed from trait bounds:
- E0283 errors should disappear (inference works again)
- E0308 errors should resolve (types match now)
- E0599 errors need checking

**Expected:** Most/all errors resolve automatically

---

## Implementation Checklist

```bash
# 1. Remove Deref bounds
# Edit: fendermint/module/src/executor.rs
type Executor: Executor<Kernel = K> + Send;
# (remove + Deref<...>)

# 2. Remove Machine: Send bound (no longer needed)
pub trait ExecutorModule<K: Kernel> {
    // Remove where clause
}

# 3. Update ModuleBundle trait similarly
# Edit: fendermint/module/src/bundle.rs
# Remove Machine: Send from where clause

# 4. Check compilation
cargo check -p fendermint_module
cargo check -p fendermint_vm_interpreter

# 5. Fix any remaining issues (should be minimal)
```

---

## Why This Will Work

**Current Problem:**
```
state.block_gas_tracker()
      ^^^^^^^^^^^^^^^^^ cannot infer DB
```

Compiler sees Deref in trait bounds and tries to use it for method resolution, creating ambiguity.

**After Fix:**
```
state.block_gas_tracker()
```

Deref is only used implicitly in the impl methods, not in trait resolution. No ambiguity!

---

## Estimated Time

- Remove Deref bounds: 5 min
- Test compilation: 10 min
- Fix any remaining errors: 30-60 min
- **Total: 45-75 minutes**

**Success probability: 90%**

---

## Alternative If Issues Remain

If removing Deref doesn't fully resolve issues:

1. Add explicit Machine accessor:
   ```rust
   impl<DB, M> FvmExecState<DB, M> {
       pub fn machine(&self) -> &<M::Kernel::CallManager as CallManager>::Machine {
           &*self.executor
       }
   }
   ```

2. Update methods to use accessor instead of direct deref

**Est. Time:** +30-60 min

---

## Current Files Status

**✅ Ready (No changes needed):**
- Most FvmExecState methods (already impl correctly)
- All type alias infrastructure
- All manager methods (already updated to generic)

**🔄 May Need Minor Tweaks:**
- Methods that call executor.method() directly
- Estimated: 5-10 locations

---

## Recommendation

**Do this now** - it's straightforward and should complete in <1 hour:

1. Remove Deref bounds (trait-level)
2. Test compilation
3. Fix remaining issues

This is the clean solution and should get us to green checkmarks.

**Ready to proceed?** I can do this now.
