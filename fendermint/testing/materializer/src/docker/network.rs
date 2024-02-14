// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

use anyhow::{anyhow, bail, Context};
use bollard::{
    network::{CreateNetworkOptions, ListNetworksOptions},
    service::{Network, NetworkCreateResponse},
    Docker,
};

use crate::TestnetName;

pub struct DockerNetwork {
    docker: Docker,
    testnet_name: TestnetName,
    network_name: String,
    /// Indicate whether this resource is managed outside the test.
    external: bool,
    id: String,
}

impl DockerNetwork {
    pub async fn get_or_create(docker: Docker, testnet_name: TestnetName) -> anyhow::Result<Self> {
        let network_name = testnet_name.path().to_string_lossy().to_string();

        let mut filters = HashMap::new();
        filters.insert("name".to_string(), vec![network_name.clone()]);

        let networks: Vec<Network> = docker
            .list_networks(Some(ListNetworksOptions { filters }))
            .await
            .context("failed to list docker networks")?;

        let networks = networks
            .into_iter()
            .filter(|n| n.name.as_ref() == Some(&network_name))
            .collect::<Vec<_>>();

        let (id, external) = match networks.len() {
            0 => {
                let network: NetworkCreateResponse = docker
                    .create_network(CreateNetworkOptions {
                        name: network_name.clone(),
                        ..Default::default()
                    })
                    .await
                    .context("failed to create docker network")?;

                let id = network
                    .id
                    .clone()
                    .ok_or_else(|| anyhow!("created docker network has no id"))?;

                (id, false)
            }
            1 => {
                let id = networks[0]
                    .id
                    .clone()
                    .ok_or_else(|| anyhow!("docker network {network_name} has no id"))?;

                (id, true)
            }
            n => bail!("there are multiple docker networks with the same name: {network_name}"),
        };

        Ok(Self {
            docker,
            testnet_name,
            network_name,
            external,
            id,
        })
    }
}

// TODO: Drop
// TODO: Test
