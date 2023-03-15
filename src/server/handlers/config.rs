// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Triggers a config reloading

use crate::config::ReloadableConfig;
use crate::server::JsonRPCRequestHandler;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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
