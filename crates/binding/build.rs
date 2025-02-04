// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::io::Write;
use std::path::PathBuf;

const SKIP_ENV_VAR_NAME: &'static str = "SKIP_BINDING_GENERATION";
/// Generate Rust bindings from the IPC Solidity Actors ABI artifacts.
///p
/// These are built by `make ipc-actors-abi`, here we just add the final step
/// so we have better code completion with Rust Analyzer.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Run with `cargo build -vv` to see output from any `eprintln!` or `println!`.
    // FIXME use a crate printer

	println!("cargo:rerun-if-env-changed={}", SKIP_ENV_VAR_NAME);
    println!("cargo:rerun-if-changed=build.rs");
    let out = std::env::var("OUTPUT").unwrap_or_else(|_| "out".to_owned());
    // PathBuf::from(std::env::var("OUT_DIR")?);
    println!("cargo:rerun-if-changed={}", out);

    // Maybe we want to skip the build and use the files as-is, could be imported as crate.
    // Enabled by default so that in the monorepo we don't have to worry about stale code.
    if let Ok(val) = std::env::var(SKIP_ENV_VAR_NAME) {
    	let val = val.trim();
    	if val == "true" || val == "1" || val == "" {
			eprintln!("cargo:warn=Skipping binding generation since {} is set by the user", SKIP_ENV_VAR_NAME);
			return Ok(());
        }
    }
    eprintln!("cargo:warn=Running binding generation...");

    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").expect("Building with cargo provides this");
    let crate_dir = PathBuf::from(crate_dir);
     
    // Where are the Solidity artifacts.
    let root_dir = crate_dir.clone()
        .parent().expect("Parent dir should exist, since we are part of a workspace")
        .parent().expect("Structure is such that we are two levels from the root")
        .to_path_buf();
    
    let contracts_dir = root_dir.join("contracts");

    // Where to place the generated binding files
    let gen_dir = crate_dir.join("src");
    fs_err::create_dir_all(&gen_dir)?;

    let lib_path = crate_dir.join("src").join("lib.rs");
    let mut lib = fs_err::File::create(&lib_path)?;

    writeln!(lib, "// This file was generated by build.rs and will be overriden")?;
    writeln!(lib, "#[macro_use]")?;
    writeln!(lib, "mod convert;")?;

    // The list of actors we need bindings for, based on how the ipc-actor uses `abigen!`.
    // With the diamond pattern, there is a contract that holds state, and there are these facets which have the code,
    // so we need bindings for the facets, but well (I think) use the same address with all of them.
    for contract_name in [
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
        "LibStaking",
        "LibStakingChangeLog",
        "LibGateway",
        "LibQuorum",
    ] {
        let contract_name_path = PathBuf::from(contract_name);
        let module_name = camel_to_snake(contract_name);

        let module_name_path = PathBuf::from(camel_to_snake(contract_name));

        let input_path = contracts_dir.join(out)
            .join(contract_name_path.with_extension("sol"))
            .join(contract_name_path.with_extension("json"));
        let output_path = gen_dir.join(module_name_path.with_extension("rs"));

        ethers::prelude::Abigen::new::<&str, String>(contract_name, input_path.display().to_string())?
            .generate()?
            .write_to_file(output_path)?;

        writeln!(lib, "#[allow(clippy::all)]")?;
        writeln!(lib, "pub mod {module_name};")?;

    }

    writeln!(
        lib,
        r#"
// The list of contracts need to convert FvmAddress to fvm_shared::Address"#
    )
    ?;
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
        writeln!(lib, "fvm_address_conversion!({module});")?;
    }

    writeln!(
        lib,
        r#"
// The list of contracts that need to convert common types between each other"#
    )?;
    let common_type_conversion = vec![
        ("SubnetActorGetterFacet", "CheckpointingFacet"),
        ("SubnetActorGetterFacet", "XnetMessagingFacet"),
    ];
    for (contract1, contract2) in common_type_conversion {
        writeln!(
            lib,
            "common_type_conversion!({}, {});",
            camel_to_snake(contract1),
            camel_to_snake(contract2)
        )
        ?;
    }

	// format file without host tooling
    // TODO use tempfile to make the formatting atomic
    let f = syn::parse_file(lib_path.display().to_string().as_str())?;
    let c = prettyplease::unparse(&f);
    fs_err::write(&lib_path, c.as_str().as_bytes())?;

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
