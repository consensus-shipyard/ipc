// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use std::fmt::Debug;

use anyhow::anyhow;
use cid::Cid;
use fil_actors_runtime::{ActorError, AsActorError};
use fvm_ipld_amt as amt;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::error::ExitCode;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// Wraps a HAMT to provide a convenient map API.
/// Any errors are returned with exit code indicating illegal state.
/// The name is not persisted in state, but adorns any error messages.
pub struct Vec<BS, V>
where
    BS: Blockstore,
    V: DeserializeOwned + Serialize,
{
    amt: amt::Amt<V, BS>,
}

/// Configuration options for an AMT instance.
#[derive(Debug, Clone)]
pub struct Config {
    /// The `bit_width` drives how wide and high the tree is going to be.
    /// Each node in the tree will have `2^bit_width` number of slots for child nodes,
    /// and consume `bit_width` number of bits from the hashed keys at each level.
    pub bit_width: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bit_width: AMT_BIT_WIDTH,
        }
    }
}

pub const AMT_BIT_WIDTH: u32 = 5;

pub const DEFAULT_AMT_CONFIG: Config = Config {
    bit_width: AMT_BIT_WIDTH,
};

impl<BS, V> Vec<BS, V>
where
    BS: Blockstore,
    V: DeserializeOwned + Serialize,
{
    /// Creates a new, empty vec.
    pub fn empty(store: BS, config: Config) -> Self {
        Self {
            amt: amt::Amt::new_with_bit_width(store, config.bit_width),
        }
    }

    /// Creates a new empty vec and flushes it to the store.
    /// Returns the CID of the empty vec root.
    pub fn flush_empty(store: BS, config: Config) -> Result<Cid, ActorError> {
        Self::empty(store, config).flush()
    }

    /// Loads a vec from the store.
    pub fn load(store: BS, root: &Cid) -> Result<Self, ActorError> {
        Ok(Self {
            amt: amt::Amt::load(root, store)
                .with_context_code(ExitCode::USR_ILLEGAL_STATE, || {
                    format!("failed to load AMT with root '{}'", root)
                })?,
        })
    }

    /// Flushes the vec's contents to the store.
    /// Returns the root node CID.
    pub fn flush(&mut self) -> Result<Cid, ActorError> {
        self.amt
            .flush()
            .with_context_code(ExitCode::USR_ILLEGAL_STATE, || "failed to flush AMT")
    }

    /// Returns a reference to the value at the given index, if present.
    pub fn get(&self, index: u64) -> Result<Option<&V>, ActorError> {
        self.amt
            .get(index)
            .with_context_code(ExitCode::USR_ILLEGAL_STATE, || {
                format!("failed to get from AMT at index {}", index)
            })
    }

    /// Inserts a value into the vec at the given index.
    pub fn set(&mut self, index: u64, value: V) -> Result<(), ActorError>
    where
        V: PartialEq,
    {
        self.amt
            .set(index, value)
            .with_context_code(ExitCode::USR_ILLEGAL_STATE, || {
                format!("failed to set AMT at index {}", index)
            })
    }

    /// Deletes a value from the vec at the given index.
    pub fn delete(&mut self, index: u64) -> Result<Option<V>, ActorError> {
        self.amt
            .delete(index)
            .with_context_code(ExitCode::USR_ILLEGAL_STATE, || {
                format!("failed to delete from AMT at index {}", index)
            })
    }

    /// Returns the height of the vec.
    pub fn height(&self) -> u32 {
        self.amt.height()
    }

    /// Returns count of elements in the vec.
    pub fn count(&self) -> u64 {
        self.amt.count()
    }

    /// Iterates and runs a function over values in the vec starting at an index up to a limit.
    /// Returns the index if there are more items.
    pub fn for_each_while_ranged<F>(
        &self,
        start_at: Option<u64>,
        limit: Option<u64>,
        mut f: F,
    ) -> Result<(u64, Option<u64>), ActorError>
    where
        F: FnMut(u64, &V) -> Result<bool, ActorError>,
    {
        match self
            .amt
            .for_each_while_ranged(start_at, limit, |i, v| f(i, v).map_err(|e| anyhow!(e)))
        {
            Ok((traversed, next)) => Ok((traversed, next)),
            Err(amt_err) => self.map_amt_error(amt_err),
        }
    }

    fn map_amt_error<T>(&self, amt_err: amt::Error) -> Result<T, ActorError> {
        match amt_err {
            amt::Error::Dynamic(e) => match e.downcast::<ActorError>() {
                Ok(actor_error) => Err(actor_error),
                Err(e) => Err(ActorError::illegal_state(format!(
                    "error in callback traversing AMT: {}",
                    e
                ))),
            },
            e => Err(ActorError::illegal_state(format!(
                "error traversing AMT: {}",
                e
            ))),
        }
    }
}
