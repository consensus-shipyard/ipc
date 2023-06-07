// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Serialization utils for config mod.

use crate::config::Subnet;
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;
use serde::ser::SerializeSeq;
use serde::Serializer;
use std::collections::HashMap;

/// A serde serialization method to serialize a hashmap of subnets with subnet id as key and
/// Subnet struct as value to a vec of subnets
pub fn serialize_subnets_to_str<S>(
    subnets: &HashMap<SubnetID, Subnet>,
    s: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let values = subnets.values().collect::<Vec<_>>();

    let mut seq = s.serialize_seq(Some(values.len()))?;
    for element in values {
        seq.serialize_element(element)?;
    }
    seq.end()
}

pub fn serialize_subnet_id_to_str<S>(id: &SubnetID, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&id.to_string())
}

pub fn serialize_address_to_str<S>(addr: &Address, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&addr.to_string())
}

pub fn serialize_accounts<S>(addrs: &Vec<Address>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(addrs.len()))?;
    for element in addrs {
        seq.serialize_element(&element.to_string())?;
    }
    seq.end()
}

#[cfg(test)]
mod tests {
    use crate::config::Config;

    const STR: &str = r#"
    [server]
    json_rpc_address = "127.0.0.1:3030"

    [[subnets]]
    id = "/r123"
    gateway_addr = "f064"
    network_name = "root"
    jsonrpc_api_http = "http://127.0.0.1:1234/rpc/v1"
    jsonrpc_api_ws = "wss://example.org/rpc/v0"
    auth_token = "YOUR TOKEN"
    accounts = ["f01"]
    "#;

    #[test]
    fn test_serialization() {
        let config = Config::from_toml_str(STR).unwrap();

        let r = toml::to_string(&config).unwrap();
        let from_str = Config::from_toml_str(&r).unwrap();
        assert_eq!(from_str, config);
    }
}
