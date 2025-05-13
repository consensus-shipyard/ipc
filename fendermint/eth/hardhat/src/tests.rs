// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use ethers_core::types as et;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use tracing::level_filters::LevelFilter;

use crate::{as_contract_name, topo_sort, ContractName, DependencyTree, SolidityActorContracts};

use super::SolidityActorContractsLoader;

fn workspace_dir() -> PathBuf {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
    let ws_dir = cargo_path.parent().unwrap().to_path_buf();
    tracing::debug!("Workspace dir found as {}", ws_dir.display());
    ws_dir
}

/// Path to the Solidity contracts, indended to be used in tests.
fn contracts_path() -> PathBuf {
    let contracts_path = std::env::var("FM_CONTRACTS_DIR")
        .map(|val| PathBuf::from_str(&val).expect("malformed contracts path"))
        .unwrap_or_else(|_| workspace_dir().join("contracts").join("out"));
    tracing::info!("Loading contracts from {}", contracts_path.display());
    contracts_path
}

fn test_hardhat() -> SolidityActorContracts {
    let _sub = tracing_subscriber::fmt()
        .with_level(true)
        .with_line_number(true)
        .with_max_level(LevelFilter::INFO)
        .try_init();

    SolidityActorContractsLoader::load_directory(&contracts_path())
        .expect("Test contracts always works")
}

// These are all the libraries based on the `scripts/deploy-libraries.ts` in `ipc-solidity-actors`.
const IPC_DEPS: [&str; 4] = [
    "AccountHelper",
    "SubnetIDHelper",
    "CrossMsgHelper",
    "LibQuorum",
];

#[test]
fn bytecode_linking() {
    // test is already performed here
    let hardhat = test_hardhat();

    let mut libraries = HashMap::new();

    for lib in IPC_DEPS {
        libraries.insert(dbg!(as_contract_name(lib)), et::Address::default());
    }

    // This one requires a subset of above libraries.
    let _bytecode = hardhat
        .resolve_library_references(dbg!(&as_contract_name("GatewayManagerFacet")), &libraries)
        .unwrap();
}

#[test]
fn bytecode_missing_link() {
    let hardhat = test_hardhat();

    // Not giving any dependency should result in a failure.
    let result = hardhat
        .resolve_library_references(&as_contract_name("SubnetActorDiamond"), &Default::default());

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("No such contract: SubnetActorDiamond"));
}

#[test]
fn library_dependencies() {
    let hardhat = test_hardhat();

    let root_contracts = Vec::<ContractName>::from_iter(
        [
            "GatewayDiamond",
            "GatewayManagerFacet",
            "CheckpointingFacet",
            "TopDownFinalityFacet",
            "XnetMessagingFacet",
            "GatewayGetterFacet",
            "GatewayMessengerFacet",
            "SubnetActorGetterFacet",
            "SubnetActorManagerFacet",
            "SubnetActorRewardFacet",
            "SubnetActorCheckpointingFacet",
            "SubnetActorPauseFacet",
        ]
        .into_iter()
        .map(as_contract_name),
    );

    // Name our top level contracts and gather all required libraries.
    let mut lib_deps = hardhat
        .dependencies(&root_contracts)
        .expect("failed to compute dependencies");

    // For the sake of testing, let's remove top libraries from the dependency list.
    lib_deps.retain(|lib_name| !root_contracts.iter().any(|root_name| root_name == lib_name));

    eprintln!("IPC dependencies: {lib_deps:?}");

    assert_eq!(
        lib_deps.len(),
        IPC_DEPS.len(),
        "should discover the same dependencies as expected"
    );

    let mut library_addresses = HashMap::default();

    for contract_name in lib_deps {
        hardhat
            .resolve_library_references(&contract_name, &library_addresses)
            .unwrap_or_else(|e| {
                panic!("failed to produce library bytecode in topo order for {contract_name}: {e}")
            });
        // Pretend that we deployed it.
        library_addresses.insert(contract_name, et::Address::default());
    }

    for name in root_contracts {
        let _linked = hardhat
            .resolve_library_references(&name, &library_addresses)
            .expect("failed to produce contract bytecode in topo order");
    }
}

#[test]
fn topo_sorting() {
    let mut tree: DependencyTree<u8> = Default::default();

    for (k, ds) in [
        (1, vec![]),
        (2, vec![1]),
        (3, vec![1, 2]),
        (4, vec![3]),
        (5, vec![4, 2]),
    ] {
        tree.entry(k).or_default().extend(ds);
    }

    let sorted = topo_sort(tree.clone()).unwrap();

    assert_eq!(sorted.len(), 5);

    for (i, k) in sorted.iter().enumerate() {
        for d in &tree[k] {
            let j = sorted.iter().position(|x| x == d).unwrap();
            assert!(j < i);
        }
    }
}
