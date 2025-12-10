// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Module bundle trait for composing all module capabilities.
//!
//! This module defines the `ModuleBundle` trait which combines all the
//! individual module traits into a single interface. A module that implements
//! `ModuleBundle` can provide custom executors, message handlers, genesis
//! initialization, services, and CLI commands.

use crate::cli::CliModule;
use crate::executor::ExecutorModule;
use crate::genesis::GenesisModule;
use crate::message::MessageHandlerModule;
use crate::service::ServiceModule;
use fvm::call_manager::{CallManager, DefaultCallManager};
use fvm::kernel::Kernel;
use fvm::machine::DefaultMachine;

/// The main module bundle trait.
///
/// This trait combines all the individual module traits (ExecutorModule,
/// MessageHandlerModule, GenesisModule, ServiceModule, CliModule) into a
/// single coherent interface.
///
/// A type that implements `ModuleBundle` must implement all five module traits,
/// providing a complete extension package for Fendermint.
///
/// # Type Parameters
///
/// * `Kernel` - The FVM kernel type used by this module's executor
///
/// # Example
///
/// ```ignore
/// struct MyModule {
///     // ... module state ...
/// }
///
/// // Implement all individual traits
/// impl<K: Kernel> ExecutorModule<K> for MyModule { ... }
/// impl MessageHandlerModule for MyModule { ... }
/// impl GenesisModule for MyModule { ... }
/// impl ServiceModule for MyModule { ... }
/// impl CliModule for MyModule { ... }
///
/// // Then implement the bundle
/// impl ModuleBundle for MyModule {
///     type Kernel = MyCustomKernel;
///
///     fn name(&self) -> &'static str {
///         "my-module"
///     }
/// }
/// ```
pub trait ModuleBundle:
    ExecutorModule<Self::Kernel>
    + MessageHandlerModule
    + GenesisModule
    + ServiceModule
    + CliModule
    + Send
    + Sync
    + 'static
where
    <<Self::Kernel as fvm::kernel::Kernel>::CallManager as fvm::call_manager::CallManager>::Machine: Send,
{
    /// The kernel type used by this module's executor.
    type Kernel: Kernel;

    /// Get the module's name.
    ///
    /// This is used for logging and debugging.
    fn name(&self) -> &'static str;

    /// Optional: Get the module version.
    ///
    /// This can be used for compatibility checks and logging.
    fn version(&self) -> &'static str {
        "0.1.0"
    }

    /// Optional: Get a description of what this module provides.
    fn description(&self) -> &'static str {
        "No description provided"
    }
}

/// Default no-op module bundle.
///
/// This provides a baseline implementation that does nothing. It's useful
/// for testing and for situations where no module extensions are needed.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpModuleBundle;

// Import the no-op implementations
use crate::cli::NoOpCliModule;
use crate::executor::NoOpExecutorModule;
use crate::externs::NoOpExterns;
use crate::genesis::NoOpGenesisModule;
use crate::message::NoOpMessageHandlerModule;
use crate::service::NoOpServiceModule;

// Implement ExecutorModule by delegating to NoOpExecutorModule
impl<K> ExecutorModule<K> for NoOpModuleBundle
where
    K: Kernel,
    <K::CallManager as CallManager>::Machine: Send,
{
    type Executor = <NoOpExecutorModule as ExecutorModule<K>>::Executor;

    fn create_executor(
        engine_pool: fvm::engine::EnginePool,
        machine: <K::CallManager as CallManager>::Machine,
    ) -> anyhow::Result<Self::Executor> {
        NoOpExecutorModule::create_executor(engine_pool, machine)
    }
}

// Implement MessageHandlerModule by delegating to NoOpMessageHandlerModule
#[async_trait::async_trait]
impl MessageHandlerModule for NoOpModuleBundle {
    async fn handle_message<DB: fvm_ipld_blockstore::Blockstore + Send + Sync>(
        &self,
        state: &mut dyn crate::message::MessageHandlerState,
        msg: &fendermint_vm_message::ipc::IpcMessage,
    ) -> anyhow::Result<Option<crate::message::ApplyMessageResponse>> {
        NoOpMessageHandlerModule.handle_message::<DB>(state, msg).await
    }

    fn message_types(&self) -> &[&str] {
        NoOpMessageHandlerModule.message_types()
    }

    async fn validate_message(
        &self,
        msg: &fendermint_vm_message::ipc::IpcMessage,
    ) -> anyhow::Result<bool> {
        NoOpMessageHandlerModule.validate_message(msg).await
    }
}

// Implement GenesisModule by delegating to NoOpGenesisModule
impl GenesisModule for NoOpModuleBundle {
    fn initialize_actors<S: crate::genesis::GenesisState>(
        &self,
        state: &mut S,
        genesis: &fendermint_vm_genesis::Genesis,
    ) -> anyhow::Result<()> {
        NoOpGenesisModule.initialize_actors(state, genesis)
    }

    fn name(&self) -> &str {
        NoOpGenesisModule.name()
    }

    fn validate_genesis(&self, genesis: &fendermint_vm_genesis::Genesis) -> anyhow::Result<()> {
        NoOpGenesisModule.validate_genesis(genesis)
    }
}

// Implement ServiceModule by delegating to NoOpServiceModule
#[async_trait::async_trait]
impl ServiceModule for NoOpModuleBundle {
    async fn initialize_services(
        &self,
        ctx: &crate::service::ServiceContext,
    ) -> anyhow::Result<Vec<tokio::task::JoinHandle<()>>> {
        NoOpServiceModule.initialize_services(ctx).await
    }

    fn resources(&self) -> crate::service::ModuleResources {
        NoOpServiceModule.resources()
    }

    async fn shutdown(&self) -> anyhow::Result<()> {
        NoOpServiceModule.shutdown().await
    }

    async fn health_check(&self) -> anyhow::Result<bool> {
        NoOpServiceModule.health_check().await
    }
}

// Implement CliModule by delegating to NoOpCliModule
#[async_trait::async_trait]
impl CliModule for NoOpModuleBundle {
    fn commands(&self) -> Vec<crate::cli::CommandDef> {
        NoOpCliModule.commands()
    }

    async fn execute(&self, args: &crate::cli::CommandArgs) -> anyhow::Result<()> {
        NoOpCliModule.execute(args).await
    }

    fn validate_args(&self, args: &crate::cli::CommandArgs) -> anyhow::Result<()> {
        NoOpCliModule.validate_args(args)
    }

    fn complete(&self, command: &str, arg: &str) -> Vec<String> {
        NoOpCliModule.complete(command, arg)
    }
}

// Finally, implement ModuleBundle itself
impl ModuleBundle for NoOpModuleBundle {
    // Use a concrete Kernel type for the no-op implementation
    // This will be different for actual modules
    type Kernel = fvm::DefaultKernel<
        DefaultCallManager<DefaultMachine<fvm_ipld_blockstore::MemoryBlockstore, NoOpExterns>>,
    >;

    fn name(&self) -> &'static str {
        "noop"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn description(&self) -> &'static str {
        "No-op module bundle that provides baseline functionality with no extensions"
    }
}

impl std::fmt::Display for NoOpModuleBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NoOpModuleBundle")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_op_bundle_default() {
        let _bundle = NoOpModuleBundle::default();
    }

    #[test]
    fn test_no_op_bundle_name() {
        let bundle = NoOpModuleBundle;
        assert_eq!(ModuleBundle::name(&bundle), "noop");
    }

    #[test]
    fn test_no_op_bundle_version() {
        let bundle = NoOpModuleBundle;
        assert_eq!(bundle.version(), "0.1.0");
    }

    #[test]
    fn test_no_op_bundle_description() {
        let bundle = NoOpModuleBundle;
        assert!(!bundle.description().is_empty());
    }

    #[test]
    fn test_no_op_bundle_clone() {
        let bundle1 = NoOpModuleBundle;
        let _bundle2 = bundle1;
        let _bundle3 = bundle1; // NoOpModuleBundle is Copy
    }

    #[test]
    fn test_no_op_bundle_display() {
        let bundle = NoOpModuleBundle;
        let display = format!("{}", bundle);
        assert_eq!(display, "NoOpModuleBundle");
    }
}
