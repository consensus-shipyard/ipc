// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use fvm_shared::clock::ChainEpoch;
use ipc_subnet_actor::Validator;
use serde::Deserialize;

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
    pub validator_set: Vec<Validator>,
}
