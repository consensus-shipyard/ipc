// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::{os::unix::fs::MetadataExt, path::Path};

use anyhow::Context;
use bollard::{
    container::{Config, RemoveContainerOptions},
    service::HostConfig,
    Docker,
};

use super::{container::DockerContainer, DockerMaterials, DockerPortRange, DockerWithDropHandle};
use crate::{materializer::NodeConfig, NodeName};

const COMETBFT_IMAGE: &str = "cometbft/cometbft:v0.37.x";
const FENDERMINT_IMAGE: &str = "fendermint:latest";

/// A Node consists of multiple docker containers.
pub struct DockerNode {
    /// Logical name of the node in the subnet hierarchy.
    node_name: NodeName,
    fendermint: DockerContainer,
    cometbft: DockerContainer,
    ethapi: Option<DockerContainer>,
}

impl DockerNode {
    pub async fn get_or_create<'a>(
        root: impl AsRef<Path>,
        dh: DockerWithDropHandle,
        node_name: &NodeName,
        node_config: NodeConfig<'a, DockerMaterials>,
        port_range: DockerPortRange,
    ) -> anyhow::Result<Self> {
        let fendermint_name = container_name(node_name, "fendermint");
        let cometbft_name = container_name(node_name, "cometbft");
        let ethapi_name = container_name(node_name, "ethapi");

        let fendermint = DockerContainer::get(&dh, fendermint_name).await?;
        let cometbft = DockerContainer::get(&dh, cometbft_name).await?;
        let ethapi = DockerContainer::get(&dh, ethapi_name).await?;

        // Directory for the node's data volumes
        let node_dir = root.as_ref().join(node_name);
        std::fs::create_dir_all(&node_dir).context("failed to create node dir")?;

        // Get the current user ID to use with docker containers.
        let user = node_dir.metadata()?.uid();

        // Use the subnet genesis file.

        // Create a directory for keys
        let keys_dir = node_dir.join("keys");
        if !keys_dir.exists() {
            std::fs::create_dir(&keys_dir)?;

            // Make the validator key available for the init script.
            if let Some(v) = node_config.validator {
                let validator_key_path = v.secret_key_path();
                std::fs::copy(validator_key_path, keys_dir.join("validator_key.sk"))
                    .context("failed to copy validator key")?;
            }
        }

        // Create a directory for cometbft
        let cometbft_dir = node_dir.join("cometbft");
        if !cometbft_dir.exists() {
            std::fs::create_dir(&cometbft_dir)?;

            // Init cometbft to establish the network key.
            let config = Config {
                image: Some(COMETBFT_IMAGE.to_string()),
                user: Some(user.to_string()),
                host_config: Some(HostConfig {
                    // Volumes
                    binds: Some(vec![format!(
                        "{}:/cometbft",
                        cometbft_dir.to_string_lossy()
                    )]),
                    ..Default::default()
                }),
                cmd: Some(vec!["init".to_string()]),
                ..Default::default()
            };

            docker_run(&dh.docker, config)
                .await
                .context("cannot init cometbft")?;

            // Convert fendermint genesis to cometbft.
            // Convert validator private key to cometbft.
            // Create a network key for the resolver.
            let config = Config {
                image: Some(FENDERMINT_IMAGE.to_string()),
                user: Some(user.to_string()),
                host_config: Some(HostConfig {
                    // Volumes for fendermint-init.sh
                    binds: Some(vec![
                        format!(
                            "{}:/scripts/fendermint-init.sh",
                            root.as_ref()
                                .join("scripts")
                                .join("fendermint-init.sh")
                                .to_string_lossy()
                        ),
                        format!("{}:/data/keys", keys_dir.to_string_lossy()),
                        format!("{}:/data/cometbft", cometbft_dir.to_string_lossy()),
                        format!(
                            "{}:/data/genesis.json",
                            node_config.genesis.path.to_string_lossy()
                        ),
                    ]),
                    ..Default::default()
                }),
                entrypoint: Some(vec!["/scripts/fendermint-init.sh".to_string()]),
                ..Default::default()
            };

            docker_run(&dh.docker, config)
                .await
                .context("cannot init fendermint")?;
        }

        // Create a directory for fendermint
        let fendermint_dir = node_dir.join("fendermint");
        if !fendermint_dir.exists() {
            std::fs::create_dir(&fendermint_dir)?;
        }

        // If there is no static env var file, create one with all the common variables.
        todo!();

        // If there is no dynamic env var file, create an empty one.
        todo!();

        if fendermint.is_none() {
            // Create a fendermint container mounting:
            // - the fendermint directory
            // - the docker-entry
            // - the env var files

            //         let fendermint = match fendermint {
            //             Some(container) => container,
            //             None => dh
            //                 .docker
            //                 .create_container(Some(CreateContainerOptions {
            //                     name: fendermint_name.clone(),
            //                     ..Default::default()
            //                 }), Config {
            //                     hostname: Some(fendermint_name.clone()),
            // user,
            // host_config: Some(HostConfig {
            //     init: Some(true ),
            //     binds: ,
            //     port_bindings: ,
            // })
            //                 })
            //                 .await
            //                 .context("failed to create fendermint container")?,
            //         };
            todo!();
        }

        if cometbft.is_none() {
            // Create a CometBFT container mounting:
            // - the cometbft directory
            // - the docker-entry
            // - the env var files
        }

        if node_config.ethapi && ethapi.is_none() {
            // Create a ethapi container mounting:
            // - the docker-entry
            // - the env var files
        }

        // Construct the DockerNode
        todo!()
    }
}

/// Create a container name from a node name and a logical container name, e.g. "cometbft".
///
/// Ideally the container name should be usable as a hostname as well.
///
/// Alternatively we could hash the name, and use the original as a tag.
fn container_name(node_name: &NodeName, container: &str) -> String {
    let name = node_name
        .path()
        .join(container)
        .to_string_lossy()
        .to_string();

    name.replace("/", "__")
}

/// Run a short lived container.
async fn docker_run(docker: &Docker, mut create_config: Config<String>) -> anyhow::Result<()> {
    create_config.attach_stderr = Some(true);
    create_config.attach_stdout = Some(true);
    if let Some(ref mut host_config) = create_config.host_config {
        host_config.auto_remove = Some(true);
        host_config.init = Some(true);
    }

    let id = docker
        .create_container::<&str, _>(None, create_config)
        .await
        .context("failed to create container")?
        .id;

    docker
        .start_container::<&str>(&id, None)
        .await
        .context("failed to start container")?;

    // TODO: Output?

    docker
        .remove_container(
            &id,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await?;

    Ok(())
}
