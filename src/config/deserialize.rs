// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Deserialization utils for config mod.

use fvm_shared::address::{Address, Network};
use ipc_sdk::subnet_id::SubnetID;
use num_traits::FromPrimitive;
use serde::de::{Error, SeqAccess};
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

/// A serde deserialization method to deserialize a u8 into a [`Network`].
pub(crate) fn deserialize_network<'de, D>(deserializer: D) -> anyhow::Result<Network, D::Error>
where
    D: Deserializer<'de>,
{
    struct NetworkVisitor;
    impl<'de> serde::de::Visitor<'de> for NetworkVisitor {
        type Value = Network;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a i64")
        }

        // We only need u8, but toml integer is mapped to serde with i64.
        // If we use u8, it will throw an error.
        fn visit_i64<E>(self, v: i64) -> std::result::Result<Self::Value, E>
        where
            E: Error,
        {
            Network::from_u8(v as u8).ok_or_else(|| Error::custom("unknown network"))
        }
    }
    deserializer.deserialize_str(NetworkVisitor)
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
