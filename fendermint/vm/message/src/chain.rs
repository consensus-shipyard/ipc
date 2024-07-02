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
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChainMessage {
    /// A message that can be passed on to the FVM as-is.
    Signed(SignedMessage),

    /// Messages involved in InterPlanetaryConsensus, which are basically top-down and bottom-up
    /// checkpoints that piggy-back on the Tendermint voting mechanism for finality and CID resolution.
    ///
    /// Possible mechanisms include:
    /// * Proposing "for resolution" - A message with a CID proposed for async resolution. These would be bottom-up
    ///     messages that need to be relayed, so they also include some relayer identity and signature, for rewards.
    /// * Proposing "for execution" - A message with a CID with proven availability and finality, ready to be executed.
    ///     Such messages are proposed by the validators themselves, and their execution might trigger rewards for others.
    ///
    /// Because of the involvement of data availability voting and CID resolution, these messages require support
    /// from the application, which is why they are handled in a special way.
    Ipc(IpcMessage),
}

#[cfg(feature = "arb")]
mod arb {

    use super::ChainMessage;
    use crate::{ipc::IpcMessage, signed::SignedMessage};

    impl quickcheck::Arbitrary for ChainMessage {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            match u8::arbitrary(g) % 2 {
                0 => ChainMessage::Signed(SignedMessage::arbitrary(g)),
                _ => ChainMessage::Ipc(IpcMessage::arbitrary(g)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::chain::{ChainMessage, IpcMessage};
    use fvm_shared::econ::TokenAmount;
    use std::str::FromStr;

    use crate::ipc::SealedTopdownProposal;
    use ipc_api::address::IPCAddress;
    use ipc_api::cross::{IpcEnvelope, IpcMsgKind};
    use quickcheck_macros::quickcheck;

    #[quickcheck]
    fn chain_message_cbor(value0: ChainMessage) {
        let repr = fvm_ipld_encoding::to_vec(&value0).expect("failed to encode");
        let value1: ChainMessage =
            fvm_ipld_encoding::from_slice(repr.as_ref()).expect("failed to decode");

        assert_eq!(value1, value0)
    }

    #[test]
    fn test_sealed_finality() {
        let sealed = SealedTopdownProposal::new(
            1735339,
            vec![
                29, 242, 28, 50, 92, 203, 187, 118, 19, 5, 6, 41, 177, 77, 62, 21, 170, 251, 221,
                205, 233, 83, 29, 92, 217, 175, 241, 171, 167, 249, 52, 66,
            ],
            vec![IpcEnvelope {
                kind: IpcMsgKind::Transfer,
                to: IPCAddress::from_str("/r314159:f410filqucpvo75uywfpmakbpnrileto6powcfxi2wea")
                    .unwrap(),
                value: TokenAmount::from_whole(1),
                from: IPCAddress::from_str("/r314159:f410fdj4tqxvnb2dt7ygeihadiy3nh3pxafgm42mxxjy")
                    .unwrap(),
                message: vec![],
                nonce: 0,
            }],
            vec![],
        );

        let msg = ChainMessage::Ipc(IpcMessage::TopDownExecV2(sealed));
        let bytes = fvm_ipld_encoding::to_vec(&msg).unwrap();
        let m = fvm_ipld_encoding::from_slice::<ChainMessage>(&bytes).unwrap();

        assert_eq!(m, msg);
    }
}
