// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Helper methods to convert between Ethereum and FVM data formats.

use ethers_core::types as et;
use ethers_core::types::transaction::eip2718::TypedTransaction;

pub use fendermint_vm_message::conv::from_eth::*;
use fvm_shared::{error::ExitCode, message::Message};

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
pub fn to_eth_transaction(
    tx: et::Eip1559TransactionRequest,
    sig: et::Signature,
    hash: et::TxHash,
) -> et::Transaction {
    et::Transaction {
        hash,
        nonce: tx.nonce.unwrap_or_default(),
        block_hash: None,
        block_number: None,
        transaction_index: None,
        from: tx.from.unwrap_or_default(),
        to: tx.to.and_then(|to| to.as_address().cloned()),
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
        input: tx.data.unwrap_or_default(),
        chain_id: tx.chain_id.map(|x| et::U256::from(x.as_u64())),
        v: et::U64::from(sig.v),
        r: sig.r,
        s: sig.s,
        transaction_type: Some(2u64.into()),
        access_list: Some(tx.access_list),
        other: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use ethers_core::types::Signature;
    use ethers_core::types::transaction::eip2718::TypedTransaction;
    use ethers_core::utils::rlp;
    use crate::conv::from_eth::to_fvm_message;

    #[test]
    fn test_legacy_transaction() {
        let raw_tx = "f86e158512bfb19e608301f8dc94c083e9947cf02b8ffc7d3090ae9aea72df98fd4789056bc75e2d63100000801ca0a254fe085f721c2abe00a2cd244110bfc0df5f4f25461c85d8ab75ebac11eb10a030b7835ba481955b20193a703ebc5fdffeab081d63117199040cdf5a91c68765";
        let raw_tx = hex::decode(raw_tx).unwrap();

        let rlp = rlp::Rlp::new(raw_tx.as_ref());
        let (tx, sig): (TypedTransaction, Signature) = TypedTransaction::decode_signed(&rlp).unwrap();

        let r = to_fvm_message(tx).unwrap();

    }
}