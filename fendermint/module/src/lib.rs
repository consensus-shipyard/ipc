// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Fendermint Module System
//!
//! This crate provides a modular extension system for Fendermint, allowing
//! functionality to be added at compile-time through a trait-based architecture.
//!
//! # Overview
//!
//! The module system consists of five core traits:
//!
//! - [`ExecutorModule`] - Customize FVM message execution
//! - [`MessageHandlerModule`] - Handle custom IPC message types
//! - [`GenesisModule`] - Initialize actors during genesis
//! - [`ServiceModule`] - Start background services
//! - [`CliModule`] - Add CLI commands
//!
//! These traits are composed together in the [`ModuleBundle`] trait, which
//! represents a complete module package.
//!
//! # Architecture
//!
//! The module system uses zero-cost static dispatch through generics. Core
//! Fendermint types become generic over `ModuleBundle`, allowing the compiler
//! to specialize code for each module configuration.
//!
//! ```text
//!                         ┌─────────────────┐
//!                         │  ModuleBundle   │
//!                         └────────┬────────┘
//!                                  │
//!                  ┌───────────────┼───────────────┐
//!                  │               │               │
//!          ┌───────▼──────┐ ┌──────▼──────┐ ┌─────▼──────┐
//!          │   Executor   │ │   Message   │ │  Genesis   │
//!          │    Module    │ │   Handler   │ │   Module   │
//!          └──────────────┘ └─────────────┘ └────────────┘
//!                  │               │               │
//!          ┌───────▼──────┐ ┌──────▼──────┐      │
//!          │   Service    │ │     CLI     │      │
//!          │    Module    │ │   Module    │      │
//!          └──────────────┘ └─────────────┘      │
//! ```
//!
//! # Example
//!
//! Creating a custom module:
//!
//! ```ignore
//! use fendermint_module::*;
//!
//! struct MyModule {
//!     // module state
//! }
//!
//! // Implement each trait
//! impl<K: Kernel> ExecutorModule<K> for MyModule {
//!     type Executor = MyCustomExecutor<K>;
//!     fn create_executor(...) -> Result<Self::Executor> { ... }
//! }
//!
//! #[async_trait]
//! impl MessageHandlerModule for MyModule {
//!     async fn handle_message(...) -> Result<Option<ApplyMessageResponse>> { ... }
//!     fn message_types(&self) -> &[&str] { ... }
//! }
//!
//! impl GenesisModule for MyModule {
//!     fn initialize_actors(...) -> Result<()> { ... }
//!     fn name(&self) -> &str { ... }
//! }
//!
//! #[async_trait]
//! impl ServiceModule for MyModule {
//!     async fn initialize_services(...) -> Result<Vec<JoinHandle<()>>> { ... }
//!     fn resources(&self) -> ModuleResources { ... }
//! }
//!
//! #[async_trait]
//! impl CliModule for MyModule {
//!     fn commands(&self) -> Vec<CommandDef> { ... }
//!     async fn execute(...) -> Result<()> { ... }
//! }
//!
//! // Compose into a bundle
//! impl ModuleBundle for MyModule {
//!     type Kernel = MyKernel;
//!     fn name(&self) -> &'static str { "my-module" }
//! }
//! ```
//!
//! # Feature Flags
//!
//! Modules are selected at compile-time using feature flags:
//!
//! ```toml
//! [features]
//! default = []
//! my-module = ["my_module_crate"]
//! ```
//!
//! # Benefits
//!
//! - **Zero Runtime Overhead** - Static dispatch, no vtables
//! - **Type Safety** - Compile-time guarantees
//! - **Modularity** - Clean separation of concerns
//! - **Extensibility** - Easy to add new modules
//! - **Testability** - Mock modules for testing

// Re-export key types from dependencies
pub use anyhow::{bail, Context, Result};
pub use async_trait::async_trait;
pub use fvm;
pub use fvm_ipld_blockstore::Blockstore;
pub use fvm_shared;

// Module trait definitions
pub mod bundle;
pub mod cli;
pub mod executor;
pub mod externs;
pub mod genesis;
pub mod message;
pub mod service;

// Re-export main types
pub use bundle::{ModuleBundle, NoOpModuleBundle};
pub use cli::{CliModule, CommandArgs, CommandDef, NoOpCliModule};
pub use executor::{DelegatingExecutor, ExecutorModule, NoOpExecutorModule};
pub use genesis::{GenesisModule, GenesisState, NoOpGenesisModule};
pub use message::{
    ApplyMessageResponse, MessageApplyRet, MessageHandlerModule, MessageHandlerState,
    NoOpMessageHandlerModule,
};
pub use service::{ModuleResources, NoOpServiceModule, ServiceContext, ServiceModule};

/// Prelude module for convenient imports.
///
/// Import everything from this module to get started quickly:
///
/// ```ignore
/// use fendermint_module::prelude::*;
/// ```
pub mod prelude {
    pub use crate::bundle::{ModuleBundle, NoOpModuleBundle};
    pub use crate::cli::{CliModule, CommandArgs, CommandDef};
    pub use crate::executor::ExecutorModule;
    pub use crate::genesis::{GenesisModule, GenesisState};
    pub use crate::message::{MessageHandlerModule, MessageHandlerState};
    pub use crate::service::{ModuleResources, ServiceContext, ServiceModule};
    pub use crate::{async_trait, bail, Context, Result};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noop_bundle_implements_all_traits() {
        let bundle = NoOpModuleBundle::default();

        // Test that it implements ModuleBundle
        assert_eq!(ModuleBundle::name(&bundle), "noop");

        // Test that it implements all sub-traits (compile-time check)
        fn _check_executor<K: fvm::kernel::Kernel>(_: &impl ExecutorModule<K>) {}
        fn _check_message(_: &impl MessageHandlerModule) {}
        fn _check_genesis(_: &impl GenesisModule) {}
        fn _check_service(_: &impl ServiceModule) {}
        fn _check_cli(_: &impl CliModule) {}

        _check_message(&bundle);
        _check_genesis(&bundle);
        _check_service(&bundle);
        _check_cli(&bundle);
    }
}
