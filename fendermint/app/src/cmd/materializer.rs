// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::Path;

use anyhow::anyhow;
use fendermint_app_options::materializer::*;
use fendermint_materializer::{
    docker::{DockerMaterializer, DropPolicy},
    manifest::Manifest,
    testnet::Testnet,
    TestnetId, TestnetName,
};

use crate::cmd;

cmd! {
  MaterializerArgs(self) {
    let m = || DockerMaterializer::new(&self.data_dir, self.seed).map(|m| m.with_policy(DropPolicy::PERSISTENT));
    match &self.command {
        MaterializerCommands::Validate(args) => args.exec(()).await,
        MaterializerCommands::Setup(args) => args.exec(m()?).await,
        MaterializerCommands::Remove(args) => args.exec(m()?).await,
    }
  }
}

cmd! {
  MaterializerValidateArgs(self) {
    validate(&self.manifest_file).await
  }
}

cmd! {
  MaterializerSetupArgs(self, m: DockerMaterializer) {
    setup(m, &self.manifest_file, self.validate).await
  }
}

cmd! {
  MaterializerRemoveArgs(self, m: DockerMaterializer) {
    remove(m, self.testnet_id.clone()).await
  }
}

/// Validate a manifest.
async fn validate(manifest_file: &Path) -> anyhow::Result<()> {
    let (name, manifest) = read_manifest(manifest_file)?;
    manifest.validate(&name).await
}

/// Setup a testnet.
async fn setup(
    mut m: DockerMaterializer,
    manifest_file: &Path,
    validate: bool,
) -> anyhow::Result<()> {
    let (name, manifest) = read_manifest(manifest_file)?;

    if validate {
        manifest.validate(&name).await?;
    }

    let _testnet = Testnet::setup(&mut m, &name, &manifest).await?;

    Ok(())
}

/// Remove a testnet.
async fn remove(mut m: DockerMaterializer, id: TestnetId) -> anyhow::Result<()> {
    m.remove(&TestnetName::new(id)).await
}

/// Read a manifest file; use its file name as the testnet name.
fn read_manifest(manifest_file: &Path) -> anyhow::Result<(TestnetName, Manifest)> {
    let testnet_id = manifest_file
        .file_stem()
        .ok_or_else(|| anyhow!("manifest file has no stem"))?
        .to_string_lossy()
        .to_string();

    let name = TestnetName::new(testnet_id);

    let manifest = Manifest::from_file(manifest_file)?;

    Ok((name, manifest))
}
