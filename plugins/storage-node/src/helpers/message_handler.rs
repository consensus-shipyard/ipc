// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Message handling for storage-node specific IPC messages.

use anyhow::Result;
use fendermint_module::message::{ApplyMessageResponse, MessageApplyRet};
use fendermint_vm_message::ipc::{IpcMessage, PendingReadRequest, ClosedReadRequest};
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address;
use fvm_shared::error::ExitCode;
use std::collections::HashMap;

/// Handle ReadRequestPending message.
///
/// This sets a read request to "pending" state, indicating that validators
/// are working on resolving it.
pub fn handle_read_request_pending(
    read_request: &PendingReadRequest,
) -> Result<ApplyMessageResponse> {
    tracing::debug!(
        request_id = %read_request.id,
        "Handling ReadRequestPending message"
    );

    // TODO: Implement actual storage logic
    // This requires access to FvmExecState to call storage_helpers::set_read_request_pending
    // For now, return a placeholder response

    Ok(ApplyMessageResponse {
        apply_ret: MessageApplyRet {
            from: Address::new_id(0),
            to: Address::new_id(1),
            method_num: 0,
            gas_limit: 10_000_000,
            exit_code: ExitCode::OK,
            gas_used: 100,
            return_data: RawBytes::default(),
            emitters: HashMap::new(),
        },
        domain_hash: None,
    })
}

/// Handle ReadRequestClosed message.
///
/// This executes the callback for a read request and closes it.
pub fn handle_read_request_closed(
    read_request: &ClosedReadRequest,
) -> Result<ApplyMessageResponse> {
    tracing::debug!(
        request_id = %read_request.id,
        "Handling ReadRequestClosed message"
    );

    // TODO: Implement actual storage logic
    // This requires access to FvmExecState to call:
    // 1. storage_helpers::read_request_callback
    // 2. storage_helpers::close_read_request

    Ok(ApplyMessageResponse {
        apply_ret: MessageApplyRet {
            from: Address::new_id(0),
            to: Address::new_id(1),
            method_num: 0,
            gas_limit: 10_000_000,
            exit_code: ExitCode::OK,
            gas_used: 100,
            return_data: RawBytes::default(),
            emitters: HashMap::new(),
        },
        domain_hash: None,
    })
}

/// Validate a storage-node IPC message.
pub fn validate_storage_message(msg: &IpcMessage) -> Result<bool> {
    match msg {
        IpcMessage::ReadRequestPending(_) | IpcMessage::ReadRequestClosed(_) => {
            // TODO: Add actual validation logic
            // - Check signatures
            // - Verify request exists
            // - Validate data format
            Ok(true)
        }
        _ => Ok(true), // Don't validate messages we don't handle
    }
}
