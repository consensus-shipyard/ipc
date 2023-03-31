// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use num_traits::Num;

use fvm_shared::{bigint::BigInt, econ::TokenAmount, version::NetworkVersion};

pub fn parse_network_version(s: &str) -> Result<NetworkVersion, String> {
    let nv: u32 = s
        .parse()
        .map_err(|_| format!("`{s}` isn't a network version"))?;
    if nv >= 18 {
        Ok(NetworkVersion::from(nv))
    } else {
        Err("the minimum network version is 18".to_owned())
    }
}

pub fn parse_token_amount(s: &str) -> Result<TokenAmount, String> {
    BigInt::from_str_radix(s, 10)
        .map_err(|e| format!("not a token amount: {e}"))
        .map(TokenAmount::from_atto)
}
