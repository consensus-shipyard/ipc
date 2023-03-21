// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use cid::Cid;
use serde::{Deserialize, Serialize};

use crate::signed::SignedMessage;

/// The different kinds of messages that can appear in blocks, ie. the transactions
/// we can receive from Tendermint through the ABCI.
///
/// Unlike Filecoin, we don't have `Unsigned` messages here. In Filecoin, the messages
/// signed by BLS signatures are aggregated to the block level, and their original
/// signatures are stripped from the messages, to save space. Tendermint Core will
/// not do this for us (perhaps with ABCI++ Vote Extensions we could do it), though.
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum ChainMessage {
    /// A message that can be passed on to the FVM as-is.
    Signed(Box<SignedMessage>),
    // TODO: ForResolution - A message CID proposed for async resolution.
    // This will not need a signature, it is proposed by the validator who made the block.
    // We might want to add a `from` and a signature anyway if we want to reward relayers.
    // Or the validator itself can be rewarded for inclusion, since a message like this
    // will be a top-down or bottom-up message, and this incentivises them to do the relaying.
    ForResolution(Cid),

    // TODO: ForExecution - A message CID proposed for execution in the containing block, assumed to be resolvable.
    // This will again not need a signature, it is proposed by the validator who made the block.
    // The reward for this should have two parts:
    // (1) go to the validator who originally proposed the resolution of this CID, and
    // (2) go to the validator who proposed the execution.
    // This should ensure that even if low-power validator poposed a CID, the others aren't neglecting it.
    // To remember after a restart who the original proposer was, the proposed CIDs have to go onto the ledger.
    ForExecution(Cid),
}

#[cfg(feature = "arb")]
mod arb {
    use fendermint_testing::arb::ArbCid;

    use super::ChainMessage;
    use crate::signed::SignedMessage;

    impl quickcheck::Arbitrary for ChainMessage {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            match u8::arbitrary(g) % 3 {
                0 => ChainMessage::Signed(Box::new(SignedMessage::arbitrary(g))),
                1 => ChainMessage::ForResolution(ArbCid::arbitrary(g).0),
                _ => ChainMessage::ForExecution(ArbCid::arbitrary(g).0),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::chain::ChainMessage;
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn chain_message_cbor(value0: ChainMessage) {
        let repr = fvm_ipld_encoding::to_vec(&value0).expect("failed to encode");
        let value1: ChainMessage =
            fvm_ipld_encoding::from_slice(repr.as_ref()).expect("failed to decode");

        assert_eq!(value1, value0)
    }
}
