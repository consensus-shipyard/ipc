// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod constants;
mod executions;
mod externs;
pub mod interpreter;
pub mod observe;
// storage_env moved to plugins/storage-node/src/storage_env.rs
// storage_helpers remains as internal implementation detail (tightly coupled to FvmExecState)
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
pub mod end_block_hook;
pub(crate) mod gas;
pub(crate) mod gas_estimation;

pub use fendermint_vm_message::query::FvmQuery;

pub type FvmMessage = fvm_shared::message::Message;
pub type BaseFee = fvm_shared::econ::TokenAmount;
pub type BlockGasLimit = u64;

// No default module - plugins are discovered at app layer
// Interpreter is fully generic over M: ModuleBundle
