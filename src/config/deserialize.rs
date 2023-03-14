// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Deserialization utils for config mod.

use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use ipc_sdk::subnet_id::SubnetID;
use serde::de::{Error, MapAccess, SeqAccess};
use serde::Deserializer;
use std::fmt::Formatter;
use std::str::FromStr;

/// A serde deserialization method to deserialize a subnet path string into a [`SubnetID`].
pub(crate) fn deserialize_subnet_id<'de, D>(deserializer: D) -> anyhow::Result<SubnetID, D::Error>
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

/// A serde deserialization method to deserialize a subnet id from map
pub fn deserialize_subnet_id_from_map<'de, D>(deserializer: D) -> anyhow::Result<SubnetID, D::Error>
where
    D: Deserializer<'de>,
{
    struct SubnetIdVisitor;
    impl<'de> serde::de::Visitor<'de> for SubnetIdVisitor {
        type Value = SubnetID;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a map")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut parent = None;
            let mut actor = None;
            while let Some((key, value)) = map
                .next_entry()?
                .map(|(k, v): (String, &'de serde_json::value::RawValue)| (k, v))
            {
                match key.as_str() {
                    "Parent" => {
                        let s = value.get();
                        if s.starts_with('"') {
                            let id =
                                SubnetID::from_str(&s[1..s.len() - 1]).map_err(A::Error::custom)?;
                            parent = Some(id);
                        } else {
                            return Err(A::Error::custom("invalid parent"));
                        }
                    }
                    "Actor" => {
                        let s = value.get();
                        if s.starts_with('"') {
                            let addr =
                                Address::from_str(&s[1..s.len() - 1]).map_err(A::Error::custom)?;
                            actor = Some(addr)
                        } else {
                            return Err(A::Error::custom("invalid actor"));
                        }
                    }
                    _ => {}
                }
            }

            if parent.is_none() || actor.is_none() {
                return Err(A::Error::custom("parent or actor not present"));
            }

            Ok(SubnetID::new_from_parent(&parent.unwrap(), actor.unwrap()))
        }
    }
    deserializer.deserialize_map(SubnetIdVisitor)
}

/// A serde deserialization method to deserialize a token amount from string
pub fn deserialize_token_amount_from_str<'de, D>(
    deserializer: D,
) -> anyhow::Result<TokenAmount, D::Error>
where
    D: Deserializer<'de>,
{
    struct TokenAmountVisitor;
    impl<'de> serde::de::Visitor<'de> for TokenAmountVisitor {
        type Value = TokenAmount;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a string")
        }

        fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
        where
            E: Error,
        {
            let u: u64 = v.parse().map_err(E::custom)?;
            Ok(TokenAmount::from_atto(u))
        }
    }
    deserializer.deserialize_str(TokenAmountVisitor)
}

/// A serde deserialization method to deserialize a list of account strings into a vector of
/// [`Address`].
pub(crate) fn deserialize_accounts<'de, D>(
    deserializer: D,
) -> anyhow::Result<Vec<Address>, D::Error>
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
