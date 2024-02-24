// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::{env::current_dir, path::PathBuf, pin::Pin, time::Duration};

use anyhow::Context;
use fendermint_testing_materializer::{
    docker::{DockerMaterializer, DockerMaterials},
    manifest::Manifest,
    testnet::Testnet,
    TestnetName,
};
use futures::Future;
use serial_test::serial;

/// Want to keep the testnet artifacts in the `tests/testnets` directory.
fn tests_dir() -> PathBuf {
    let dir = current_dir().unwrap();
    debug_assert!(
        dir.ends_with("materializer"),
        "expected the current directory to be the crate"
    );
    dir.join("tests")
}

/// Parse a manifest file in the `manifests` directory, clean up any corresponding
/// testnet resources, then materialize a testnet and run some tests.
async fn with_testnet<F>(manifest_name: &str, f: F) -> anyhow::Result<()>
where
    F: FnOnce(
        &mut DockerMaterializer,
        Testnet<DockerMaterials, DockerMaterializer>,
        Manifest,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>>>>,
{
    let root = tests_dir();
    let manifest = root.join("manifests").join(format!("{manifest_name}.yaml"));
    let manifest = std::fs::read_to_string(manifest).context("failed to read manifest")?;
    let manifest = serde_yaml::from_str(&manifest).context("failed to parse manifest")?;

    let testnet_name = TestnetName::new(manifest_name);

    let mut materializer = DockerMaterializer::new(&root, 0).unwrap();

    materializer
        .remove(&testnet_name)
        .await
        .context("failed to remove testnet")?;

    let testnet = Testnet::setup(&mut materializer, &testnet_name, &manifest)
        .await
        .context("failed to set up testnet")?;

    let res = f(&mut materializer, testnet, manifest).await;

    // Allow some time for containers to be dropped.
    tokio::time::sleep(Duration::from_secs(5)).await;

    res
}

// Run these tests serially because they share a common `materializer-state.json` file with the port mappings.
#[serial]
mod materializer_tests {
    use super::with_testnet;

    #[tokio::test]
    async fn test_root_only() {
        with_testnet("root-only", |_materializer, testnet, _manifest| {
            Box::pin(async move {
                let node1 = testnet.root().node("node-1");
                let _dnode1 = testnet.node(&node1)?;
                Ok(())
            })
        })
        .await
        .unwrap()
    }
}
