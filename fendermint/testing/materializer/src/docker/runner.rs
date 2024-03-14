// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

use anyhow::{bail, Context};
use bollard::{
    container::{
        AttachContainerOptions, AttachContainerResults, Config, CreateContainerOptions,
        RemoveContainerOptions,
    },
    network::ConnectNetworkOptions,
    secret::{ContainerInspectResponse, HostConfig, PortBinding},
    Docker,
};
use futures::StreamExt;

use crate::NodeName;

use super::{
    container::DockerContainer,
    dropper::{DropChute, DropPolicy},
    DockerConstruct, DockerNetwork, Volumes,
};

pub struct DockerRunner {
    docker: Docker,
    dropper: DropChute,
    drop_policy: DropPolicy,
    node_name: NodeName,
    user: u32,
    image: String,
    volumes: Volumes,
}

impl DockerRunner {
    pub fn new(
        docker: Docker,
        dropper: DropChute,
        drop_policy: DropPolicy,
        node_name: NodeName,
        user: u32,
        image: &str,
        volumes: Volumes,
    ) -> Self {
        Self {
            docker,
            dropper,
            drop_policy,
            node_name,
            user,
            image: image.to_string(),
            volumes,
        }
    }

    // Tag containers with resource names.
    fn labels(&self) -> HashMap<String, String> {
        [
            ("testnet", self.node_name.testnet().path()),
            ("node", self.node_name.path()),
        ]
        .into_iter()
        .map(|(n, p)| (n.to_string(), p.to_string_lossy().to_string()))
        .collect()
    }

    /// Run a short lived container.
    pub async fn run_cmd(&self, cmd: &str) -> anyhow::Result<Vec<String>> {
        let cmdv = cmd.split(' ').map(|s| s.to_string()).collect();
        let config = Config {
            image: Some(self.image.clone()),
            user: Some(self.user.to_string()),
            cmd: Some(cmdv),
            attach_stderr: Some(true),
            attach_stdout: Some(true),
            tty: Some(true),
            labels: Some(self.labels()),
            host_config: Some(HostConfig {
                // We'll remove it explicitly at the end after collecting the output.
                auto_remove: Some(false),
                init: Some(true),
                binds: Some(
                    self.volumes
                        .iter()
                        .map(|(h, c)| format!("{}:{c}", h.to_string_lossy()))
                        .collect(),
                ),
                ..Default::default()
            }),
            ..Default::default()
        };

        let id = self
            .docker
            .create_container::<&str, _>(None, config)
            .await
            .context("failed to create container")?
            .id;

        let AttachContainerResults { mut output, .. } = self
            .docker
            .attach_container::<String>(
                &id,
                Some(AttachContainerOptions {
                    stdout: Some(true),
                    stderr: Some(true),
                    stream: Some(true),
                    ..Default::default()
                }),
            )
            .await
            .context("failed to attach to container")?;

        self.docker
            .start_container::<&str>(&id, None)
            .await
            .context("failed to start container")?;

        // Collect docker attach output
        let mut out = Vec::new();
        while let Some(Ok(output)) = output.next().await {
            out.push(output.to_string());
        }

        eprintln!("NODE: {}", self.node_name);
        eprintln!("CMD: {cmd}");
        for o in out.iter() {
            eprint!("OUT: {o}");
        }
        eprintln!("---");

        let inspect: ContainerInspectResponse = self
            .docker
            .inspect_container(&id, None)
            .await
            .context("failed to inspect container")?;

        self.docker
            .remove_container(
                &id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await?;

        if let Some(state) = inspect.state {
            let exit_code = state.exit_code.unwrap_or_default();
            if exit_code != 0 {
                bail!(
                    "ctonainer exited with code {exit_code}: {}",
                    state.error.unwrap_or_default()
                );
            }
        }

        Ok(out)
    }

    /// Create a container to be started later.
    pub async fn create(
        &self,
        name: String,
        network: &DockerNetwork,
        // Host <-> Container port mappings
        ports: Vec<(u32, u32)>,
        entrypoint: Vec<String>,
    ) -> anyhow::Result<DockerContainer> {
        let config = Config {
            hostname: Some(name.clone()),
            image: Some(self.image.clone()),
            user: Some(self.user.to_string()),
            entrypoint: Some(entrypoint),
            labels: Some(self.labels()),
            cmd: None,
            host_config: Some(HostConfig {
                init: Some(true),
                binds: Some(
                    self.volumes
                        .iter()
                        .map(|(h, c)| format!("{}:{c}", h.to_string_lossy()))
                        .collect(),
                ),
                port_bindings: Some(
                    ports
                        .into_iter()
                        .flat_map(|(h, c)| {
                            let binding = PortBinding {
                                host_ip: None,
                                host_port: Some(h.to_string()),
                            };
                            // Emitting both TCP and UDP, just in case.
                            vec![
                                (format!("{c}/tcp"), Some(vec![binding.clone()])),
                                (format!("{c}/udp"), Some(vec![binding])),
                            ]
                        })
                        .collect(),
                ),
                ..Default::default()
            }),
            ..Default::default()
        };

        let id = self
            .docker
            .create_container::<String, _>(
                Some(CreateContainerOptions {
                    name: name.clone(),
                    ..Default::default()
                }),
                config,
            )
            .await
            .context("failed to create container")?
            .id;

        eprintln!("NODE: {}", self.node_name);
        eprintln!("CREATED CONTAINER: {} ({})", name, id);
        eprintln!("---");

        // host_config.network_mode should work as well.
        self.docker
            .connect_network(
                network.network_name(),
                ConnectNetworkOptions {
                    container: id.clone(),
                    ..Default::default()
                },
            )
            .await
            .context("failed to connect container to network")?;

        Ok(DockerContainer::new(
            self.docker.clone(),
            self.dropper.clone(),
            DockerConstruct {
                id,
                name,
                keep: self.drop_policy.keep(true),
            },
        ))
    }
}
