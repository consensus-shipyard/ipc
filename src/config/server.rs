use std::net::SocketAddr;
use serde::Deserialize;

pub const JSON_RPC_ENDPOINT: &str = "json_rpc";

#[derive(Deserialize, Clone)]
pub struct Server {
    pub json_rpc_address: SocketAddr,
}