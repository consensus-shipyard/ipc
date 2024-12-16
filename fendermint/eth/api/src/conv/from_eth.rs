// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Helper methods to convert between Ethereum and FVM data formats.

use ethers_core::types as et;
use ethers_core::types::transaction::eip2718::TypedTransaction;

pub use fendermint_vm_message::conv::from_eth::*;
use fvm_shared::{error::ExitCode, message::Message};

use crate::error::JsonRpcError;
use crate::{error, JsonRpcResult};

pub fn to_fvm_message(tx: TypedTransaction) -> JsonRpcResult<Message> {
    match tx {
        TypedTransaction::Eip1559(ref tx) => Ok(fvm_message_from_eip1559(tx)?),
        TypedTransaction::Legacy(ref tx) => Ok(fvm_message_from_legacy(tx)?),
        _ => error(
            ExitCode::USR_ILLEGAL_ARGUMENT,
            "unexpected transaction type",
        ),
    }
}

/// Turn a request into the DTO returned by the API.
pub fn to_eth_transaction_response(
    tx: &TypedTransaction,
    sig: et::Signature,
    hash: et::TxHash,
) -> Result<et::Transaction, JsonRpcError> {
    match &tx {
        TypedTransaction::Legacy(_) => todo!(),
        TypedTransaction::Eip1559(tx) => {
            Ok(et::Transaction {
                hash,
                nonce: tx.nonce.unwrap_or_default(),
                block_hash: None,
                block_number: None,
                transaction_index: None,
                from: tx.from.unwrap_or_default(),
                to: tx.to.clone().and_then(|to| to.as_address().cloned()),
                value: tx.value.unwrap_or_default(),
                gas: tx.gas.unwrap_or_default(),
                max_fee_per_gas: tx.max_fee_per_gas,
                max_priority_fee_per_gas: tx.max_priority_fee_per_gas,
                // Strictly speaking a "Type 2" transaction should not need to set this, but we do because Blockscout
                // has a database constraint that if a transaction is included in a block this can't be null.
                gas_price: Some(
                    tx.max_fee_per_gas.unwrap_or_default()
                        + tx.max_priority_fee_per_gas.unwrap_or_default(),
                ),
                input: tx.data.clone().unwrap_or_default(),
                chain_id: tx.chain_id.map(|x| et::U256::from(x.as_u64())),
                v: et::U64::from(sig.v),
                r: sig.r,
                s: sig.s,
                transaction_type: Some(2u64.into()),
                access_list: Some(tx.access_list.clone()),
                other: Default::default(),
            })
        }
        _ => error::error_with_revert(
            ExitCode::USR_ILLEGAL_ARGUMENT,
            "txn type not supported",
            None::<Vec<u8>>,
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::conv::from_eth::to_fvm_message;
    use ethers_core::types::transaction::eip2718::TypedTransaction;
    use ethers_core::types::Signature;
    use ethers_core::utils::rlp;
    use fendermint_vm_message::signed::{OriginKind, SignedMessage};
    use fvm_shared::chainid::ChainID;

    #[test]
    fn test_legacy_transaction() {
        let raw_tx = "f8ac821dac850df8475800830186a09465292eeadf1426cd2df1c4793a3d7519f253913b80b844a9059cbb000000000000000000000000cd50511c4e355f2bc3c084d854253cc17b2230bf00000000000000000000000000000000000000000000148a616ad7f95aa0000025a0a4f3a70a01cfb3969c4a12510ebccd7d08250a4d34181123bebae3f865392643a063116147193f2badc611fa20dfa1c339bca299f50e470353ee4f676bc236479d";
        let raw_tx = hex::decode(raw_tx).unwrap();

        let rlp = rlp::Rlp::new(raw_tx.as_ref());
        let (tx, sig): (TypedTransaction, Signature) =
            TypedTransaction::decode_signed(&rlp).unwrap();

        let msg = to_fvm_message(tx).unwrap();

        let signed_msg = SignedMessage {
            origin_kind: OriginKind::EthereumLegacy,
            message: msg,
            signature: fvm_shared::crypto::signature::Signature::new_secp256k1(sig.to_vec()),
        };
        assert!(signed_msg.verify(&ChainID::from(1)).is_ok());
    }
}
