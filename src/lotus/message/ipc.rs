// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use ipc_gateway::Status;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};

use crate::lotus::message::deserialize::{
    deserialize_subnet_id_from_map, deserialize_token_amount_from_str,
};
use crate::lotus::message::serialize::{
    serialize_subnet_id_to_str, serialize_token_amount_to_atto,
};
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
///
/// Note that the serialization and deserialization casing are different. Reason because for deserialization,
/// it is from the fvm actor, which is `PascalCase`. When serialize, we are using rust's default casing
#[derive(Debug, Serialize, Deserialize)]
pub struct SubnetInfo {
    /// Id of the subnet.
    #[serde(rename(deserialize = "ID"))]
    #[serde(deserialize_with = "deserialize_subnet_id_from_map")]
    #[serde(serialize_with = "serialize_subnet_id_to_str")]
    pub id: SubnetID,
    /// Collateral staked in the subnet.
    #[serde(rename(deserialize = "Stake"))]
    #[serde(deserialize_with = "deserialize_token_amount_from_str")]
    #[serde(serialize_with = "serialize_token_amount_to_atto")]
    pub stake: TokenAmount,
    /// Circulating supply available in the subnet.
    #[serde(rename(deserialize = "CircSupply"))]
    #[serde(deserialize_with = "deserialize_token_amount_from_str")]
    #[serde(serialize_with = "serialize_token_amount_to_atto")]
    pub circ_supply: TokenAmount,
    /// State of the Subnet (Initialized, Active, Killed)
    #[serde(rename(deserialize = "Status"))]
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
    pub weight: String,
}
