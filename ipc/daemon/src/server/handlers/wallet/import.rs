// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! wallet handlers and parameters

use crate::server::JsonRPCRequestHandler;
use async_trait::async_trait;
use base64::Engine;
use fvm_shared::crypto::signature::SignatureType;
use ipc_identity::json::KeyInfoJson;
use ipc_identity::{EvmKeyInfo, EvmKeyStore, KeyInfo, PersistentKeyStore, Wallet};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use zeroize::Zeroize;

#[derive(Deserialize, Serialize, Debug)]
#[serde(tag = "network_type")]
pub enum WalletImportParams {
    #[serde(rename = "fvm")]
    Fvm(FvmImportParams),
    #[serde(rename = "evm")]
    Evm(EvmImportParams),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FvmImportParams {
    pub key_type: u8,
    /// Base64 encoded private key string
    pub private_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EvmImportParams {
    /// Hex encoded private key string
    pub private_key: String,
}

impl Drop for WalletImportParams {
    fn drop(&mut self) {
        match self {
            WalletImportParams::Fvm(p) => {
                p.private_key.zeroize();
            }
            WalletImportParams::Evm(p) => {
                p.private_key.zeroize();
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletImportResponse {
    pub address: String,
}

/// Send value between two addresses within a subnet
pub(crate) struct WalletImportHandler {
    fvm_wallet: Arc<RwLock<Wallet>>,
    evm_keystore: Arc<RwLock<PersistentKeyStore<ethers::types::Address>>>,
}

impl WalletImportHandler {
    pub(crate) fn new(
        fvm_wallet: Arc<RwLock<Wallet>>,
        evm_keystore: Arc<RwLock<PersistentKeyStore<ethers::types::Address>>>,
    ) -> Self {
        Self {
            fvm_wallet,
            evm_keystore,
        }
    }

    fn import_fvm(&self, request: &FvmImportParams) -> anyhow::Result<WalletImportResponse> {
        let mut wallet = self.fvm_wallet.write().unwrap();

        let key_type = if request.key_type == SignatureType::BLS as u8 {
            SignatureType::BLS
        } else {
            SignatureType::Secp256k1
        };

        let key_info = KeyInfoJson(KeyInfo::new(
            key_type,
            base64::engine::general_purpose::STANDARD.decode(&request.private_key)?,
        ));
        let key_info = KeyInfo::try_from(key_info)?;
        let address = wallet.import(key_info)?;

        Ok(WalletImportResponse {
            address: address.to_string(),
        })
    }

    fn import_evm(&self, request: &EvmImportParams) -> anyhow::Result<WalletImportResponse> {
        let mut keystore = self.evm_keystore.write().unwrap();

        let private_key = if !request.private_key.starts_with("0x") {
            hex::decode(&request.private_key)?
        } else {
            hex::decode(&request.private_key.as_str()[2..])?
        };
        let addr = keystore.put(EvmKeyInfo::new(private_key))?;

        Ok(WalletImportResponse {
            address: format!("{:}", addr),
        })
    }
}

#[async_trait]
impl JsonRPCRequestHandler for WalletImportHandler {
    type Request = WalletImportParams;
    type Response = WalletImportResponse;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        match &request {
            WalletImportParams::Fvm(p) => self.import_fvm(p),
            WalletImportParams::Evm(p) => self.import_evm(p),
        }
    }
}
