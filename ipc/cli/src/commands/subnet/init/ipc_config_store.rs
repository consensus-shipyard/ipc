// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use anyhow::Context;
use fvm_shared::address::Address;
use ipc_provider::config::{Config, EVMSubnet, Subnet, SubnetConfig};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use url::Url;

use crate::GlobalArguments;
use ipc_api::subnet_id::SubnetID;

/// Manages an `ipc_provider::config::Config`, providing shared ownership and persistence.
#[derive(Clone)]
pub struct IpcConfigStore {
    inner: Arc<Mutex<Config>>,
    path: PathBuf,
}

impl IpcConfigStore {
    /// Load the config from disk, or if none exists, create one with defaults, persist it, and return it.
    pub async fn load_or_init(global: &GlobalArguments) -> anyhow::Result<Self> {
        let path = global.config_path();
        let path = PathBuf::from(path);

        let cfg = if path.exists() {
            Config::from_file(&path)?
        } else {
            let default = Config::default();
            default.write_to_file_async(&path).await?;
            default
        };

        Ok(Self {
            inner: Arc::new(Mutex::new(cfg)),
            path: path.to_owned(),
        })
    }

    /// Returns a shared reference to the `Config`.
    pub async fn snapshot(&self) -> Config {
        let guard = self.inner.lock().await;
        guard.clone()
    }

    /// Adds a new subnet to the config and persists it to disk.
    pub async fn add_subnet(
        &self,
        subnet_id: SubnetID,
        provider_http: Url,
        gateway_addr: Address,
        registry_addr: Address,
    ) -> anyhow::Result<()> {
        let mut cfg = self.inner.lock().await;
        cfg.add_subnet(Subnet {
            id: subnet_id,
            config: SubnetConfig::Fevm(EVMSubnet {
                provider_http,
                provider_timeout: None,
                auth_token: None,
                gateway_addr,
                registry_addr,
            }),
        });

        cfg.write_to_file_async(&self.path)
            .await
            .context("writing updated IPC provider config to disk")?;
        Ok(())
    }

    /// Returns the subnet with the given ID
    pub async fn get_subnet(&self, subnet_id: &SubnetID) -> Option<Subnet> {
        let cfg = self.inner.lock().await;
        cfg.subnets.get(subnet_id).cloned()
    }
}
