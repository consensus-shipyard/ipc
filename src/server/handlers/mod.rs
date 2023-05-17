// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! The module contains the handlers implementation for the json rpc server.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde_json::Value;

pub use config::ReloadConfigParams;
use fvm_shared::econ::TokenAmount;
use manager::create::CreateSubnetHandler;
use manager::join::JoinSubnetHandler;
use manager::kill::KillSubnetHandler;
use manager::leave::LeaveSubnetHandler;
use manager::subnet::SubnetManagerPool;
pub use manager::*;

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
use crate::server::handlers::wallet::balances::WalletBalancesHandler;
use crate::server::handlers::wallet::new::WalletNewHandler;
use crate::server::list_checkpoints::ListBottomUpCheckpointsHandler;
use crate::server::net_addr::SetValidatorNetAddrHandler;
use crate::server::JsonRPCRequestHandler;
use ipc_identity::Wallet;

use self::config::new_keystore_from_config;
pub use self::config::new_keystore_from_path;
use self::rpc::RPCSubnetHandler;
use self::topdown_executed::LastTopDownExecHandler;
use self::wallet::export::WalletExportHandler;
use self::wallet::import::WalletImportHandler;

mod config;
mod manager;
mod validator;
pub mod wallet;

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

        // Load the wallet manager from keystore
        let wallet = Arc::new(RwLock::new(Wallet::new(new_keystore_from_config(
            config.clone(),
        )?)));

        // subnet manager methods
        let pool = Arc::new(SubnetManagerPool::new(config.clone(), wallet.clone()));
        let h: Box<dyn HandlerWrapper> = Box::new(CreateSubnetHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::CREATE_SUBNET), h);

        let h: Box<dyn HandlerWrapper> = Box::new(LeaveSubnetHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::LEAVE_SUBNET), h);

        let h: Box<dyn HandlerWrapper> = Box::new(KillSubnetHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::KILL_SUBNET), h);

        let h: Box<dyn HandlerWrapper> = Box::new(JoinSubnetHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::JOIN_SUBNET), h);

        let h: Box<dyn HandlerWrapper> = Box::new(RPCSubnetHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::RPC_SUBNET), h);

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

        let h: Box<dyn HandlerWrapper> = Box::new(WalletNewHandler::new(wallet.clone()));
        handlers.insert(String::from(json_rpc_methods::WALLET_NEW), h);

        let h: Box<dyn HandlerWrapper> = Box::new(WalletImportHandler::new(wallet.clone()));
        handlers.insert(String::from(json_rpc_methods::WALLET_IMPORT), h);

        let _h: Box<dyn HandlerWrapper> = Box::new(WalletExportHandler::new(wallet.clone()));
        // FIXME: For security reasons currently not exposing the ability to export wallet
        // remotely through the RPC API, only directly through the CLI.
        // We can consider re-enabling once we have RPC authentication in the agent.
        // handlers.insert(String::from(json_rpc_methods::WALLET_EXPORT), h);

        let h: Box<dyn HandlerWrapper> = Box::new(WalletBalancesHandler::new(pool.clone(), wallet));
        handlers.insert(String::from(json_rpc_methods::WALLET_BALANCES), h);

        let h: Box<dyn HandlerWrapper> = Box::new(SetValidatorNetAddrHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::SET_VALIDATOR_NET_ADDR), h);

        let h: Box<dyn HandlerWrapper> = Box::new(ListSubnetsHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::LIST_CHILD_SUBNETS), h);

        let h: Box<dyn HandlerWrapper> =
            Box::new(ListBottomUpCheckpointsHandler::new(pool.clone()));
        handlers.insert(String::from(json_rpc_methods::LIST_BOTTOMUP_CHECKPOINTS), h);

        let h: Box<dyn HandlerWrapper> = Box::new(LastTopDownExecHandler::new(pool));
        handlers.insert(String::from(json_rpc_methods::LAST_TOPDOWN_EXECUTED), h);

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

pub(crate) fn f64_to_token_amount(f: f64) -> anyhow::Result<TokenAmount> {
    let precision = TokenAmount::PRECISION as f64;
    // no rounding, just the integer part
    let amount = TokenAmount::from_atto(f64::trunc(f * precision) as u64);

    if !amount.is_positive() {
        Err(anyhow!("invalid token amount: {f:}"))
    } else {
        Ok(amount)
    }
}

#[cfg(test)]
mod tests {
    use crate::server::handlers::f64_to_token_amount;
    use fvm_shared::econ::TokenAmount;

    #[test]
    fn test_amount() {
        let amount = f64_to_token_amount(1.2f64).unwrap();
        assert_eq!(amount, TokenAmount::from_atto(1200000000000000000u64));
    }
}
