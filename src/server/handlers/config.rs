// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Triggers a config reloading

use crate::config::ReloadableConfig;
use crate::server::JsonRPCRequestHandler;
use anyhow::anyhow;
use async_trait::async_trait;
use ipc_identity::{KeyStore, KeyStoreConfig, KEYSTORE_NAME};
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::Arc};

#[derive(Debug, Deserialize, Serialize)]
pub struct ReloadConfigParams {
    pub path: Option<String>,
}

/// The create subnet json rpc method handler.
pub(crate) struct ReloadConfigHandler {
    config: Arc<ReloadableConfig>,
}

impl ReloadConfigHandler {
    pub fn new(config: Arc<ReloadableConfig>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for ReloadConfigHandler {
    type Request = ReloadConfigParams;
    type Response = ();

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        log::info!("received request to reload config: {request:?}");

        if request.path.is_some() {
            self.config.set_path(request.path.unwrap());
        }
        self.config.reload().await
    }
}

pub(crate) fn new_keystore_from_config(config: Arc<ReloadableConfig>) -> anyhow::Result<KeyStore> {
    let repo_str = config.get_config_repo();
    if let Some(repo_str) = repo_str {
        new_keystore_from_path(&repo_str)
    } else {
        Err(anyhow!("No keystore repo found in config"))
    }
}

pub fn new_keystore_from_path(repo_str: &str) -> anyhow::Result<KeyStore> {
    let repo = Path::new(&repo_str);
    let keystore_config = KeyStoreConfig::Persistent(repo.join(KEYSTORE_NAME));
    // TODO: we currently only support persistent keystore in the default repo directory.
    KeyStore::new(keystore_config).map_err(|e| anyhow!("Failed to create keystore: {}", e))
}
