// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use color_eyre::{Result, bail};

const CONTRACTS_DIR: &str = concat!(env!("CARGO_WORKSPACE_DIR"), "/../../contracts/out/"); 

use fendermint_eth_hardhat as hardhat;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed={}", CONTRACTS_DIR);
    // TODO Hardhat::new(..)
    Ok(())
}
