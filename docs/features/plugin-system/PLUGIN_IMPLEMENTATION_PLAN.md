# Module System Implementation Plan

**Status:** Phase 1 In Progress
**Approved Architecture:** Multi-Trait Hook System with zero-cost generics
**Terminology:** Using "module" instead of "plugin"
**Branch:** modular-plugable-architecture

---

## Design Decisions (Finalized)

1. ✅ **Performance**: Zero-cost via static dispatch (generics)
2. ✅ **Executor Design**: Full executor replacement (Option A)
   - RecallExecutor has complex 3-way gas accounting
   - Cannot be achieved with pre/post hooks
   - Plugin provides entire `Executor` implementation
3. ✅ **Message Types**: Plugins can define new message types
4. ✅ **Type Propagation**: Core types generic over `PluginBundle`
5. ✅ **Migration**: Clean cut - remove all 22 `#[cfg]` directives

---

## Phase 1: Foundation (Days 1-5)

### Goal: Create plugin framework crate with all trait definitions

**Tasks:**

1. **Create `fendermint/plugin/` crate**
   ```toml
   [package]
   name = "fendermint_plugin"
   description = "Plugin system for extending Fendermint functionality"

   [dependencies]
   anyhow = { workspace = true }
   async-trait = { workspace = true }
   # ... minimal deps
   ```

2. **Define `ExecutorPlugin` trait**
   ```rust
   // fendermint/plugin/src/executor.rs
   pub trait ExecutorPlugin<K: Kernel> {
       type Executor: Executor<Kernel = K>;

       fn create_executor(
           engine_pool: EnginePool,
           machine: <K::CallManager as CallManager>::Machine,
       ) -> Result<Self::Executor>;
   }

   // Default implementation using FVM's DefaultExecutor
   pub struct NoOpExecutorPlugin;
   ```

3. **Define `MessageHandlerPlugin` trait**
   ```rust
   // fendermint/plugin/src/message.rs
   pub trait MessageHandlerPlugin: Send + Sync {
       fn handle_message<DB: Blockstore>(
           &self,
           state: &mut FvmExecState<DB>,
           msg: &IpcMessage,
       ) -> Result<Option<ApplyMessageResponse>>;

       fn message_types(&self) -> &[&str];
   }
   ```

4. **Define `GenesisPlugin` trait**
   ```rust
   // fendermint/plugin/src/genesis.rs
   pub trait GenesisPlugin: Send + Sync {
       fn initialize_actors<BS: Blockstore>(
           &self,
           state: &mut FvmGenesisState<BS>,
           genesis: &Genesis,
       ) -> Result<()>;

       fn name(&self) -> &str;
   }
   ```

5. **Define `ServicePlugin` trait**
   ```rust
   // fendermint/plugin/src/service.rs
   pub trait ServicePlugin: Send + Sync {
       fn initialize_services(
           &self,
           ctx: &mut ServiceContext,
       ) -> Result<Vec<JoinHandle<()>>>;

       fn resources(&self) -> Box<dyn Any + Send + Sync>;
   }

   pub struct ServiceContext {
       pub settings: Settings,
       pub validator_keypair: Option<SecretKey>,
       pub db: RocksDb,
       pub state_store: NamespaceBlockstore,
       // ... other resources
   }
   ```

6. **Define `CliPlugin` trait**
   ```rust
   // fendermint/plugin/src/cli.rs
   pub trait CliPlugin: Send + Sync {
       fn commands(&self) -> Vec<Command>;

       async fn execute(&self, cmd: &str, matches: &ArgMatches) -> Result<()>;
   }

   pub struct Command {
       pub name: String,
       pub about: String,
       pub subcommands: Vec<Command>,
   }
   ```

7. **Define `PluginBundle` composition trait**
   ```rust
   // fendermint/plugin/src/bundle.rs
   pub trait PluginBundle:
       ExecutorPlugin<Self::Kernel> +
       MessageHandlerPlugin +
       GenesisPlugin +
       ServicePlugin +
       CliPlugin +
       Send + Sync + 'static
   {
       type Kernel: Kernel;

       fn name(&self) -> &'static str;
   }
   ```

8. **Implement `NoOpPluginBundle`**
   ```rust
   pub struct NoOpPluginBundle;

   impl<K: Kernel> ExecutorPlugin<K> for NoOpPluginBundle {
       type Executor = DefaultExecutor<K>;
       fn create_executor(...) -> Result<Self::Executor> {
           DefaultExecutor::new(engine_pool, machine)
       }
   }

   // ... implement all traits with no-op versions

   impl PluginBundle for NoOpPluginBundle {
       type Kernel = DefaultKernel<DefaultCallManager<...>>;
       fn name(&self) -> &'static str { "noop" }
   }
   ```

9. **Write comprehensive tests**
   ```rust
   #[cfg(test)]
   mod tests {
       // Test trait implementations
       // Test no-op plugin
       // Test plugin composition
   }
   ```

10. **Documentation**
    - API documentation for all traits
    - Plugin development guide
    - Example plugin template

**Deliverables:**
- ✅ `fendermint/plugin/` crate compiles
- ✅ All trait definitions complete
- ✅ No-op plugin bundle functional
- ✅ Comprehensive tests pass
- ✅ Documentation complete

---

## Phase 2: Core Integration - Make Generic (Days 6-10)

### Goal: Make core fendermint generic over `PluginBundle`

**Tasks:**

1. **Update `FvmExecState` to be generic**
   ```rust
   // fendermint/vm/interpreter/src/fvm/state/exec.rs

   // BEFORE:
   pub struct FvmExecState<DB> {
       executor: RecallExecutor<...>,
   }

   // AFTER:
   pub struct FvmExecState<DB, P: PluginBundle> {
       executor: P::Executor,
       plugin: Arc<P>,
   }
   ```

2. **Update `FvmMessagesInterpreter` to be generic**
   ```rust
   // fendermint/vm/interpreter/src/fvm/interpreter.rs

   pub struct FvmMessagesInterpreter<P: PluginBundle> {
       plugin: Arc<P>,
       // ... other fields
   }

   impl<P: PluginBundle> FvmMessagesInterpreter<P> {
       pub fn new(plugin: P) -> Self {
           Self {
               plugin: Arc::new(plugin),
               // ...
           }
       }
   }
   ```

3. **Update message handling to use plugin**
   ```rust
   // In apply_message:
   match msg {
       ChainMessage::Ipc(ipc_msg) => {
           // Try plugin handler first
           if let Some(response) = self.plugin.handle_message(state, &ipc_msg)? {
               return Ok(response);
           }

           // REMOVE all #[cfg(feature = "storage-node")] conditionals
           // Fall back to core message handling
           match ipc_msg {
               // ... core handlers only
           }
       }
   }
   ```

4. **Update genesis to use plugin**
   ```rust
   // fendermint/vm/interpreter/src/genesis.rs

   impl<'a, P: PluginBundle> GenesisBuilder<'a, P> {
       pub fn build(&mut self) -> Result<()> {
           // Initialize core actors
           self.initialize_core_actors()?;

           // Let plugin initialize its actors
           self.plugin.initialize_actors(&mut self.state, &self.genesis)?;

           Ok(())
       }
   }

   // REMOVE all #[cfg(feature = "storage-node")] from genesis
   ```

5. **Update app to be generic**
   ```rust
   // fendermint/app/src/lib.rs

   pub struct App<P: PluginBundle> {
       plugin: Arc<P>,
       // ... other fields
   }
   ```

6. **Add type aliases for convenience**
   ```rust
   // fendermint/app/src/lib.rs

   #[cfg(feature = "storage-node")]
   pub type DefaultPlugin = storage_node_plugin::StorageNodePlugin;

   #[cfg(not(feature = "storage-node"))]
   pub type DefaultPlugin = fendermint_plugin::NoOpPluginBundle;

   pub type DefaultApp = App<DefaultPlugin>;
   pub type DefaultInterpreter = FvmMessagesInterpreter<DefaultPlugin>;
   ```

7. **Update service initialization**
   ```rust
   // fendermint/app/src/service/node.rs

   pub async fn create_node<P: PluginBundle>(
       settings: Settings,
       plugin: P,
   ) -> Result<Node<P>> {
       // ... setup ...

       // REMOVE all #[cfg(feature = "storage-node")]

       // Let plugin initialize services
       let plugin_handles = plugin.initialize_services(&mut ctx)?;

       // ...
   }
   ```

8. **Update CLI to use plugin**
   ```rust
   // fendermint/app/options/src/lib.rs

   pub enum Commands<P: PluginBundle> {
       Config(ConfigArgs),
       Run(RunArgs),
       // ... core commands ...

       // Dynamic plugin commands
       Plugin(PluginCommand<P>),
   }

   // REMOVE #[cfg(feature = "storage-node")] Objects variant
   ```

9. **Update all type signatures**
   - Propagate `P: PluginBundle` through call stack
   - Update function signatures
   - Update struct definitions
   - Update trait implementations

10. **Remove ALL `#[cfg(feature = "storage-node")]` from core**
    - Search for all 22 occurrences
    - Replace with plugin calls
    - Verify no conditionals remain in core

**Deliverables:**
- ✅ Core is fully generic over `PluginBundle`
- ✅ All `#[cfg]` removed from core code
- ✅ Compiles with `NoOpPluginBundle`
- ✅ Type inference works correctly
- ✅ Tests pass with no-op plugin

---

## Phase 3: Storage Node Plugin (Days 11-18)

### Goal: Implement storage-node as a plugin

**Tasks:**

1. **Create `storage-node/plugin/` crate**
   ```toml
   [package]
   name = "storage_node_plugin"

   [dependencies]
   fendermint_plugin = { path = "../../fendermint/plugin" }
   storage_node_executor = { path = "../executor" }
   storage_node_kernel = { path = "../kernel" }
   # ... all storage-node deps
   ```

2. **Implement `ExecutorPlugin`**
   ```rust
   // storage-node/plugin/src/executor.rs

   impl<K: Kernel> ExecutorPlugin<K> for StorageNodePlugin {
       type Executor = RecallExecutor<K>;

       fn create_executor(
           engine_pool: EnginePool,
           machine: <K::CallManager as CallManager>::Machine,
       ) -> Result<Self::Executor> {
           RecallExecutor::new(engine_pool, machine)
       }
   }
   ```

3. **Implement `MessageHandlerPlugin`**
   ```rust
   // storage-node/plugin/src/message.rs

   impl MessageHandlerPlugin for StorageNodePlugin {
       fn handle_message<DB: Blockstore>(
           &self,
           state: &mut FvmExecState<DB>,
           msg: &IpcMessage,
       ) -> Result<Option<ApplyMessageResponse>> {
           match msg {
               IpcMessage::ReadRequestPending(req) => {
                   // Move logic from interpreter here
                   let ret = set_read_request_pending(state, req.id)?;
                   Ok(Some(ApplyMessageResponse { ... }))
               }
               IpcMessage::ReadRequestClosed(req) => {
                   // Move logic from interpreter here
                   read_request_callback(state, req)?;
                   let ret = close_read_request(state, req.id)?;
                   Ok(Some(ApplyMessageResponse { ... }))
               }
               _ => Ok(None),
           }
       }

       fn message_types(&self) -> &[&str] {
           &["ReadRequestPending", "ReadRequestClosed"]
       }
   }
   ```

4. **Implement `GenesisPlugin`**
   ```rust
   // storage-node/plugin/src/genesis.rs

   impl GenesisPlugin for StorageNodePlugin {
       fn initialize_actors<BS: Blockstore>(
           &self,
           state: &mut FvmGenesisState<BS>,
           genesis: &Genesis,
       ) -> Result<()> {
           // Move storage actor initialization from genesis.rs here
           self.init_storage_config_actor(state)?;
           self.init_blobs_actor(state)?;
           self.init_blob_reader_actor(state)?;
           self.init_adm_actor(state)?;
           Ok(())
       }

       fn name(&self) -> &str {
           "storage-node"
       }
   }
   ```

5. **Implement `ServicePlugin`**
   ```rust
   // storage-node/plugin/src/service.rs

   impl ServicePlugin for StorageNodePlugin {
       fn initialize_services(
           &self,
           ctx: &mut ServiceContext,
       ) -> Result<Vec<JoinHandle<()>>> {
           let mut handles = vec![];

           // Move Iroh resolver initialization here
           let blob_pool = ResolvePool::new();
           let read_request_pool = ResolvePool::new();

           if let Some(ref key) = ctx.validator_keypair {
               // Blob resolver
               let resolver = IrohResolver::new(...);
               handles.push(tokio::spawn(async move {
                   resolver.run().await
               }));

               // Read request resolver
               // ...
           }

           Ok(handles)
       }

       fn resources(&self) -> Box<dyn Any + Send + Sync> {
           Box::new(StorageNodeResources {
               blob_pool,
               read_request_pool,
           })
       }
   }
   ```

6. **Implement `CliPlugin`**
   ```rust
   // storage-node/plugin/src/cli.rs

   impl CliPlugin for StorageNodePlugin {
       fn commands(&self) -> Vec<Command> {
           vec![Command {
               name: "objects".to_string(),
               about: "Manage storage objects/blobs".to_string(),
               subcommands: vec![
                   // run, get, put, etc.
               ],
           }]
       }

       async fn execute(&self, cmd: &str, matches: &ArgMatches) -> Result<()> {
           match cmd {
               "objects" => self.handle_objects_command(matches).await,
               _ => bail!("Unknown command: {}", cmd),
           }
       }
   }
   ```

7. **Implement `PluginBundle`**
   ```rust
   // storage-node/plugin/src/lib.rs

   pub struct StorageNodePlugin {
       // Plugin state
   }

   impl PluginBundle for StorageNodePlugin {
       type Kernel = RecallKernel<DefaultCallManager<...>>;

       fn name(&self) -> &'static str {
           "storage-node"
       }
   }

   impl Default for StorageNodePlugin {
       fn default() -> Self {
           Self { /* ... */ }
       }
   }
   ```

8. **Move storage-specific code to plugin**
   - Move `storage_env` module
   - Move `storage_helpers` module
   - Move Iroh resolver code
   - Update imports

9. **Update dependencies**
   ```toml
   # fendermint/app/Cargo.toml

   [dependencies]
   fendermint_plugin = { path = "../plugin" }

   [dependencies.storage-node-plugin]
   path = "../../storage-node/plugin"
   optional = true

   [features]
   default = []
   storage-node = ["storage-node-plugin"]
   ```

10. **Plugin selection in main**
    ```rust
    // fendermint/app/src/main.rs

    #[cfg(feature = "storage-node")]
    type AppPlugin = storage_node_plugin::StorageNodePlugin;

    #[cfg(not(feature = "storage-node"))]
    type AppPlugin = fendermint_plugin::NoOpPluginBundle;

    fn main() {
        let plugin = AppPlugin::default();
        let app = App::new(plugin);
        // ...
    }
    ```

**Deliverables:**
- ✅ `storage-node/plugin/` crate complete
- ✅ All storage-node functionality moved to plugin
- ✅ Plugin implements all traits correctly
- ✅ Compiles with feature flag
- ✅ Tests pass with storage-node plugin

---

## Phase 4: Integration Testing (Days 19-23)

### Goal: Verify both configurations work correctly

**Tasks:**

1. **Test with NoOpPlugin**
   ```bash
   cargo build --no-default-features
   cargo test --no-default-features
   ./target/debug/fendermint --help  # No objects command
   ```

2. **Test with StorageNodePlugin**
   ```bash
   cargo build --features storage-node
   cargo test --features storage-node
   ./target/debug/fendermint objects --help  # Has objects command
   ```

3. **Genesis tests**
   - Verify storage actors initialized with plugin
   - Verify no storage actors without plugin
   - Test both configurations

4. **Message handling tests**
   - Test ReadRequest messages with plugin
   - Test messages are rejected without plugin
   - Test message routing

5. **Service tests**
   - Verify Iroh resolvers start with plugin
   - Verify no resolvers without plugin
   - Test service lifecycle

6. **CLI tests**
   - Verify Objects command with plugin
   - Verify no Objects command without plugin
   - Test command execution

7. **Executor tests**
   - Test RecallExecutor with plugin
   - Test DefaultExecutor without plugin
   - Test sponsor gas logic

8. **Integration tests**
   - Full node startup with both configs
   - Message processing end-to-end
   - Genesis to execution flow

9. **Performance testing**
   - Benchmark with/without plugin
   - Verify zero overhead (static dispatch)
   - Memory usage comparison

10. **Documentation updates**
    - Update architecture docs
    - Update deployment docs
    - Plugin development guide

**Deliverables:**
- ✅ All tests pass in both configurations
- ✅ No performance regression
- ✅ Documentation updated
- ✅ Both binaries work correctly

---

## Phase 5: Polish & Migration (Days 24-28)

### Goal: Clean up and prepare for production

**Tasks:**

1. **Code cleanup**
   - Remove dead code
   - Clean up imports
   - Fix clippy warnings
   - Format all code

2. **Documentation**
   - API documentation
   - Plugin development guide
   - Migration guide for other plugins
   - Architecture decision records

3. **Examples**
   - Minimal plugin example
   - Custom executor plugin
   - Custom message handler plugin

4. **CI/CD updates**
   - Test both configurations
   - Build both binaries
   - Run integration tests

5. **Performance validation**
   - Benchmark against old implementation
   - Verify no regression
   - Document results

6. **Security review**
   - Review plugin API surface
   - Check for unsafe code
   - Validate error handling

7. **Migration testing**
   - Test upgrade path
   - Verify state compatibility
   - Test rollback procedures

8. **Release preparation**
   - Update CHANGELOG
   - Version bumps
   - Release notes

**Deliverables:**
- ✅ Production-ready code
- ✅ Complete documentation
- ✅ CI/CD configured
- ✅ Ready for merge

---

## Success Criteria

- ✅ Zero `#[cfg(feature = "storage-node")]` in core code
- ✅ Both configurations build and run
- ✅ All tests pass in both modes
- ✅ No performance regression
- ✅ Clean, maintainable architecture
- ✅ Comprehensive documentation
- ✅ Easy to add new plugins

---

## Timeline

- **Phase 1:** Days 1-5 (Foundation)
- **Phase 2:** Days 6-10 (Core Integration)
- **Phase 3:** Days 11-18 (Storage Node Plugin)
- **Phase 4:** Days 19-23 (Testing)
- **Phase 5:** Days 24-28 (Polish)

**Total: 28 days (5.6 weeks)**

---

## Risk Mitigation

1. **Type complexity**: Use type aliases liberally
2. **Compilation time**: Keep plugin trait bounds minimal
3. **Breaking changes**: Version carefully, document migration
4. **Testing**: Comprehensive test coverage in both modes
5. **Performance**: Continuous benchmarking

---

## Next Steps

1. Get final approval on this plan
2. Create feature branch `plugin-architecture`
3. Begin Phase 1 implementation
4. Daily progress updates
5. Review after each phase

---

**Ready to start implementation!** 🚀
