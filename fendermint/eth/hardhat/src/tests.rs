use ethers_core::types as et;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::{topo_sort, DependencyTree};

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
    cargo_path.parent().unwrap().to_path_buf()
}

/// Path to the Solidity contracts, indended to be used in tests.
fn contracts_path() -> PathBuf {
    let contracts_path = std::env::var("FM_CONTRACTS_DIR").unwrap_or_else(|_| {
        workspace_dir()
            .join("contracts/out")
            .to_string_lossy()
            .into_owned()
    });

    PathBuf::from_str(&contracts_path).expect("malformed contracts path")
}

fn test_hardhat() -> SolidityActorContracts {
    SolidityActorContractsLoader::load_directory(contracts_path())
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
    let hardhat = test_hardhat();

    let mut libraries = HashMap::new();

    for lib in IPC_DEPS {
        libraries.insert(lib.to_owned(), et::Address::default());
    }

    // This one requires a subset of above libraries.
    let _bytecode = hardhat
        .resolve_library_references("GatewayManagerFacet.sol", "GatewayManagerFacet", &libraries)
        .unwrap();
}

#[test]
fn bytecode_missing_link() {
    let hardhat = test_hardhat();

    // Not giving any dependency should result in a failure.
    let result = hardhat.resolve_library_references(
        "SubnetActorDiamond.sol",
        "SubnetActorDiamond",
        &Default::default(),
    );

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("failed to resolve library"));
}

#[test]
fn library_dependencies() {
    let hardhat = test_hardhat();

    let root_contracts: Vec<(String, &str)> = vec![
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
    .map(|c| (format!("{c}.sol"), c))
    .collect();

    // Name our top level contracts and gather all required libraries.
    let mut lib_deps = hardhat
        .dependencies(&root_contracts)
        .expect("failed to compute dependencies");

    // For the sake of testing, let's remove top libraries from the dependency list.
    lib_deps.retain(|(_, d)| !root_contracts.iter().any(|(_, c)| c == d));

    eprintln!("IPC dependencies: {lib_deps:?}");

    assert_eq!(
        lib_deps.len(),
        IPC_DEPS.len(),
        "should discover the same dependencies as expected"
    );

    let mut libs = HashMap::default();

    for (bytes, c) in lib_deps {
        hardhat
            .resolve_library_references(&bytes, &c, &libs)
            .unwrap_or_else(|e| {
                panic!("failed to produce library bytecode in topo order for {c}: {e}")
            });
        // Pretend that we deployed it.
        libs.insert(hardhat.fully_qualified_name(&c, &bytes), et::Address::default());
    }

    for (src, name) in root_contracts {
        hardhat
            .resolve_library_references(src, name, &libs)
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
