# Recall Migration Status

## Current Progress

### ✅ Phase 0: Preparation - COMPLETED
- [x] Created `recall-migration` branch from latest main (commit: 984fc4a4)
- [x] Copied `recall/` directory from ipc-recall branch
- [x] Added recall modules to workspace Cargo.toml
- [x] Added missing workspace dependencies:
  - `ambassador = "0.3.5"`
  - `iroh = "0.35"`
  - `iroh-base = "0.35"`
  - `iroh-blobs = "0.35"`
  - `iroh-relay = "0.35"`
  - `iroh-quinn = "0.13"`
  - `n0-future = "0.1.2"`
  - `quic-rpc = "0.20"`
  - `replace_with = "0.1.7"`
  - `entangler` (git dependency)
  - `entangler_storage` (git dependency)

### 🔄 Phase 1: Core Dependencies - IN PROGRESS

**Current Status:** Setting up recall modules

**Blockers Identified:**
1. `recall/executor` depends on `fendermint_actor_blobs_shared` which doesn't exist on main yet
2. `recall_sol_facade` workspace dependency reference found but source unknown
3. Need to port Recall actors before executor can compile

**Next Steps:**
1. Copy Recall actor components from ipc-recall:
   - `fendermint/actors/blobs/` (full directory with shared/)
   - `fendermint/actors/bucket/`
   - `fendermint/actors/blob_reader/`
   - `fendermint/actors/recall_config/`
2. Update workspace to include these actors
3. Try compiling recall/ipld, recall/kernel first (no actor dependencies)
4. Then move to recall/syscalls, recall/executor

## Branch Information

**Branch Name:** `recall-migration`
**Based On:** `main` @ commit `984fc4a4` (feat: add f3 cert actor)
**Original Branch:** `ipc-recall` @ commit `567108af` (fix: non-determinism from actor debug flag)
**Gap:** 959 commits behind, 77 commits ahead

## Components Ported So Far

### ✅ Ported
- `recall/` directory structure (7 modules)
- Workspace dependencies added
- Documentation:
  - `docs/ipc/recall-vote-tally.md`
  - `docs/ipc/recall-migration-guide.md`

### ⏳ Pending
- Recall actors (blobs, bucket, blob_reader, recall_config, timehub)
- VM integration (iroh_resolver)
- Application layer integration
- Contract updates
- Tests

## Build Status

**Current Error:**
```
error: failed to load manifest for workspace member `/Users/philip/github/ipc/recall/executor`

Caused by:
  failed to parse manifest at `/Users/philip/github/ipc/recall/executor/Cargo.toml`

Caused by:
  cannot find `fendermint_actor_blobs_shared` in workspace
```

**Resolution:** Need to port actors first

## Recommended Next Actions

### Immediate (Today)
1. **Copy Recall actors from ipc-recall branch:**
   ```bash
   git checkout ipc-recall -- fendermint/actors/blobs/
   git checkout ipc-recall -- fendermint/actors/bucket/
   git checkout ipc-recall -- fendermint/actors/blob_reader/
   git checkout ipc-recall -- fendermint/actors/recall_config/
   ```

2. **Add actors to workspace Cargo.toml**

3. **Test basic compilation:**
   ```bash
   cargo check -p recall_ipld
   cargo check -p recall_kernel
   cargo check -p fendermint_actor_blobs_shared
   ```

### Short-term (This Week)
1. Fix FVM API compatibility issues in recall modules
2. Update contract binding imports in actor sol_facades
3. Port iroh_resolver VM component
4. Update chain interpreter for blob messages

### Medium-term (Next Week)
1. Integration testing of uploaded → resolution → finalization flow
2. Genesis integration
3. Application layer (app.rs) updates
4. End-to-end testing

## Risks & Mitigations

### High Risk Items
1. **FVM 4.3 → 4.7.4 upgrade**
   - **Risk:** API incompatibilities in kernel/executor
   - **Mitigation:** Incremental testing, FVM changelog review

2. **Iroh 0.35 compatibility**
   - **Risk:** P2P layer might not work
   - **Mitigation:** Test early, have fallback plan

3. **Actor dependencies**
   - **Risk:** Circular dependencies, complex build order
   - **Mitigation:** Port in dependency order

### Medium Risk Items
1. **Contract binding paths changed**
   - **Mitigation:** Straightforward find/replace

2. **Vote tally integration**
   - **Mitigation:** Existing code in topdown/voting.rs

## Key Decisions Made

1. **Use incremental migration approach** rather than direct merge
2. **Start with recall/ modules** before fendermint components
3. **Use Iroh 0.35** (one version ahead of what recall branch had)
4. **Keep entanglement as external git dependency**

## Timeline Estimate

- **Phase 0 (Prep):** ✅ Complete (1 day)
- **Phase 1 (Core):** 🔄 In Progress (2-3 days remaining)
- **Phase 2 (Iroh):** ⏳ Not Started (2-3 days)
- **Phase 3 (Executor):** ⏳ Not Started (3-4 days)
- **Phase 4 (Actors):** ⏳ Not Started (5-7 days)
- **Phase 5+ (Integration):** ⏳ Not Started (8-10 days)

**Total Estimated:** 6-8 weeks (realistic scenario)

## Files Modified

```
Modified:
  Cargo.toml                              (workspace configuration)

Added:
  recall/                                  (entire directory)
  docs/ipc/recall-vote-tally.md           (documentation)
  docs/ipc/recall-migration-guide.md      (documentation)
  docs/ipc/recall-migration-status.md     (this file)
```

## Useful Commands

```bash
# Check status
git status

# See what's in recall/ on ipc-recall
git show ipc-recall:recall/

# See what actors exist on ipc-recall
git show ipc-recall:fendermint/actors/

# Test compilation
cargo check -p recall_kernel

# See dependency tree
cargo tree -p recall_kernel

# Check for FVM usage
rg "fvm::" recall/

# View migration guide
code docs/ipc/recall-migration-guide.md
```

## Notes

- All recall code uses FVM workspace dependencies, so will pick up FVM 4.7.4
- Iroh bumped to 0.35 (was 0.34 in recall branch guide)
- Entanglement library hosted at github.com/recallnet/entanglement
- Some components will need iterative fixes as dependencies are resolved

---

**Last Updated:** 2024-11-04
**Status:** Phase 1 in progress
**Next Milestone:** Complete recall module compilation

