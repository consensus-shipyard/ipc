// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::runtime::builtins::Type;
use fil_actors_runtime::runtime::Runtime;
use fil_actors_runtime::{actor_error, ActorError};
use fvm_ipld_encoding::{strict_bytes, tuple::*};
use fvm_shared::address::Address;
use fvm_shared::bigint::BigUint;
use fvm_shared::econ::TokenAmount;
use fvm_shared::error::ErrorNumber;
use fvm_shared::event::{ActorEvent, Entry, Flags};
use fvm_shared::IPLD_RAW;
use recall_sol_facade::primitives::IntoLogData;

pub fn hash_rm(hash: [u8; 32]) -> Result<(), ErrorNumber> {
    unsafe { sys::hash_rm(hash.as_ptr()) }
}

mod sys {
    use fvm_sdk::sys::fvm_syscalls;

    fvm_syscalls! {
        module = "recall";
        pub fn hash_rm(hash_ptr: *const u8) -> Result<()>;
    }
}

/// Returns an error if the address does not match the message origin or caller.
pub fn require_addr_is_origin_or_caller(
    rt: &impl Runtime,
    address: Address,
) -> Result<(), ActorError> {
    let address = to_id_address(rt, address, false)?;
    if address == rt.message().origin() || address == rt.message().caller() {
        return Ok(());
    }
    Err(ActorError::illegal_argument(format!(
        "address {} does not match origin or caller",
        address
    )))
}

/// Resolves ID address of an actor.
/// If `require_delegated` is `true`, the address must be of type
/// EVM (a Solidity contract), EthAccount (an Ethereum-style EOA), or Placeholder (a yet to be
/// determined EOA or Solidity contract).
pub fn to_id_address(
    rt: &impl Runtime,
    address: Address,
    require_delegated: bool,
) -> Result<Address, ActorError> {
    let actor_id = rt
        .resolve_address(&address)
        .ok_or(ActorError::not_found(format!(
            "actor {} not found",
            address
        )))?;
    if require_delegated {
        let code_cid = rt.get_actor_code_cid(&actor_id).ok_or_else(|| {
            ActorError::not_found(format!("actor {} code cid not found", address))
        })?;
        if !matches!(
            rt.resolve_builtin_actor_type(&code_cid),
            Some(Type::Placeholder | Type::EVM | Type::EthAccount)
        ) {
            return Err(ActorError::forbidden(format!(
                "invalid address: address {} is not delegated",
                address,
            )));
        }
    }
    Ok(Address::new_id(actor_id))
}

pub trait TryIntoEVMEvent {
    type Target: IntoLogData;
    fn try_into_evm_event(self) -> Result<Self::Target, anyhow::Error>;
}

/// The event key prefix for the Ethereum log topics.
const EVENT_TOPIC_KEY_PREFIX: &str = "t";

/// The event key for the Ethereum log data.
const EVENT_DATA_KEY: &str = "d";

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

/// Resolves an address to its ID address and external delegated address.
pub fn to_id_and_delegated_address(
    rt: &impl Runtime,
    address: Address,
) -> Result<(Address, Address), ActorError> {
    let actor_id = rt
        .resolve_address(&address)
        .ok_or(ActorError::not_found(format!(
            "actor {} not found",
            address
        )))?;
    let delegated = rt
        .lookup_delegated_address(actor_id)
        .ok_or(ActorError::not_found(format!(
            "invalid address: actor {} is not delegated",
            address
        )))?;
    Ok((Address::new_id(actor_id), delegated))
}

/// Resolves an address to its external delegated address.
pub fn to_delegated_address(rt: &impl Runtime, address: Address) -> Result<Address, ActorError> {
    Ok(to_id_and_delegated_address(rt, address)?.1)
}

/// Returns the [`TokenAmount`] as a [`BigUint`].
/// If the given amount is negative, the value returned will be zero.
pub fn token_to_biguint(amount: Option<TokenAmount>) -> BigUint {
    amount
        .unwrap_or_default()
        .atto()
        .to_biguint()
        .unwrap_or_default()
}

#[derive(Default, Serialize_tuple, Deserialize_tuple)]
#[serde(transparent)]
pub struct InvokeContractParams {
    #[serde(with = "strict_bytes")]
    pub input_data: Vec<u8>,
}

/// EVM call with selector (first 4 bytes) and calldata (remaining bytes)
pub struct InputData(Vec<u8>);

impl InputData {
    pub fn selector(&self) -> [u8; 4] {
        let mut selector = [0u8; 4];
        selector.copy_from_slice(&self.0[0..4]);
        selector
    }

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

#[derive(Serialize_tuple, Deserialize_tuple)]
#[serde(transparent)]
pub struct InvokeContractReturn {
    #[serde(with = "strict_bytes")]
    pub output_data: Vec<u8>,
}
