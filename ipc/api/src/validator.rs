// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use fvm_shared::{address::Address, econ::TokenAmount};
use ipc_actors_abis::subnet_actor_getter_facet;

use crate::{
    eth_to_fil_amount, ethers_address_to_fil_address,
    evm::{fil_to_eth_amount, payload_to_evm_address},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenesisValidator {
    pub addr: Address,
    pub metadata: Vec<u8>,
    pub collateral: TokenAmount,
    pub federated_power: TokenAmount,
}

impl TryFrom<GenesisValidator> for subnet_actor_getter_facet::GenesisValidator {
    type Error = anyhow::Error;

    fn try_from(value: GenesisValidator) -> Result<Self, Self::Error> {
        Ok(subnet_actor_getter_facet::GenesisValidator {
            addr: payload_to_evm_address(value.addr.payload())?,
            collateral: fil_to_eth_amount(&value.collateral)?,
            federated_power: fil_to_eth_amount(&value.federated_power)?,
            metadata: ethers::core::types::Bytes::from(value.metadata),
        })
    }
}

impl TryFrom<subnet_actor_getter_facet::GenesisValidator> for GenesisValidator {
    type Error = anyhow::Error;

    fn try_from(value: subnet_actor_getter_facet::GenesisValidator) -> Result<Self, Self::Error> {
        Ok(GenesisValidator {
            addr: ethers_address_to_fil_address(&value.addr)?,
            collateral: eth_to_fil_amount(&value.collateral)?,
            federated_power: eth_to_fil_amount(&value.federated_power)?,
            metadata: value.metadata.to_vec(),
        })
    }
}

pub fn vec_try_from<T, W: TryFrom<T>>(values: Vec<T>) -> anyhow::Result<Vec<W>, W::Error> {
    let out = values
        .into_iter()
        .map(W::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(out)
}

