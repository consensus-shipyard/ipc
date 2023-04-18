// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Create subnet handler and parameters

use crate::manager::SubnetManager;
use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::handlers::manager::{check_subnet, parse_from};
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::{ConsensusType, ConstructParams};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSubnetParams {
    pub from: Option<String>,
    pub parent: String,
    pub name: String,
    pub min_validator_stake: u64,
    pub min_validators: u64,
    pub bottomup_check_period: ChainEpoch,
    pub topdown_check_period: ChainEpoch,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSubnetResponse {
    /// The address of the created subnet
    pub address: String,
}

/// The create subnet json rpc method handler.
pub(crate) struct CreateSubnetHandler {
    pool: Arc<SubnetManagerPool>,
}

impl CreateSubnetHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for CreateSubnetHandler {
    type Request = CreateSubnetParams;
    type Response = CreateSubnetResponse;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let parent = SubnetID::from_str(&request.parent)?;
        let conn = match self.pool.get(&parent) {
            None => return Err(anyhow!("target parent subnet not found")),
            Some(conn) => conn,
        };

        let subnet_config = conn.subnet();
        check_subnet(subnet_config)?;

        let constructor_params = ConstructParams {
            parent,
            name: request.name,
            ipc_gateway_addr: subnet_config.gateway_addr.id()?,
            consensus: ConsensusType::Mir,
            min_validator_stake: TokenAmount::from_whole(request.min_validator_stake), // In FIL
            min_validators: request.min_validators,
            bottomup_check_period: request.bottomup_check_period,
            topdown_check_period: request.topdown_check_period,
            genesis: vec![],
        };

        let from = parse_from(subnet_config, request.from)?;

        let created_subnet_addr = conn
            .manager()
            .create_subnet(from, constructor_params)
            .await?;

        Ok(CreateSubnetResponse {
            address: created_subnet_addr.to_string(),
        })
    }
}
