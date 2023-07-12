// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! wallet handlers and parameters

use crate::lotus::message::wallet::WalletKeyType;
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use ethers::types::Address;
use fvm_shared::crypto::signature::SignatureType;
use ipc_identity::{random_key_info, EvmKeyStore, PersistentKeyStore, Wallet};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::{Arc, RwLock};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "network_type")]
pub enum WalletNewParams {
    #[serde(rename = "fvm")]
    Fvm(NewFvmWallet),
    #[serde(rename = "evm")]
    Evm,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewFvmWallet {
    pub key_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletNewResponse {
    pub address: String,
}

/// Send value between two addresses within a subnet
pub(crate) struct WalletNewHandler {
    fvm_wallet: Arc<RwLock<Wallet>>,
    evm_keystore: Arc<RwLock<PersistentKeyStore<Address>>>,
}

impl WalletNewHandler {
    pub(crate) fn new(
        fvm_wallet: Arc<RwLock<Wallet>>,
        evm_keystore: Arc<RwLock<PersistentKeyStore<Address>>>,
    ) -> Self {
        Self {
            fvm_wallet,
            evm_keystore,
        }
    }
}

impl WalletNewHandler {
    fn new_fvm(&self, request: NewFvmWallet) -> anyhow::Result<WalletNewResponse> {
        let tp = match WalletKeyType::from_str(&request.key_type)? {
            WalletKeyType::BLS => SignatureType::BLS,
            WalletKeyType::Secp256k1 => SignatureType::Secp256k1,
            WalletKeyType::Secp256k1Ledger => return Err(anyhow!("ledger key type not supported")),
        };
        let mut wallet = self.fvm_wallet.write().unwrap();
        let address = wallet.generate_addr(tp)?;

        Ok(WalletNewResponse {
            address: address.to_string(),
        })
    }

    fn new_evm(&self) -> anyhow::Result<WalletNewResponse> {
        let key_info = random_key_info();

        let mut keystore = self.evm_keystore.write().unwrap();
        let addr = keystore.put(key_info)?;

        Ok(WalletNewResponse {
            address: format!("{:?}", addr),
        })
    }
}
#[async_trait]
impl JsonRPCRequestHandler for WalletNewHandler {
    type Request = WalletNewParams;
    type Response = WalletNewResponse;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        match request {
            WalletNewParams::Fvm(p) => self.new_fvm(p),
            WalletNewParams::Evm => self.new_evm(),
        }
    }
}
