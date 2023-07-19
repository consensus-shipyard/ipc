// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::config::subnet::SubnetConfig;
use crate::manager::evm::ethers_address_to_fil_address;
use crate::manager::SubnetManager;
use crate::server::handlers::manager::subnet::SubnetManagerPool;
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use futures_util::future::join_all;
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use ipc_identity::EvmKeyStore;
use ipc_identity::{PersistentKeyStore, Wallet};
use ipc_sdk::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletBalancesParams {
    pub subnet: String,
}

/// Key is the address as string and value is the token amount as string
pub type WalletBalancesResponse = HashMap<String, String>;

/// Send value between two addresses within a subnet
pub(crate) struct WalletBalancesHandler {
    pool: Arc<SubnetManagerPool>,
    fvm_wallet: Arc<RwLock<Wallet>>,
    evm_keystore: Arc<RwLock<PersistentKeyStore<ethers::types::Address>>>,
}

impl WalletBalancesHandler {
    pub(crate) fn new(
        pool: Arc<SubnetManagerPool>,
        fvm_wallet: Arc<RwLock<Wallet>>,
        evm_keystore: Arc<RwLock<PersistentKeyStore<ethers::types::Address>>>,
    ) -> Self {
        Self {
            pool,
            fvm_wallet,
            evm_keystore,
        }
    }
}

impl WalletBalancesHandler {
    async fn fvm_balances(
        &self,
        manager: &dyn SubnetManager,
    ) -> anyhow::Result<WalletBalancesResponse> {
        let wallet = Arc::clone(&self.fvm_wallet);
        let addresses = wallet.read().unwrap().list_addrs()?;

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

    async fn fevm_balances(
        &self,
        manager: &dyn SubnetManager,
    ) -> anyhow::Result<WalletBalancesResponse> {
        let keystore = Arc::clone(&self.evm_keystore);
        let addresses = keystore.read().unwrap().list()?;

        let r = addresses
            .iter()
            .map(|addr| async move {
                manager
                    .wallet_balance(&ethers_address_to_fil_address(addr)?)
                    .await
                    .map(|balance| (balance, addr))
            })
            .collect::<Vec<_>>();

        let mut hashmap = HashMap::new();
        let r = join_all(r)
            .await
            .into_iter()
            .collect::<anyhow::Result<Vec<(TokenAmount, &ethers::types::H160)>>>()?;
        for (balance, addr) in r {
            hashmap.insert(format!("{addr:?}"), balance.to_string());
        }
        Ok(hashmap)
    }
}

#[async_trait]
impl JsonRPCRequestHandler for WalletBalancesHandler {
    type Request = WalletBalancesParams;
    type Response = WalletBalancesResponse;

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        let subnet = SubnetID::from_str(&request.subnet)?;
        let conn = match self.pool.get(&subnet) {
            None => return Err(anyhow!("target subnet not found")),
            Some(conn) => conn,
        };
        let manager = conn.manager();

        match conn.subnet().config {
            SubnetConfig::Fvm(_) => self.fvm_balances(manager).await,
            SubnetConfig::Fevm(_) => self.fevm_balances(manager).await,
        }
    }
}
