// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use fvm_shared::clock::ChainEpoch;
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

/// We need to redefine the struct here due to:
/// In the actor, it is `Deserialize_tuple`, but when returned from json rpc endpoints, it's
/// actually `json` struct. The deserialization is not working because the agent is interpreting
/// the tuple as json.
#[derive(Deserialize, Serialize, Debug)]
pub struct ValidatorSet {
    validators: Option<Vec<Validator>>,
    // sequence number that uniquely identifies a validator set
    configuration_number: u64,
}

/// The validator struct. See `ValidatorSet` comment on why we need this duplicated definition.
#[derive(Deserialize, Serialize, Debug)]
pub struct Validator {
    pub addr: String,
    pub net_addr: String,
    pub weight: u64,
}
