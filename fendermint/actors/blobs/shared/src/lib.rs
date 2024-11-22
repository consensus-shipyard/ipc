// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::runtime::builtins::Type;
use fil_actors_runtime::runtime::Runtime;
use fil_actors_runtime::{deserialize_block, extract_send_result, ActorError, AsActorError};
use fvm_ipld_encoding::ipld_block::IpldBlock;
use fvm_shared::address::Address;
use fvm_shared::bigint::BigUint;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::error::ExitCode;
use fvm_shared::sys::SendFlags;
use fvm_shared::{ActorID, MethodNum, METHOD_CONSTRUCTOR};
use num_derive::FromPrimitive;
use num_traits::Zero;

use crate::state::{Account, CreditApproval, Subscription};

pub mod params;
pub mod state;

mod ext;

pub const BLOBS_ACTOR_ID: ActorID = 66;
pub const BLOBS_ACTOR_ADDR: Address = Address::new_id(BLOBS_ACTOR_ID);

pub enum ActorType {
    Account,
    EthAccount,
    Evm,
    Machine,
}

/// Resolve robust address and ensure it is not a Machine actor type.
/// See `resolve_external`.
pub fn resolve_external_non_machine(
    rt: &impl Runtime,
    address: Address,
) -> Result<Address, ActorError> {
    let (address, actor_type) = resolve_external(rt, address)?;
    if matches!(actor_type, ActorType::Machine) {
        Err(ActorError::illegal_argument(format!(
            "address {} cannot be a machine",
            address
        )))
    } else {
        Ok(address)
    }
}

/// Resolves robust address of an actor.
pub fn resolve_external(
    rt: &impl Runtime,
    address: Address,
) -> Result<(Address, ActorType), ActorError> {
    let actor_id = rt
        .resolve_address(&address)
        .ok_or(ActorError::not_found(format!(
            "actor {} not found",
            address
        )))?;
    let code_cid = rt
        .get_actor_code_cid(&actor_id)
        .expect("failed to lookup caller code");
    match rt.resolve_builtin_actor_type(&code_cid) {
        Some(Type::Account) => {
            let result = rt
                .send(
                    &address,
                    ext::account::PUBKEY_ADDRESS_METHOD,
                    None,
                    Zero::zero(),
                    None,
                    SendFlags::READ_ONLY,
                )
                .context_code(
                    ExitCode::USR_ASSERTION_FAILED,
                    "account failed to return its key address",
                )?;
            if !result.exit_code.is_success() {
                return Err(ActorError::checked(
                    result.exit_code,
                    "failed to retrieve account robust address".to_string(),
                    None,
                ));
            }
            let robust_addr: Address = deserialize_block(result.return_data)?;
            Ok((robust_addr, ActorType::Account))
        }
        Some(Type::EthAccount) => {
            let delegated_addr =
                rt.lookup_delegated_address(actor_id)
                    .ok_or(ActorError::forbidden(format!(
                        "actor {} does not have delegated address",
                        actor_id
                    )))?;
            Ok((delegated_addr, ActorType::EthAccount))
        }
        Some(Type::EVM) => {
            let delegated_addr =
                rt.lookup_delegated_address(actor_id)
                    .ok_or(ActorError::forbidden(format!(
                        "actor {} does not have delegated address",
                        actor_id
                    )))?;
            Ok((delegated_addr, ActorType::Evm))
        }
        Some(t) => Err(ActorError::forbidden(format!(
            "disallowed caller type {} for address {}",
            t.name(),
            address
        ))),
        None => {
            // The caller might be a machine
            let result = rt
                .send(
                    &address,
                    fendermint_actor_machine::GET_ADDRESS_METHOD,
                    None,
                    Zero::zero(),
                    None,
                    SendFlags::READ_ONLY,
                )
                .context_code(
                    ExitCode::USR_ASSERTION_FAILED,
                    "machine failed to return its key address",
                )?;
            if !result.exit_code.is_success() {
                return Err(ActorError::forbidden(format!(
                    "disallowed caller code {code_cid}"
                )));
            }
            let robust_addr: Address = deserialize_block(result.return_data)?;
            Ok((robust_addr, ActorType::Machine))
        }
    }
}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    GetStats = frc42_dispatch::method_hash!("GetStats"),
    BuyCredit = frc42_dispatch::method_hash!("BuyCredit"),
    ApproveCredit = frc42_dispatch::method_hash!("ApproveCredit"),
    GetCreditApproval = frc42_dispatch::method_hash!("GetCreditApproval"),
    RevokeCredit = frc42_dispatch::method_hash!("RevokeCredit"),
    GetAccount = frc42_dispatch::method_hash!("GetAccount"),
    DebitAccounts = frc42_dispatch::method_hash!("DebitAccounts"),
    AddBlob = frc42_dispatch::method_hash!("AddBlob"),
    GetBlob = frc42_dispatch::method_hash!("GetBlob"),
    GetBlobStatus = frc42_dispatch::method_hash!("GetBlobStatus"),
    GetAddedBlobs = frc42_dispatch::method_hash!("GetAddedBlobs"),
    GetPendingBlobs = frc42_dispatch::method_hash!("GetPendingBlobs"),
    SetBlobPending = frc42_dispatch::method_hash!("SetBlobPending"),
    FinalizeBlob = frc42_dispatch::method_hash!("FinalizeBlob"),
    DeleteBlob = frc42_dispatch::method_hash!("DeleteBlob"),
}

pub fn buy_credit(rt: &impl Runtime, recipient: Address) -> Result<Account, ActorError> {
    deserialize_block(extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::BuyCredit as MethodNum,
        IpldBlock::serialize_cbor(&params::BuyCreditParams(recipient))?,
        rt.message().value_received(),
    ))?)
}

pub fn approve_credit(
    rt: &impl Runtime,
    from: Address,
    receiver: Address,
    required_caller: Option<Address>,
    limit: Option<BigUint>,
    ttl: Option<ChainEpoch>,
) -> Result<CreditApproval, ActorError> {
    deserialize_block(extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::ApproveCredit as MethodNum,
        IpldBlock::serialize_cbor(&params::ApproveCreditParams {
            from,
            receiver,
            required_caller,
            limit,
            ttl,
        })?,
        rt.message().value_received(),
    ))?)
}

pub fn get_credit_approval(
    rt: &impl Runtime,
    from: Address,
    receiver: Address,
    caller: Address,
) -> Result<Option<CreditApproval>, ActorError> {
    let params = params::GetCreditApprovalParams {
        from,
        receiver,
        caller,
    };

    deserialize_block(extract_send_result(rt.send(
        &BLOBS_ACTOR_ADDR,
        Method::GetCreditApproval as MethodNum,
        IpldBlock::serialize_cbor(&params)?,
        rt.message().value_received(),
        None,
        SendFlags::READ_ONLY,
    ))?)
}

pub fn revoke_credit(
    rt: &impl Runtime,
    from: Address,
    receiver: Address,
    required_caller: Option<Address>,
) -> Result<(), ActorError> {
    extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::RevokeCredit as MethodNum,
        IpldBlock::serialize_cbor(&params::RevokeCreditParams {
            from,
            receiver,
            required_caller,
        })?,
        rt.message().value_received(),
    ))?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn add_blob(
    rt: &impl Runtime,
    sponsor: Option<Address>,
    source: state::PublicKey,
    hash: state::Hash,
    metadata_hash: state::Hash,
    id: state::SubscriptionId,
    size: u64,
    ttl: Option<ChainEpoch>,
) -> Result<Subscription, ActorError> {
    let params = IpldBlock::serialize_cbor(&params::AddBlobParams {
        sponsor,
        source,
        hash,
        metadata_hash,
        id,
        size,
        ttl,
    })?;
    deserialize_block(extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::AddBlob as MethodNum,
        params,
        rt.message().value_received(),
    ))?)
}

pub fn get_blob(rt: &impl Runtime, hash: state::Hash) -> Result<Option<state::Blob>, ActorError> {
    deserialize_block(extract_send_result(rt.send(
        &BLOBS_ACTOR_ADDR,
        Method::GetBlob as MethodNum,
        IpldBlock::serialize_cbor(&params::GetBlobParams(hash))?,
        rt.message().value_received(),
        None,
        SendFlags::READ_ONLY,
    ))?)
}

pub fn delete_blob(
    rt: &impl Runtime,
    sponsor: Option<Address>,
    hash: state::Hash,
    id: state::SubscriptionId,
) -> Result<(), ActorError> {
    extract_send_result(rt.send_simple(
        &BLOBS_ACTOR_ADDR,
        Method::DeleteBlob as MethodNum,
        IpldBlock::serialize_cbor(&params::DeleteBlobParams { sponsor, hash, id })?,
        rt.message().value_received(),
    ))?;
    Ok(())
}
