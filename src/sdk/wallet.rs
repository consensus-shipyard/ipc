// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Wallet related API calls

use crate::config::json_rpc_methods;
use crate::jsonrpc::JsonRpcClient;
use crate::lotus::message::wallet::WalletKeyType;
use crate::sdk::IpcAgentClient;
use crate::server::wallet::import::{WalletImportParams, WalletImportResponse};
use fvm_shared::crypto::signature::SignatureType;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use zeroize::Zeroize;

impl<T: JsonRpcClient> IpcAgentClient<T> {
    /// Import a wallet address in the form of lotus json
    pub async fn import_lotus_json(&self, key_type: LotusJsonKeyType) -> anyhow::Result<String> {
        let params = WalletImportParams {
            key_type: SignatureType::try_from(WalletKeyType::from_str(&key_type.r#type)?)? as u8,
            private_key: key_type.private_key.clone(),
        };
        self.import(params).await
    }

    pub async fn import(&self, params: WalletImportParams) -> anyhow::Result<String> {
        Ok(self
            .json_rpc_client
            .request::<WalletImportResponse>(
                json_rpc_methods::WALLET_IMPORT,
                serde_json::to_value(params)?,
            )
            .await?
            .address)
    }
}

/// Lotus JSON keytype format
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct LotusJsonKeyType {
    pub r#type: String,
    pub private_key: String,
}

impl FromStr for LotusJsonKeyType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = serde_json::from_str(s)?;
        Ok(v)
    }
}

impl Drop for LotusJsonKeyType {
    fn drop(&mut self) {
        self.private_key.zeroize();
    }
}
