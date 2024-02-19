// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use super::{container::DockerContainer, DockerMaterials, DockerPortRange, DockerWithDropHandle};
use crate::{materializer::NodeConfig, NodeName};

/// A Node consists of multiple docker containers.
pub struct DockerNode {
    /// Logical name of the node in the subnet hierarchy.
    node_name: NodeName,
    fendermint: DockerContainer,
    cometbft: DockerContainer,
    ethapi: Option<DockerContainer>,
}

impl DockerNode {
    /// Check if externally managed containers already exist;
    /// if not, create new containers for the node.
    pub async fn get_or_create<'a>(
        dh: DockerWithDropHandle,
        node_name: &NodeName,
        node_config: NodeConfig<'a, DockerMaterials>,
        port_range: DockerPortRange,
    ) -> anyhow::Result<Self> {
        let fendermint_name = container_name(node_name, "fendermint");
        let cometbft_name = container_name(node_name, "cometbft");
        let ethapi_name = if node_config.ethapi {
            Some(container_name(node_name, "ethapi"))
        } else {
            None
        };

        let fendermint = DockerContainer::get(&dh, fendermint_name).await?;
        let cometbft = DockerContainer::get(&dh, cometbft_name).await?;
        let ethapi = if let Some(n) = ethapi_name {
            DockerContainer::get(&dh, n).await?
        } else {
            None
        };

        // Create a common env file for all the containers.
        let env_path = node_name.path().join(".env");

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
