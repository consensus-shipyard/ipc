// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use super::*;

#[test]
fn evm_addr_derivation_integrity() {
    let sk = ethers::core::k256::ecdsa::SigningKey::random(&mut rand::thread_rng());
    let addr0 = ethers::utils::secret_key_to_address(&sk);
    let addr0 = hex::encode(addr0.to_fixed_bytes().as_slice());

    let info = EvmKeyInfo::new(sk.to_bytes().as_slice().to_vec());
    let addr1 = info.as_address();
    assert_eq!(&addr0, &addr1.to_string());
}
