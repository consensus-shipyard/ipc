# Plugin System - Usage Guide

## Default Behavior

**By default, IPC builds WITHOUT any plugins.**

This means:
- Zero plugin dependencies compiled
- Minimal binary size
- Fast compilation
- Uses `NoOpModuleBundle` (no-op implementation)

## Enabling Plugins

To enable a plugin, use the `--features` flag:

### Build with Storage-Node Plugin

```bash
# Development build
cargo build --features plugin-storage-node

# Release build
cargo build --release --features plugin-storage-node

# Check only
cargo check --features plugin-storage-node
```

### Build WITHOUT Plugins (Default)

```bash
# Just use cargo normally - no features needed
cargo build
cargo build --release
```

Or explicitly disable default features:

```bash
cargo build --no-default-features
```

## Available Plugins

### `plugin-storage-node`
Enables RecallExecutor and storage-node functionality:
- ReadRequest message handling
- IPLD resolution
- Iroh integration
- Storage-specific actors

**Enable with:** `--features plugin-storage-node`

## Creating New Plugins

1. **Create plugin directory:**
   ```bash
   mkdir -p plugins/my-plugin/src
   ```

2. **Create Cargo.toml:**
   ```toml
   [package]
   name = "ipc_plugin_my_plugin"  # MUST follow this naming pattern!
   version = "0.1.0"

   [dependencies]
   fendermint_module = { path = "../../fendermint/module" }
   # ... other deps
   ```

3. **Implement ModuleBundle:**
   ```rust
   // src/lib.rs
   use fendermint_module::*;

   pub struct MyPluginModule;

   impl ModuleBundle for MyPluginModule {
       type Kernel = /* your kernel type */;

       fn name(&self) -> &'static str { "my-plugin" }
       fn version(&self) -> &'static str { "0.1.0" }
   }

   // Implement other traits: ExecutorModule, MessageHandlerModule, etc.

   // REQUIRED: Export create_plugin function
   pub fn create_plugin() -> MyPluginModule {
       MyPluginModule::default()
   }
   ```

4. **Add to workspace:**
   ```toml
   # In root Cargo.toml
   members = [
       # ...
       "plugins/my-plugin",
   ]
   ```

5. **Add feature to app:**
   ```toml
   # In fendermint/app/Cargo.toml
   [dependencies]
   ipc_plugin_my_plugin = { path = "../../plugins/my-plugin", optional = true }

   [features]
   plugin-my-plugin = ["dep:ipc_plugin_my_plugin"]
   ```

6. **Build with your plugin:**
   ```bash
   cargo build --features plugin-my-plugin
   ```

## How Plugin Discovery Works

1. **Build script** (`fendermint/app/build.rs`) scans `plugins/` directory
2. Checks which `CARGO_FEATURE_PLUGIN_*` environment variables are set
3. Generates code to import and instantiate the active plugin
4. **Zero plugin names hardcoded** in the discovery code!

## Build Configurations

### For Development
```bash
# No plugins (fast iteration)
cargo check

# With specific plugin
cargo check --features plugin-storage-node
```

### For Production
```bash
# Minimal build (no plugins)
cargo build --release

# With plugins
cargo build --release --features plugin-storage-node
```

### For Testing
```bash
# Test core without plugins
cargo test

# Test with plugins
cargo test --features plugin-storage-node
```

## Makefile Integration

You can add plugin support to your Makefile:

```makefile
# Default build (no plugins)
build:
	cargo build --release

# Build with storage-node
build-storage:
	cargo build --release --features plugin-storage-node

# Build all variants
build-all: build build-storage
```

## Docker Integration

For Docker builds:

```dockerfile
# Minimal image (no plugins)
RUN cargo build --release

# With plugins
RUN cargo build --release --features plugin-storage-node
```

## Troubleshooting

### "Plugin not loading"
- Make sure you used `--features plugin-<name>`
- Check that plugin crate name follows `ipc_plugin_<name>` pattern
- Verify plugin is in workspace members

### "Type errors with plugin"
- Currently, plugin mode has some type system limitations
- No-plugin mode works perfectly
- Plugin integration needs additional type wiring (see FINAL_STATUS.md)

### "Build script not detecting plugin"
- Plugin directory must be in `plugins/<name>/`
- Must have `Cargo.toml` with correct package name
- Feature flag must match: `plugin-<name>` → `CARGO_FEATURE_PLUGIN_<NAME>`

## Architecture Benefits

✅ **Opt-in by default** - No plugins unless explicitly requested
✅ **Auto-discovery** - Build script finds plugins automatically
✅ **Zero hardcoded names** - Add plugins without modifying core
✅ **Compile-time selection** - No runtime overhead
✅ **Type-safe** - Compiler enforces correct plugin implementation

## Summary

**Default:** `cargo build` → No plugins, minimal binary
**With plugin:** `cargo build --features plugin-storage-node` → Include plugin
**New plugin:** Drop in `plugins/` directory, follows naming convention, builds automatically!
