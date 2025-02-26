// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::state::TokenCreditRate;
use fil_actors_runtime::runtime::Runtime;
use fil_actors_runtime::{deserialize_block, extract_send_result, ActorError};
use fvm_ipld_encoding::tuple::*;
use fvm_shared::address::Address;
use fvm_shared::bigint::BigInt;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use fvm_shared::sys::SendFlags;
use fvm_shared::{ActorID, MethodNum, METHOD_CONSTRUCTOR};
use num_derive::FromPrimitive;
use num_traits::Zero;
use serde::{Deserialize, Serialize};

pub const RECALL_CONFIG_ACTOR_ID: ActorID = 70;
pub const RECALL_CONFIG_ACTOR_ADDR: Address = Address::new_id(RECALL_CONFIG_ACTOR_ID);

/// The updatable config.
#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct RecallConfig {
    /// The total storage capacity of the subnet.
    pub blob_capacity: u64,
    /// The token to credit rate.
    pub token_credit_rate: TokenCreditRate,
    /// Epoch interval at which to debit all credit accounts.
    pub blob_credit_debit_interval: ChainEpoch,
    /// The minimum epoch duration a blob can be stored.
    pub blob_min_ttl: ChainEpoch,
    /// The default epoch duration a blob is stored.
    pub blob_default_ttl: ChainEpoch,
    /// Maximum number of blobs to delete in a single batch during debit.
    pub blob_delete_batch_size: u64,
    /// Maximum number of accounts to process in a single batch during debit.
    pub account_debit_batch_size: u64,
}

impl Default for RecallConfig {
    fn default() -> Self {
        Self {
            blob_capacity: 10 * 1024 * 1024 * 1024 * 1024, // 10 TiB
            // 1 RECALL buys 1e18 credits ~ 1 RECALL buys 1e36 atto credits.
            token_credit_rate: TokenCreditRate::from(BigInt::from(10u128.pow(36))),
            // This needs to be low enough to avoid out-of-gas errors.
            // TODO: Stress test with max-throughput (~100 blobs/s)
            blob_credit_debit_interval: ChainEpoch::from(60 * 10), // ~10 min
            blob_min_ttl: ChainEpoch::from(60 * 60),               // ~1 hour
            blob_default_ttl: ChainEpoch::from(60 * 60 * 24),      // ~1 day
            blob_delete_batch_size: 100,
            account_debit_batch_size: 1000,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SetAdminParams(pub Address);

pub type SetConfigParams = RecallConfig;

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    Constructor = METHOD_CONSTRUCTOR,
    SetAdmin = frc42_dispatch::method_hash!("SetAdmin"),
    GetAdmin = frc42_dispatch::method_hash!("GetAdmin"),
    SetConfig = frc42_dispatch::method_hash!("SetConfig"),
    GetConfig = frc42_dispatch::method_hash!("GetConfig"),
}

pub fn get_admin(rt: &impl Runtime) -> Result<Option<Address>, ActorError> {
    deserialize_block(extract_send_result(rt.send(
        &RECALL_CONFIG_ACTOR_ADDR,
        Method::GetAdmin as MethodNum,
        None,
        TokenAmount::zero(),
        None,
        SendFlags::READ_ONLY,
    ))?)
}

/// Requires caller is the Recall Admin.
pub fn require_caller_is_admin(rt: &impl Runtime) -> Result<(), ActorError> {
    let admin = get_admin(rt)?;
    if admin.is_none() {
        Err(ActorError::illegal_state(
            "admin address not set".to_string(),
        ))
    } else {
        Ok(rt.validate_immediate_caller_is(std::iter::once(&admin.unwrap()))?)
    }
}

pub fn get_config(rt: &impl Runtime) -> Result<RecallConfig, ActorError> {
    deserialize_block(extract_send_result(rt.send(
        &RECALL_CONFIG_ACTOR_ADDR,
        Method::GetConfig as MethodNum,
        None,
        TokenAmount::zero(),
        None,
        SendFlags::READ_ONLY,
    ))?)
}
