// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::HashMap;

use anyhow::{anyhow, bail, Context};
use bollard::{
    network::{CreateNetworkOptions, ListNetworksOptions},
    service::{Network, NetworkCreateResponse},
};

use crate::TestnetName;

use super::{DockerConstruct, DockerWithDropHandle};

pub struct DockerNetwork {
    dh: DockerWithDropHandle,
    /// There is a single docker network created for the entire testnet.
    testnet_name: TestnetName,
    network: DockerConstruct,
}

impl DockerNetwork {
    pub fn testnet_name(&self) -> &TestnetName {
        &self.testnet_name
    }

    pub fn network(&self) -> &DockerConstruct {
        &self.network
    }

    /// Check if an externally managed network already exists;
    /// if not, create a new docker network for the testnet.
    pub async fn get_or_create(
        dh: DockerWithDropHandle,
        testnet_name: TestnetName,
    ) -> anyhow::Result<Self> {
        let network_name = testnet_name.path().to_string_lossy().to_string();

        let mut filters = HashMap::new();
        filters.insert("name".to_string(), vec![network_name.clone()]);

        let networks: Vec<Network> = dh
            .docker
            .list_networks(Some(ListNetworksOptions { filters }))
            .await
            .context("failed to list docker networks")?;

        // let networks = networks
        //     .into_iter()
        //     .filter(|n| n.name.as_ref() == Some(&network_name))
        //     .collect::<Vec<_>>();

        let (id, external) = match networks.len() {
            0 => {
                let network: NetworkCreateResponse = dh
                    .docker
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
            dh,
            testnet_name,
            network: DockerConstruct {
                id,
                name: network_name,
                external,
            },
        })
    }
}

impl Drop for DockerNetwork {
    fn drop(&mut self) {
        if !self.network.external {
            let network_name = self.network.name.clone();
            let docker = self.dh.docker.clone();
            self.dh.drop_handle.spawn(async move {
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
    use std::time::Duration;

    use super::DockerNetwork;
    use crate::{docker::DockerWithDropHandle, TestnetName};

    #[tokio::test]
    async fn test_network() {
        let tn = TestnetName::new("test-network");

        let docker = Docker::connect_with_local_defaults().expect("failed to connect to docker");
        let dh = DockerWithDropHandle::from_current(docker.clone());

        let n1 = DockerNetwork::get_or_create(dh.clone(), tn.clone())
            .await
            .expect("failed to create network");

        let n2 = DockerNetwork::get_or_create(dh.clone(), tn.clone())
            .await
            .expect("failed to get network");

        assert!(
            !n1.network.external,
            "when created, the network should not be external"
        );
        assert!(
            n2.network.external,
            "when already exists, the network should be external"
        );
        assert_eq!(n1.network.id, n2.network.id);
        assert_eq!(n1.network.name, n2.network.name);
        assert_eq!(n1.network.name, "testnets/test-network");

        let id = n1.network.id.clone();

        let exists = || async {
            tokio::time::sleep(Duration::from_millis(250)).await;
            let ns = docker.list_networks::<String>(None).await.unwrap();
            ns.iter().any(|n| n.id == Some(id.clone()))
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
