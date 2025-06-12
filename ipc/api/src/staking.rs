// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Staking module related types and functions

use crate::{eth_to_fil_amount, ethers_address_to_fil_address};
use ethers::utils::hex;
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use ipc_actors_abis::{lib_power_change_log, subnet_actor_getter_facet};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub type ConfigurationNumber = u64;

#[derive(Clone, Debug, num_enum::TryFromPrimitive, Deserialize, Serialize)]
#[non_exhaustive]
#[repr(u8)]
pub enum PowerOperation {
    SetMetadata = 0,
    SetPower = 1,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PowerChangeRequest {
    pub configuration_number: ConfigurationNumber,
    pub change: PowerChange,
}

/// The change request to validator staking
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PowerChange {
    pub op: PowerOperation,
    pub payload: Vec<u8>,
    pub validator: Address,
}

impl TryFrom<lib_power_change_log::NewPowerChangeRequestFilter> for PowerChangeRequest {
    type Error = anyhow::Error;

    fn try_from(
        value: lib_power_change_log::NewPowerChangeRequestFilter,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            configuration_number: value.configuration_number,
            change: PowerChange {
                op: PowerOperation::try_from(value.op)?,
                payload: value.payload.to_vec(),
                validator: ethers_address_to_fil_address(&value.validator)?,
            },
        })
    }
}

/// The staking validator information
#[derive(Clone, Debug)]
pub struct ValidatorStakingInfo {
    current_power: TokenAmount,
    next_power: TokenAmount,
    metadata: Vec<u8>,
}

impl Display for ValidatorStakingInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ValidatorStaking(current_power: {}, next_power: {}, metadata: 0x{})",
            self.current_power,
            self.next_power,
            hex::encode(&self.metadata)
        )
    }
}

impl TryFrom<subnet_actor_getter_facet::ValidatorInfo> for ValidatorStakingInfo {
    type Error = anyhow::Error;

    fn try_from(value: subnet_actor_getter_facet::ValidatorInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            current_power: eth_to_fil_amount(&value.current_power)?,
            next_power: eth_to_fil_amount(&value.next_power)?,
            metadata: value.metadata.to_vec(),
        })
    }
}

/// The full validator information with
#[derive(Clone, Debug)]
pub struct ValidatorInfo {
    pub staking: ValidatorStakingInfo,
    /// If the validator is active in block production
    pub is_active: bool,
    /// If the validator is current waiting to be promoted to active
    pub is_waiting: bool,
}

impl Display for ValidatorInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ValidatorInfo(staking: {}, is_active: {}, is_waiting: {})",
            self.staking, self.is_active, self.is_waiting
        )
    }
}
