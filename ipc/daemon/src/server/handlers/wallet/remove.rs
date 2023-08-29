// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! wallet handlers and parameters

use crate::server::JsonRPCRequestHandler;
use async_trait::async_trait;
use ethers::types::Address;
use ipc_identity::{EvmKeyStore, PersistentKeyStore, Wallet};
use serde::{Deserialize, Serialize};
use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

use super::WalletType;

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletRemoveParams {
    pub wallet_type: WalletType,
    pub address: String,
}

/// Send value between two addresses within a subnet
pub(crate) struct WalletRemoveHandler {
    fvm_wallet: Arc<RwLock<Wallet>>,
    evm_keystore: Arc<RwLock<PersistentKeyStore<Address>>>,
}

impl WalletRemoveHandler {
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

impl WalletRemoveHandler {
    fn rm_fvm(&self, request: WalletRemoveParams) -> anyhow::Result<()> {
        let mut wallet = self.fvm_wallet.write().unwrap();
        let address = fvm_shared::address::Address::from_str(&request.address)?;
        wallet.remove(&address)?;
        Ok(())
    }

    fn rm_evm(&self, request: WalletRemoveParams) -> anyhow::Result<()> {
        let address = ethers::types::Address::from_str(&request.address)?;
        let mut keystore = self.evm_keystore.write().unwrap();
        keystore.remove(&address)
    }
}

#[async_trait]
impl JsonRPCRequestHandler for WalletRemoveHandler {
    type Request = WalletRemoveParams;
    type Response = ();

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        match request.wallet_type {
            WalletType::Fvm => self.rm_fvm(request),
            WalletType::Evm => self.rm_evm(request),
        }
    }
}
