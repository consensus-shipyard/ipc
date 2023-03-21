// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use fvm_shared::address::{set_current_network, Network};
use std::str::FromStr;

use crate::lotus::message::deserialize::{
    deserialize_subnet_id_from_map, deserialize_token_amount_from_str,
};
use crate::manager::SubnetInfo;
use fvm_shared::econ::TokenAmount;
use ipc_gateway::Status;
use ipc_sdk::subnet_id::SubnetID;

#[test]
fn test_subnet_from_map() {
    use serde::Deserialize;

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "PascalCase")]
    struct SubnetIdWrapper {
        #[allow(dead_code)]
        #[serde(rename = "ID")]
        #[serde(deserialize_with = "deserialize_subnet_id_from_map")]
        id: SubnetID,
    }

    let raw_str = r#"
    {
        "ID": {
            "Parent": "/root/f01",
            "Actor": "f064"
        }
    }"#;

    let w: Result<SubnetIdWrapper, _> = serde_json::from_str(raw_str);
    assert!(w.is_ok());
    assert_eq!(w.unwrap().id, SubnetID::from_str("/root/f01/f064").unwrap())
}

#[test]
fn test_subnet_from_map_error() {
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct SubnetIdWrapper {
        #[allow(dead_code)]
        #[serde(rename = "ID")]
        #[serde(deserialize_with = "deserialize_subnet_id_from_map")]
        id: SubnetID,
    }

    let raw_str = r#"
    {
        "Id": {
            "Parent": 65,
            "Actor": "f064"
        }
    }"#;

    let w: Result<SubnetIdWrapper, _> = serde_json::from_str(raw_str);
    assert!(w.is_err());
}

#[test]
fn test_token_amount_from_str() {
    use serde::Deserialize;

    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct Wrapper {
        #[allow(dead_code)]
        #[serde(deserialize_with = "deserialize_token_amount_from_str")]
        token_amount: TokenAmount,
    }

    let raw_str = r#"
    {
        "TokenAmount": "20000000000000000000"
    }"#;

    let w: Result<Wrapper, _> = serde_json::from_str(raw_str);
    assert!(w.is_ok());
    assert_eq!(w.unwrap().token_amount, TokenAmount::from_whole(20));
}

#[test]
fn test_subnet_info_to_str() {
    let s = SubnetInfo {
        id: Default::default(),
        stake: Default::default(),
        circ_supply: Default::default(),
        status: Status::Active,
    };

    let w = serde_json::to_string(&s);
    assert!(w.is_ok());
}

#[test]
fn test_subnet_info_from_str() {
    set_current_network(Network::Testnet);

    let raw_str = r#"
    {
        "ID": {
            "Parent": "/root",
            "Actor": "t010000000002"
        },
        "Stake": "10000000000000000000",
        "TopDownMsgs": {
            "/": "bafy2bzacedijw74yui7otvo63nfl3hdq2vdzuy7wx2tnptwed6zml4vvz7wee"
        },
        "Nonce": 0,
        "CircSupply": "0",
        "Status": 0,
        "PrevCheckpoint": null
    }
    "#;

    let w: Result<SubnetInfo, _> = serde_json::from_str(raw_str);
    assert!(w.is_ok());
}
