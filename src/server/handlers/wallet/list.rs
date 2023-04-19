use std::collections::HashMap;
// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::manager::SubnetManager;
use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use futures_util::future::join_all;
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletListParams {
    pub subnet: String,
}

/// Key is the address as string and value is the token amount as string
pub type WalletListResponse = HashMap<String, String>;

/// Send value between two addresses within a subnet
pub(crate) struct WalletListHandler {
    pool: Arc<SubnetManagerPool>,
}

impl WalletListHandler {
    pub(crate) fn new(pool: Arc<SubnetManagerPool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for WalletListHandler {
    type Request = WalletListParams;
    type Response = WalletListResponse;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let subnet = SubnetID::from_str(&request.subnet)?;
        let conn = match self.pool.get(&subnet) {
            None => return Err(anyhow!("target subnet not found")),
            Some(conn) => conn,
        };

        let manager = conn.manager();
        let addresses = manager.wallet_list().await?;

        let r = addresses
            .iter()
            .map(|addr| async move {
                manager
                    .wallet_balance(addr)
                    .await
                    .map(|balance| (balance, addr))
            })
            .collect::<Vec<_>>();

        let mut hashmap = HashMap::new();
        let r = join_all(r)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<(TokenAmount, &Address)>>>()?;
        for (balance, addr) in r {
            hashmap.insert(addr.to_string(), balance.to_string());
        }
        Ok(hashmap)
    }
}
