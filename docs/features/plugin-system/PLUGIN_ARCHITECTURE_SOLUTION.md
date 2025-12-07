# True Plugin Architecture - Zero Core References

## Current Problem

You're right! Even with the module system, we still have hardcoded references:

**In `fendermint/vm/interpreter/Cargo.toml`:**
```toml
storage_node_executor = { path = "../../../storage-node/executor", optional = true }
storage_node_module = { path = "../../../storage-node/module", optional = true }
# ... more storage-node deps

[features]
storage-node = [
    "dep:storage_node_executor",
    "dep:storage_node_module",
    # ...
]
```

**In `fendermint/vm/interpreter/src/fvm/default_module.rs`:**
```rust
#[cfg(feature = "storage-node")]
pub type DefaultModule = storage_node_module::StorageNodeModule;
```

This violates the plugin architecture principle! ❌

## Solution: Move Plugin Selection to Application Layer

### Architecture Change

```
┌─────────────────────────────────────────┐
│  Core Layer (NO plugin references)     │
│  - fendermint_vm_interpreter            │
│  - fendermint_module (traits only)      │
│  - Generic over M: ModuleBundle          │
└─────────────────────────────────────────┘
                    ▲
                    │ depends on (generic)
                    │
┌─────────────────────────────────────────┐
│  Plugin Layer (separate crates)         │
│  - storage_node_module                   │
│  - other_plugin_module                   │
│  - custom_modules...                     │
└─────────────────────────────────────────┘
                    ▲
                    │ imports & selects
                    │
┌─────────────────────────────────────────┐
│  Application Layer                       │
│  - fendermint_app                        │
│  - Chooses which plugin to use           │
│  - Wires everything together             │
└─────────────────────────────────────────┘
```

## Implementation Steps

### Step 1: Remove Plugin References from Core

**`fendermint/vm/interpreter/Cargo.toml`:**
```toml
[dependencies]
# Core dependencies only - NO plugin references
fendermint_module = { path = "../../module" }
fvm = { workspace = true }
# ... other core deps

# REMOVE these:
# storage_node_executor = { ... }
# storage_node_module = { ... }

[features]
# Keep this generic
bundle = []
# REMOVE storage-node feature entirely
```

**`fendermint/vm/interpreter/src/fvm/default_module.rs`:**
```rust
// Remove this file entirely, or make it export nothing
// The module selection happens in the app layer now
```

**`fendermint/vm/interpreter/src/fvm/mod.rs`:**
```rust
// Remove the DefaultModule type alias
// Everything stays generic over M: ModuleBundle
```

### Step 2: Keep Core Fully Generic

**`fendermint/vm/interpreter/src/fvm/state/exec.rs`:**
```rust
// Already generic - no changes needed!
pub struct FvmExecState<DB, M = fendermint_module::NoOpModuleBundle>
where
    DB: Blockstore + Clone + 'static,
    M: ModuleBundle,
{
    // ...
}
```

**`fendermint/vm/interpreter/src/fvm/interpreter.rs`:**
```rust
// Already generic - no changes needed!
pub struct FvmMessagesInterpreter<DB, M>
where
    DB: Blockstore + Clone + Send + Sync + 'static,
    M: ModuleBundle,
{
    // ...
}
```

### Step 3: Move Plugin Selection to App Layer

**`fendermint/app/Cargo.toml`:**
```toml
[dependencies]
fendermint_module = { path = "../module" }
fendermint_vm_interpreter = { path = "../vm/interpreter" }

# Plugin imports happen HERE, not in core
storage_node_module = { path = "../../storage-node/module", optional = true }
# other_plugin_module = { path = "../../plugins/other", optional = true }

[features]
default = ["plugin-storage-node"]

# Feature flags control which plugin the APP uses
plugin-storage-node = ["dep:storage_node_module"]
plugin-other = ["dep:other_plugin_module"]
plugin-none = []  # Use baseline NoOpModuleBundle
```

**`fendermint/app/src/plugin_selector.rs`** (new file):
```rust
//! Plugin selection at the application layer.
//!
//! This is the ONLY place that knows about specific plugins.

use fendermint_module::{ModuleBundle, NoOpModuleBundle};
use std::sync::Arc;

/// Select which module to use based on compile-time features.
///
/// This function is the single point where plugin selection happens.
/// Core code remains generic and never imports plugins directly.
pub fn select_module() -> Arc<dyn ModuleBundle<Kernel = /* appropriate type */>> {
    #[cfg(feature = "plugin-storage-node")]
    {
        tracing::info!("Loading plugin: storage-node");
        Arc::new(storage_node_module::StorageNodeModule::default())
    }

    #[cfg(all(feature = "plugin-other", not(feature = "plugin-storage-node")))]
    {
        tracing::info!("Loading plugin: other");
        Arc::new(other_plugin_module::OtherModule::default())
    }

    #[cfg(all(
        not(feature = "plugin-storage-node"),
        not(feature = "plugin-other")
    ))]
    {
        tracing::info!("No plugin loaded, using baseline NoOpModuleBundle");
        Arc::new(NoOpModuleBundle::default())
    }
}
```

**`fendermint/app/src/service/node.rs`:**
```rust
use crate::plugin_selector;

pub async fn run(...) {
    // Select module at app layer
    let module = plugin_selector::select_module();

    let interpreter = FvmMessagesInterpreter::new(
        module,
        // ... rest of params
    );

    // ...
}
```

## Alternative: Runtime Plugin Registry

For even more flexibility, use a registry pattern:

**`fendermint/module/src/registry.rs`:**
```rust
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

type ModuleConstructor = Box<dyn Fn() -> Arc<dyn ModuleBundle> + Send + Sync>;

static PLUGIN_REGISTRY: Lazy<RwLock<HashMap<String, ModuleConstructor>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Register a plugin constructor
pub fn register_plugin<F>(name: &str, constructor: F)
where
    F: Fn() -> Arc<dyn ModuleBundle> + Send + Sync + 'static,
{
    PLUGIN_REGISTRY
        .write()
        .unwrap()
        .insert(name.to_string(), Box::new(constructor));
}

/// Get a plugin by name
pub fn get_plugin(name: &str) -> Option<Arc<dyn ModuleBundle>> {
    PLUGIN_REGISTRY
        .read()
        .unwrap()
        .get(name)
        .map(|ctor| ctor())
}

/// List all registered plugins
pub fn list_plugins() -> Vec<String> {
    PLUGIN_REGISTRY
        .read()
        .unwrap()
        .keys()
        .cloned()
        .collect()
}
```

**Plugin auto-registers itself:**
```rust
// storage-node/module/src/lib.rs

use fendermint_module::registry;

// Auto-register on load
#[used]
static REGISTER: () = {
    registry::register_plugin("storage-node", || {
        Arc::new(StorageNodeModule::default())
    });
};
```

**App selects by name:**
```rust
// fendermint/app/src/service/node.rs

let plugin_name = settings.module.plugin_name.unwrap_or("storage-node");
let module = fendermint_module::registry::get_plugin(&plugin_name)
    .unwrap_or_else(|| Arc::new(NoOpModuleBundle::default()));
```

## Comparison of Approaches

### Approach 1: Compile-Time Selection (Recommended)

**Pros:**
- ✅ Zero runtime overhead
- ✅ Compile-time type checking
- ✅ Clear and explicit
- ✅ Easy to understand
- ✅ No magic behavior

**Cons:**
- ❌ Requires recompilation to change plugins
- ❌ Slightly more boilerplate

**Use when:** You want clean architecture with compile-time safety (recommended for most cases)

### Approach 2: Runtime Registry

**Pros:**
- ✅ Can load plugins without recompilation
- ✅ Configuration-based selection
- ✅ Easy to add new plugins

**Cons:**
- ❌ More complex
- ❌ Runtime overhead (minimal)
- ❌ Type erasure via trait objects
- ❌ Potential for runtime errors

**Use when:** You need to swap plugins without rebuilding, or load plugins from config files

### Approach 3: Dynamic Loading (.so/.dylib)

**Pros:**
- ✅ True runtime plugin system
- ✅ Plugins compiled separately
- ✅ Can update plugins independently

**Cons:**
- ❌ Very complex
- ❌ Requires unsafe code
- ❌ C FFI compatibility needed
- ❌ Platform-specific behavior
- ❌ Harder debugging

**Use when:** You need binary-compatible plugins distributed separately (rarely needed)

## Recommended Implementation

For IPC, I recommend **Approach 1 (Compile-Time Selection)** because:

1. **Clean Architecture:** Core has zero plugin knowledge
2. **Type Safety:** Full compile-time checks
3. **Performance:** Zero runtime overhead
4. **Simplicity:** Easy to understand and maintain
5. **Rust Philosophy:** Uses Rust's strength (zero-cost abstractions)

The app layer is the perfect place for "composition" - it knows about all the pieces and wires them together, while the core stays generic and reusable.

## Summary

**Old way (what we have now):**
```
Core (interpreter) → directly depends on → storage_node_module
```

**New way (true plugin architecture):**
```
Core (interpreter) → stays generic over M: ModuleBundle
                            ↑
                            │
App layer → imports plugins → wires them together
```

This achieves **true separation** - the core crate has no idea plugins even exist! 🎉
