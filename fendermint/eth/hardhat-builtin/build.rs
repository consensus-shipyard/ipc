// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use build_rs_utils::{echo, rerun_if_changed};
use color_eyre::{bail, Result};

const CONTRACTS_FORGE_BUILD_OUT_DIR: &str =
    concat!(env!("CARGO_WORKSPACE_DIR"), "/../../contracts/out/");

use fendermint_eth_hardhat as hardhat;

fn main() -> Result<()> {
    // must be in sync with `lib.rs`
    let compiled_handover_path = std::env::var("OUT_DIR").ok_or_eyre("Must have set OUT_DIR")?;
    let compiled_handover_path = std::path::Path::new(compiled_handover_path).join("super.json");

    rerun_if_changed("build.rs");
    rerun_if_changed(CONTRACTS_FORGE_BUILD_OUT_DIR);
    rerun_if_changed(compiled_handover_path);

    let sol_contracts =
        hardhat::SolidityActorContractsLoader::load_directory(CONTRACTS_FORGE_BUILD_OUT_DIR)?;
    let sol_contracts_json = sol_contracts.to_json();

    fs_err::write(compiled_handover_path, sol_contracts_json.as_bytes())?;

    echo!(
        "solidity",
        yellow,
        "Prepared solidity top-level and libs actor contracts for inclusion"
    );

    Ok(())
}
