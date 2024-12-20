// Copyright 2024 Hoku Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

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

pub const HOKU_CONFIG_ACTOR_ID: ActorID = 70;
pub const HOKU_CONFIG_ACTOR_ADDR: Address = Address::new_id(HOKU_CONFIG_ACTOR_ID);

/// The updatable config.
#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct HokuConfig {
    /// The total storage capacity of the subnet.
    pub blob_capacity: u64,
    /// The token to credit rate. The amount of atto credits that 1 atto buys.
    pub token_credit_rate: BigInt,
    /// Block interval at which to debit all credit accounts.
    pub blob_credit_debit_interval: ChainEpoch,
    /// The minimum epoch duration a blob can be stored.
    pub blob_min_ttl: ChainEpoch,
    /// The rolling epoch duration used for non-expiring blobs.
    pub blob_auto_renew_ttl: ChainEpoch,
}

impl Default for HokuConfig {
    fn default() -> Self {
        Self {
            blob_capacity: 10 * 1024 * 1024 * 1024 * 1024, // 10 TiB
            token_credit_rate: BigInt::from(1_000_000_000_000_000_000u64), // 1 atto = 1 credit (1e18 atto credit)
            blob_credit_debit_interval: ChainEpoch::from(3600),
            blob_min_ttl: ChainEpoch::from(3600),
            blob_auto_renew_ttl: ChainEpoch::from(3600),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SetAdminParams(pub Address);

pub type SetConfigParams = HokuConfig;

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
        &HOKU_CONFIG_ACTOR_ADDR,
        Method::GetAdmin as MethodNum,
        None,
        TokenAmount::zero(),
        None,
        SendFlags::READ_ONLY,
    ))?)
}

pub fn get_config(rt: &impl Runtime) -> Result<HokuConfig, ActorError> {
    deserialize_block(extract_send_result(rt.send(
        &HOKU_CONFIG_ACTOR_ADDR,
        Method::GetConfig as MethodNum,
        None,
        TokenAmount::zero(),
        None,
        SendFlags::READ_ONLY,
    ))?)
}
