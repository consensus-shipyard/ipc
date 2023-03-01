//! The ipc-agent json rpc node.

use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;
use std::fmt::Debug;

pub mod jsonrpc;
pub mod request;
pub mod response;
mod handlers;

pub use handlers::*;

/// The JSON RPC server request handler trait.
#[async_trait]
pub trait JsonRPCRequestHandler {
    type Request: Debug;
    type Response: Debug + Serialize;

    /// Handles the request sent to the json rpc server. Returns a response back.
    async fn handle(&self, request: Self::Request) -> Result<Self::Response>;
}
