// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! wallet handlers and parameters

use crate::lotus::message::wallet::WalletKeyType;
use crate::manager::SubnetManager;
use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletNewParams {
    pub subnet: String,
    pub key_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletNewResponse {
    pub address: String,
}

/// Send value between two addresses within a subnet
pub(crate) struct WalletNewHandler {
    pool: Arc<SubnetManagerPool>,
}

impl WalletNewHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for WalletNewHandler {
    type Request = WalletNewParams;
    type Response = WalletNewResponse;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let conn = match self.pool.get(&request.subnet) {
            None => return Err(anyhow!("target subnet not found")),
            Some(conn) => conn,
        };

        let address = conn
            .manager()
            .wallet_new(WalletKeyType::from_str(&request.key_type)?)
            .await?;
        Ok(WalletNewResponse {
            address: address.to_string(),
        })
    }
}
