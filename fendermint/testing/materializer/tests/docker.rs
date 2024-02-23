// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_testing_materializer::{
    docker::DockerMaterializer,
    manifest::{self, Manifest},
};

mod manifests {
    use fendermint_testing_materializer::manifest::Manifest;

    pub const ROOT_ONLY: &str = include_str!("../manifests/root-only.yaml");

    pub fn parse_yaml(yaml: &str) -> Manifest {
        serde_yaml::from_str(yaml).expect("failed to parse manifest")
    }
}

#[tokio::test]
async fn test_root_only() {
    let _manifest = manifests::parse_yaml(manifests::ROOT_ONLY);
}
