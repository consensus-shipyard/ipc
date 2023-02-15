//! Provides a simple way of reading configuration files.
//!
//! Reads a TOML config file for the IPC Agent and deserializes it in a type-safe way into a
//! [`Config`] struct.

use std::collections::HashMap;
use std::fmt::Formatter;
use std::fs;
use std::str::FromStr;

use anyhow::Result;
use fvm_shared::address::Address;
use ipc_sdk::subnet_id::SubnetID;
use serde::de::{Error, SeqAccess};
use serde::{Deserialize, Deserializer};
use url::Url;

/// The top-level struct representing the config. Calls to [`Config::from_file`] deserialize into
/// this struct.
#[derive(Deserialize)]
pub(crate) struct Config {
    pub subnets: HashMap<String, Subnet>,
}

/// Represents a subnet declaration in the config.
#[derive(Deserialize)]
pub struct Subnet {
    #[serde(deserialize_with = "deserialize_path")]
    id: SubnetID,
    jsonrpc_api_http: Url,
    jsonrpc_api_ws: Option<Url>,
    auth_token: Option<String>,
    #[serde(deserialize_with = "deserialize_accounts", default)]
    accounts: Vec<Address>,
}

impl Config {
    /// Reads a TOML configuration in the `s` string and returns a [`Config`] struct.
    pub fn from_str(s: &str) -> Result<Self> {
        let config = toml::from_str(&s)?;
        Ok(config)
    }

    /// Reads a TOML configuration file specified in the `path` and returns a [`Config`] struct.
    pub fn from_file(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: Config = Config::from_str(contents.as_str())?;
        Ok(config)
    }
}

/// A serde deserialization method to deserialize a subnet path string into a [`SubnetID`].
fn deserialize_path<'de, D>(deserializer: D) -> Result<SubnetID, D::Error>
where
    D: Deserializer<'de>,
{
    struct SubnetIDVisitor;
    impl<'de> serde::de::Visitor<'de> for SubnetIDVisitor {
        type Value = SubnetID;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a string")
        }

        fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
        where
            E: Error,
        {
            SubnetID::from_str(v).map_err(E::custom)
        }
    }
    deserializer.deserialize_str(SubnetIDVisitor)
}

/// A serde deserialization method to deserialize a list of account strings into a vector of
/// [`Address`].
fn deserialize_accounts<'de, D>(deserializer: D) -> Result<Vec<Address>, D::Error>
where
    D: Deserializer<'de>,
{
    struct AddressSeqVisitor;
    impl<'de> serde::de::Visitor<'de> for AddressSeqVisitor {
        type Value = Vec<Address>;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a sequence of strings")
        }

        fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut vec: Vec<Address> = Vec::new();
            while let Some(value) = seq.next_element::<String>()? {
                let a = Address::from_str(value.as_str()).map_err(Error::custom)?;
                vec.push(a);
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_str(AddressSeqVisitor)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use fvm_shared::address::Address;
    use indoc::formatdoc;
    use ipc_sdk::subnet_id::{SubnetID, ROOTNET_ID};
    use url::Url;

    use crate::config::Config;

    #[test]
    fn read_config() {
        // Arguments for the config's fields
        let root_id = "/root";
        let child_id = "/root/f0100";
        let root_auth_token = "root_auth_token";
        let child_auth_token = "child_auth_token";
        let jsonrpc_api_http = "https://example.org/rpc/v0";
        let jsonrpc_api_ws = "ws://example.org/rpc/v0";
        let account_address = "f3thgjtvoi65yzdcoifgqh6utjbaod3ukidxrx34heu34d6avx6z7r5766t5jqt42a44ehzcnw3u5ehz47n42a";

        let config_str = formatdoc!(
            r#"
            [subnets]

            [subnets.root]
            id = "{root_id}"
            jsonrpc_api_http = "{jsonrpc_api_http}"
            jsonrpc_api_ws = "{jsonrpc_api_ws}"
            auth_token = "{root_auth_token}"

            [subnets.child]
            id = "{child_id}"
            jsonrpc_api_http = "{jsonrpc_api_http}"
            auth_token = "{child_auth_token}"
            accounts = ["{account_address}"]
        "#
        );

        println!("{}", config_str);
        let config = Config::from_str(config_str.as_str()).unwrap();

        let root = &config.subnets["root"];
        assert_eq!(root.id, *ROOTNET_ID);
        assert_eq!(
            root.jsonrpc_api_http,
            Url::from_str(jsonrpc_api_http).unwrap()
        );
        assert_eq!(
            root.jsonrpc_api_ws.as_ref().unwrap(),
            &Url::from_str(jsonrpc_api_ws).unwrap()
        );
        assert_eq!(root.auth_token.as_ref().unwrap(), root_auth_token);

        let child = &config.subnets["child"];
        assert_eq!(child.id, SubnetID::from_str(child_id).unwrap());
        assert_eq!(
            child.jsonrpc_api_http,
            Url::from_str(jsonrpc_api_http).unwrap()
        );
        assert_eq!(child.auth_token.as_ref().unwrap(), child_auth_token);
        assert_eq!(
            child.accounts.as_ref(),
            vec![Address::from_str(account_address).unwrap()]
        );
    }
}
