// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Reloadable config

use crate::config::Config;
use anyhow::Result;
use std::ops::DerefMut;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

/// Reloadable configuration exposes the latest config through `get_config` method. Use this you
/// will always the latest config. At the same time, it also exposes `new_subscriber`. If caller
/// needs to be notified when config has updated, just make a new subscription. Once received a
/// notification, read the config again to obtain the latest config.
pub struct ReloadableConfig {
    config: RwLock<Arc<Config>>,
    broadcast_tx: broadcast::Sender<()>,
    /// We keep at least one channel active, so that we dont encounter a `SendError`. We might need to use it later.
    #[allow(dead_code)]
    broadcast_rx: broadcast::Receiver<()>,
}

impl ReloadableConfig {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        // we dont really need a big channel, the frequency should be very very low
        let (broadcast_tx, broadcast_rx) = broadcast::channel(8);

        let config = RwLock::new(Arc::new(Config::from_file(path)?));

        Ok(Self {
            config,
            broadcast_tx,
            broadcast_rx,
        })
    }

    /// Read from the config file.
    pub fn get_config(&self) -> Arc<Config> {
        let config = self.config.read().unwrap();
        config.clone()
    }

    /// Triggers a reload of the config from the target path
    pub async fn reload(&self, path: String) -> Result<()> {
        let new_config = Config::from_file_async(path).await?;

        let mut config = self.config.write().unwrap();
        let r = config.deref_mut();
        *r = Arc::new(new_config);

        self.broadcast_tx.send(()).unwrap_or_default();

        Ok(())
    }

    pub fn new_subscriber(&self) -> broadcast::Receiver<()> {
        self.broadcast_tx.subscribe()
    }
}
