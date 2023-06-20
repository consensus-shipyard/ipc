// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Helper methods to convert between Ethereum and FVM data formats.

use anyhow::Context;
use ethers_core::types::{Eip1559TransactionRequest, NameOrAddress, H160, H256, U256};
use fendermint_vm_actor_interface::{
    eam::{self, EthAddress},
    evm,
};
use fvm_ipld_encoding::RawBytes;
use fvm_shared::{
    address::Address,
    bigint::{BigInt, Sign},
    econ::TokenAmount,
    message::Message,
};

// https://github.com/filecoin-project/lotus/blob/594c52b96537a8c8728389b446482a2d7ea5617c/chain/types/ethtypes/eth_transactions.go#L152
pub fn to_fvm_message(tx: &Eip1559TransactionRequest) -> anyhow::Result<Message> {
    // FIP-55 says that we should use `InvokeContract` for transfers instead of `METHOD_SEND`,
    // because if we are sending to some Ethereum actor by ID using `METHOD_SEND`, they will
    // get the tokens but the contract might not provide any way of retrieving them.
    // The `Account` actor has been modified to accept any method call, so it will not fail
    // even if it receives tokens using `InvokeContract`.
    let (method_num, to) = match tx.to {
        None => (eam::Method::CreateExternal as u64, eam::EAM_ACTOR_ADDR),
        Some(NameOrAddress::Address(to)) => {
            let to = to_fvm_address(to);
            (evm::Method::InvokeContract as u64, to)
        }
        Some(NameOrAddress::Name(_)) => {
            anyhow::bail!("Turning name to address would require ENS which is not supported.")
        }
    };

    // The `from` of the transaction is inferred from the signature.
    // As long as the client and the server use the same hashing scheme,
    // this should be usable as a delegated address.
    let from = to_fvm_address(tx.from.unwrap_or_default());

    let msg = Message {
        version: 0,
        from,
        to,
        sequence: tx.nonce.unwrap_or_default().as_u64(),
        value: to_fvm_tokens(&tx.value.unwrap_or_default()),
        method_num,
        params: RawBytes::new(tx.data.clone().unwrap_or_default().to_vec()),
        gas_limit: tx
            .gas
            .map(|gas| gas.min(U256::from(u64::MAX)).as_u64())
            .unwrap_or_default(),
        gas_fee_cap: to_fvm_tokens(&tx.max_fee_per_gas.unwrap_or_default()),
        gas_premium: to_fvm_tokens(&tx.max_fee_per_gas.unwrap_or_default()),
    };

    Ok(msg)
}

pub fn to_fvm_address(addr: H160) -> Address {
    Address::from(EthAddress(addr.0))
}

pub fn to_fvm_tokens(value: &U256) -> TokenAmount {
    let mut bz = [0u8; 256 / 8];
    value.to_big_endian(&mut bz);
    let atto = BigInt::from_bytes_be(Sign::Plus, &bz);
    TokenAmount::from_atto(atto)
}

pub fn to_tm_hash(value: &H256) -> anyhow::Result<tendermint::Hash> {
    tendermint::Hash::try_from(value.as_bytes().to_vec())
        .context("failed to convert to Tendermint Hash")
}

#[cfg(test)]
mod tests {

    use fendermint_testing::arb::ArbTokenAmount;
    use quickcheck_macros::quickcheck;

    use crate::conv::from_fvm::to_eth_tokens;

    use super::to_fvm_tokens;

    #[quickcheck]
    fn prop_to_token_amount(tokens: ArbTokenAmount) -> bool {
        let tokens0 = tokens.0;
        if let Ok(value) = to_eth_tokens(&tokens0) {
            let tokens1 = to_fvm_tokens(&value);
            return tokens0 == tokens1;
        }
        true
    }
}
