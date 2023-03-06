// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Triggers a config reloading

use crate::config::ReloadableConfig;
use crate::server::JsonRPCRequestHandler;
use async_trait::async_trait;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct ReloadConfigParams {
    pub path: Option<String>,
}

/// The create subnet json rpc method handler.
pub(crate) struct ReloadConfigHandler {
    config: Arc<ReloadableConfig>,
    default_config_path: String,
}

impl ReloadConfigHandler {
    pub fn new(config: Arc<ReloadableConfig>, default_config_path: String) -> Self {
        Self {
            config,
            default_config_path,
        }
    }
}

#[async_trait]
impl JsonRPCRequestHandler for ReloadConfigHandler {
    type Request = ReloadConfigParams;
    type Response = ();

    async fn handle(&self, request: Self::Request) -> anyhow::Result<Self::Response> {
        log::info!("received request to reload config: {request:?}");

        let path = request
            .path
            .unwrap_or_else(|| self.default_config_path.clone());
        self.config.reload(path).await
    }
}
