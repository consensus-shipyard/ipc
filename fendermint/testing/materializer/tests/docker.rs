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
use lazy_static::lazy_static;
use serial_test::serial;

lazy_static! {
    static ref CI_PROFILE: bool = std::env::var("PROFILE").unwrap_or_default() == "ci";
    static ref STARTUP_WAIT_SECS: u64 = if *CI_PROFILE { 20 } else { 15 };
    static ref TEARDOWN_WAIT_SECS: u64 = 5;
    static ref PRINT_LOGS_ON_ERROR: bool = *CI_PROFILE;
}

/// Want to keep the testnet artifacts in the `tests/testnets` directory.
fn tests_dir() -> PathBuf {
    let dir = current_dir().unwrap();
    debug_assert!(
        dir.ends_with("materializer"),
        "expected the current directory to be the crate"
    );
    dir.join("tests")
}

/// Parse a manifest from the `tests/manifests` directory.
fn read_manifest(file_name: &str) -> anyhow::Result<Manifest> {
    let manifest = tests_dir().join("manifests").join(file_name);
    let manifest = std::fs::read_to_string(&manifest).with_context(|| {
        format!(
            "failed to read manifest from {}",
            manifest.to_string_lossy()
        )
    })?;
    let manifest = serde_yaml::from_str(&manifest).context("failed to parse manifest")?;
    Ok(manifest)
}

/// Parse a manifest file in the `manifests` directory, clean up any corresponding
/// testnet resources, then materialize a testnet and run some tests.
async fn with_testnet<F>(manifest_file_name: &str, f: F) -> anyhow::Result<()>
where
    // https://users.rust-lang.org/t/function-that-takes-a-closure-with-mutable-reference-that-returns-a-future/54324
    F: for<'a> FnOnce(
        &Manifest,
        &mut DockerMaterializer,
        &'a mut Testnet<DockerMaterials, DockerMaterializer>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<()>> + 'a>>,
{
    let testnet_name = TestnetName::new(
        PathBuf::from(manifest_file_name)
            .file_stem()
            .expect("there is a file step")
            .to_string_lossy()
            .to_string(),
    );
    let manifest = read_manifest(manifest_file_name)?;

    let mut materializer = DockerMaterializer::new(&tests_dir().join("docker-materializer"), 0)?;

    materializer
        .remove(&testnet_name)
        .await
        .context("failed to remove testnet")?;

    let mut testnet = Testnet::setup(&mut materializer, &testnet_name, &manifest)
        .await
        .context("failed to set up testnet")?;

    // Allow time for things to consolidate and blocks to be created.
    tokio::time::sleep(Duration::from_secs(*STARTUP_WAIT_SECS)).await;

    let res = f(&manifest, &mut materializer, &mut testnet).await;

    // Print all logs on failure.
    // Some might be available in logs in the files which are left behind,
    // e.g. for `fendermint` we have logs, but maybe not for `cometbft`.
    if res.is_err() && *PRINT_LOGS_ON_ERROR {
        for (name, node) in testnet.nodes() {
            let name = name.path_string();
            for log in node.fendermint_logs().await {
                eprintln!("{name}/fendermint: {log}");
            }
            for log in node.cometbft_logs().await {
                eprintln!("{name}/cometbft: {log}");
            }
            for log in node.ethapi_logs().await {
                eprintln!("{name}/ethapi: {log}");
            }
        }
    }

    // Tear down the testnet.
    drop(testnet);

    // Allow some time for containers to be dropped.
    // This only happens if the testnet setup succeeded,
    // otherwise the system shuts down too quick, but
    // at least we can inspect the containers.
    // If they don't all get dropped, `docker system prune` helps.
    tokio::time::sleep(Duration::from_secs(*TEARDOWN_WAIT_SECS)).await;

    res
}

// Run these tests serially because they share a common `materializer-state.json` file with the port mappings.
#[serial]
mod materializer_tests {

    use anyhow::{anyhow, bail};
    use ethers::{providers::Middleware, types::U64};
    use fendermint_testing_materializer::HasEthApi;
    use futures::FutureExt;

    use super::with_testnet;

    #[tokio::test]
    async fn test_root_only() {
        with_testnet("root-only.yaml", |_materializer, _manifest, testnet| {
            let test = async {
                // Check that node2 is following node1.
                let node2 = testnet.root().node("node-2");
                let dnode2 = testnet.node(&node2)?;

                let provider = dnode2
                    .ethapi_http_provider()?
                    .ok_or_else(|| anyhow!("node-2 has ethapi enabled"))?;

                let bn = provider.get_block_number().await?;

                if bn <= U64::one() {
                    bail!("expected positive block number");
                }

                Ok(())
            };

            test.boxed_local()
        })
        .await
        .unwrap()
    }
}
