// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, Context};
use std::collections::HashMap;

use bollard::{container::ListContainersOptions, service::ContainerSummary, Docker};

use super::{
    dropper::{DropCommand, DropHandle},
    DockerConstruct,
};

/// Time to wait before killing the container if it doesn't want to stop.
const KILL_TIMEOUT_SECS: i64 = 5;

pub struct DockerContainer {
    docker: Docker,
    dropper: DropHandle,
    container: DockerConstruct,
}

impl DockerContainer {
    pub fn new(docker: Docker, dropper: DropHandle, container: DockerConstruct) -> Self {
        Self {
            docker,
            dropper,
            container,
        }
    }

    pub fn hostname(&self) -> &str {
        &self.container.name
    }

    /// Get a container by name, if it exists.
    pub async fn get(
        docker: Docker,
        dropper: DropHandle,
        name: String,
    ) -> anyhow::Result<Option<Self>> {
        let mut filters = HashMap::new();
        filters.insert("name".to_string(), vec![name.clone()]);

        let containers: Vec<ContainerSummary> = docker
            .list_containers(Some(ListContainersOptions {
                all: true,
                filters,
                ..Default::default()
            }))
            .await
            .context("failed to list docker containers")?;

        match containers.first() {
            None => Ok(None),
            Some(container) => {
                let id = container
                    .id
                    .clone()
                    .ok_or_else(|| anyhow!("docker container {name} has no id"))?;

                Ok(Some(Self {
                    docker,
                    dropper,
                    container: DockerConstruct {
                        id,
                        name,
                        external: true,
                    },
                }))
            }
        }
    }

    /// Start the container, unless it's already running.
    pub async fn start(&self) -> anyhow::Result<()> {
        // TODO: Check if the container is running.

        self.docker
            .start_container::<&str>(&self.container.id, None)
            .await
            .with_context(|| {
                format!(
                    "failed to start container: {} ({})",
                    self.container.name, self.container.id
                )
            })?;

        Ok(())
    }
}

impl Drop for DockerContainer {
    fn drop(&mut self) {
        if !self.container.external {
            if self
                .dropper
                .send(DropCommand::DropContainer(self.container.name.clone()))
                .is_err()
            {
                tracing::error!(
                    container_name = self.container.name,
                    "dropper no longer listening"
                );
            }
        }
    }
}
