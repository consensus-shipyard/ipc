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
use serde::{Deserialize, Serialize};

/// ID used in the builtin-actors bundle manifest
pub const MANIFEST_ID: &str = "ipc_subnet_actor";

const DEFAULT_ACTIVE_VALIDATORS: u16 = 100;
const DEFAULT_POWER_SCALE: i8 = 3;
/// The majority vote percentage for checkpoint submission when creating a subnet.
const SUBNET_MAJORITY_PERCENTAGE: u8 = 67;

/// Determines the permission mode for validators.
#[repr(u8)]
#[derive(
    Copy,
    Debug,
    Clone,
    Serialize_repr,
    Deserialize_repr,
    PartialEq,
    Eq,
    strum::EnumString,
    strum::VariantNames,
)]
#[strum(serialize_all = "snake_case")]
pub enum PermissionMode {
    /// Validator power is determined by the collateral staked
    Collateral,
    /// Validator power is assigned by the owner of the subnet
    Federated,
    /// Validator power is determined by the initial collateral staked and does not change anymore
    Static,
}

/// Defines the supply source of a subnet on its parent subnet.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupplySource {
    /// The kind of supply.
    pub kind: SupplyKind,
    /// The address of the ERC20 token if that supply kind is selected.
    pub token_address: Option<Address>,
}

/// Determines the type of supply used by the subnet.
#[repr(u8)]
#[derive(
    Copy,
    Debug,
    Clone,
    Serialize_repr,
    Deserialize_repr,
    PartialEq,
    Eq,
    strum::EnumString,
    strum::VariantNames,
)]
#[strum(serialize_all = "snake_case")]
pub enum SupplyKind {
    Native,
    ERC20,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConstructParams {
    /// The gateway address. If not specified, it will be injected
    /// with the default value based on each subnet configuration
    pub ipc_gateway_addr: Option<Address>,

    pub power_scale: i8,
    pub consensus: ConsensusType,
    pub majority_percentage: u8,
    pub active_validators_limit: u16,
    pub parent: SubnetID,
    pub min_validator_stake: TokenAmount,
    pub min_validators: u64,
    pub bottomup_check_period: ChainEpoch,
    pub min_cross_msg_fee: TokenAmount,
    pub permission_mode: PermissionMode,
    pub supply_source: SupplySource,
}

impl ConstructParams {
    pub fn new(
        parent: SubnetID,
        min_validators: u64,
        min_validator_stake: TokenAmount,
        bottomup_check_period: ChainEpoch,
        min_cross_msg_fee: TokenAmount,
        permission_mode: PermissionMode,
        supply_source: SupplySource,
    ) -> Self {
        Self {
            ipc_gateway_addr: None,

            power_scale: DEFAULT_POWER_SCALE,
            consensus: ConsensusType::Fendermint,
            majority_percentage: SUBNET_MAJORITY_PERCENTAGE,
            active_validators_limit: DEFAULT_ACTIVE_VALIDATORS,

            parent,
            min_validator_stake,
            min_validators,
            bottomup_check_period,
            min_cross_msg_fee,
            permission_mode,
            supply_source,
        }
    }
}

/// Consensus types supported by hierarchical consensus
#[derive(PartialEq, Eq, Clone, Copy, Debug, Deserialize_repr, Serialize_repr)]
#[repr(u64)]
pub enum ConsensusType {
    Fendermint,
}
