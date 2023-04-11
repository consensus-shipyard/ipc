// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use fvm_shared::address::Address;
use fvm_shared::bigint::BigInt;
use fvm_shared::econ::TokenAmount;
use num_traits::Num;
use serde::de::{DeserializeOwned, Error};
use serde::{de, Deserialize, Serialize, Serializer};
use serde_with::{DeserializeAs, SerializeAs};
use std::any::type_name;
use std::fmt::Display;
use std::str::FromStr;

use cid::Cid;

/// Serializer which can be used together with the [`serde_with`] crate to annotate
/// fields that we want to appear as strings in human readable formats like JSON,
/// and leave as their default serialization formats otherwise (ie. bytes, which
/// would appear as array of numbers in JSON).
///
/// # Example
///
/// ```ignore
/// #[serde_as(as = "Option<IsHumanReadable>")]
/// pub delegated_address: Option<Address>,
/// ```
pub struct IsHumanReadable;

fn serialize_str<T, S>(source: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: ToString + Serialize,
    S: Serializer,
{
    {
        if serializer.is_human_readable() {
            source.to_string().serialize(serializer)
        } else {
            source.serialize(serializer)
        }
    }
}

fn deserialize_str<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr + DeserializeOwned,
    <T as FromStr>::Err: Display,
    D: de::Deserializer<'de>,
{
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            match T::from_str(&s) {
                Ok(a) => Ok(a),
                Err(e) => Err(D::Error::custom(format!(
                    "error deserializing {}: {}",
                    type_name::<T>(),
                    e
                ))),
            }
        } else {
            T::deserialize(deserializer)
        }
    }
}

/// Create [`SerializeAs`] and [`DeserializeAs`] instances for `IsHumanReadable` for the
/// given type assuming it implements [`ToString`] and [`FromStr`].
#[macro_export]
macro_rules! human_readable_str {
    ($typ: ty) => {
        impl SerializeAs<$typ> for IsHumanReadable {
            fn serialize_as<S>(addr: &$typ, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serialize_str(addr, serializer)
            }
        }

        impl<'de> DeserializeAs<'de, $typ> for IsHumanReadable {
            fn deserialize_as<D>(deserializer: D) -> Result<$typ, D::Error>
            where
                D: de::Deserializer<'de>,
            {
                deserialize_str(deserializer)
            }
        }
    };
}

human_readable_str!(Address);
human_readable_str!(Cid);

impl SerializeAs<TokenAmount> for IsHumanReadable {
    /// Serialize tokens as human readable decimal string.
    fn serialize_as<S>(tokens: &TokenAmount, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            tokens.atto().to_str_radix(10).serialize(serializer)
        } else {
            tokens.serialize(serializer)
        }
    }
}

impl<'de> DeserializeAs<'de, TokenAmount> for IsHumanReadable {
    /// Deserialize tokens from human readable decimal format.
    fn deserialize_as<D>(deserializer: D) -> Result<TokenAmount, D::Error>
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
}
