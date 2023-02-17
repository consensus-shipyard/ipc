// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use cid::Cid;
use fvm_shared::{address::Address, econ::TokenAmount};
use serde::{Deserialize, Serialize};

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
/// I changed `Serialize_tuple` into `Serialize` - could be better as a
/// message exchange format if the field names are in tact.
#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub struct ActorState {
    /// Link to code for the actor.
    pub code: Cid,
    /// Link to the state of the actor.
    pub state: Cid,
    /// Sequence of the actor.
    pub sequence: u64,
    /// Tokens available to the actor.
    pub balance: TokenAmount,
    /// The actor's "delegated" address, if assigned.
    ///
    /// This field is set on actor creation and never modified.
    pub delegated_address: Option<Address>,
}

#[cfg(feature = "arb")]
mod arb {
    use fvm_shared::{address::Address, econ::TokenAmount};

    use crate::arb::{cid::arbitrary_cid, fix_address, fix_tokens};

    use super::{ActorState, FvmQuery};

    impl quickcheck::Arbitrary for FvmQuery {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            match u8::arbitrary(g) % 2 {
                0 => FvmQuery::Ipld(arbitrary_cid(g)),
                _ => FvmQuery::ActorState(fix_address(Address::arbitrary(g))),
            }
        }
    }

    impl quickcheck::Arbitrary for ActorState {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            Self {
                code: arbitrary_cid(g),
                state: arbitrary_cid(g),
                sequence: u64::arbitrary(g),
                balance: fix_tokens(TokenAmount::arbitrary(g)),
                delegated_address: Option::<Address>::arbitrary(g).map(fix_address),
            }
        }
    }
}
