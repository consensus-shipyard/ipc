// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod constants;
mod executions;
mod externs;
pub mod interpreter;
pub mod observe;
#[cfg(feature = "storage-node")]
pub mod storage_env;
#[cfg(feature = "storage-node")]
pub mod storage_helpers;
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
