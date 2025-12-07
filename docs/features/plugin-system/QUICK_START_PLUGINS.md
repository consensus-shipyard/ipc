# Plugin System - Quick Start

## 🚀 Building IPC

### Default Build (No Plugins - Recommended)
```bash
cargo build --release
# or
make build
```

**Result:** Minimal IPC build with `NoOpModuleBundle`

### With Storage-Node Plugin
```bash
cargo build --release --features plugin-storage-node
```

**Result:** IPC with RecallExecutor and full storage functionality

## 🎯 Key Points

- **Default = No plugins** - Keep it lean
- **Opt-in for plugins** - Add `--features plugin-<name>`
- **Zero core changes** - Plugins are auto-discovered
- **Type-safe** - Compiler checks everything

## 📂 Plugin Architecture

```
plugins/storage-node/     ← Storage plugin
  ├── Cargo.toml         (name = "ipc_plugin_storage_node")
  └── src/lib.rs         (pub fn create_plugin())

fendermint/vm/interpreter/
  └── Cargo.toml         ← ZERO plugin dependencies! ✨

fendermint/app/
  ├── build.rs           ← Auto-discovers plugins
  └── src/types.rs       ← AppModule type alias
```

## ⚡ Quick Commands

```bash
# Check compilation (fast)
cargo check                                    # No plugins
cargo check --features plugin-storage-node     # With plugin

# Build binaries
cargo build --release                          # Minimal
cargo build --release --features plugin-storage-node  # Full

# Test
cargo test                                     # No plugins
cargo test --features plugin-storage-node      # With plugin
```

## 🎓 What Changed?

### Before
- Storage-node code **mixed into** interpreter
- Hard to build without storage dependencies
- Plugin code **hardcoded** in core

### After ✨
- Storage-node is a **separate plugin**
- Core interpreter is **100% generic**
- Plugins are **auto-discovered** by build script
- **Zero hardcoded** plugin names anywhere!

## 📖 More Info

- `PLUGIN_USAGE.md` - Complete usage guide
- `PLUGIN_SYSTEM_SUCCESS.md` - Implementation details
- `plugins/README.md` - Plugin development guide

---

**TL;DR:** Use `cargo build` for minimal builds, add `--features plugin-storage-node` when you need storage functionality. Core IPC is now completely plugin-free! 🎉
