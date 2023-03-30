// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Expose the subnet actor validator set

use crate::config::ReloadableConfig;
use crate::lotus::client::LotusJsonRPCClient;
use crate::lotus::message::ipc::ValidatorSet;
use crate::lotus::LotusClient;
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use cid::Cid;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryValidatorSetParams {
    pub subnet: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryValidatorSetResponse {
    /// The validator set for the subnet fetched from the parent.
    pub validator_set: ValidatorSet,
    /// Minimum number of validators required by the subnet
    pub min_validators: u64,
}

/// The create subnet json rpc method handler.
pub(crate) struct QueryValidatorSetHandler {
    config: Arc<ReloadableConfig>,
}

impl QueryValidatorSetHandler {
    pub(crate) fn new(config: Arc<ReloadableConfig>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for QueryValidatorSetHandler {
    type Request = QueryValidatorSetParams;
    type Response = QueryValidatorSetResponse;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let subnet_id = SubnetID::from_str(&request.subnet)?;
        let parent = subnet_id
            .parent()
            .ok_or_else(|| anyhow!("cannot get for root"))?;

        let config = self.config.get_config();
        let subnet = match config.subnets.get(&parent) {
            None => return Err(anyhow!("target parent subnet not found")),
            Some(s) => s,
        };

        let lotus = LotusJsonRPCClient::from_subnet(subnet);

        // Read the parent's chain head and obtain the tip set CID.
        // FIXME: This is used all over the place, make it a more
        // compact function
        let parent_head = lotus.chain_head().await?;
        let cid_map = parent_head.cids.first().unwrap().clone();
        let tip_set = Cid::try_from(cid_map)?;

        let response = lotus
            .ipc_read_subnet_actor_state(&subnet_id, tip_set)
            .await?;

        Ok(QueryValidatorSetResponse {
            validator_set: response.validator_set,
            min_validators: response.min_validators,
        })
    }
}
