// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::io::Write;
use std::path::{Path, PathBuf};

use build_rs_utils::{echo, rerun_if_changed, rerun_if_env_changed, run_command_with_stdio};
use color_eyre::eyre::eyre;

const SKIP_ENV_VAR_NAME: &str = "SKIP_BINDING_GENERATION";

fn find_program(name: &str) -> color_eyre::Result<PathBuf> {
    let binary = which::which(name).map_err(|e| eyre!("Could not find binary `{name}`: {e}"))?;
    Ok(binary)
}

#[allow(dead_code)]
async fn run_forge_test(_contracts_dir: &Path, _out: &Path) -> color_eyre::Result<()> {
    // forge test -vvv --ffi
    todo!("forge test -vvv --ffi")
}

/// Run `forge build`
async fn run_forge_build(contracts_dir: &Path, out: &Path) -> color_eyre::Result<()> {
    // Re-run on any change `contracts_dir` directory and subtree
    rerun_if_changed(&contracts_dir);

    const FORGE_VERBOSITY_ENV_NAME: &str = "CONTRACT_BINDINGS_FORGE_BUILD_VERBOSITY";
    rerun_if_env_changed(FORGE_VERBOSITY_ENV_NAME);
    let verbosity = std::env::var(FORGE_VERBOSITY_ENV_NAME).unwrap_or_else(|_| "-q".to_owned());

    let forge = find_program("forge")?;

    fs_err::create_dir_all(&out)?;

    let mut cmd = std::process::Command::new(forge);
    cmd.current_dir(contracts_dir);
    cmd.args(
        format!(
            "{verbosity} build -C ./src/ --lib-paths lib/ --via-ir --sizes --skip test --out={out}",
            verbosity = verbosity,
            out = out.display()
        )
        .split(" "),
    );
    run_command_with_stdio("forge", cmd).await
}

/// `pnpm install`
///
/// It's a pre-requisite to make `forge build` work
async fn run_pnpm_install(cwd: &Path) -> color_eyre::Result<()> {
    let pnpm = find_program("pnpm")?;
    let mut cmd = std::process::Command::new(pnpm);
    cmd.current_dir(cwd);
    cmd.arg("install");
    run_command_with_stdio("pnpm", cmd).await
}

/// Generate Rust contract-bindings from the IPC Solidity Actors ABI artifacts.
///
/// These are built by `make ipc-actors-abi`, here we just add the final step
/// so we have better code completion with Rust Analyzer.
#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Run with `cargo build -vv` to see output from any `eprintln!` or `println!`.
    // FIXME use a crate printer

    rerun_if_env_changed(SKIP_ENV_VAR_NAME);
    rerun_if_changed("build.rs");

    let out = std::env::var("OUTPUT").unwrap_or_else(|_| "out".to_owned());
    rerun_if_changed(&out);

    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Building with cargo provides this");
    let crate_dir = PathBuf::from(crate_dir);

    // Where to place the generated binding files
    // TODO `OUT_DIR` would be a even better place
    // TODO and then using `include_bytes!` from `lib.rs`
    // PathBuf::from(std::env::var("OUT_DIR")?);
    let gen_dir = crate_dir.join("src").join("gen");
    let mod_path = gen_dir.join("mod.rs");

    rerun_if_changed(crate_dir.join("src"));

    // Maybe we want to skip the build and use the files as-is, could be imported as crate.
    // Enabled by default so that in the monorepo we don't have to worry about stale code.
    if let Ok(val) = std::env::var(SKIP_ENV_VAR_NAME) {
        let val = val.trim();
        if val == "true" || val == "1" || val.is_empty() {
            echo!(
                "contract-bindings",
                yellow,
                "Skipping binding generation since {} is set by the user",
                SKIP_ENV_VAR_NAME
            );
            return Ok(());
        }
    }
    echo!("contract-bindings", yellow, "Running binding generation...");

    // Where are the Solidity artifacts.
    let workspace_dir = fs_err::canonicalize(crate_dir.clone().parent().expect(
        "Structure is such that we are 2 levels from the root, two levels up should work",
    ))?;

    let contracts_dir = workspace_dir.join("contracts");

    // build the contracts from solidity
    run_pnpm_install(&contracts_dir).await?;
    run_forge_build(&contracts_dir, contracts_dir.join(&out).as_path()).await?;

    fs_err::create_dir_all(&gen_dir)?;
    let mut mod_f = fs_err::File::create(&mod_path)?;

    writeln!(
        mod_f,
        "//! This file was generated by build.rs and will be overriden"
    )?;
    writeln!(mod_f)?;

    // The list of actors we need contract-bindings for, based on how the ipc-actor uses `abigen!`.
    // With the diamond pattern, there is a contract that holds state, and there are these facets which have the code,
    // so we need contract-bindings for the facets, but well (I think) use the same address with all of them.
    let all_contracts = [
        "IDiamond",
        "DiamondLoupeFacet",
        "DiamondCutFacet",
        "OwnershipFacet",
        "GatewayDiamond",
        "GatewayManagerFacet",
        "GatewayGetterFacet",
        "CheckpointingFacet",
        "TopDownFinalityFacet",
        "XnetMessagingFacet",
        "GatewayMessengerFacet",
        "SubnetActorActivityFacet",
        "SubnetActorCheckpointingFacet",
        "SubnetActorDiamond",
        "SubnetActorGetterFacet",
        "SubnetActorManagerFacet",
        "SubnetActorPauseFacet",
        "SubnetActorRewardFacet",
        "SubnetRegistryDiamond",
        "RegisterSubnetFacet",
        "SubnetGetterFacet",
        "LibPower",
        "LibPowerChangeLog",
        "LibGateway",
        "LibQuorum",
    ];

    for contract_name in all_contracts {
        let contract_name_path = PathBuf::from(contract_name);
        let module_name = camel_to_snake(contract_name);

        let module_name_path = PathBuf::from(camel_to_snake(contract_name));

        let input_path = contracts_dir
            .join(&out)
            .join(contract_name_path.with_extension("sol"))
            .join(contract_name_path.with_extension("json"));
        let output_path = gen_dir.join(module_name_path.with_extension("rs"));

        ethers::prelude::Abigen::new::<&str, String>(
            contract_name,
            input_path.display().to_string(),
        )?
        .generate()?
        .write_to_file(output_path)?;

        writeln!(mod_f, "#[allow(clippy::all)]")?;
        writeln!(mod_f, "pub mod {module_name};")?;
    }
    writeln!(mod_f)?;
    writeln!(
        mod_f,
        "// The list of contracts need to convert FvmAddress to fvm_shared::Address"
    )?;
    let fvm_address_conversion = vec![
        "GatewayManagerFacet",
        "GatewayGetterFacet",
        "XnetMessagingFacet",
        "GatewayMessengerFacet",
        "SubnetActorCheckpointingFacet",
        "SubnetActorGetterFacet",
        "LibGateway",
        "CheckpointingFacet",
    ];

    let modules = fvm_address_conversion.into_iter().map(camel_to_snake);
    for module in modules {
        writeln!(mod_f, "fvm_address_conversion!({module});")?;
    }

    writeln!(mod_f)?;

    writeln!(
        mod_f,
        "// The list of contracts that need to convert common types between each other"
    )?;
    let common_type_conversion = vec![
        ("SubnetActorGetterFacet", "CheckpointingFacet"),
        ("SubnetActorGetterFacet", "XnetMessagingFacet"),
    ];
    for (contract1, contract2) in common_type_conversion {
        writeln!(
            mod_f,
            "common_type_conversion!({}, {});",
            camel_to_snake(contract1),
            camel_to_snake(contract2)
        )?;
    }

    writeln!(mod_f, "\n\n")?;
    error_mapping_gen(&mut mod_f, &all_contracts)?;

    mod_f.flush()?;
    mod_f.sync_all()?;

    // format file without host tooling
    // TODO use tempfile to make the formatting atomic
    let unformatted = fs_err::read_to_string(&mod_path)?;
    let f = syn::parse_str(&unformatted)?;
    let c = prettyplease::unparse(&f);
    fs_err::write(&mod_path, c.as_bytes())?;

    Ok(())
}

/// Convert ContractName to contract_name so we can use it as a Rust module.
///
/// We could just lowercase, but this is what `Abigen` does as well, and it's more readable with complex names.
fn camel_to_snake(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        match (i, c) {
            (0, c) if c.is_uppercase() => {
                out.push(c.to_ascii_lowercase());
            }
            (_, c) if c.is_uppercase() => {
                out.push('_');
                out.push(c.to_ascii_lowercase());
            }
            (_, c) => {
                out.push(c);
            }
        }
    }
    out
}

/// generate the mapping between contract error selectors to the ethers abi error type for parsing
/// potential contract errors.
/// This function generates a rust file that creates the error mapping, see [`ipc_actors_abis::extend_contract_error_mapping`]
/// macro for how it works internally.
/// This function will write the macro call [`ipc_actors_abis::extend_contract_error_mapping`] and loops all the contract names to watch and
/// fill the macro rule.
fn error_mapping_gen(mod_f: &mut impl Write, all_contracts: &[&str]) -> color_eyre::Result<()> {
    writeln!(mod_f, "crate::extend_contract_error_mapping!(")?;

    let map_name_to_macro_rule = |s| format!("[{}, {}_ABI]", camel_to_snake(s), s.to_uppercase());

    let extend_map_code = all_contracts
        .iter()
        .map(|s| map_name_to_macro_rule(s))
        .collect::<Vec<_>>()
        .join(",\n");

    writeln!(mod_f, "{}", extend_map_code)?;
    writeln!(mod_f, ");")?;
    Ok(())
}
