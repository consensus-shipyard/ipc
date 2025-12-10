// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! State operation traits for plugin access to FVM execution state.
//!
//! These traits provide a controlled interface for plugins to interact with
//! the execution state without exposing internal implementation details.

use anyhow::Result;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::{address::Address, message::Message, MethodNum};

/// Return type for implicit message execution.
///
/// This is a simplified version of FvmApplyRet that plugins can use.
#[derive(Debug, Clone)]
pub struct ImplicitMessageResult {
    pub return_data: Vec<u8>,
    pub gas_used: u64,
    pub exit_code: fvm_shared::error::ExitCode,
}

/// Trait for executing implicit (system) messages.
///
/// This allows plugins to send messages as system actors without
/// going through the normal transaction flow.
pub trait ImplicitMessageExecutor {
    /// Execute an implicit message (system call).
    ///
    /// # Arguments
    ///
    /// * `to` - Destination actor address
    /// * `method` - Method number to call
    /// * `params` - CBOR-encoded parameters
    /// * `gas_limit` - Gas limit for execution
    ///
    /// # Returns
    ///
    /// The result of the message execution
    fn execute_implicit(
        &mut self,
        to: Address,
        method: MethodNum,
        params: RawBytes,
        gas_limit: u64,
    ) -> Result<ImplicitMessageResult>;

    /// Execute a full implicit message.
    ///
    /// This variant takes a complete Message struct for more control.
    fn execute_implicit_message(
        &mut self,
        msg: Message,
    ) -> Result<ImplicitMessageResult>;
}

/// Trait for plugins that need access to execution state operations.
///
/// This provides a safe, controlled interface for plugins to interact
/// with the FVM execution state during message handling.
pub trait PluginStateAccess: ImplicitMessageExecutor + Send + Sync {
    /// Get the current block height.
    fn block_height(&self) -> fvm_shared::clock::ChainEpoch;

    /// Get the current timestamp.
    fn timestamp(&self) -> fendermint_vm_core::Timestamp;

    /// Get the current base fee.
    fn base_fee(&self) -> &fvm_shared::econ::TokenAmount;

    /// Get the chain ID.
    fn chain_id(&self) -> u64;
}
