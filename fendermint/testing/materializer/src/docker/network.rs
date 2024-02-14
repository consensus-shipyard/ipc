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

impl Drop for DockerNetwork {
    fn drop(&mut self) {
        if !self.external {
            let network_name = self.network_name.clone();
            let docker = self.docker.clone();
            // TODO: Handle this in a more linearlised way, e.g. it could happen that we are still stopping and
            // removing containers when we try to remove the network, which will thus fail. Maybe the materializer
            // should have a background worker listening to these events and execute commands one after the other.
            // Or maybe it should have a single threaded tokio runtime that we can use with `block_on`. If that
            // runtime isn't the one that is being used to run all the regular tasks, perhaps it can block here.
            tokio::runtime::Handle::current().spawn(async move {
                if let Err(e) = docker.remove_network(&network_name).await {
                    tracing::error!(
                        error = e.to_string(),
                        network_name,
                        "failed to remove docker network"
                    );
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use bollard::Docker;

    use super::DockerNetwork;
    use crate::TestnetName;

    #[tokio::test]
    async fn test_network() {
        let tn = TestnetName::new("test-network");

        let docker = Docker::connect_with_local_defaults().expect("failed to connect to docker");

        let n1 = DockerNetwork::get_or_create(docker.clone(), tn.clone())
            .await
            .expect("failed to create network");

        let n2 = DockerNetwork::get_or_create(docker.clone(), tn.clone())
            .await
            .expect("failed to get network");

        assert_eq!(
            n1.external, false,
            "when created, the network should not be external"
        );
        assert_eq!(
            n2.external, true,
            "when already exists, the network should be external"
        );
        assert_eq!(n1.id, n2.id);
        assert_eq!(n1.network_name, n2.network_name);
        assert_eq!(n1.network_name, "testnets/test-network");

        let id = n1.id.clone();

        let exists = || async {
            let ns = docker.list_networks::<String>(None).await.unwrap();
            ns.iter().find(|n| n.id == Some(id.clone())).is_some()
        };

        drop(n2);
        assert!(exists().await, "network still exists after n2 dropped");

        drop(n1);
        assert!(
            !exists().await,
            "network should be removed when n1 is dropped"
        );
    }
}
