// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! wallet handlers and parameters

use crate::server::JsonRPCRequestHandler;
use async_trait::async_trait;
use base64::Engine;
use fvm_shared::crypto::signature::SignatureType;
use ipc_identity::json::KeyInfoJson;
use ipc_identity::{KeyInfo, Wallet};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletImportParams {
    pub key_type: u8,
    /// Base64 encoded private key string
    pub private_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletImportResponse {
    pub address: String,
}

/// Send value between two addresses within a subnet
pub(crate) struct WalletImportHandler {
    wallet: Arc<RwLock<Wallet>>,
}

impl WalletImportHandler {
    pub(crate) fn new(wallet: Arc<RwLock<Wallet>>) -> Self {
        Self { wallet }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for WalletImportHandler {
    type Request = WalletImportParams;
    type Response = WalletImportResponse;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let mut wallet = self.wallet.write().unwrap();

        let key_type = if request.key_type == SignatureType::BLS as u8 {
            SignatureType::BLS
        } else {
            SignatureType::Secp256k1
        };

        let key_info = KeyInfoJson(KeyInfo::new(
            key_type,
            base64::engine::general_purpose::STANDARD.decode(request.private_key)?,
        ));
        let key_info = KeyInfo::try_from(key_info)?;
        let address = wallet.import(key_info)?;

        Ok(WalletImportResponse {
            address: address.to_string(),
        })
    }
}
