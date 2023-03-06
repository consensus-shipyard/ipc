// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! The module contains the handlers implementation for the json rpc server.

mod config;
pub mod create;
mod subnet;

use crate::config::json_rpc_methods;
use crate::config::ReloadableConfig;
use crate::server::create::CreateSubnetHandler;
use crate::server::handlers::config::ReloadConfigHandler;
use crate::server::handlers::subnet::SubnetManagerPool;
use crate::server::JsonRPCRequestHandler;
use anyhow::{anyhow, Result};
pub use create::{CreateSubnetParams, CreateSubnetResponse};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub type Method = String;

/// A util enum to avoid Box<dyn> mess in Handlers struct
enum HandlerWrapper {
    CreateSubnet(CreateSubnetHandler),
    ReloadConfig(ReloadConfigHandler),
}

/// The collection of all json rpc handlers
pub struct Handlers {
    handlers: HashMap<Method, HandlerWrapper>,
}

impl Handlers {
    /// We test the handlers separately and individually instead of from the handlers.
    /// Convenient method for json rpc to test routing.
    #[cfg(test)]
    pub fn empty_handlers() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn new(config_path_string: String) -> Result<Self> {
        let mut handlers = HashMap::new();

        let config = Arc::new(ReloadableConfig::new(config_path_string.clone())?);
        handlers.insert(
            String::from(json_rpc_methods::RELOAD_CONFIG),
            HandlerWrapper::ReloadConfig(ReloadConfigHandler::new(
                config.clone(),
                config_path_string,
            )),
        );

        let pool = Arc::new(SubnetManagerPool::from_reload_config(config));
        handlers.insert(
            String::from(json_rpc_methods::CREATE_SUBNET),
            HandlerWrapper::CreateSubnet(CreateSubnetHandler::new(pool)),
        );

        Ok(Self { handlers })
    }

    pub async fn handle(&self, method: Method, params: Value) -> Result<Value> {
        if let Some(wrapper) = self.handlers.get(&method) {
            match wrapper {
                HandlerWrapper::CreateSubnet(handler) => {
                    let r = handler.handle(serde_json::from_value(params)?).await?;
                    Ok(serde_json::to_value(r)?)
                }
                HandlerWrapper::ReloadConfig(handler) => {
                    handler.handle(serde_json::from_value(params)?).await?;
                    Ok(serde_json::to_value(())?)
                }
            }
        } else {
            Err(anyhow!("method not supported"))
        }
    }
}
