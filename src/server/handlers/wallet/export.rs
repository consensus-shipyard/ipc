// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! wallet handlers and parameters

use crate::server::JsonRPCRequestHandler;
use async_trait::async_trait;
use fvm_shared::address::Address;
use ipc_identity::json::KeyInfoJson;
use ipc_identity::Wallet;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::{Arc, RwLock};

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletExportParams {
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletExportResponse {
    pub key_info: KeyInfoJson,
}

/// Send value between two addresses within a subnet
pub(crate) struct WalletExportHandler {
    wallet: Arc<RwLock<Wallet>>,
}

impl WalletExportHandler {
    pub(crate) fn new(wallet: Arc<RwLock<Wallet>>) -> Self {
        Self { wallet }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for WalletExportHandler {
    type Request = WalletExportParams;
    type Response = WalletExportResponse;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let mut wallet = self.wallet.write().unwrap();
        let address = Address::from_str(&request.address)?;
        let key_info = wallet.export(&address)?;

        Ok(WalletExportResponse {
            key_info: KeyInfoJson(key_info),
        })
    }
}
