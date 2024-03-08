// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::{fmt::Display, path::Path};

use anyhow::Context;
use bollard::Docker;

use crate::{
    docker::{
        runner::{split_cmd, DockerRunner},
        user_id, FENDERMINT_IMAGE,
    },
    materializer::SubmitConfig,
    materials::{DefaultAccount, DefaultSubnet},
    RelayerName, ResourceHash, TestnetResource,
};

use super::{
    container::DockerContainer, dropper::DropChute, DockerMaterials, DockerNode, DropPolicy,
};

pub struct DockerRelayer {
    relayer_name: RelayerName,
    relayer: DockerContainer,
}

impl Display for DockerRelayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.relayer_name, f)
    }
}

impl DockerRelayer {
    pub async fn get_or_create<'a>(
        root: impl AsRef<Path>,
        docker: Docker,
        dropper: DropChute,
        drop_policy: &DropPolicy,
        relayer_name: &RelayerName,
        parent_submit_config: &SubmitConfig<'a, DockerMaterials>,
        subnet: &DefaultSubnet,
        submitter: &DefaultAccount,
        follow_node: &DockerNode,
    ) -> anyhow::Result<Self> {
        let container_name = container_name(relayer_name);

        // If the container exists, return it.
        if let Some(relayer) = DockerContainer::get(
            docker.clone(),
            dropper.clone(),
            drop_policy,
            container_name.clone(),
        )
        .await?
        {
            return Ok(Self {
                relayer_name: relayer_name.clone(),
                relayer,
            });
        }

        // We'll need to mount the IPC configuration for the relayer.
        let ipc_dir = root.as_ref().join(subnet.name.testnet()).join("ipc");

        let user = user_id(&ipc_dir)?;
        let network_name = follow_node.network_name().clone();

        // TODO: Logs?
        let volumes = vec![(ipc_dir, "/fendermint/.ipc")];

        let creator = DockerRunner::new(
            docker,
            dropper,
            drop_policy.clone(),
            relayer_name.clone(),
            user,
            FENDERMINT_IMAGE,
            volumes,
            Some(network_name),
        );

        // TODO: Do we need to use any env vars with the relayer?
        let entrypoint = split_cmd(&format!(
            "ipc-cli \
                --config-path /fendermint/.ipc/config.toml \
                checkpoint relayer \
                    --subnet {} \
                    --submitter {:?} \
            ",
            subnet.subnet_id,
            submitter.eth_addr()
        ));

        let relayer = creator
            .create(container_name, Default::default(), entrypoint)
            .await
            .context("failed to create relayer")?;

        Ok(Self {
            relayer_name: relayer_name.clone(),
            relayer,
        })
    }
}

/// Create a container name from the relayer name.
///
/// It consists of `{relayer-id}-relayer-{hash(relayer-name)}`
fn container_name(relayer_name: &RelayerName) -> String {
    let relayer_id = relayer_name
        .path()
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let hash = ResourceHash::digest(relayer_name.path_string());
    let hash = hash.to_string();
    let hash = &hash.as_str()[..6];
    format!("{relayer_id}-relayer-{}", hash)
}
