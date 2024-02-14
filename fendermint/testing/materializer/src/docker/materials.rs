// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::{
    materials::{DefaultAccount, DefaultDeployment, DefaultGenesis, DefaultSubnet, Materials},
    NodeName, TestnetName,
};

pub struct DockerMaterials;

impl Materials for DockerMaterials {
    type Deployment = DefaultDeployment;
    type Account = DefaultAccount;
    type Genesis = DefaultGenesis;
    type Subnet = DefaultSubnet;

    type Network = DockerNetwork;
    type Node = DockerNode;
    type Relayer = DockerRelayer;
}

pub struct DockerNetwork {
    name: TestnetName,
    /// Indicate whether this resource is managed outside the test.
    external: bool,
    network_name: String,
}

/// A docker node consists of multiple containers.
pub struct DockerNode {
    name: NodeName,
    /// Indicate whether this resource is managed outside the test.
    external: bool,
    cometbft_container_name: String,
    fendermint_container_name: String,
    ethapi_container_name: Option<String>,
    // TODO: Ports
}

pub struct DockerRelayer {
    name: NodeName,
    /// Indicate whether this resource is managed outside the test.
    external: bool,
    relayer_container_name: String,
}
