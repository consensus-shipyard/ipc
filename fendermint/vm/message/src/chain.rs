// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use serde::{Deserialize, Serialize};

use crate::{ipc::IpcMessage, signed::SignedMessage};

/// The different kinds of messages that can appear in blocks, ie. the transactions
/// we can receive from Tendermint through the ABCI.
///
/// Unlike Filecoin, we don't have `Unsigned` messages here. In Filecoin, the messages
/// signed by BLS signatures are aggregated to the block level, and their original
/// signatures are stripped from the messages, to save space. Tendermint Core will
/// not do this for us (perhaps with ABCI++ Vote Extensions we could do it), though.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChainMessage {
    /// A message that can be passed on to the FVM as-is.
    Signed(SignedMessage),

    /// The validator messages for IPC to function correctly.
    Validator(ValidatorMessage),
}

/// The messages sent from validators that perform various on chain duties.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidatorMessage {
    SignBottomUpCheckpoint(SignedMessage),
    TopdownPropose(SignedMessage),
}

impl From<SignedMessage> for ChainMessage {
    fn from(msg: SignedMessage) -> Self {
        ChainMessage::Signed(msg)
    }
}

#[cfg(feature = "arb")]
mod arb {

    use super::ChainMessage;
    use crate::{ipc::IpcMessage, signed::SignedMessage};
    use crate::chain::ValidatorMessage;

    impl quickcheck::Arbitrary for ChainMessage {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            match u8::arbitrary(g) % 2 {
                0 => ChainMessage::Signed(SignedMessage::arbitrary(g)),
                _ => ChainMessage::Validator(ValidatorMessage::SignBottomUpCheckpoint(SignedMessage::arbitrary(g))),
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
