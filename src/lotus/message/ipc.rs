// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use ipc_gateway::Status;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};

use crate::lotus::message::CIDMap;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct IPCGetPrevCheckpointForChildResponse {
    #[serde(rename = "CID")]
    pub cid: CIDMap,
}

/// The state of a gateway actor. The struct omits all fields that are not relevant for the
/// execution of the IPC agent.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct IPCReadGatewayStateResponse {
    pub check_period: ChainEpoch,
}

/// The state of a subnet actor. The struct omits all fields that are not relevant for the
/// execution of the IPC agent.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct IPCReadSubnetActorStateResponse {
    pub check_period: ChainEpoch,
    pub validator_set: ValidatorSet,
}

/// SubnetInfo is an auxiliary struct that collects relevant information about the state of a subnet
#[derive(Debug, Serialize, Deserialize)]
pub struct SubnetInfo {
    /// Id of the subnet.
    pub id: SubnetID,
    /// Collateral staked in the subnet.
    pub stake: TokenAmount,
    /// Circulating supply available in the subnet.
    pub circ_supply: TokenAmount,
    /// State of the Subnet (Initialized, Active, Killed)
    pub status: Status,
}

/// We need to redefine the struct here due to:
/// In the actor, it is `Deserialize_tuple`, but when returned from json rpc endpoints, it's
/// actually `json` struct. The deserialization is not working because the agent is interpreting
/// the tuple as json.
#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorSet {
    pub validators: Option<Vec<Validator>>,
    // sequence number that uniquely identifies a validator set
    pub configuration_number: u64,
}

/// The validator struct. See `ValidatorSet` comment on why we need this duplicated definition.
#[derive(Deserialize, Serialize, Debug)]
pub struct Validator {
    pub addr: String,
    pub net_addr: String,
    pub weight: u64,
}
