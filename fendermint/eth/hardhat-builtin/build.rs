// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{Context, Result};
use build_rs_utils::{echo, rerun_if_changed};

use fendermint_eth_hardhat as hardhat;

fn main() -> Result<()> {
    // must be in sync with `lib.rs`
    let compiled_handover_path = std::env::var("OUT_DIR").context("Must have set OUT_DIR")?;
    let compiled_handover_path =
        std::path::Path::new(compiled_handover_path.as_str()).join("super.json");

    // FIXME TODO tests currently rely on this
    let contracts_forge_build_out_dir =
        std::env::var("CARGO_MANIFEST_DIR").context("Must have set CARGO_MANIFEST_DIR")?;
    let contracts_forge_build_out_dir =
        std::path::Path::new(contracts_forge_build_out_dir.as_str())
            .join("../../../contracts/out/");

    rerun_if_changed("build.rs");
    rerun_if_changed(&contracts_forge_build_out_dir);
    rerun_if_changed(&compiled_handover_path);

    let sol_contracts =
        hardhat::SolidityActorContractsLoader::load_directory(&contracts_forge_build_out_dir)?;
    let sol_contracts_json = sol_contracts.to_json()?;

    fs_err::write(compiled_handover_path, sol_contracts_json.as_bytes())?;

    echo!(
        "solidity",
        yellow,
        "Prepared solidity top-level and libs actor contracts for inclusion"
    );

    Ok(())
}
