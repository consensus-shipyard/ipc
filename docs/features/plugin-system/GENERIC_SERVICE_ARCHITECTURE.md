# Generic Service Architecture - The Right Way

## Problem

Current `node.rs` has **hardcoded storage-node references**:

```rust
// ❌ HARDCODED - Defeats the purpose of generic modules
#[cfg(feature = "storage-node")]
use ipc_plugin_storage_node::{BlobPool, ReadRequestPool};
#[cfg(feature = "storage-node")]
use ipc_plugin_storage_node::resolver::IrohResolver;

#[cfg(feature = "storage-node")]
let blob_pool: BlobPool = ResolvePool::new();
// ... manual initialization of storage services
```

This means:
- ❌ Each plugin requires modifying `node.rs`
- ❌ Not truly modular
- ❌ Defeats the generic `ServiceModule` trait

---

## Solution: Use Generic Module APIs

### Step 1: Module Provides Services (Already Have This!)

```rust
// In plugins/storage-node/src/lib.rs
impl ServiceModule for StorageNodeModule {
    async fn initialize_services(
        &self,
        ctx: &ServiceContext,
    ) -> Result<Vec<JoinHandle<()>>> {
        // Plugin spawns its own services
        let mut handles = vec![];

        // Create pools
        let blob_pool = ResolvePool::new();
        let read_request_pool = ResolvePool::new();

        // Spawn resolvers
        let blob_resolver = IrohResolver::new(...);
        handles.push(tokio::spawn(async move {
            blob_resolver.run().await
        }));

        // Return all handles
        Ok(handles)
    }

    fn resources(&self) -> ModuleResources {
        // Expose pools and resolvers
        ModuleResources::new(StorageResources {
            blob_pool,
            read_request_pool,
        })
    }
}
```

### Step 2: App Calls Generic Methods (Need to Add This!)

```rust
// In fendermint/app/src/service/node.rs

// ✅ GENERIC - Works with ANY module
let module = std::sync::Arc::new(AppModule::default());

// Build service context
let service_ctx = ServiceContext::new(Box::new(settings.clone()))
    .with_validator_keypair(validator_keypair.as_ref().map(|k| k.to_vec()));

// ✅ Generic call - module decides what services to start
let service_handles = module.initialize_services(&service_ctx)
    .await
    .context("failed to initialize module services")?;

// ✅ Generic - get resources from module
let module_resources = module.resources();

// Store handles to keep services running
app_state.service_handles = service_handles;
```

---

## Benefits of Generic Approach

### 1. **No Hardcoded References** ✅
- No `#[cfg(feature = "storage-node")]` in node.rs
- No importing plugin-specific types
- node.rs stays clean

### 2. **True Modularity** ✅
- Add new plugins without touching node.rs
- Plugin owns its initialization logic
- Clear separation of concerns

### 3. **Resource Sharing** ✅
```rust
// Other components can access resources generically
if let Some(storage) = module_resources.get::<StorageResources>() {
    // Use storage pools
}
```

---

## Current Status

### What We Have: ✅
- ✅ `ServiceModule` trait defined
- ✅ `ServiceContext` for passing settings
- ✅ `ModuleResources` for sharing state
- ✅ Plugin implements `ServiceModule`
- ✅ Build script discovers plugins

### What's Missing: ⚠️
- ⚠️ `node.rs` still has hardcoded storage initialization (lines 136-224)
- ⚠️ `module.initialize_services()` not called in node.rs
- ⚠️ Plugin's `initialize_services()` is a stub

---

## Implementation Plan

### Phase 1: Plugin Implements Full Service Initialization

```rust
// In plugins/storage-node/src/lib.rs

pub struct StorageResources {
    pub blob_pool: Arc<BlobPool>,
    pub read_request_pool: Arc<ReadRequestPool>,
}

impl ServiceModule for StorageNodeModule {
    async fn initialize_services(
        &self,
        ctx: &ServiceContext,
    ) -> Result<Vec<JoinHandle<()>>> {
        // Extract settings
        let settings = ctx.settings_as::<AppSettings>()
            .ok_or_else(|| anyhow!("missing settings"))?;

        let validator_key = ctx.validator_keypair.as_ref()
            .ok_or_else(|| anyhow!("validator key required"))?;

        // Create pools
        let blob_pool = Arc::new(ResolvePool::new());
        let read_request_pool = Arc::new(ResolvePool::new());

        let mut handles = vec![];

        // Spawn blob resolver
        let blob_resolver = IrohResolver::new(
            /* ... configure from settings ... */
        );
        handles.push(tokio::spawn(async move {
            blob_resolver.run().await
        }));

        // Spawn read request resolver
        // ... similar ...

        // Store resources for other components
        self.resources.set(StorageResources {
            blob_pool,
            read_request_pool,
        });

        Ok(handles)
    }

    fn resources(&self) -> ModuleResources {
        ModuleResources::new(self.resources.get().unwrap())
    }
}
```

### Phase 2: Update node.rs to Call Generic Methods

```rust
// In fendermint/app/src/service/node.rs

// REMOVE lines 13-28 (hardcoded imports)
// REMOVE lines 136-224 (hardcoded initialization)

// ADD generic call:
let module = Arc::new(AppModule::default());

// Prepare context
let service_ctx = ServiceContext::new(Box::new(settings.clone()))
    .with_validator_keypair(
        validator_keypair.as_ref().map(|k| k.secret_bytes())
    )
    .with_extra(Arc::new(ExtraContext {
        client: client.clone(),
        vote_tally: parent_finality_votes.clone(),
        subnet_id: own_subnet_id.clone(),
    }));

// Generic service initialization
let service_handles = module
    .initialize_services(&service_ctx)
    .await
    .context("failed to initialize module services")?;

tracing::info!(
    "Module '{}' started {} background services",
    module.name(),
    service_handles.len()
);

// Keep handles alive
spawn_services_monitor(service_handles);
```

### Phase 3: Remove Hardcoded Feature Flags

After Phase 1 & 2, these can be removed:
- Line 13-14: `use ipc_plugin_storage_node::{BlobPool, ReadRequestPool};`
- Line 17-20: `use ipc_plugin_storage_node::resolver::...`
- Line 27-28: `use ipc_plugin_storage_node::{IPCBlobFinality, ...}`
- Line 136-224: All hardcoded storage initialization

---

## Example: Adding Another Plugin

With generic architecture:

```rust
// In plugins/caching-node/src/lib.rs
impl ServiceModule for CachingNodeModule {
    async fn initialize_services(&self, ctx: &ServiceContext)
        -> Result<Vec<JoinHandle<()>>>
    {
        // Start cache invalidation service
        Ok(vec![tokio::spawn(async { /* cache work */ })])
    }
}
```

**No changes needed to node.rs!** ✅

---

## Trade-offs

### Current Approach (Hardcoded):
- ✅ Simple to understand
- ✅ Explicit initialization
- ❌ Not truly modular
- ❌ Each plugin requires node.rs changes
- ❌ Defeats purpose of module system

### Generic Approach:
- ✅ Truly modular
- ✅ Add plugins without touching node.rs
- ✅ Clean architecture
- ❌ Slightly more complex (indirection)
- ❌ Requires passing context properly

---

## Recommendation

**Implement the Generic Approach** because:

1. **Aligns with original intent** - You created `ServiceModule` trait for this!
2. **True plugin system** - Currently it's compile-time selection, not true plugins
3. **Future-proof** - Easy to add more plugins
4. **Clean boundaries** - Plugin owns its initialization

**Effort:** ~2-3 hours to:
1. Implement full `initialize_services()` in plugin
2. Update `node.rs` to call generic methods
3. Remove hardcoded storage references

---

## Current Status: Hybrid Approach

Right now we have:
- ✅ Generic traits defined
- ⚠️ Hardcoded initialization in practice
- ⚠️ Module system not fully utilized

**This is why you noticed the storage-node references!** The infrastructure is there, but not fully wired up. The question is: do you want to complete the generic wiring, or keep the pragmatic hardcoded approach?

Both are valid depending on your goals:
- **Hardcoded**: Simpler, faster to implement, good enough for 1-2 plugins
- **Generic**: More complex, better architecture, scales to many plugins
