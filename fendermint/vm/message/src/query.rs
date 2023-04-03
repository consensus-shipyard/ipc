// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use cid::Cid;
use fvm_shared::{address::Address, econ::TokenAmount};
use serde::{Deserialize, Serialize};

use crate::encoding::{deserialize_cid, deserialize_tokens, serialize_cid, serialize_tokens};

// TODO: Use `serde_with` to get rid of `ActorAddr`.

/// Wrapper around [`Address`] to provide human readable serialization in JSON format.
///
/// An alternative would be the `serde_with` crate.
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActorAddr(pub Address);

/// Queries over the IPLD blockstore or the state tree.
///
/// Maybe we can have some common queries over the known state of built-in actors,
/// and actors supporting IPC, or FEVM.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FvmQuery {
    /// Query something from the IPLD store.
    ///
    /// The response is the raw bytes from the store.
    Ipld(Cid),
    /// Query the state of an actor.
    ///
    /// The response is IPLD encoded `ActorState`.
    ActorState(Address),
}

/// State of all actor implementations.
///
/// This is a copy of `fvm::state_tree::ActorState` so that this crate
/// doesn't need a dependency on `fvm` itself, only `fvm_shared`.
///
/// Note that it changes changes `Serialize_tuple` into `Serialize`
/// to preserve the field names; the intention is to display the results
/// as JSON, where tuple serialization wouldn't be as useful.
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct ActorState {
    /// Link to code for the actor.
    #[serde(serialize_with = "serialize_cid", deserialize_with = "deserialize_cid")]
    pub code: Cid,
    /// Link to the state of the actor.
    #[serde(serialize_with = "serialize_cid", deserialize_with = "deserialize_cid")]
    pub state: Cid,
    /// Sequence of the actor.
    pub sequence: u64,
    /// Tokens available to the actor.
    #[serde(
        serialize_with = "serialize_tokens",
        deserialize_with = "deserialize_tokens"
    )]
    pub balance: TokenAmount,
    /// The actor's "delegated" address, if assigned.
    ///
    /// This field is set on actor creation and never modified.
    pub delegated_address: Option<ActorAddr>,
}

#[cfg(feature = "arb")]
mod arb {
    use fendermint_testing::arb::{ArbAddress, ArbCid, ArbTokenAmount};

    use super::{ActorAddr, ActorState, FvmQuery};

    impl quickcheck::Arbitrary for FvmQuery {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            match u8::arbitrary(g) % 2 {
                0 => FvmQuery::Ipld(ArbCid::arbitrary(g).0),
                _ => FvmQuery::ActorState(ArbAddress::arbitrary(g).0),
            }
        }
    }

    impl quickcheck::Arbitrary for ActorState {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Self {
                code: ArbCid::arbitrary(g).0,
                state: ArbCid::arbitrary(g).0,
                sequence: u64::arbitrary(g),
                balance: ArbTokenAmount::arbitrary(g).0,
                delegated_address: Option::<ArbAddress>::arbitrary(g).map(|a| ActorAddr(a.0)),
            }
        }
    }
}
