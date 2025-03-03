// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use std::fmt::Debug;
use std::marker::PhantomData;

use anyhow::anyhow;
use cid::Cid;
use fil_actors_runtime::{ActorError, AsActorError};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_hamt as hamt;
use fvm_shared::address::Address;
use fvm_shared::error::ExitCode;
use integer_encoding::VarInt;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::Hasher;

/// Wraps a HAMT to provide a convenient map API.
/// Any errors are returned with exit code indicating illegal state.
/// The name is not persisted in state, but adorns any error messages.
pub struct Map<BS, K, V>
where
    BS: Blockstore,
    K: MapKey,
    V: DeserializeOwned + Serialize,
{
    hamt: hamt::Hamt<BS, V, hamt::BytesKey, Hasher>,
    name: String,
    key_type: PhantomData<K>,
}

pub trait MapKey: Sized + Debug {
    fn from_bytes(b: &[u8]) -> Result<Self, String>;
    fn to_bytes(&self) -> Result<Vec<u8>, String>;
}

pub type Config = hamt::Config;

pub const DEFAULT_HAMT_CONFIG: Config = Config {
    bit_width: 5,
    min_data_depth: 2,
    max_array_width: 1,
};

impl<BS, K, V> Map<BS, K, V>
where
    BS: Blockstore,
    K: MapKey,
    V: DeserializeOwned + Serialize,
{
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Creates a new, empty map.
    pub fn empty(store: BS, config: Config, name: String) -> Self {
        Self {
            hamt: hamt::Hamt::new_with_config(store, config),
            name,
            key_type: Default::default(),
        }
    }

    /// Creates a new empty map and flushes it to the store.
    /// Returns the CID of the empty map root.
    pub fn flush_empty(store: BS, config: Config) -> Result<Cid, ActorError> {
        // This CID is constant regardless of the HAMT's configuration, so as an optimization,
        // we could hard-code it and merely check it is already stored.
        Self::empty(store, config, "empty".into()).flush()
    }

    /// Loads a map from the store.
    // There is no version of this method that doesn't take an explicit config parameter.
    // The caller must know the configuration to interpret the HAMT correctly.
    // Forcing them to provide it makes it harder to accidentally use an incorrect default.
    pub fn load(store: BS, root: &Cid, config: Config, name: String) -> Result<Self, ActorError> {
        Ok(Self {
            hamt: hamt::Hamt::load_with_config(root, store, config)
                .with_context_code(ExitCode::USR_ILLEGAL_STATE, || {
                    format!("failed to load HAMT '{}'", name)
                })?,
            name,
            key_type: Default::default(),
        })
    }

    /// Flushes the map's contents to the store.
    /// Returns the root node CID.
    pub fn flush(&mut self) -> Result<Cid, ActorError> {
        self.hamt
            .flush()
            .with_context_code(ExitCode::USR_ILLEGAL_STATE, || {
                format!("failed to flush HAMT '{}'", self.name)
            })
    }

    /// Returns a reference to the value associated with a key, if present.
    pub fn get(&self, key: &K) -> Result<Option<&V>, ActorError> {
        let k = key
            .to_bytes()
            .context_code(ExitCode::USR_ASSERTION_FAILED, "invalid key")?;
        self.hamt
            .get(&k)
            .with_context_code(ExitCode::USR_ILLEGAL_STATE, || {
                format!("failed to get key {key:?} from HAMT '{}'", self.name)
            })
    }

    pub fn contains_key(&self, key: &K) -> Result<bool, ActorError> {
        let k = key
            .to_bytes()
            .context_code(ExitCode::USR_ASSERTION_FAILED, "invalid key")?;
        self.hamt
            .contains_key(&k)
            .with_context_code(ExitCode::USR_ILLEGAL_STATE, || {
                format!("failed to check key {key:?} in HAMT '{}'", self.name)
            })
    }

    /// Inserts a key-value pair into the map.
    /// Returns any value previously associated with the key.
    pub fn set(&mut self, key: &K, value: V) -> Result<Option<V>, ActorError>
    where
        V: PartialEq,
    {
        let k = key
            .to_bytes()
            .context_code(ExitCode::USR_ASSERTION_FAILED, "invalid key")?;
        self.hamt
            .set(k.into(), value)
            .with_context_code(ExitCode::USR_ILLEGAL_STATE, || {
                format!("failed to set key {key:?} in HAMT '{}'", self.name)
            })
    }

    /// Inserts a key-value pair only if the key does not already exist.
    /// Returns whether the map was modified (i.e. key was absent).
    pub fn set_if_absent(&mut self, key: &K, value: V) -> Result<bool, ActorError>
    where
        V: PartialEq,
    {
        let k = key
            .to_bytes()
            .context_code(ExitCode::USR_ASSERTION_FAILED, "invalid key")?;
        self.hamt
            .set_if_absent(k.into(), value)
            .with_context_code(ExitCode::USR_ILLEGAL_STATE, || {
                format!("failed to set key {key:?} in HAMT '{}'", self.name)
            })
    }

    pub fn delete(&mut self, key: &K) -> Result<Option<V>, ActorError> {
        let k = key
            .to_bytes()
            .with_context_code(ExitCode::USR_ASSERTION_FAILED, || {
                format!("invalid key {key:?}")
            })?;
        self.hamt
            .delete(&k)
            .map(|delete_result| delete_result.map(|(_k, v)| v))
            .with_context_code(ExitCode::USR_ILLEGAL_STATE, || {
                format!("failed to delete key {key:?} from HAMT '{}'", self.name)
            })
    }

    /// Iterates over all key-value pairs in the map.
    #[allow(clippy::blocks_in_conditions)]
    pub fn for_each<F>(&self, mut f: F) -> Result<(), ActorError>
    where
        // Note the result type of F uses ActorError.
        // The implementation will extract and propagate any ActorError
        // wrapped in a hamt::Error::Dynamic.
        F: FnMut(K, &V) -> Result<(), ActorError>,
    {
        match self.hamt.for_each(|k, v| {
            let key = K::from_bytes(k).context_code(ExitCode::USR_ILLEGAL_STATE, "invalid key")?;
            f(key, v).map_err(|e| anyhow!(e))
        }) {
            Ok(_) => Ok(()),
            Err(hamt_err) => self.map_hamt_error(hamt_err),
        }
    }

    /// Iterates over key-value pairs in the map starting at a key up to a max.
    /// Returns the next key if there are more items in the map.
    #[allow(clippy::blocks_in_conditions)]
    pub fn for_each_ranged<F>(
        &self,
        starting_key: Option<&hamt::BytesKey>,
        max: Option<usize>,
        mut f: F,
    ) -> Result<(usize, Option<K>), ActorError>
    where
        // Note the result type of F uses ActorError.
        // The implementation will extract and propagate any ActorError
        // wrapped in a hamt::Error::Dynamic.
        F: FnMut(K, &V) -> Result<(), ActorError>,
    {
        match self.hamt.for_each_ranged(starting_key, max, |k, v| {
            let key = K::from_bytes(k).context_code(ExitCode::USR_ILLEGAL_STATE, "invalid key")?;
            f(key, v).map_err(|e| anyhow!(e))
        }) {
            Ok((traversed, next)) => {
                let next = if let Some(next) = next {
                    Some(
                        K::from_bytes(&next)
                            .context_code(ExitCode::USR_ILLEGAL_STATE, "invalid key")?,
                    )
                } else {
                    None
                };
                Ok((traversed, next))
            }
            Err(hamt_err) => self.map_hamt_error(hamt_err),
        }
    }

    /// Iterates over key-value pairs in the map starting at a key up to an ending_key (included).
    #[allow(clippy::blocks_in_conditions)]
    pub fn for_each_until<F>(
        &self,
        starting_key: Option<&hamt::BytesKey>,
        ending_key: &hamt::BytesKey,
        mut f: F,
    ) -> Result<(), ActorError>
    where
        F: FnMut(K, &V) -> Result<(), ActorError>,
    {
        let iter = match starting_key {
            Some(key) => self.hamt.iter_from(key).map_err(|error| {
                ActorError::illegal_state(format!("error traversing HAMT {}: {}", self.name, error))
            })?,
            None => self.hamt.iter(),
        };
        for res in iter.fuse().by_ref() {
            match res {
                Ok((k, v)) => {
                    if k.le(ending_key) {
                        let k = K::from_bytes(k)
                            .context_code(ExitCode::USR_ILLEGAL_STATE, "invalid key")?;
                        f(k, v)?;
                    }
                }
                Err(hamt_err) => {
                    return self.map_hamt_error(hamt_err);
                }
            }
        }
        Ok(())
    }

    pub fn iter(&self) -> hamt::Iter<BS, V, hamt::BytesKey, Hasher> {
        self.hamt.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.hamt.is_empty()
    }

    fn map_hamt_error<T>(&self, hamt_err: hamt::Error) -> Result<T, ActorError> {
        match hamt_err {
            hamt::Error::Dynamic(e) => match e.downcast::<ActorError>() {
                Ok(actor_error) => Err(actor_error),
                Err(e) => Err(ActorError::illegal_state(format!(
                    "error in callback traversing HAMT {}: {}",
                    self.name, e
                ))),
            },
            e => Err(ActorError::illegal_state(format!(
                "error traversing HAMT {}: {}",
                self.name, e
            ))),
        }
    }
}

impl MapKey for Vec<u8> {
    fn from_bytes(b: &[u8]) -> Result<Self, String> {
        Ok(b.to_vec())
    }

    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        Ok(self.clone())
    }
}

impl MapKey for String {
    fn from_bytes(b: &[u8]) -> Result<Self, String> {
        String::from_utf8(b.to_vec()).map_err(|e| e.to_string())
    }

    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        Ok(self.as_bytes().to_vec())
    }
}

impl MapKey for u64 {
    fn from_bytes(b: &[u8]) -> Result<Self, String> {
        if let Some((result, size)) = VarInt::decode_var(b) {
            if size != b.len() {
                return Err(format!("trailing bytes after varint in {:?}", b));
            }
            Ok(result)
        } else {
            Err(format!("failed to decode varint in {:?}", b))
        }
    }

    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        Ok(self.encode_var_vec())
    }
}

impl MapKey for i64 {
    fn from_bytes(b: &[u8]) -> Result<Self, String> {
        if let Some((result, size)) = VarInt::decode_var(b) {
            if size != b.len() {
                return Err(format!("trailing bytes after varint in {:?}", b));
            }
            Ok(result)
        } else {
            Err(format!("failed to decode varint in {:?}", b))
        }
    }

    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        Ok(self.encode_var_vec())
    }
}

impl MapKey for Address {
    fn from_bytes(b: &[u8]) -> Result<Self, String> {
        Address::from_bytes(b).map_err(|e| e.to_string())
    }

    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        Ok(Address::to_bytes(*self))
    }
}

impl MapKey for Cid {
    fn from_bytes(b: &[u8]) -> Result<Self, String> {
        Cid::try_from(b).map_err(|e| e.to_string())
    }

    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        Ok(self.to_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fvm_ipld_blockstore::MemoryBlockstore;

    #[test]
    fn basic_put_get() {
        let bs = MemoryBlockstore::new();
        let mut m = Map::<_, u64, String>::empty(bs, DEFAULT_HAMT_CONFIG, "empty".into());
        m.set(&1234, "1234".to_string()).unwrap();
        assert!(m.get(&2222).unwrap().is_none());
        assert_eq!(&"1234".to_string(), m.get(&1234).unwrap().unwrap());
    }

    #[test]
    fn for_each_callback_exitcode_propagates() {
        let bs = MemoryBlockstore::new();
        let mut m = Map::<_, u64, String>::empty(bs, DEFAULT_HAMT_CONFIG, "empty".into());
        m.set(&1234, "1234".to_string()).unwrap();
        let res = m.for_each(|_, _| Err(ActorError::forbidden("test".to_string())));
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), ActorError::forbidden("test".to_string()));
    }
}
