// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Create subnet handler and parameters

use crate::server::JsonRPCRequestHandler;
use async_trait::async_trait;
use fvm_shared::clock::ChainEpoch;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSubnetParams {
    pub parent: String,
    pub name: String,
    pub min_validator_stake: u64,
    pub min_validators: u64,
    pub finality_threshold: ChainEpoch,
    pub check_period: ChainEpoch,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSubnetResponse {
    /// The address of the created subnet
    pub address: String,
}

/// The create subnet json rpc method handler.
pub(crate) struct CreateSubnetHandler {}

#[async_trait]
impl JsonRPCRequestHandler for CreateSubnetHandler {
    type Request = CreateSubnetParams;
    type Response = CreateSubnetResponse;

    async fn handle(&self, _request: Self::Request) -> anyhow::Result<Self::Response> {
        Ok(CreateSubnetResponse {
            address: String::from("/root"),
        })
    }
}
