// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Cid;
use fil_actors_runtime::{
    deserialize_block, extract_send_result,
    runtime::{builtins::Type, Runtime},
    ActorError, ADM_ACTOR_ADDR,
};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::sys::SendFlags;
use fvm_shared::{address::Address, bigint::BigUint, econ::TokenAmount, MethodNum};
use num_traits::Zero;
use serde::{Deserialize, Serialize};

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

/// Resolves an address to its external delegated address.
pub fn to_delegated_address(rt: &impl Runtime, address: Address) -> Result<Address, ActorError> {
    Ok(to_id_and_delegated_address(rt, address)?.1)
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
        .ok_or(ActorError::forbidden(format!(
            "invalid address: actor {} is not delegated",
            address
        )))?;
    Ok((Address::new_id(actor_id), delegated))
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

/// The kinds of machines available.
#[derive(Debug, Serialize, Deserialize)]
pub enum Kind {
    /// A bucket with S3-like key semantics.
    Bucket,
    /// An MMR accumulator, used for timestamping data.
    Timehub,
}

pub fn is_bucket_address(rt: &impl Runtime, address: Address) -> Result<bool, ActorError> {
    let caller_code_cid = rt
        .resolve_address(&address)
        .and_then(|actor_id| rt.get_actor_code_cid(&actor_id));
    if let Some(caller_code_cid) = caller_code_cid {
        let bucket_code_cid = deserialize_block::<Cid>(extract_send_result(rt.send(
            &ADM_ACTOR_ADDR,
            2892692559 as MethodNum,
            IpldBlock::serialize_cbor(&Kind::Bucket)?,
            TokenAmount::zero(),
            None,
            SendFlags::READ_ONLY,
        ))?)?;
        Ok(caller_code_cid.eq(&bucket_code_cid))
    } else {
        Ok(false)
    }
}
