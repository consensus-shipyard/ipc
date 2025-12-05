# Fendermint Plugin Architecture Design

**Goal:** Replace hard-coded `#[cfg(feature = "storage-node")]` conditionals with a dynamic, compile-time plugin system that allows storage-node and future extensions to integrate cleanly without modifying core code.

---

## Current Hard-Coded Integration Points

Based on code analysis, storage-node is currently integrated via **22 conditional compilation directives** across:

1. **Executor** (`storage-node/executor/`) - Custom `RecallExecutor` wrapper
2. **Message Handlers** (vm/interpreter) - ReadRequestPending, ReadRequestClosed
3. **Genesis** (vm/interpreter) - Storage actor initialization
4. **Service Layer** (app/service) - Iroh resolvers, BlobPool, ReadRequestPool
5. **CLI** (app/options) - Objects command
6. **Settings** (app/settings) - Objects configuration
7. **Module Exports** (fvm/mod.rs) - storage_env, storage_helpers

---

## Design Goals

1. **Zero-Cost Abstraction**: No runtime overhead compared to current implementation
2. **Compile-Time Only**: No dynamic library loading, fully static
3. **Type Safety**: Leverage Rust's type system to enforce correct plugin usage
4. **Minimal Boilerplate**: Easy to add new plugins
5. **Core Independence**: Core fendermint code has no knowledge of storage-node
6. **Feature Parity**: Same functionality as current hard-coded approach
7. **Composability**: Multiple plugins can coexist

---

## Proposed Architecture: Multi-Trait Hook System

### Overview

Use a **trait-based hook system** with **compile-time plugin registration** via:
- Trait definitions for extension points
- Generic parameters with trait bounds
- Static dispatch (zero runtime cost)
- Feature-gated plugin implementations

### Key Components

```
┌─────────────────────────────────────────────────────────┐
│                    Fendermint Core                      │
│  (No knowledge of plugins)                              │
│                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ Executor     │  │ Interpreter  │  │ Service      │ │
│  │ (Generic)    │  │ (Hooks)      │  │ (Hooks)      │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│         ▲                  ▲                  ▲         │
└─────────┼──────────────────┼──────────────────┼─────────┘
          │                  │                  │
     Plugin Traits       Plugin Traits     Plugin Traits
          │                  │                  │
┌─────────┼──────────────────┼──────────────────┼─────────┐
│         │                  │                  │         │
│  ┌──────┴──────┐    ┌──────┴──────┐    ┌─────┴──────┐ │
│  │ Executor    │    │ Message     │    │ Service    │ │
│  │ Plugin API  │    │ Handler API │    │ Plugin API │ │
│  └─────────────┘    └─────────────┘    └────────────┘ │
│                                                         │
│              Plugin Interface Layer                     │
└─────────────────────────────────────────────────────────┘
          │                  │                  │
┌─────────┼──────────────────┼──────────────────┼─────────┐
│         ▼                  ▼                  ▼         │
│  ┌─────────────────────────────────────────────────┐   │
│  │                                                   │   │
│  │         Storage Node Plugin                      │   │
│  │  (Implements all plugin traits)                  │   │
│  │                                                   │   │
│  │  - ExecutorPlugin                                │   │
│  │  - MessageHandlerPlugin                          │   │
│  │  - GenesisPlugin                                 │   │
│  │  - ServicePlugin                                 │   │
│  │  - CliPlugin                                     │   │
│  │                                                   │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│           storage-node/ (separate crate)               │
└─────────────────────────────────────────────────────────┘
```

---

## Detailed Design

### 1. Plugin Trait Definitions

Location: `fendermint/plugin/` (new crate)

```rust
// fendermint/plugin/src/executor.rs

/// Plugin that can wrap or replace the FVM executor
pub trait ExecutorPlugin<K: Kernel> {
    type Executor: Executor<Kernel = K>;

    /// Create an executor instance
    fn create_executor(
        engine_pool: EnginePool,
        machine: <K::CallManager as CallManager>::Machine,
    ) -> Result<Self::Executor>;
}

/// Default no-op plugin uses standard FVM executor
pub struct NoOpExecutorPlugin;

impl<K: Kernel> ExecutorPlugin<K> for NoOpExecutorPlugin {
    type Executor = DefaultExecutor<K>;

    fn create_executor(
        engine_pool: EnginePool,
        machine: <K::CallManager as CallManager>::Machine,
    ) -> Result<Self::Executor> {
        DefaultExecutor::new(engine_pool, machine)
    }
}
```

```rust
// fendermint/plugin/src/message.rs

/// Plugin that can handle custom message types
pub trait MessageHandlerPlugin {
    /// Handle a custom IPC message
    /// Return None if plugin doesn't handle this message type
    fn handle_message<DB: Blockstore>(
        &self,
        state: &mut FvmExecState<DB>,
        msg: &IpcMessage,
    ) -> Result<Option<ApplyMessageResponse>>;

    /// List message types this plugin handles
    fn message_types(&self) -> &[&str];
}

/// Default no-op plugin handles no messages
pub struct NoOpMessageHandlerPlugin;

impl MessageHandlerPlugin for NoOpMessageHandlerPlugin {
    fn handle_message<DB: Blockstore>(
        &self,
        _state: &mut FvmExecState<DB>,
        _msg: &IpcMessage,
    ) -> Result<Option<ApplyMessageResponse>> {
        Ok(None) // Don't handle any messages
    }

    fn message_types(&self) -> &[&str] {
        &[]
    }
}
```

```rust
// fendermint/plugin/src/genesis.rs

/// Plugin that can add custom actors during genesis
pub trait GenesisPlugin {
    /// Initialize plugin-specific actors
    fn initialize_actors<BS: Blockstore>(
        &self,
        state: &mut FvmGenesisState<BS>,
        genesis: &Genesis,
    ) -> Result<()>;

    /// Plugin name for logging
    fn name(&self) -> &str;
}

pub struct NoOpGenesisPlugin;

impl GenesisPlugin for NoOpGenesisPlugin {
    fn initialize_actors<BS: Blockstore>(
        &self,
        _state: &mut FvmGenesisState<BS>,
        _genesis: &Genesis,
    ) -> Result<()> {
        Ok(())
    }

    fn name(&self) -> &str {
        "noop"
    }
}
```

```rust
// fendermint/plugin/src/service.rs

/// Plugin that can add custom services
pub trait ServicePlugin {
    /// Initialize plugin services
    fn initialize_services<DB: Blockstore>(
        &self,
        ctx: &ServiceContext<DB>,
    ) -> Result<Vec<JoinHandle<()>>>;

    /// Provide any resources needed by other components
    fn resources(&self) -> PluginResources;
}

pub struct PluginResources {
    // Could contain shared state, channels, etc.
    pub data: HashMap<String, Box<dyn Any + Send + Sync>>,
}

pub struct NoOpServicePlugin;

impl ServicePlugin for NoOpServicePlugin {
    fn initialize_services<DB: Blockstore>(
        &self,
        _ctx: &ServiceContext<DB>,
    ) -> Result<Vec<JoinHandle<()>>> {
        Ok(vec![])
    }

    fn resources(&self) -> PluginResources {
        PluginResources { data: HashMap::new() }
    }
}
```

```rust
// fendermint/plugin/src/cli.rs

/// Plugin that can add CLI commands
pub trait CliPlugin {
    /// Get CLI command definitions
    fn commands(&self) -> Vec<CommandDescriptor>;

    /// Execute a command
    async fn execute_command(&self, cmd: &str, args: &[String]) -> Result<()>;
}

pub struct CommandDescriptor {
    pub name: String,
    pub about: String,
    pub args: Vec<ArgDescriptor>,
}

pub struct NoOpCliPlugin;

impl CliPlugin for NoOpCliPlugin {
    fn commands(&self) -> Vec<CommandDescriptor> {
        vec![]
    }

    async fn execute_command(&self, _cmd: &str, _args: &[String]) -> Result<()> {
        bail!("No CLI commands available")
    }
}
```

---

### 2. Plugin Composition

Location: `fendermint/plugin/src/bundle.rs`

```rust
/// Bundle of all plugin traits
pub trait PluginBundle:
    ExecutorPlugin<Self::Kernel> +
    MessageHandlerPlugin +
    GenesisPlugin +
    ServicePlugin +
    CliPlugin
{
    type Kernel: Kernel;

    fn name(&self) -> &str;
}

/// No-op plugin bundle (default)
pub struct NoOpPluginBundle;

impl ExecutorPlugin<DefaultCallManager<...>> for NoOpPluginBundle {
    // Use NoOpExecutorPlugin implementation
}

impl MessageHandlerPlugin for NoOpPluginBundle {
    // Use NoOpMessageHandlerPlugin implementation
}

// ... implement all traits with no-op versions

impl PluginBundle for NoOpPluginBundle {
    type Kernel = DefaultKernel<DefaultCallManager<...>>;

    fn name(&self) -> &str {
        "noop"
    }
}
```

---

### 3. Storage Node Plugin Implementation

Location: `storage-node/plugin/` (new crate)

```rust
// storage-node/plugin/src/lib.rs

pub struct StorageNodePlugin {
    // Plugin state
}

impl<K: Kernel> ExecutorPlugin<K> for StorageNodePlugin {
    type Executor = RecallExecutor<K>;

    fn create_executor(
        engine_pool: EnginePool,
        machine: <K::CallManager as CallManager>::Machine,
    ) -> Result<Self::Executor> {
        RecallExecutor::new(engine_pool, machine)
    }
}

impl MessageHandlerPlugin for StorageNodePlugin {
    fn handle_message<DB: Blockstore>(
        &self,
        state: &mut FvmExecState<DB>,
        msg: &IpcMessage,
    ) -> Result<Option<ApplyMessageResponse>> {
        match msg {
            IpcMessage::ReadRequestPending(req) => {
                let ret = set_read_request_pending(state, req.id)?;
                Ok(Some(ApplyMessageResponse {
                    applied_message: ret.into(),
                    domain_hash: None,
                }))
            }
            IpcMessage::ReadRequestClosed(req) => {
                read_request_callback(state, req)?;
                let ret = close_read_request(state, req.id)?;
                Ok(Some(ApplyMessageResponse {
                    applied_message: ret.into(),
                    domain_hash: None,
                }))
            }
            _ => Ok(None), // Don't handle other messages
        }
    }

    fn message_types(&self) -> &[&str] {
        &["ReadRequestPending", "ReadRequestClosed"]
    }
}

impl GenesisPlugin for StorageNodePlugin {
    fn initialize_actors<BS: Blockstore>(
        &self,
        state: &mut FvmGenesisState<BS>,
        genesis: &Genesis,
    ) -> Result<()> {
        // Initialize storage config actor
        let storage_config_state = fendermint_actor_storage_config::State {
            admin: None,
            config: fendermint_actor_storage_config_shared::StorageConfig::default(),
        };
        state.create_custom_actor(
            fendermint_actor_storage_config::ACTOR_NAME,
            storage_config::STORAGE_CONFIG_ACTOR_ID,
            &storage_config_state,
            TokenAmount::zero(),
            None,
        )?;

        // Initialize blobs actor
        // ... etc

        Ok(())
    }

    fn name(&self) -> &str {
        "storage-node"
    }
}

impl ServicePlugin for StorageNodePlugin {
    fn initialize_services<DB: Blockstore>(
        &self,
        ctx: &ServiceContext<DB>,
    ) -> Result<Vec<JoinHandle<()>>> {
        let mut handles = vec![];

        // Create blob and read request pools
        let blob_pool: BlobPool = ResolvePool::new();
        let read_request_pool: ReadRequestPool = ResolvePool::new();

        // Spawn Iroh resolvers
        if let Some(ref key) = ctx.validator_keypair {
            let iroh_resolver = IrohResolver::new(/* ... */);
            handles.push(tokio::spawn(async move {
                iroh_resolver.run().await
            }));

            // Read request resolver
            // ...
        }

        Ok(handles)
    }

    fn resources(&self) -> PluginResources {
        // Provide blob_pool, read_request_pool, etc.
        PluginResources { /* ... */ }
    }
}

impl CliPlugin for StorageNodePlugin {
    fn commands(&self) -> Vec<CommandDescriptor> {
        vec![CommandDescriptor {
            name: "objects".to_string(),
            about: "Subcommands related to the Objects/Blobs storage HTTP API".to_string(),
            args: vec![/* ... */],
        }]
    }

    async fn execute_command(&self, cmd: &str, args: &[String]) -> Result<()> {
        match cmd {
            "objects" => {
                // Handle objects command
                Ok(())
            }
            _ => bail!("Unknown command: {}", cmd),
        }
    }
}

impl PluginBundle for StorageNodePlugin {
    type Kernel = RecallKernel<DefaultCallManager<...>>;

    fn name(&self) -> &str {
        "storage-node"
    }
}
```

---

### 4. Core Integration (Generic over Plugin)

Location: `fendermint/vm/interpreter/src/fvm/interpreter.rs`

```rust
// BEFORE (hard-coded):
#[cfg(feature = "storage-node")]
IpcMessage::ReadRequestPending(req) => { /* ... */ }

// AFTER (plugin-based):
pub struct FvmMessagesInterpreter<P: PluginBundle> {
    plugin: P,
    // ... other fields
}

impl<P: PluginBundle> FvmMessagesInterpreter<P> {
    async fn apply_message(&self, msg: ChainMessage) -> Result<ApplyMessageResponse> {
        match msg {
            ChainMessage::Ipc(ipc_msg) => {
                // Try plugin first
                if let Some(response) = self.plugin.handle_message(state, &ipc_msg)? {
                    return Ok(response);
                }

                // Handle core messages
                match ipc_msg {
                    // ... core message handlers
                }
            }
        }
    }
}
```

---

### 5. Feature-Gated Plugin Selection

Location: `fendermint/app/Cargo.toml` and `fendermint/app/src/lib.rs`

```toml
[features]
default = ["storage-node"]
storage-node = ["storage-node-plugin"]

[dependencies]
fendermint-plugin = { path = "../plugin" }

# Only included when feature is enabled
storage-node-plugin = { path = "../../storage-node/plugin", optional = true }
```

```rust
// fendermint/app/src/lib.rs

#[cfg(feature = "storage-node")]
type AppPlugin = storage_node_plugin::StorageNodePlugin;

#[cfg(not(feature = "storage-node"))]
type AppPlugin = fendermint_plugin::NoOpPluginBundle;

// Use AppPlugin throughout the application
pub fn create_interpreter() -> FvmMessagesInterpreter<AppPlugin> {
    FvmMessagesInterpreter::new(AppPlugin::default())
}
```

---

## Alternative Approaches Considered

### Option B: Inventory-Based Runtime Registration

**Pros:**
- More flexible, plugins can self-register
- No need to modify core type parameters

**Cons:**
- Runtime overhead (trait object dispatch)
- More complex lifetime management
- Harder to ensure type safety

### Option C: Macro-Based Code Generation

**Pros:**
- Maximum flexibility in generated code
- Can generate optimal code paths

**Cons:**
- Complex macro implementation
- Harder to debug
- IDE support challenges

### Option D: Dependency Injection Container

**Pros:**
- Familiar pattern from other languages
- Flexible service wiring

**Cons:**
- Runtime overhead
- Not idiomatic Rust
- Loses compile-time guarantees

---

## Implementation Plan

### Phase 1: Foundation (3-5 days)
1. Create `fendermint/plugin/` crate
2. Define all plugin trait interfaces
3. Implement no-op plugin bundle
4. Add comprehensive documentation and examples

### Phase 2: Executor Plugin (3-4 days)
1. Make executor generic over `ExecutorPlugin`
2. Extract `RecallExecutor` to storage-node plugin
3. Test with both plugins
4. Verify zero performance regression

### Phase 3: Message Handler Plugin (3-4 days)
1. Add message handler hooks to interpreter
2. Move storage message handling to plugin
3. Remove `#[cfg]` from interpreter
4. Test message routing

### Phase 4: Genesis Plugin (2-3 days)
1. Add genesis hooks
2. Move storage actor initialization to plugin
3. Remove `#[cfg]` from genesis code
4. Test genesis with both plugins

### Phase 5: Service Plugin (3-4 days)
1. Add service initialization hooks
2. Move Iroh resolvers to plugin
3. Remove `#[cfg]` from service code
4. Test service lifecycle

### Phase 6: CLI Plugin (2-3 days)
1. Add CLI extension mechanism
2. Move Objects command to plugin
3. Dynamic command registration
4. Test CLI with both plugins

### Phase 7: Integration & Testing (3-5 days)
1. Full integration testing
2. Performance benchmarking
3. Documentation updates
4. Migration guide

**Total Estimate: 19-28 days**

---

## Questions for Clarification

1. **Performance Requirements:**
   - Is zero runtime overhead mandatory? (implies static dispatch via generics)
   - Or is minimal runtime overhead acceptable? (allows trait objects, more flexible)

2. **Plugin Scope:**
   - Should plugins only extend existing functionality, or add entirely new features?
   - Do we need plugin-to-plugin communication/dependencies?

3. **Executor Flexibility:**
   - The `RecallExecutor` wraps the entire FVM executor. Should we use:
     - **Option A:** Plugin provides entire executor (current approach)
     - **Option B:** Plugin provides hooks into execution lifecycle (more granular)
     - **Option C:** Executor has pre/post hooks, plugin implements those

4. **Message Types:**
   - Should plugins be able to define entirely new message types?
   - Or only handle existing IpcMessage variants?

5. **Type Parameters:**
   - Are you comfortable with core types being generic over plugins? E.g.:
     ```rust
     FvmMessagesInterpreter<P: PluginBundle>
     ```
   - This propagates through the codebase but is zero-cost

6. **Plugin Discovery:**
   - Compile-time only (via feature flags)?
   - Or should we support some form of plugin discovery?

7. **Backward Compatibility:**
   - Do we need to maintain the current `#[cfg]` approach as well?
   - Or can we do a clean migration?

8. **Testing Strategy:**
   - Should plugins have their own test suites?
   - How do we test plugin interactions?

---

## Recommendation

I recommend **Option A: Multi-Trait Hook System** because it:
- ✅ Zero runtime overhead (static dispatch)
- ✅ Type-safe at compile time
- ✅ Idiomatic Rust (traits + generics)
- ✅ Clean separation of concerns
- ✅ Easy to test (mock plugins)
- ✅ Extensible to future plugins

The main trade-off is that types become generic over plugin bundles, but this is a compile-time concern only and provides maximum safety and performance.

---

## Next Steps

Please review and provide feedback on:
1. Overall architecture approach
2. Answers to clarification questions
3. Any concerns about the design
4. Priority of features/phases

Once approved, I can begin implementation starting with Phase 1 (Foundation).
