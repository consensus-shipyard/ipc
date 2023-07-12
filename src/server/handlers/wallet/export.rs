// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! wallet handlers and parameters

use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::address::Address;
use ipc_identity::json::KeyInfoJson;
use ipc_identity::{EvmKeyStore, PersistentKeyStore, Wallet};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::{Arc, RwLock};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "network_type")]
pub enum WalletExportParams {
    #[serde(rename = "fvm")]
    Fvm { address: String },
    #[serde(rename = "evm")]
    Evm { address: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FvmExportParams {
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvmExportParams {
    pub address: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "network_type")]
pub enum WalletExportResponse {
    #[serde(rename = "fvm")]
    Fvm(KeyInfoJson),
    #[serde(rename = "evm")]
    Evm { private_key: String },
}

/// Send value between two addresses within a subnet
pub(crate) struct WalletExportHandler {
    fvm_wallet: Arc<RwLock<Wallet>>,
    evm_keystore: Arc<RwLock<PersistentKeyStore<ethers::types::Address>>>,
}

impl WalletExportHandler {
    pub(crate) fn new(
        fvm_wallet: Arc<RwLock<Wallet>>,
        evm_keystore: Arc<RwLock<PersistentKeyStore<ethers::types::Address>>>,
    ) -> Self {
        Self {
            fvm_wallet,
            evm_keystore,
        }
    }

    fn export_fvm(&self, address: String) -> anyhow::Result<WalletExportResponse> {
        let mut wallet = self.fvm_wallet.write().unwrap();
        let address = Address::from_str(&address)?;
        let key_info = wallet.export(&address)?;

        Ok(WalletExportResponse::Fvm(KeyInfoJson(key_info)))
    }

    fn export_evm(&self, address: String) -> anyhow::Result<WalletExportResponse> {
        let keystore = self.evm_keystore.read().unwrap();
        let address = ethers::types::Address::from_str(&address)?;

        let key_info = keystore
            .get(&address)?
            .ok_or_else(|| anyhow!("key does not exists"))?;
        Ok(WalletExportResponse::Evm {
            private_key: hex::encode(key_info.private_key()),
        })
    }
}

#[async_trait]
impl JsonRPCRequestHandler for WalletExportHandler {
    type Request = WalletExportParams;
    type Response = WalletExportResponse;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        match request {
            WalletExportParams::Fvm { address } => self.export_fvm(address),
            WalletExportParams::Evm { address } => self.export_evm(address),
        }
    }
}
