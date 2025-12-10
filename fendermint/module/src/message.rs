// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Message handler module trait for processing custom IPC messages.
//!
//! This trait allows modules to handle custom message types that extend
//! the core IPC message set. Modules can intercept and process messages
//! before they reach the default handler.

use anyhow::Result;
use async_trait::async_trait;
use fendermint_vm_core::Timestamp;
use fendermint_vm_message::ipc::IpcMessage;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use fvm_shared::MethodNum;
use std::collections::HashMap;
use std::fmt;

/// Response from applying a message to the chain state.
///
/// This mirrors the structure used in the interpreter for consistency.
#[derive(Clone, Debug)]
pub struct ApplyMessageResponse {
    /// The result of applying the message
    pub apply_ret: MessageApplyRet,
    /// Optional domain hash for the message
    pub domain_hash: Option<[u8; 32]>,
}

/// Result of applying a message to the state.
#[derive(Clone, Debug)]
pub struct MessageApplyRet {
    /// Message sender address
    pub from: Address,
    /// Message receiver address
    pub to: Address,
    /// Method number called
    pub method_num: MethodNum,
    /// Gas limit for the message
    pub gas_limit: u64,
    /// Exit code from execution
    pub exit_code: fvm_shared::error::ExitCode,
    /// Gas used during execution
    pub gas_used: u64,
    /// Return value from the message
    pub return_data: fvm_ipld_encoding::RawBytes,
    /// Event emitter delegated addresses
    pub emitters: HashMap<fvm_shared::ActorID, Address>,
}

/// State context provided to message handlers.
///
/// This is a simplified view of the execution state that message handlers
/// can use to interact with the FVM.
pub trait MessageHandlerState: Send + Sync {
    /// Get the current block height
    fn block_height(&self) -> ChainEpoch;

    /// Get the current timestamp
    fn timestamp(&self) -> Timestamp;

    /// Get the current base fee
    fn base_fee(&self) -> &TokenAmount;

    /// Get the chain ID
    fn chain_id(&self) -> u64;
}

/// Module trait for handling custom IPC messages.
///
/// Modules can implement this trait to handle specific message types.
/// When a message is received, the interpreter will try each module's
/// handler in order. The first module to return `Some(response)` will
/// handle the message.
///
/// # Example
///
/// ```ignore
/// struct MyModule;
///
/// #[async_trait]
/// impl MessageHandlerModule for MyModule {
///     async fn handle_message<DB: Blockstore + Send + Sync>(
///         &self,
///         state: &mut dyn MessageHandlerState,
///         msg: &IpcMessage,
///     ) -> Result<Option<ApplyMessageResponse>> {
///         match msg {
///             IpcMessage::MyCustomMessage(data) => {
///                 // Handle the message
///                 let response = process_my_message(state, data)?;
///                 Ok(Some(response))
///             }
///             _ => Ok(None), // Don't handle other messages
///         }
///     }
///
///     fn message_types(&self) -> &[&str] {
///         &["MyCustomMessage"]
///     }
/// }
/// ```
#[async_trait]
pub trait MessageHandlerModule: Send + Sync {
    /// Handle a message.
    ///
    /// # Arguments
    ///
    /// * `state` - The current execution state
    /// * `msg` - The IPC message to handle
    ///
    /// # Returns
    ///
    /// * `Ok(Some(response))` if this module handled the message
    /// * `Ok(None)` if this module does not handle this message type
    /// * `Err(e)` if an error occurred while handling the message
    async fn handle_message<DB: Blockstore + Send + Sync>(
        &self,
        state: &mut dyn MessageHandlerState,
        msg: &IpcMessage,
    ) -> Result<Option<ApplyMessageResponse>>;

    /// List the message types this module handles.
    ///
    /// This is used for logging and debugging. It should return a list
    /// of human-readable message type names (e.g., "ReadRequestPending").
    fn message_types(&self) -> &[&str];

    /// Validate a message before it's included in a block.
    ///
    /// This is called during the message preparation phase. Modules can
    /// reject messages that don't meet their requirements.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if the message is valid
    /// * `Ok(false)` if the message should be rejected
    /// * `Err(e)` if an error occurred during validation
    async fn validate_message(&self, _msg: &IpcMessage) -> Result<bool> {
        Ok(true) // Default: accept all messages
    }
}

/// Default no-op message handler that doesn't handle any messages.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoOpMessageHandlerModule;

#[async_trait]
impl MessageHandlerModule for NoOpMessageHandlerModule {
    async fn handle_message<DB: Blockstore + Send + Sync>(
        &self,
        _state: &mut dyn MessageHandlerState,
        _msg: &IpcMessage,
    ) -> Result<Option<ApplyMessageResponse>> {
        Ok(None) // Don't handle any messages
    }

    fn message_types(&self) -> &[&str] {
        &[] // No message types handled
    }

    async fn validate_message(&self, _msg: &IpcMessage) -> Result<bool> {
        Ok(true) // Accept all messages (no validation)
    }
}

impl fmt::Display for NoOpMessageHandlerModule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NoOpMessageHandler")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full integration test is skipped because it requires complex setup.
    // The trait implementation is verified at compile time.

    #[test]
    fn test_no_op_handler_message_types() {
        let handler = NoOpMessageHandlerModule;
        assert_eq!(handler.message_types().len(), 0);
    }

    #[tokio::test]
    async fn test_no_op_handler_validates_all() {
        use fendermint_vm_message::ipc::ParentFinality;

        let handler = NoOpMessageHandlerModule;
        let msg = IpcMessage::TopDownExec(ParentFinality {
            height: 0,
            block_hash: vec![],
        });

        let result = handler.validate_message(&msg).await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
