// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

//! Staking module related types and functions

use crate::{eth_to_fil_amount, ethers_address_to_fil_address};
use ethers::contract::EthEvent;
use ethers::types::U256;
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;

pub type ConfigurationNumber = u64;
pub type StakingChangeRequest = (ConfigurationNumber, StakingChange);

#[derive(Clone, Debug)]
pub enum StakingOperation {
    Deposit,
    Withdraw,
}

impl From<u8> for StakingOperation {
    fn from(value: u8) -> Self {
        if value == 0 {
            Self::Deposit
        } else {
            Self::Withdraw
        }
    }
}

/// The change request to validator staking
#[derive(Clone, Debug)]
pub struct StakingChange {
    pub op: StakingOperation,
    pub amount: TokenAmount,
    pub validator: Address,
}

/// The event emitted when a staking request is perform in solidity contracts
#[derive(Clone, Debug, EthEvent)]
pub struct NewStakingRequest {
    op: u8,
    validator: ethers::types::Address,
    amount: U256,
    configuration_number: u64,
}

impl TryFrom<NewStakingRequest> for StakingChangeRequest {
    type Error = anyhow::Error;

    fn try_from(value: NewStakingRequest) -> Result<Self, Self::Error> {
        Ok((
            value.configuration_number,
            StakingChange {
                op: StakingOperation::from(value.op),
                amount: eth_to_fil_amount(&value.amount)?,
                validator: ethers_address_to_fil_address(&value.validator)?,
            },
        ))
    }
}
