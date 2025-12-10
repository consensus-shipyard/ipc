# Module System - Phase 2 Progress

**Status:** 🔄 In Progress
**Phase:** 2 - Core Integration
**Started:** December 4, 2025

---

## Goal

Make core Fendermint components generic over `ModuleBundle`, removing hardcoded conditional compilation directives.

## Progress Tracker

### Step 1: Add Module Dependency ✅
- [x] Add `fendermint_module` to interpreter Cargo.toml

### Step 2: Make FvmExecState Generic 🔄
- [ ] Add generic parameter `M: ModuleBundle`
- [ ] Replace hardcoded `RecallExecutor` with `M::Executor`
- [ ] Store module instance
- [ ] Update `new()` constructor
- [ ] Update all methods using executor

### Step 3: Make FvmMessagesInterpreter Generic
- [ ] Add generic parameter `M: ModuleBundle`
- [ ] Store module instance
- [ ] Update message handling to use module
- [ ] Remove `#[cfg(feature = "storage-node")]` from interpreter

### Step 4: Make App Generic
- [ ] Add generic parameter to `App<M>`
- [ ] Update service initialization
- [ ] Remove `#[cfg]` from app layer

### Step 5: Feature-Gated Type Aliases
- [ ] Create `DefaultModule` type alias
- [ ] Create `DefaultApp` type alias
- [ ] Create `DefaultInterpreter` type alias

### Step 6: Remove All #[cfg] Directives
Progress: 0/22 locations

### Step 7: Verification
- [ ] Compile with storage-node feature
- [ ] Compile without storage-node feature
- [ ] Run tests in both configurations

---

## Current Work

Working on: Making `FvmExecState` generic over `ModuleBundle`

## Notes

- Using terminology "module" instead of "plugin" throughout
- Maintaining zero-cost abstraction principle
- All changes preserve backward compatibility via type aliases

### Files Updated
- ✅ fvm/state/exec.rs - FvmExecState<DB, M>
- ✅ fvm/interpreter.rs - FvmMessagesInterpreter<DB, M>
- ✅ fvm/executions.rs - execution functions
- ✅ fvm/state/genesis.rs - FvmGenesisState<DB, M>
- ✅ fvm/upgrades.rs - MigrationFunc<DB, M>
