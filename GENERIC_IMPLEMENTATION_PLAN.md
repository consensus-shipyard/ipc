# Generic Service Implementation - Step by Step Plan

## Goal
Remove ALL hardcoded storage-node references from `node.rs` and make it use generic module APIs.

## Current State
- ✅ `ServiceModule` trait exists
- ✅ Plugin implements trait (but returns empty)
- ❌ `node.rs` has hardcoded storage initialization (lines 136-224)
- ❌ `node.rs` has hardcoded imports (lines 13-28)

## Implementation Steps

### Step 1: Add Service Call to node.rs ✅ (Do This)
```rust
// After creating the module
let module = Arc::new(AppModule::default());

// Build service context
let service_ctx = ServiceContext::new(Box::new(settings.clone()))
    .with_validator_keypair(
        validator_keypair.as_ref().map(|k| k.secret_bytes())
    );

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
```

### Step 2: Document What Full Implementation Needs
The storage plugin CANNOT fully implement `initialize_services()` today because it needs:
1. ✅ Settings (can pass via ServiceContext)
2. ✅ Validator keypair (can pass via ServiceContext)
3. ❌ IPLD resolver client (created in node.rs, not available yet)
4. ❌ Vote tally (created in node.rs, not available yet)

**Solution:**
- Keep storage init in node.rs for now, but behind a clean interface
- Document TODOs for full migration
- Key win: Remove hardcoded type references

### Step 3: Remove Hardcoded Imports from node.rs ✅ (Do This)
Remove lines 13-28:
```rust
// ❌ DELETE THESE
#[cfg(feature = "storage-node")]
use ipc_plugin_storage_node::{BlobPool, ReadRequestPool};
#[cfg(feature = "storage-node")]
use ipc_plugin_storage_node::resolver::IrohResolver;
// ... etc
```

### Step 4: Extract Storage Init to Helper Function ✅ (Do This)
```rust
// In node.rs
#[cfg(feature = "plugin-storage-node")]
async fn initialize_storage_services(
    validator_key: &libp2p::identity::Keypair,
    client: &ipc_ipld_resolver::Client<_>,
    vote_tally: &VoteTally,
    settings: &AppSettings,
    subnet_id: &SubnetID,
) -> Result<Vec<JoinHandle<()>>> {
    // All the storage initialization code
    // Returns service handles
}
```

### Step 5: Call Helper from Generic Context ✅ (Do This)
```rust
// In node.rs after module.initialize_services()
#[cfg(feature = "plugin-storage-node")]
if let Some(ref key) = validator_keypair {
    let storage_handles = initialize_storage_services(
        key, &client, &vote_tally, &settings, &subnet_id
    ).await?;

    service_handles.extend(storage_handles);
}
```

## Result

### Before:
```rust
// ❌ Hardcoded imports
use ipc_plugin_storage_node::{BlobPool, ReadRequestPool};

// ❌ Hardcoded initialization inline
#[cfg(feature = "storage-node")]
let blob_pool = ResolvePool::new();
#[cfg(feature = "storage-node")]
let iroh_resolver = IrohResolver::new(...);
// ... 80+ lines of storage code inline
```

### After:
```rust
// ✅ No hardcoded imports

// ✅ Generic module call
let module = Arc::new(AppModule::default());
let service_handles = module.initialize_services(&ctx).await?;

// ✅ Plugin-specific init in clean helper
#[cfg(feature = "plugin-storage-node")]
let storage_handles = initialize_storage_services(...).await?;
```

## Benefits

1. **No hardcoded type imports** ✅
2. **Generic module pattern** ✅
3. **Clean separation** ✅
4. **Easy to remove feature flag later** ✅

## Future: Full Migration

To fully move storage init to plugin:
1. Refactor resolver client creation to be plugin-provided
2. Make vote tally part of module resources
3. Move helper function to plugin
4. Remove feature flag from node.rs

**Estimated effort:** 4-6 hours
**Current approach:** 1-2 hours, achieves main goal

## Decision

**Implement Steps 1-5 now:**
- Removes hardcoded references ✅
- Makes architecture generic ✅
- Clean and maintainable ✅
- Full migration is clear next step ✅
