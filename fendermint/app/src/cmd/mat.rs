// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::PathBuf;

use anyhow::anyhow;
use fendermint_app_options::mat::*;
use fendermint_materializer::{
    docker::DockerMaterializer, manifest::Manifest, testnet::Testnet, TestnetId, TestnetName,
};

use crate::cmd;

cmd! {
  MaterializerArgs(self) {
    let m = DockerMaterializer::new(&self.data_dir, self.seed)?;
    match &self.command {
        MaterializerCommands::Setup(args) => args.exec(m).await,
        MaterializerCommands::Remove(args) => args.exec(m).await,
    }
  }
}

cmd! {
  MaterializerSetupArgs(self, m: DockerMaterializer) {
    setup(m, self.manifest_file.clone()).await
  }
}

cmd! {
  MaterializerRemoveArgs(self, m: DockerMaterializer) {
    remove(m, self.testnet_id.clone()).await
  }
}

async fn setup(mut m: DockerMaterializer, manifest_file: PathBuf) -> anyhow::Result<()> {
    let testnet_id = manifest_file
        .file_stem()
        .ok_or_else(|| anyhow!("manifest file has no stem"))?
        .to_string_lossy()
        .to_string();

    let name = TestnetName::new(testnet_id);

    let manifest = Manifest::from_file(&manifest_file)?;

    let _testnet = Testnet::setup(&mut m, &name, &manifest).await?;

    Ok(())
}

async fn remove(mut m: DockerMaterializer, id: TestnetId) -> anyhow::Result<()> {
    m.remove(&TestnetName::new(id)).await
}
