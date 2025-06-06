// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use ethers::{abi::ethereum_types::Secret, core::k256::SecretKey, types::Address};

use super::*;

#[test]
fn evm_addr_derivation_integrity() {
    let sk = SecretKey::random(&mut rand::thread_rng());
    let addr0: Address = ethers::utils::secret_key_to_address(&sk).unwrap();
    let addr0 = hex::encode(addr0.as_bytes());

    let info = EvmKeyInfo::new(sk.to_bytes());
    let addr1: String = info.as_address();
    assert_eq!(&addr1, &addr0);
}
