// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::{collections::BTreeMap, os::unix::fs::MetadataExt, path::Path};

use anyhow::Context;
use bollard::{
    container::{Config, CreateContainerOptions, RemoveContainerOptions},
    service::HostConfig,
    Docker,
};

use super::{container::DockerContainer, DockerMaterials, DockerPortRange, DockerWithDropHandle};
use crate::{materializer::NodeConfig, NodeName};

const DOCKER_ENTRY: &str = include_str!("../../scripts/docker-entry.sh");

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

        // Create a directory for cometbft
        let cometbft_dir = node_dir.join("cometbft");
        if !cometbft_dir.exists() {
            std::fs::create_dir(&cometbft_dir)?;
            // Init cometbft to establish the network key.
            todo!()
        }

        // Create a directory for fendermint
        let fendermint_dir = node_dir.join("fendermint");
        if !fendermint_dir.exists() {
            std::fs::create_dir(&fendermint_dir)?;
            // Export fendermint genesis file.
            // Convert fendermint genesis to cometbft.
            // Convert validator private key to cometbft.
            // Create a network key for the resolver.
            todo!()
        }

        // Export the docker entry point to an executable script.
        todo!();

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
async fn docker_run(docker: &Docker, create_config: Config<&str>) -> anyhow::Result<()> {
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
