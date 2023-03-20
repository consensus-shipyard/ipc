// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! The module contains the handlers implementation for the json rpc server.

mod config;
mod manager;
mod validator;

use crate::config::json_rpc_methods;
use crate::config::ReloadableConfig;
use crate::server::handlers::config::ReloadConfigHandler;
use crate::server::handlers::manager::fund::FundHandler;
use crate::server::handlers::manager::list_subnets::ListSubnetsHandler;
use crate::server::handlers::manager::propagate::PropagateHandler;
use crate::server::handlers::manager::release::ReleaseHandler;
use crate::server::handlers::manager::whitelist::WhitelistPropagatorHandler;
use crate::server::handlers::send_value::SendValueHandler;
use crate::server::handlers::validator::QueryValidatorSetHandler;
use crate::server::JsonRPCRequestHandler;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
pub use config::ReloadConfigParams;
use manager::create::CreateSubnetHandler;
use manager::join::JoinSubnetHandler;
use manager::kill::KillSubnetHandler;
use manager::leave::LeaveSubnetHandler;
use manager::subnet::SubnetManagerPool;
pub use manager::*;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub type Method = String;

/// The collection of all json rpc handlers
pub struct Handlers {
    handlers: HashMap<Method, Box<dyn HandlerWrapper>>,
}

/// A util trait to avoid Box<dyn> and associated type mess in Handlers struct
#[async_trait]
trait HandlerWrapper: Send + Sync {
    async fn handle(&self, params: Value) -> Result<Value>;
}

#[async_trait]
impl<H: JsonRPCRequestHandler + Send + Sync> HandlerWrapper for H {
    async fn handle(&self, params: Value) -> Result<Value> {
        let p = serde_json::from_value(params)?;
        let r = self.handle(p).await?;
        Ok(serde_json::to_value(r)?)
    }
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

    pub fn new(config: Arc<ReloadableConfig>) -> Result<Self> {
        let mut handlers = HashMap::new();

        let h: Box<dyn HandlerWrapper> = Box::new(ReloadConfigHandler::new(config.clone()));
        handlers.insert(String::from(json_rpc_methods::RELOAD_CONFIG), h);

        // subnet manager methods
        let pool = Arc::new(SubnetManagerPool::from_reload_config(config.clone()));
        let h: Box<dyn HandlerWrapper> = Box::new(CreateSubnetHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::CREATE_SUBNET), h);

        let h: Box<dyn HandlerWrapper> = Box::new(LeaveSubnetHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::LEAVE_SUBNET), h);

        let h: Box<dyn HandlerWrapper> = Box::new(KillSubnetHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::KILL_SUBNET), h);

        let h: Box<dyn HandlerWrapper> = Box::new(JoinSubnetHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::JOIN_SUBNET), h);

        let h: Box<dyn HandlerWrapper> = Box::new(FundHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::FUND), h);

        let h: Box<dyn HandlerWrapper> = Box::new(ReleaseHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::RELEASE), h);

        let h: Box<dyn HandlerWrapper> = Box::new(PropagateHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::PROPAGATE), h);

        let h: Box<dyn HandlerWrapper> = Box::new(WhitelistPropagatorHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::WHITELIST_PROPAGATOR), h);

        let h: Box<dyn HandlerWrapper> = Box::new(SendValueHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::SEND_VALUE), h);

        let h: Box<dyn HandlerWrapper> = Box::new(ListSubnetsHandler::new(pool));
        handlers.insert(String::from(json_rpc_methods::LIST_CHILD_SUBNETS), h);

        // query validator
        let h: Box<dyn HandlerWrapper> = Box::new(QueryValidatorSetHandler::new(config));
        handlers.insert(String::from(json_rpc_methods::QUERY_VALIDATOR_SET), h);

        Ok(Self { handlers })
    }

    pub async fn handle(&self, method: Method, params: Value) -> Result<Value> {
        if let Some(wrapper) = self.handlers.get(&method) {
            wrapper.handle(params).await
        } else {
            Err(anyhow!("method not supported"))
        }
    }
}
