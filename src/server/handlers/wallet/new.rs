// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! wallet handlers and parameters

use crate::lotus::message::wallet::WalletKeyType;
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use fvm_shared::crypto::signature::SignatureType;
use ipc_identity::Wallet;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::{Arc, RwLock};

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletNewParams {
    pub key_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletNewResponse {
    pub address: String,
}

/// Send value between two addresses within a subnet
pub(crate) struct WalletNewHandler {
    wallet: Arc<RwLock<Wallet>>,
}

impl WalletNewHandler {
    pub(crate) fn new(wallet: Arc<RwLock<Wallet>>) -> Self {
        Self { wallet }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for WalletNewHandler {
    type Request = WalletNewParams;
    type Response = WalletNewResponse;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let tp = match WalletKeyType::from_str(&request.key_type)? {
            WalletKeyType::BLS => SignatureType::BLS,
            WalletKeyType::Secp256k1 => SignatureType::Secp256k1,
            WalletKeyType::Secp256k1Ledger => return Err(anyhow!("ledger key type not supported")),
        };
        let mut wallet = self.wallet.write().unwrap();
        let address = wallet.generate_addr(tp)?;

        Ok(WalletNewResponse {
            address: address.to_string(),
        })
    }
}
