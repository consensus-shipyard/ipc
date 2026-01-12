// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::{actor_error, runtime::Runtime, ActorError};
use fvm_ipld_encoding::{strict_bytes, tuple::*};
use fvm_shared::event::{ActorEvent, Entry, Flags};
use fvm_shared::IPLD_RAW;
use recall_sol_facade::primitives::IntoLogData;

/// The event key prefix for the Ethereum log topics.
const EVENT_TOPIC_KEY_PREFIX: &str = "t";

/// The event key for the Ethereum log data.
const EVENT_DATA_KEY: &str = "d";

pub trait TryIntoEVMEvent {
    type Target: IntoLogData;
    fn try_into_evm_event(self) -> Result<Self::Target, anyhow::Error>;
}

/// Returns an [`ActorEvent`] from an EVM event.
pub fn to_actor_event<T: TryIntoEVMEvent>(event: T) -> Result<ActorEvent, ActorError> {
    let event = event
        .try_into_evm_event()
        .map_err(|e| actor_error!(illegal_argument; "failed to build evm event: {}", e))?;
    let log = event.to_log_data();
    let num_entries = log.topics().len() + 1; // +1 for log data

    let mut entries: Vec<Entry> = Vec::with_capacity(num_entries);
    for (i, topic) in log.topics().iter().enumerate() {
        let key = format!("{}{}", EVENT_TOPIC_KEY_PREFIX, i + 1);
        entries.push(Entry {
            flags: Flags::FLAG_INDEXED_ALL,
            key,
            codec: IPLD_RAW,
            value: topic.to_vec(),
        });
    }
    entries.push(Entry {
        flags: Flags::FLAG_INDEXED_ALL,
        key: EVENT_DATA_KEY.to_owned(),
        codec: IPLD_RAW,
        value: log.data.to_vec(),
    });

    Ok(entries.into())
}

/// Emits an [`ActorEvent`] from an EVM event.
pub fn emit_evm_event<T: TryIntoEVMEvent>(rt: &impl Runtime, event: T) -> Result<(), ActorError> {
    let actor_event = to_actor_event(event)?;
    rt.emit_event(&actor_event)
}

/// Params for invoking a contract.
#[derive(Default, Serialize_tuple, Deserialize_tuple)]
#[serde(transparent)]
pub struct InvokeContractParams {
    #[serde(with = "strict_bytes")]
    pub input_data: Vec<u8>,
}

/// EVM call with selector (first 4 bytes) and calldata (remaining bytes).
pub struct InputData(Vec<u8>);

impl InputData {
    /// Returns the selector bytes.
    pub fn selector(&self) -> [u8; 4] {
        let mut selector = [0u8; 4];
        selector.copy_from_slice(&self.0[0..4]);
        selector
    }

    /// Returns the calldata bytes.
    pub fn calldata(&self) -> &[u8] {
        &self.0[4..]
    }
}

impl TryFrom<InvokeContractParams> for InputData {
    type Error = ActorError;

    fn try_from(value: InvokeContractParams) -> Result<Self, Self::Error> {
        if value.input_data.len() < 4 {
            return Err(ActorError::illegal_argument("input too short".to_string()));
        }
        Ok(InputData(value.input_data))
    }
}

#[macro_export]
macro_rules! declare_abi_call {
    () => {
        pub trait AbiCall {
            type Params;
            type Returns;
            type Output;
            fn params(&self) -> Self::Params;
            fn returns(&self, returns: Self::Returns) -> Self::Output;
        }

        pub trait AbiCallRuntime {
            type Params;
            type Returns;
            type Output;
            fn params(&self, rt: &impl fil_actors_runtime::runtime::Runtime) -> Self::Params;
            fn returns(&self, returns: Self::Returns) -> Self::Output;
        }

        #[derive(Debug, Clone)]
        pub struct AbiEncodeError {
            message: String,
        }

        impl From<anyhow::Error> for AbiEncodeError {
            fn from(error: anyhow::Error) -> Self {
                Self {
                    message: format!("failed to abi encode {}", error),
                }
            }
        }

        impl From<String> for AbiEncodeError {
            fn from(message: String) -> Self {
                Self { message }
            }
        }

        impl From<fil_actors_runtime::ActorError> for AbiEncodeError {
            fn from(error: fil_actors_runtime::ActorError) -> Self {
                Self {
                    message: format!("{}", error),
                }
            }
        }

        impl From<AbiEncodeError> for fil_actors_runtime::ActorError {
            fn from(error: AbiEncodeError) -> Self {
                fil_actors_runtime::actor_error!(serialization, error.message)
            }
        }
    };
}

/// Returned when invoking a contract.
#[derive(Serialize_tuple, Deserialize_tuple)]
#[serde(transparent)]
pub struct InvokeContractReturn {
    #[serde(with = "strict_bytes")]
    pub output_data: Vec<u8>,
}
