use ipc_sdk::subnet_id::SubnetID;
use url::Url;
use fvm_shared::address::Address;
use serde::Deserialize;

use crate::config::deserialize::{deserialize_accounts, deserialize_subnet_id};

/// Represents a subnet declaration in the config.
#[derive(Deserialize, Clone)]
pub struct Subnet {
    #[serde(deserialize_with = "deserialize_subnet_id")]
    pub id: SubnetID,
    pub jsonrpc_api_http: Url,
    pub jsonrpc_api_ws: Option<Url>,
    pub auth_token: Option<String>,
    #[serde(deserialize_with = "deserialize_accounts", default)]
    pub accounts: Vec<Address>,
}
