// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use std::str::FromStr;

use fvm_shared::bigint::BigInt;
use fvm_shared::{address::Address, econ::TokenAmount};
use num_traits::Num;
use serde::de::Error;
use serde::{de, Deserialize, Serialize, Serializer};

use crate::ActorAddr;

impl Serialize for ActorAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            self.0.to_string().serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}

impl<'de> Deserialize<'de> for ActorAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            match Address::from_str(&s) {
                Ok(a) => Ok(Self(a)),
                Err(e) => Err(D::Error::custom(format!(
                    "error deserializing address: {}",
                    e
                ))),
            }
        } else {
            Address::deserialize(deserializer).map(Self)
        }
    }
}

/// Serialize tokens as human readable string.
pub fn serialize_tokens<S>(tokens: &TokenAmount, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if serializer.is_human_readable() {
        tokens.atto().to_str_radix(10).serialize(serializer)
    } else {
        tokens.serialize(serializer)
    }
}

/// Deserialize tokens from human readable decimal format.
pub fn deserialize_tokens<'de, D>(deserializer: D) -> Result<TokenAmount, D::Error>
where
    D: de::Deserializer<'de>,
{
    if deserializer.is_human_readable() {
        let s = String::deserialize(deserializer)?;
        match BigInt::from_str_radix(&s, 10) {
            Ok(a) => Ok(TokenAmount::from_atto(a)),
            Err(e) => Err(D::Error::custom(format!(
                "error deserializing tokens: {}",
                e
            ))),
        }
    } else {
        TokenAmount::deserialize(deserializer)
    }
}
