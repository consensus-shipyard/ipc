# Recall Migration Session Log

## Session Date: 2024-11-04

### Progress Summary

**Branch:** `recall-migration` (based on main @ `984fc4a4`)

#### ✅ Completed

1. **Phase 0 - Preparation** (COMPLETE)
   - Created `recall-migration` branch from latest main
   - Copied `recall/` directory structure (7 modules)
   - Added recall modules to workspace Cargo.toml
   - Created comprehensive migration documentation
   - **Commit:** `c4262763` - "feat: initial recall migration setup"

2. **Phase 1 - Core Dependencies** (PARTIAL)
   - Ported all Recall actors:
     - `fendermint/actors/blobs/` (with shared/ and testing/)
     - `fendermint/actors/bucket/`
     - `fendermint/actors/blob_reader/`
     - `fendermint/actors/machine/`
     - `fendermint/actors/timehub/`
     - `fendermint/actors/recall_config/` (with shared/)
   - Added workspace dependencies:
     - `iroh` 0.35
     - `iroh-base` 0.35
     - `iroh-blobs` 0.35
     - `iroh-relay` 0.35
     - `iroh-quinn` 0.13
     - `ambassador` 0.3.5
     - `n0-future` 0.1.2
     - `quic-rpc` 0.20
     - `replace_with` 0.1.7
     - `blake3` 1.5
     - `data-encoding` 2.3.3
     - `entangler` (git dependency)
     - `entangler_storage` (git dependency)
     - `recall_sol_facade` (git dependency)

#### 🔄 Current Status

**Blocker:** `fil_actor_adm` dependency missing

The `fendermint_actor_machine` depends on `fil_actor_adm` which doesn't exist in the main branch's builtin-actors.

**Investigation Findings:**
- Main branch uses upstream `builtin-actors` from GitHub (no local copy)
- ipc-recall branch has custom `builtin-actors/actors/adm/` but it's not in the git tree
- ADM (Autonomous Data Management) appears to be a Recall-specific actor
- Need to determine source of ADM actor or remove machine actor dependency

#### ⏸️ Next Actions

**Option 1: Find ADM Actor Source**
- Check if ADM exists in a separate Recall repository
- Add as external dependency if available
- Or implement minimal ADM interface

**Option 2: Remove Machine Actor** (temporary)
- Remove `fendermint/actors/machine/` from migration for now
- Update bucket actor to not depend on machine
- Add machine back later when ADM is available

**Option 3: Mock ADM Actor** (for compilation)
- Create minimal ADM actor stub to satisfy dependencies
- Focus on getting recall_ipld and other core modules compiling first
- Come back to full ADM implementation later

### Recommended Approach

**Continue with Option 2** - Remove machine actor temporarily:
1. Remove `fendermint/actors/machine/` and `fendermint/actors/timehub/` from workspace
2. Check if bucket actually needs machine or if it's optional
3. Get core recall modules compiling first (ipld, kernel, iroh_manager)
4. Then work on actors that have fewer dependencies

### Dependencies Successfully Resolved

```toml
# Iroh P2P
iroh = "0.35"
iroh-base = "0.35"
iroh-blobs = "0.35"
iroh-relay = "0.35"
iroh-quinn = "0.13"

# Recall-specific
ambassador = "0.3.5"
n0-future = "0.1.2"
quic-rpc = "0.20"
replace_with = "0.1.7"
blake3 = "1.5"
data-encoding = "2.3.3"

# External Recall libraries
entangler (github.com/recallnet/entanglement)
entangler_storage (github.com/recallnet/entanglement)  
recall_sol_facade (github.com/recallnet/contracts)
```

### Key Learnings

1. **Dependency Chain Complexity**
   - Recall actors have deep dependency trees
   - Custom builtin actors (ADM) not in upstream
   - Need incremental approach: start with low-dependency modules

2. **FVM Version**
   - Main uses FVM 4.7.4
   - Recall code uses FVM workspace deps (will automatically use 4.7.4)
   - May need API compatibility fixes later

3. **Contract Bindings**
   - Recall uses external `recall_sol_facade` from recallnet/contracts repo
   - Includes facades for: blobs, credit, gas, bucket, blob-reader, machine, config

4. **Architecture Differences**
   - Main: builtin-actors from upstream GitHub
   - ipc-recall: custom builtin-actors directory (but not tracked properly)
   - Need to reconcile actor architecture

### Files Changed So Far

```
M  Cargo.toml (workspace configuration)
A  recall/ (7 modules, 28 files)
A  fendermint/actors/blobs/ (with shared/, testing/)
A  fendermint/actors/bucket/
A  fendermint/actors/blob_reader/
A  fendermint/actors/machine/
A  fendermint/actors/timehub/
A  fendermint/actors/recall_config/ (with shared/)
A  docs/ipc/recall-migration-guide.md
A  docs/ipc/recall-migration-status.md
A  docs/ipc/recall-vote-tally.md
```

### Next Session TODO

1. **Investigate ADM Actor:**
   - Search recallnet GitHub org for ADM
   - Check if ADM is essential or optional
   - Determine migration path for machine actor

2. **Simplify Dependency Tree:**
   - Remove machine/timehub temporarily
   - Get basic recall modules compiling:
     - recall_ipld ✓
     - recall_kernel_ops ✓
     - recall_kernel
     - recall_iroh_manager
     - recall_syscalls
   
3. **Test Basic Components:**
   ```bash
   cargo check -p recall_ipld
   cargo check -p recall_kernel
   cargo check -p recall_iroh_manager
   cargo test -p recall_ipld
   ```

4. **Actor Compilation:**
   - Start with simplest actors (recall_config, blob_reader)
   - Then blobs actor (most complex)
   - Leave bucket for later if it needs machine

### Time Invested

- Setup & Documentation: ~2 hours
- Dependency Resolution: ~1 hour  
- **Total:** ~3 hours

### Estimated Remaining

- Phase 1 completion: 4-6 hours
- Phase 2-4: 20-30 hours
- Testing & Integration: 10-15 hours
- **Total Remaining:** 35-50 hours (1-1.5 weeks full-time)

---

**Status:** Paused at dependency resolution  
**Blocker:** fil_actor_adm not found  
**Next:** Investigate ADM source or remove machine actor

