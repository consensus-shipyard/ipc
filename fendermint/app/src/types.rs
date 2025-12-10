// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Type aliases for the app layer.
//!
//! This module provides conditional type aliases based on enabled feature flags.
//! This allows the app to work with different module types without complex generics.

use fendermint_vm_interpreter::fvm::interpreter::FvmMessagesInterpreter;
use fendermint_vm_interpreter::fvm::state::FvmExecState;

/// The active module type, selected at compile time based on feature flags.
///
/// - With `plugin-storage-node`: Uses StorageNodeModule
/// - Without plugins: Uses NoOpModuleBundle (default)
#[cfg(feature = "plugin-storage-node")]
pub type AppModule = ipc_plugin_storage_node::StorageNodeModule;

#[cfg(not(feature = "plugin-storage-node"))]
pub type AppModule = fendermint_module::NoOpModuleBundle;

/// Type alias for the interpreter using the active module.
///
/// This simplifies type signatures throughout the app.
pub type AppInterpreter<DB> = FvmMessagesInterpreter<DB, AppModule>;

/// Type alias for execution state using the active module.
pub type AppExecState<DB> = FvmExecState<DB, AppModule>;
