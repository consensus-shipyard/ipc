//! Expose the subnet actor validator set

use crate::config::ReloadableConfig;
use crate::lotus::client::LotusJsonRPCClient;
use crate::lotus::LotusClient;
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use cid::Cid;
use ipc_subnet_actor::Validator;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryValidatorSetParams {
    pub subnet: String,
    pub tip_set: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryValidatorSetResponse {
    /// The address of the created subnet
    pub validator_set: Vec<Validator>,
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
        let tip_set = Cid::from_str(&request.tip_set)?;

        let config = self.config.get_config();
        let subnet = match config.subnets.get(&request.subnet) {
            None => return Err(anyhow!("target parent subnet not found")),
            Some(s) => s,
        };

        let lotus = LotusJsonRPCClient::from_subnet(subnet);
        let response = lotus.ipc_read_subnet_actor_state(tip_set).await?;

        Ok(QueryValidatorSetResponse {
            validator_set: response.validator_set,
        })
    }
}
