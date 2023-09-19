// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Serialization utils for config mod.

use crate::config::Subnet;
use anyhow::anyhow;
use fvm_shared::address::{Address, Payload};
use ipc_sdk::subnet_id::SubnetID;
use primitives::EthAddress;
use serde::ser::{Error, SerializeSeq};
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

pub fn serialize_eth_address_to_str<S>(addr: &Address, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let addr = address_to_eth_address(addr).map_err(S::Error::custom)?;
    s.serialize_str(&format!("0x{:?}", addr))
}

pub fn serialize_eth_accounts<S>(addrs: &Vec<Address>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut seq = s.serialize_seq(Some(addrs.len()))?;
    for element in addrs {
        let addr = address_to_eth_address(element).map_err(S::Error::custom)?;
        seq.serialize_element(&format!("0x{:?}", addr))?;
    }
    seq.end()
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

fn address_to_eth_address(addr: &Address) -> anyhow::Result<EthAddress> {
    match addr.payload() {
        Payload::Delegated(inner) => {
            let mut bytes = [0; 20];
            bytes.copy_from_slice(&inner.subaddress()[0..20]);
            Ok(EthAddress(bytes))
        }
        _ => Err(anyhow!("not eth address")),
    }
}

#[cfg(test)]
mod tests {
    use crate::config::subnet::{EVMSubnet, SubnetConfig};
    use crate::config::{Config, Server, Subnet};
    use fvm_shared::address::Address;
    use ipc_sdk::subnet_id::SubnetID;
    use primitives::EthAddress;
    use std::str::FromStr;

    const STR: &str = r#"
    [server]
    json_rpc_address = "127.0.0.1:3030"

    [[subnets]]
    id = "/r123"
    network_name = "test"

    [subnets.config]
    network_type = "fvm"
    gateway_addr = "f01"
    jsonrpc_api_http = "http://127.0.0.1:3030/rpc/v1"
    accounts = ["f01", "f01"]

    [[subnets]]
    id = "/r1234"
    network_name = "test2"

    [subnets.config]
    network_type = "fevm"
    provider_http = "http://127.0.0.1:3030/rpc/v1"
    registry_addr = "0x6be1ccf648c74800380d0520d797a170c808b624"
    gateway_addr = "0x6be1ccf648c74800380d0520d797a170c808b624"
    private_key = "0x6BE1Ccf648c74800380d0520D797a170c808b624"
    accounts = ["0x6be1ccf648c74800380d0520d797a170c808b624", "0x6be1ccf648c74800380d0520d797a170c808b624"]
    "#;

    #[test]
    fn test_serialization2() {
        let config = Config::from_toml_str(STR).unwrap();

        let r = toml::to_string(&config).unwrap();
        let from_str = Config::from_toml_str(&r).unwrap();
        assert_eq!(from_str, config);
    }

    #[test]
    fn test_serialization() {
        let mut config = Config {
            server: Server {
                json_rpc_address: "127.0.0.1:3030".parse().unwrap(),
            },
            subnets: Default::default(),
        };

        let eth_addr1 = EthAddress::from_str("0x6BE1Ccf648c74800380d0520D797a170c808b624").unwrap();
        let subnet2 = Subnet {
            id: SubnetID::new_root(1234),
            network_name: "test2".to_string(),
            config: SubnetConfig::Fevm(EVMSubnet {
                gateway_addr: Address::from(eth_addr1),
                provider_http: "http://127.0.0.1:3030/rpc/v1".parse().unwrap(),
                auth_token: None,
                accounts: vec![Address::from(eth_addr1), Address::from(eth_addr1)],
                registry_addr: Address::from(eth_addr1),
            }),
        };
        config.add_subnet(subnet2);
        assert!(toml::to_string(&config).is_ok());
    }
}
