// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Helper methods to convert between FVM and Ethereum data formats.

use std::str::FromStr;

use anyhow::anyhow;
use ethers_core::types as et;
use ethers_core::types::transaction::eip2718::TypedTransaction;
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_actor_interface::eam::EAM_ACTOR_ID;
use fvm_shared::address::Address;
use fvm_shared::bigint::BigInt;
use fvm_shared::chainid::ChainID;
use fvm_shared::crypto::signature::Signature;
use fvm_shared::crypto::signature::SignatureType;
use fvm_shared::crypto::signature::SECP_SIG_LEN;
use fvm_shared::message::Message;
use fvm_shared::{address::Payload, econ::TokenAmount};
use lazy_static::lazy_static;
use libsecp256k1::RecoveryId;

lazy_static! {
    pub static ref MAX_U256: BigInt = BigInt::from_str(&et::U256::MAX.to_string()).unwrap();
}

pub fn to_eth_tokens(amount: &TokenAmount) -> anyhow::Result<et::U256> {
    if amount.atto() > &MAX_U256 {
        Err(anyhow!("TokenAmount > U256.MAX"))
    } else {
        let (_sign, bz) = amount.atto().to_bytes_be();
        Ok(et::U256::from_big_endian(&bz))
    }
}

pub fn to_eth_address(addr: &Address) -> Option<et::H160> {
    match addr.payload() {
        Payload::Delegated(d) if d.namespace() == EAM_ACTOR_ID && d.subaddress().len() == 20 => {
            Some(et::H160::from_slice(d.subaddress()))
        }
        // It should be possible to send to an ethereum account by ID.
        Payload::ID(id) => Some(et::H160::from_slice(&EthAddress::from_id(*id).0)),
        // XXX: The following fit into the type but are not valid ethereum addresses.
        Payload::Secp256k1(h) => Some(et::H160::from_slice(h)),
        Payload::Actor(h) => Some(et::H160::from_slice(h)),
        _ => None, // BLS or an invalid delegated address. Just move on.
    }
}

fn parse_secp256k1(
    sig: &[u8],
) -> anyhow::Result<(libsecp256k1::RecoveryId, libsecp256k1::Signature)> {
    if sig.len() != SECP_SIG_LEN {
        return Err(anyhow!("unexpected Secp256k1 length: {}", sig.len()));
    }

    // generate types to recover key from
    let rec_id = RecoveryId::parse(sig[64])?;

    // Signature value without recovery byte
    let mut s = [0u8; 64];
    s.clone_from_slice(&sig[..64]);

    // generate Signature
    let sig = libsecp256k1::Signature::parse_standard(&s)?;

    Ok((rec_id, sig))
}

pub fn to_eth_signature(sig: &Signature) -> anyhow::Result<et::Signature> {
    let (v, sig) = match sig.sig_type {
        SignatureType::Secp256k1 => parse_secp256k1(&sig.bytes)?,
        other => return Err(anyhow!("unexpected signature type: {other:?}")),
    };

    let sig = et::Signature {
        v: et::U64::from(v.serialize()).as_u64(),
        r: et::U256::from_big_endian(sig.r.b32().as_ref()),
        s: et::U256::from_big_endian(sig.s.b32().as_ref()),
    };

    Ok(sig)
}

pub fn to_eth_transaction(msg: &Message, chain_id: &ChainID) -> anyhow::Result<TypedTransaction> {
    let chain_id: u64 = (*chain_id).into();

    let Message {
        version: _,
        from,
        to,
        sequence,
        value,
        method_num: _,
        params,
        gas_limit,
        gas_fee_cap,
        gas_premium,
    } = msg;

    let mut tx = et::Eip1559TransactionRequest::new()
        .chain_id(chain_id)
        .from(to_eth_address(from).unwrap_or_default())
        .nonce(*sequence)
        .value(to_eth_tokens(value)?)
        .gas(*gas_limit)
        .max_fee_per_gas(to_eth_tokens(gas_fee_cap)?)
        .max_priority_fee_per_gas(to_eth_tokens(gas_premium)?)
        .data(et::Bytes::from(params.to_vec()));

    tx.to = to_eth_address(to).map(et::NameOrAddress::Address);

    Ok(tx.into())
}

#[cfg(test)]
pub mod tests {

    use std::{array, str::FromStr};

    use fendermint_testing::arb::{ArbMessage, ArbTokenAmount};
    use fendermint_vm_actor_interface::{
        eam::{EthAddress, EAM_ACTOR_ID},
        evm,
    };
    use fendermint_vm_message::signed::SignedMessage;
    use fvm_shared::{
        address::Address,
        bigint::{BigInt, Integer},
        chainid::ChainID,
        econ::TokenAmount,
        message::Message,
    };
    use libsecp256k1::SecretKey;
    use quickcheck_macros::quickcheck;
    use rand::{rngs::StdRng, SeedableRng};

    use crate::conv::from_eth::to_fvm_message;

    use super::{to_eth_signature, to_eth_tokens, to_eth_transaction, MAX_U256};

    #[derive(Clone, Debug)]
    struct EthDelegatedAddress(Address);

    impl quickcheck::Arbitrary for EthDelegatedAddress {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut subaddr: [u8; 20] = array::from_fn(|_| u8::arbitrary(g));
            while EthAddress(subaddr).is_masked_id() {
                subaddr[0] = u8::arbitrary(g);
            }
            Self(Address::new_delegated(EAM_ACTOR_ID, &subaddr).unwrap())
        }
    }

    #[derive(Clone, Debug)]
    struct EthTokenAmount(TokenAmount);

    impl quickcheck::Arbitrary for EthTokenAmount {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let t = ArbTokenAmount::arbitrary(g).0;
            let (_, t) = t.atto().div_mod_floor(&MAX_U256);
            Self(TokenAmount::from_atto(t))
        }
    }

    /// Message that only contains data which can survive a roundtrip.
    #[derive(Clone, Debug)]
    pub struct EthMessage(pub Message);

    impl quickcheck::Arbitrary for EthMessage {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut m = ArbMessage::arbitrary(g).0;
            m.version = 0;
            m.method_num = evm::Method::InvokeContract as u64;
            m.from = EthDelegatedAddress::arbitrary(g).0;
            m.to = EthDelegatedAddress::arbitrary(g).0;
            m.value = EthTokenAmount::arbitrary(g).0;
            m.gas_fee_cap = EthTokenAmount::arbitrary(g).0;
            m.gas_premium = EthTokenAmount::arbitrary(g).0;
            Self(m)
        }
    }

    #[quickcheck]
    fn prop_to_eth_tokens(tokens: ArbTokenAmount) -> bool {
        let tokens = tokens.0;
        if let Ok(u256_from_tokens) = to_eth_tokens(&tokens) {
            let tokens_as_str = tokens.atto().to_str_radix(10);
            let u256_from_str = ethers_core::types::U256::from_dec_str(&tokens_as_str).unwrap();
            return u256_from_str == u256_from_tokens;
        }
        true
    }

    #[test]
    fn test_to_eth_tokens() {
        let atto = BigInt::from_str(
            "99191064924191451313862974502415542781658129482631472725645205117646186753315",
        )
        .unwrap();

        let tokens = TokenAmount::from_atto(atto);

        to_eth_tokens(&tokens).unwrap();
    }

    #[quickcheck]
    fn prop_signature(msg: SignedMessage, seed: u64, chain_id: u64) -> Result<(), String> {
        let chain_id = ChainID::from(chain_id);

        let mut rng = StdRng::seed_from_u64(seed);
        let sk = SecretKey::random(&mut rng);

        let msg = SignedMessage::new_secp256k1(msg.into_message(), &sk, &chain_id)
            .map_err(|e| format!("failed to sign: {e}"))?;

        let sig0 = msg.signature();

        let sig1 =
            to_eth_signature(sig0).map_err(|e| format!("failed to convert signature: {e}"))?;

        let sig2 = fvm_shared::crypto::signature::Signature::new_secp256k1(sig1.to_vec());

        if *sig0 != sig2 {
            return Err(format!("signatures don't match: {sig0:?} != {sig2:?}"));
        }
        Ok(())
    }

    #[quickcheck]
    fn prop_to_and_from_eth_transaction(msg: EthMessage, chain_id: u64) {
        let chain_id = ChainID::from(chain_id);
        let msg0 = msg.0;
        let tx = to_eth_transaction(&msg0, &chain_id).unwrap();
        let tx = tx.as_eip1559_ref().unwrap();
        let msg1 = to_fvm_message(tx).unwrap();

        assert_eq!(msg1, msg0)
    }
}
