# Architecture Decision: Storage Plugin Isolation Level

## Context

We've successfully moved storage actors from `fendermint/actors/` to `storage-node/actors/`, achieving the stated goal of "not having any references to the storage plugin in the core code."

However, there are still `#[cfg(feature = "storage-node")]` feature flags throughout fendermint for:
- Genesis initialization (1 location)
- Message handling (2 locations)  
- Service initialization (4 locations)
- Plus ~1000 lines of storage-specific code in fendermint core

## Question

**How far should we go with plugin isolation?**

## Options

### Option A: Pragmatic Hybrid (Current State + Minor Cleanup) ⚡ FAST

**What it is:**
- Actors live in `storage-node/actors/` ✅ (DONE)
- Integration code stays in fendermint behind feature flags
- Plugin is primarily for actor ownership and executor

**Pros:**
- ✅ Actors are already isolated
- ✅ Minimal additional work (2-3 days)
- ✅ No complex API changes needed
- ✅ Storage functionality is opt-in via feature flag
- ✅ Good enough for most modularity goals

**Cons:**
- ⚠️ Fendermint still has storage-specific code
- ⚠️ Compile-time coupling via feature flags
- ⚠️ Can't add new storage plugins without modifying fendermint

**Work Required:**
1. Document the hybrid architecture
2. Clean up dependencies in Cargo.toml
3. Maybe: Move storage_resolver to plugin
4. Test that feature flag works correctly

**Effort:** 2-3 days

---

### Option B: Full Plugin Extraction 🔨 THOROUGH

**What it is:**
- Zero `#[cfg(feature = "storage-node")]` in fendermint
- All storage code lives in plugin
- Module system extended to support runtime plugin hooks
- Plugin-based genesis, messages, and services

**Pros:**
- ✅ True zero compile-time coupling
- ✅ Future plugins can follow same pattern
- ✅ Fendermint is completely storage-agnostic
- ✅ Cleanest architecture

**Cons:**
- ⚠️ 2-3 weeks of development
- ⚠️ Requires significant module system enhancements
- ⚠️ More complex plugin API surface
- ⚠️ Potential for bugs during refactoring
- ⚠️ Might be over-engineering for current needs

**Work Required:**
1. Extend module system with new traits/APIs
2. Move storage_resolver, storage_helpers, storage_env to plugin
3. Create generic topdown finality types
4. Implement full plugin hooks
5. Remove all feature flags
6. Extensive testing

**Effort:** 2-3 weeks

---

### Option C: Incremental Enhancement 🔄 BALANCED

**What it is:**
- Start with Option A
- Gradually extract components as needed
- Extend module system incrementally
- No big-bang refactor

**Pros:**
- ✅ Ship improvements incrementally
- ✅ Learn what APIs are actually needed
- ✅ Lower risk than big refactor
- ✅ Can stop when good enough

**Cons:**
- ⚠️ Might never reach full extraction
- ⚠️ Could leave architecture in limbo
- ⚠️ Multiple rounds of changes

**Work Required:**
1. Start with Option A (actor isolation)
2. Move storage_resolver next (low coupling)
3. Add plugin hooks for genesis (medium coupling)
4. Add plugin hooks for messages (high coupling)
5. Remove feature flags one by one

**Effort:** Variable, spread over time

---

## Recommendation

**Start with Option A (Pragmatic Hybrid)**

**Reasoning:**
1. **Goal achieved:** Actors are isolated ✅
2. **Good enough:** Feature flags provide modularity
3. **Low risk:** Minimal changes to working code
4. **Fast delivery:** 2-3 days vs 2-3 weeks
5. **Can evolve:** Can move to Option C later if needed

**The 80/20 rule applies here:**
- 80% of the modularity benefit from actor isolation (done)
- 20% from removing feature flags (expensive)

**When to reconsider:**
- Need to support multiple storage plugins
- Want to compile fendermint without any storage code
- Storage plugin becomes independently versioned/released

---

## Implementation for Option A

### 1. Document Architecture (1 day)
- ✅ Create `STORAGE_DEPENDENCIES_MAP.md` (DONE)
- ✅ Create `STORAGE_PLUGIN_MIGRATION_PLAN.md` (DONE)
- Write architecture decision record
- Update project README

### 2. Clean Up Dependencies (1 day)
- Remove unused storage imports
- Consolidate feature flags where possible
- Update Cargo.toml with clear comments
- Test compilation with/without feature

### 3. Optional: Move storage_resolver (1 day)
- Move `fendermint/vm/storage_resolver/` → `plugins/storage-node/src/resolver/`
- Update imports
- Keep feature flag in node.rs for now
- Test functionality

### 4. Test & Verify
- Ensure storage-node works with feature enabled
- Document how to build with/without plugin
- Update CI if needed

---

## Decision

**[TO BE FILLED IN BY MAINTAINERS]**

- [ ] Option A: Pragmatic Hybrid
- [ ] Option B: Full Extraction
- [ ] Option C: Incremental Enhancement

**Reasoning:**

**Action Items:**

**Timeline:**
