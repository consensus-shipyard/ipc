// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use ethers_contract::{ContractRevert, EthError};
use fendermint_vm_actor_interface::ipc::subnet::SubnetActorErrors;
use fvm_shared::error::ExitCode;
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl From<anyhow::Error> for JsonRpcError {
    fn from(value: anyhow::Error) -> Self {
        Self {
            code: 0,
            message: format!("{:#}", value),
            data: None,
        }
    }
}

impl From<tendermint_rpc::Error> for JsonRpcError {
    fn from(value: tendermint_rpc::Error) -> Self {
        Self {
            code: 0,
            message: format!("Tendermint RPC error: {value}"),
            data: None,
        }
    }
}

impl From<JsonRpcError> for jsonrpc_v2::Error {
    fn from(value: JsonRpcError) -> Self {
        Self::Full {
            code: value.code,
            message: value.message,
            data: value.data.map(|d| {
                let d: Box<dyn erased_serde::Serialize + Send> = Box::new(d);
                d
            }),
        }
    }
}

pub fn error<T>(exit_code: ExitCode, msg: impl ToString) -> Result<T, JsonRpcError> {
    Err(JsonRpcError {
        code: exit_code.value().into(),
        message: msg.to_string(),
        data: None,
    })
}

pub fn error_with_data<T, E: Serialize>(
    exit_code: ExitCode,
    msg: impl ToString,
    data: Option<E>,
) -> Result<T, JsonRpcError> {
    let data = data.map(|data| match serde_json::to_value(data) {
        Ok(v) => v,
        Err(e) => serde_json::Value::String(format!("failed to serialize error data: {e}")),
    });
    Err(JsonRpcError {
        code: exit_code.value().into(),
        message: msg.to_string(),
        data,
    })
}

/// Try to parse the data returned from the EVM as a revert string and append it to the message,
/// so we have a bit more human readable feedback than just hexadecimal strings with the selector
/// we can see in for example [here](https://github.com/gakonst/ethers-rs/commit/860100535812cbfe5e3cc417872392a6d76a159c).
///
/// The goal is that if Solidity has something like `require(x > 0, "X must be positive")` then we see the message in the JSON-RPC response.
pub fn error_with_revert<T>(
    exit_code: ExitCode,
    msg: impl ToString,
    data: Option<impl AsRef<[u8]>>,
) -> Result<T, JsonRpcError> {
    let msg = msg.to_string();
    let (msg, data) = match data {
        None => (msg, None),
        Some(data) => {
            // Try the simplest case of just a string, even though it's covered by the `SubnetActorErrors` as well.
            // Then see if it's an error that one of our known IPC actor facets are producing.
            let revert = if let Some(revert) = String::decode_with_selector(data.as_ref()) {
                Some(revert)
            } else {
                SubnetActorErrors::decode_with_selector(data.as_ref()).map(|e| e.to_string())
            };

            (
                revert.map(|rev| format!("{msg}\n{rev}")).unwrap_or(msg),
                Some(hex::encode(data)),
            )
        }
    };
    error_with_data(exit_code, msg, data)
}

impl std::fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (code: {})", self.message, self.code)
    }
}

impl std::error::Error for JsonRpcError {}
