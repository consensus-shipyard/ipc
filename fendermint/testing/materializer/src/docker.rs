// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use async_trait::async_trait;
use std::path::{Path, PathBuf};

use crate::{
    materializer::Materializer,
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

pub struct DockerMaterializer {
    dir: PathBuf,
}

impl DockerMaterializer {
    pub fn new(dir: &Path) -> Self {
        Self { dir: dir.into() }
    }
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

#[async_trait]
impl Materializer<DockerMaterials> for DockerMaterializer {
    async fn create_network(
        &mut self,
        testnet_name: &TestnetName,
    ) -> anyhow::Result<<DockerMaterials as Materials>::Network> {
        todo!()
    }

    fn create_account(
        &mut self,
        account_name: &crate::AccountName,
    ) -> anyhow::Result<<DockerMaterials as Materials>::Account> {
        todo!()
    }

    async fn fund_from_faucet<'s, 'a>(
        &'s mut self,
        account: &'a <DockerMaterials as Materials>::Account,
        reference: Option<crate::ResourceHash>,
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        todo!()
    }

    async fn new_deployment<'s, 'a>(
        &'s mut self,
        subnet_name: &crate::SubnetName,
        deployer: &'a <DockerMaterials as Materials>::Account,
        urls: Vec<tendermint_rpc::Url>,
    ) -> anyhow::Result<<DockerMaterials as Materials>::Deployment>
    where
        's: 'a,
    {
        todo!()
    }

    fn existing_deployment(
        &mut self,
        subnet_name: &crate::SubnetName,
        gateway: ethers::types::H160,
        registry: ethers::types::H160,
    ) -> anyhow::Result<<DockerMaterials as Materials>::Deployment> {
        todo!()
    }

    fn default_deployment(
        &mut self,
        subnet_name: &crate::SubnetName,
    ) -> anyhow::Result<<DockerMaterials as Materials>::Deployment> {
        todo!()
    }

    fn create_root_genesis<'a>(
        &mut self,
        subnet_name: &crate::SubnetName,
        validators: std::collections::BTreeMap<
            &'a <DockerMaterials as Materials>::Account,
            fendermint_vm_genesis::Collateral,
        >,
        balances: std::collections::BTreeMap<
            &'a <DockerMaterials as Materials>::Account,
            crate::manifest::Balance,
        >,
    ) -> anyhow::Result<<DockerMaterials as Materials>::Genesis> {
        todo!()
    }

    async fn create_node<'s, 'a>(
        &'s mut self,
        node_name: &NodeName,
        node_config: crate::materializer::NodeConfig<'a, DockerMaterials>,
    ) -> anyhow::Result<<DockerMaterials as Materials>::Node>
    where
        's: 'a,
    {
        todo!()
    }

    async fn start_node<'s, 'a>(
        &'s mut self,
        node: &'a <DockerMaterials as Materials>::Node,
        seed_nodes: &'a [&'a <DockerMaterials as Materials>::Node],
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        todo!()
    }

    async fn create_subnet<'s, 'a>(
        &'s mut self,
        parent_submit_config: &crate::materializer::SubmitConfig<'a, DockerMaterials>,
        subnet_name: &crate::SubnetName,
        subnet_config: crate::materializer::SubnetConfig<'a, DockerMaterials>,
    ) -> anyhow::Result<<DockerMaterials as Materials>::Subnet>
    where
        's: 'a,
    {
        todo!()
    }

    async fn fund_subnet<'s, 'a>(
        &'s mut self,
        parent_submit_config: &crate::materializer::SubmitConfig<'a, DockerMaterials>,
        account: &'a <DockerMaterials as Materials>::Account,
        subnet: &'a <DockerMaterials as Materials>::Subnet,
        amount: fvm_shared::econ::TokenAmount,
        reference: Option<crate::ResourceHash>,
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        todo!()
    }

    async fn join_subnet<'s, 'a>(
        &'s mut self,
        parent_submit_config: &crate::materializer::SubmitConfig<'a, DockerMaterials>,
        account: &'a <DockerMaterials as Materials>::Account,
        subnet: &'a <DockerMaterials as Materials>::Subnet,
        collateral: fendermint_vm_genesis::Collateral,
        balance: crate::manifest::Balance,
        reference: Option<crate::ResourceHash>,
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        todo!()
    }

    async fn create_subnet_genesis<'s, 'a>(
        &'s mut self,
        parent_submit_config: &crate::materializer::SubmitConfig<'a, DockerMaterials>,
        subnet: &'a <DockerMaterials as Materials>::Subnet,
    ) -> anyhow::Result<<DockerMaterials as Materials>::Genesis>
    where
        's: 'a,
    {
        todo!()
    }

    async fn create_relayer<'s, 'a>(
        &'s mut self,
        parent_submit_config: &crate::materializer::SubmitConfig<'a, DockerMaterials>,
        relayer_name: &crate::RelayerName,
        subnet: &'a <DockerMaterials as Materials>::Subnet,
        submitter: &'a <DockerMaterials as Materials>::Account,
        follow_node: &'a <DockerMaterials as Materials>::Node,
    ) -> anyhow::Result<<DockerMaterials as Materials>::Relayer>
    where
        's: 'a,
    {
        todo!()
    }
}
