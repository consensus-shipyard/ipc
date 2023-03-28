// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use anyhow::anyhow;
use ipc_sdk::subnet_id::SubnetID;
use serde_json::{json, Value};

/// A trait to convert a type to a JSON value.
pub(crate) trait ToJson {
    fn to_json(&self) -> Value;
}

/// Implement the `ToJson` trait for `SubnetID`.
impl ToJson for SubnetID {
    fn to_json(&self) -> Value {
        let parent = self
            .parent()
            .ok_or_else(|| anyhow!("no parent found"))
            .unwrap()
            .to_string();
        let actor = self.subnet_actor().to_string();
        json!({"Parent": parent, "Actor": actor})
    }
}

#[cfg(test)]
#[test]
fn test_to_json() {
    use std::str::FromStr;
    let subnet_id = SubnetID::from_str("/root/t0102").unwrap();
    let json = subnet_id.to_json();
    assert_eq!(
        json,
        json!({"Parent": "/root", "Actor": "t0102"}),
        "subnet id to json failed"
    );
}
