// Copyright 2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fil_actors_runtime::runtime::Runtime;
use fil_actors_runtime::ActorError;
use fvm_ipld_encoding::tuple::*;
use fvm_shared::econ::TokenAmount;
use num_derive::FromPrimitive;

pub type Gas = u64;

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct Reading {
    pub block_gas_limit: Gas,
    pub base_fee: TokenAmount,
}

#[derive(Serialize_tuple, Deserialize_tuple, Debug, Clone)]
pub struct Utilization {
    pub block_gas_used: Gas,
}

#[derive(FromPrimitive)]
#[repr(u64)]
pub enum Method {
    CurrentReading = frc42_dispatch::method_hash!("CurrentReading"),
    UpdateUtilization = frc42_dispatch::method_hash!("UpdateUtilization"),
}

pub trait GasMarket {
    fn current_reading(rt: &impl Runtime) -> Result<Reading, ActorError>;

    fn update_utilization(
        rt: &impl Runtime,
        utilization: Utilization,
    ) -> Result<Reading, ActorError>;
}
