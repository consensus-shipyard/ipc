// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::{env::current_dir, time::Duration};

use fendermint_testing_materializer::{docker::DockerMaterializer, testnet::Testnet};

mod manifests {
    use fendermint_testing_materializer::manifest::Manifest;

    pub const ROOT_ONLY: &str = include_str!("../manifests/root-only.yaml");

    pub fn parse_yaml(yaml: &str) -> Manifest {
        serde_yaml::from_str(yaml).expect("failed to parse manifest")
    }
}

#[tokio::test]
async fn test_root_only() {
    let manifest = manifests::parse_yaml(manifests::ROOT_ONLY);

    // The current directory should be this crate.
    let root_dir = current_dir().unwrap().join("tests");
    let mut materializer = DockerMaterializer::new(&root_dir, 0).unwrap();

    let testnet = Testnet::setup(&mut materializer, "test-root-only", &manifest)
        .await
        .unwrap();

    let node1 = testnet.root().node("node-1");
    let _dnode1 = testnet.node(&node1).unwrap();

    tokio::time::sleep(Duration::from_secs(60)).await;
}
