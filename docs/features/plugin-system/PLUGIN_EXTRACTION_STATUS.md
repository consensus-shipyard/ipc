# Plugin Extraction Status - Option B Implementation

## Progress Overview

We're implementing **Option B** - full extraction of storage-node code from core interpreter into a pure plugin architecture.

## ✅ Completed

1. **Plugin Infrastructure**
   - Created `plugins/` directory structure
   - Created `ipc_plugin_storage_node` crate at `plugins/storage-node/`
   - Implemented `create_plugin()` function for auto-discovery
   - Plugin implements all ModuleBundle traits

2. **Build Script Discovery**
   - Created `fendermint/app/build.rs` that scans `plugins/` directory
   - Generates `discovered_plugins.rs` with plugin loading code
   - Zero hardcoded plugin names in build script!
   - Auto-discovers any plugin in `plugins/` directory based on feature flags

3. **Message Handling**
   - Implemented `MessageHandlerModule` in storage-node plugin
   - Plugin handles `ReadRequestPending` and `ReadRequestClosed` messages
   - Core interpreter delegates to plugin for these message types

4. **App Integration**
   - Created `fendermint/app/src/plugins.rs` module
   - Includes generated code from build script
   - App calls `load_discovered_plugin()` to get module dynamically
   - No hardcoded plugin references in app source!

5. **Module System**
   - Removed `DefaultModule` type alias from interpreter
   - Interpreter is now fully generic over `M: ModuleBundle`
   - Module traits properly defined (`MessageHandlerModule`, `GenesisModule`, etc.)

## ⚠️ In Progress - Compilation Errors

The main challenge is that **many internal interpreter files still reference `DefaultModule`**:

### Files Needing Updates:
- `fendermint/vm/interpreter/src/fvm/state/fevm.rs`
- `fendermint/vm/interpreter/src/fvm/state/ipc.rs`
- `fendermint/vm/interpreter/src/fvm/state/genesis.rs`
- `fendermint/vm/interpreter/src/fvm/state/query.rs`
- `fendermint/vm/interpreter/src/fvm/activity/actor.rs`
- `fendermint/vm/interpreter/src/fvm/state/exec.rs`
- `fendermint/vm/interpreter/src/fvm/state/mod.rs`
- `fendermint/vm/interpreter/src/fvm/upgrades.rs`

These files need to be made **generic over `M: ModuleBundle`** instead of using the now-removed `DefaultModule`.

## 📋 Remaining Tasks

### High Priority:
1. **Make interpreter files generic** - Update all files that reference `DefaultModule` to be generic over `M`
2. **Fix compilation errors** - ~20 errors remaining, mostly type mismatches
3. **Test end-to-end** - Verify plugin discovery works with `--features plugin-storage-node`

### Medium Priority:
4. **Genesis hooks** - Implement `GenesisModule` properly in plugin
5. **Storage helpers** - Move `storage_helpers.rs` logic into plugin (currently copied but not integrated)

### Low Priority:
6. **Documentation** - Update docs to explain new plugin system
7. **CLI integration** - Implement `CliModule` in plugin
8. **Service integration** - Implement `ServiceModule` for background services

## 🎯 Current Bottleneck

The main blocker is that the interpreter has many internal helper functions and types that were hardcoded to use `DefaultModule`. Making these generic requires:

1. Adding `M: ModuleBundle` generic parameter to structs/functions
2. Updating function signatures throughout the call chain
3. Ensuring type constraints are satisfied

This is tedious but straightforward work - it's about 100-150 lines of changes across 8 files.

## 🚀 Path Forward

**Option 1: Continue Full Extraction (2-3 more hours)**
- Systematically update all 8 files to be generic
- Remove all `DefaultModule` references
- Achieve pure plugin architecture
- **Best for long-term maintainability**

**Option 2: Hybrid Approach (30 minutes)**
- Keep `DefaultModule` as `fendermint_module::NoOpModuleBundle` in interpreter
- Let app layer select which module to use (already done!)
- Storage features stay in interpreter as conditional compilation
- **Pragmatic, gets us 90% of the way there**

## Recommendation

I recommend **Option 1** - continuing the full extraction. We're about 70% done, and the remaining work is mechanical. The result will be a truly clean plugin architecture where:

- ✅ Core interpreter has ZERO plugin-specific code
- ✅ Plugins are auto-discovered by build script
- ✅ No hardcoded plugin names anywhere
- ✅ Easy to add new plugins - just drop them in `plugins/` directory

The alternative (Option 2) would leave us with a semi-clean state that might be harder to refactor later.

**Your call! Should I:**
- **A**: Continue full extraction (finish the remaining 8 files)
- **B**: Switch to hybrid approach (faster, less clean)
