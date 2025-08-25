// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

/// This type definitions are borrowed from
/// https://github.com/consensus-shipyard/ipc-actors/blob/main/subnet-actor/src/types.rs
/// to ensure that they are in sync in this project.
/// However, we should either deprecate the native actors, or make
/// them use the types from this sdk directly.
use crate::subnet_id::SubnetID;
use fvm_ipld_encoding::repr::*;
use fvm_shared::{address::Address, clock::ChainEpoch, econ::TokenAmount};
use serde::{de, Deserialize, Deserializer, Serialize};
use std::str::FromStr;
use strum::{EnumString, VariantNames};

/// ID used in the builtin-actors bundle manifest
pub const MANIFEST_ID: &str = "ipc_subnet_actor";

/// Determines the permission mode for validators.
#[repr(u8)]
#[derive(Copy, Debug, Clone, Serialize_repr, PartialEq, Eq, EnumString, VariantNames)]
#[strum(serialize_all = "snake_case")]
pub enum PermissionMode {
    /// Validator power is determined by the collateral staked
    Collateral,
    /// Validator power is assigned by the owner of the subnet
    Federated,
    /// Validator power is determined by the initial collateral staked and does not change anymore
    Static,
}

impl<'de> Deserialize<'de> for PermissionMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;
        impl de::Visitor<'_> for Visitor {
            type Value = PermissionMode;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a u8 (0–2) or one of {:?}", PermissionMode::VARIANTS)
            }

            fn visit_u64<E>(self, v: u64) -> Result<PermissionMode, E>
            where
                E: de::Error,
            {
                match v {
                    0 => Ok(PermissionMode::Collateral),
                    1 => Ok(PermissionMode::Federated),
                    2 => Ok(PermissionMode::Static),
                    other => Err(E::invalid_value(de::Unexpected::Unsigned(other), &self)),
                }
            }

            fn visit_str<E>(self, s: &str) -> Result<PermissionMode, E>
            where
                E: de::Error,
            {
                PermissionMode::from_str(s)
                    .map_err(|_| E::invalid_value(de::Unexpected::Str(s), &self))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

/// Defines a generic token of a subnet on its parent subnet.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Asset {
    /// The kind of supply.
    pub kind: AssetKind,
    /// The address of the ERC20 token if that supply kind is selected.
    pub token_address: Option<Address>,
}

impl Default for Asset {
    fn default() -> Self {
        Self {
            kind: AssetKind::Native,
            token_address: None,
        }
    }
}

/// Determines the type of a token used by the subnet.
#[repr(u8)]
#[derive(
    Copy, Debug, Clone, Serialize_repr, PartialEq, Eq, strum::EnumString, strum::VariantNames,
)]
#[strum(serialize_all = "snake_case")]
pub enum AssetKind {
    Native,
    ERC20,
}

impl<'de> Deserialize<'de> for AssetKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;
        impl de::Visitor<'_> for Visitor {
            type Value = AssetKind;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "a u8 (0–2) or one of {:?}", AssetKind::VARIANTS)
            }

            fn visit_u64<E>(self, v: u64) -> Result<AssetKind, E>
            where
                E: de::Error,
            {
                match v {
                    0 => Ok(AssetKind::Native),
                    1 => Ok(AssetKind::ERC20),
                    other => Err(E::invalid_value(de::Unexpected::Unsigned(other), &self)),
                }
            }

            fn visit_str<E>(self, s: &str) -> Result<AssetKind, E>
            where
                E: de::Error,
            {
                AssetKind::from_str(s).map_err(|_| E::invalid_value(de::Unexpected::Str(s), &self))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConstructParams {
    pub parent: SubnetID,
    pub ipc_gateway_addr: Address,
    pub consensus: ConsensusType,
    pub min_validator_stake: TokenAmount,
    pub min_validators: u64,
    pub bottomup_check_period: ChainEpoch,
    pub active_validators_limit: u16,
    pub min_cross_msg_fee: TokenAmount,
    pub permission_mode: PermissionMode,
    pub supply_source: Asset,
    pub collateral_source: Asset,
    pub validator_gater: Address,
    pub validator_rewarder: Address,
    pub genesis_subnet_ipc_contracts_owner: ethers::types::Address,
    pub chain_id: u64,
}

/// Consensus types supported by hierarchical consensus
#[derive(PartialEq, Eq, Clone, Copy, Debug, Deserialize_repr, Serialize_repr)]
#[repr(u64)]
pub enum ConsensusType {
    Fendermint,
}
