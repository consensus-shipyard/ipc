// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod constants;
mod executions;
mod externs;
pub mod interpreter;
pub mod observe;
// storage_env and storage_helpers removed - these should be in the storage-node plugin
// If needed, they can be re-added to the plugin itself
pub mod state;
pub mod store;
pub mod topdown;
pub mod upgrades;
pub use interpreter::FvmMessagesInterpreter;

#[cfg(any(test, feature = "bundle"))]
pub mod bundle;

pub mod activity;
pub mod default_module;
pub mod end_block_hook;
pub(crate) mod gas;
pub(crate) mod gas_estimation;

pub use fendermint_vm_message::query::FvmQuery;

pub type FvmMessage = fvm_shared::message::Message;
pub type BaseFee = fvm_shared::econ::TokenAmount;
pub type BlockGasLimit = u64;

// Convenient type aliases using the default module
pub use default_module::DefaultModule;
pub type DefaultFvmExecState<DB> = state::FvmExecState<DB, DefaultModule>;
pub type DefaultFvmMessagesInterpreter<DB> = interpreter::FvmMessagesInterpreter<DB, DefaultModule>;
