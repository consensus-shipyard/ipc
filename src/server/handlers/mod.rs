//! The module contains the handlers implementation for the json rpc server.

pub mod create;

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use serde_json::Value;
pub use create::{CreateSubnetResponse, CreateSubnetParams};
use crate::server::create::CreateSubnetHandler;
use crate::server::JsonRPCRequestHandler;

pub type Method = String;

/// A util enum to avoid Box<dyn> mess in Handlers struct
enum HandlerWrapper {
    CreateSubnet(CreateSubnetHandler),
}

/// The collection of all json rpc handlers
pub struct Handlers {
    handlers: HashMap<Method, HandlerWrapper>,
}

impl Handlers {
    pub fn new() -> Self {
        let mut handlers = HashMap::new();

        let create_subnet = HandlerWrapper::CreateSubnet(CreateSubnetHandler{});
        handlers.insert(String::from("create_subnet"), create_subnet);

        Self { handlers }
    }

    pub async fn handle(&self, method: Method, params: Value) -> Result<Value> {
        if let Some(wrapper) = self.handlers.get(&method) {
            match wrapper {
                HandlerWrapper::CreateSubnet(handler) => {
                    let r = handler.handle(serde_json::from_value(params)?).await?;
                    Ok(serde_json::to_value(r)?)
                }
            }
        } else {
            Err(anyhow!("method not supported"))
        }
    }
}