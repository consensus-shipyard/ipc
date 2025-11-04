# Recall Migration Session Log

## Session Date: 2024-11-04

### Progress Summary

**Branch:** `recall-migration` (based on main @ `984fc4a4`)  
**Latest Commit:** `e986d08e` - "fix: temporarily disable sol_facade"

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

#### 🔄 Current Status (Updated 10:47 AM)

**✅ Phase 0: COMPLETE**  
**🟡 Phase 1: PARTIAL** - 3/7 recall modules compiling

**Successfully Compiling:**
- ✅ `recall_ipld` - Custom IPLD data structures
- ✅ `recall_kernel_ops` - Kernel operations interface
- ✅ `recall_actor_sdk` - Actor SDK (with warnings, no sol_facade)

**Blocked by netwatch (upstream issue):**
- ⏸️ `recall_syscalls` - Blob operation syscalls
- ⏸️ `recall_kernel` - Custom FVM kernel
- ⏸️ `iroh_manager` - Iroh P2P node management

**Disabled Temporarily:**
- 🚫 `fendermint/actors/machine` - needs fil_actor_adm
- 🚫 `fendermint/actors/bucket` - depends on machine
- 🚫 `fendermint/actors/timehub` - depends on machine

**Previous Blocker:** `fil_actor_adm` dependency missing - **RESOLVED** by temporarily disabling dependent actors

The `fendermint_actor_machine` depends on `fil_actor_adm` which doesn't exist in the main branch's builtin-actors.

**Investigation Findings:**
- Main branch uses upstream `builtin-actors` from GitHub (no local copy)
- ipc-recall branch has custom `builtin-actors/actors/adm/` but it's not in the git tree
- ADM (Autonomous Data Management) appears to be a Recall-specific actor
- Need to determine source of ADM actor or remove machine actor dependency

#### 🚨 Critical Blocker: FVM Version Incompatibility

**Problem:** `recall_sol_facade` (from recallnet/contracts @ ad096f2) requires FVM ~4.3.0, but IPC main uses FVM 4.7.4.

**Impact:**
- All Recall actors depend on `recall_sol_facade` for Solidity event emission
- Cargo cannot resolve the conflicting FVM versions
- Cannot compile any Recall actors until resolved

**Resolution Options:**

**Option A: Upgrade recall_sol_facade (Recommended)**
1. Fork recallnet/contracts
2. Upgrade FVM dependency from 4.3.0 to 4.7.4
3. Fix any API breaking changes
4. Use forked version temporarily
5. Submit PR to upstream recallnet/contracts

**Option B: Remove sol_facade Temporarily**
1. Comment out `recall_sol_facade` dependencies in actor Cargo.toml files
2. Comment out Solidity event emission code
3. Get basic actor functionality compiling
4. Add back sol_facade support once upgraded

**Option C: Downgrade IPC FVM (Not Recommended)**
1. Would require downgrading entire IPC main branch
2. Not feasible - FVM 4.7 has critical fixes
3. Would break other components

**Recommended Path Forward:** Option B for now, then Option A in parallel

---

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

### Issues Encountered & Resolved

**1. FVM Version Conflict** (MAJOR BLOCKER - WORKAROUND APPLIED)
- **Problem:** recall_sol_facade requires FVM 4.3.0, IPC main uses FVM 4.7.4
- **Solution:** Temporarily commented out all sol_facade dependencies
- **Impact:** EVM event emission disabled, basic functionality intact
- **Status:** ✅ Workaround applied, TODO: upgrade sol_facade later

**2. ADM Actor Missing** (BLOCKER - WORKAROUND APPLIED)  
- **Problem:** machine/bucket/timehub actors need fil_actor_adm (not in main)
- **Solution:** Temporarily disabled these actors
- **Impact:** Bucket storage and timehub features unavailable
- **Status:** ✅ Workaround applied, TODO: port ADM actor later

**3. netwatch Compilation Error** (BLOCKING PROGRESS)
- **Problem:** netwatch 0.5.0 incompatible with socket2 (upstream issue)
- **Error:** `Type::RAW` not found, `From<Socket>` trait issue
- **Affects:** recall_syscalls, recall_kernel, iroh_manager
- **Status:** 🚨 **CURRENT BLOCKER** - need to fix or work around

### Commits Made

1. **c4262763** - "feat: initial recall migration setup"
   - Created branch, copied recall modules
   - Added workspace configuration and documentation

2. **b1b8491f** - "feat: port recall actors and resolve dependencies"
   - Copied all Recall actors from ipc-recall
   - Added missing dependencies (blake3, data-encoding, etc.)
   - Added recall_sol_facade dependency

3. **4003012b** - "docs: document FVM version incompatibility blocker"
   - Documented FVM 4.3 vs 4.7.4 conflict
   - Outlined resolution options
   - Temporarily disabled machine/bucket/timehub

4. **e986d08e** - "fix: temporarily disable sol_facade to resolve FVM version conflict"
   - Commented out sol_facade in all Cargo.toml files
   - Disabled EVM event emission code
   - Got 3 recall modules compiling successfully

### Time Invested

- Setup & Documentation: ~2 hours
- Dependency Resolution: ~2 hours
- FVM Compatibility Fixes: ~1 hour
- **Total:** ~5 hours

### Estimated Remaining

- Fix netwatch issue: 1-2 hours
- Phase 1 completion: 2-4 hours
- Phase 2-4: 20-30 hours
- Testing & Integration: 10-15 hours
- **Total Remaining:** 33-51 hours (1-1.5 weeks full-time)

---

**Status:** Blocked by netwatch compilation error  
**Current Blocker:** netwatch 0.5.0 socket2 incompatibility  
**Next:** Fix netwatch or work around dependency

