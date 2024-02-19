// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, bail, Context};
use std::collections::HashMap;

use bollard::{
    container::{ListContainersOptions, RemoveContainerOptions, StopContainerOptions},
    service::ContainerSummary,
};

use super::{DockerConstruct, DockerWithDropHandle};

pub struct DockerContainer {
    pub dh: DockerWithDropHandle,
    pub container: DockerConstruct,
}

impl DockerContainer {
    /// Get a container by name, if it exists.
    pub async fn get(dh: &DockerWithDropHandle, name: String) -> anyhow::Result<Option<Self>> {
        let mut filters = HashMap::new();
        filters.insert("name".to_string(), vec![name.clone()]);

        let containers: Vec<ContainerSummary> = dh
            .docker
            .list_containers(Some(ListContainersOptions {
                all: true,
                filters,
                ..Default::default()
            }))
            .await
            .context("failed to list docker containers")?;

        match containers.len() {
            0 => Ok(None),
            1 => {
                let id = containers[0]
                    .id
                    .clone()
                    .ok_or_else(|| anyhow!("docker container {name} has no id"))?;

                Ok(Some(Self {
                    dh: dh.clone(),
                    container: DockerConstruct {
                        id,
                        name,
                        external: true,
                    },
                }))
            }
            n => bail!("there are multiple docker container with the same name: {name}"),
        }
    }
}

impl Drop for DockerContainer {
    fn drop(&mut self) {
        if !self.container.external {
            let container_name = self.container.name.clone();
            let docker = self.dh.docker.clone();
            self.dh.drop_handle.spawn(async move {
                if let Err(e) = docker
                    .stop_container(&container_name, Some(StopContainerOptions { t: 5 }))
                    .await
                {
                    tracing::error!(
                        error = e.to_string(),
                        container_name,
                        "failed to stop docker container"
                    );
                }
                if let Err(e) = docker
                    .remove_container(
                        &container_name,
                        Some(RemoveContainerOptions {
                            force: true,
                            v: true,
                            link: true,
                        }),
                    )
                    .await
                {
                    tracing::error!(
                        error = e.to_string(),
                        container_name,
                        "failed to remove docker container"
                    );
                }
            });
        }
    }
}
