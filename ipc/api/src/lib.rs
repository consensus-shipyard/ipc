// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
use fvm_shared::{address::Address, econ::TokenAmount};
use ipc_types::EthAddress;
use serde::de::Error as SerdeError;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

pub mod address;
pub mod checkpoint;
pub mod cross;
pub mod error;
pub mod gateway;
#[cfg(feature = "fil-actor")]
mod runtime;
pub mod subnet;
pub mod subnet_id;
pub mod validator;

pub mod evm;
pub mod staking;

/// Converts an ethers::U256 TokenAmount into a FIL amount.
pub fn eth_to_fil_amount(amount: &ethers::types::U256) -> anyhow::Result<TokenAmount> {
    let v = fvm_shared::bigint::BigInt::from_str(&amount.to_string())?;
    Ok(TokenAmount::from_atto(v))
}

pub fn ethers_address_to_fil_address(addr: &ethers::types::Address) -> anyhow::Result<Address> {
    let raw_addr = format!("{addr:?}");
    log::debug!("raw evm subnet addr: {raw_addr:}");

    let eth_addr = EthAddress::from_str(&raw_addr)?;
    Ok(Address::from(eth_addr))
}

/// Marker type for human readable form.
pub struct HumanReadable;

impl<T: ToString> serde_with::SerializeAs<T> for HumanReadable {
    fn serialize_as<S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        value.to_string().serialize(serializer)
    }
}

impl<'de, T: FromStr> serde_with::DeserializeAs<'de, T> for HumanReadable {
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        T::from_str(&s).map_err(|_| {
            D::Error::custom(format!(
                "cannot parse from str {}",
                core::any::type_name::<T>()
            ))
        })
    }
}
