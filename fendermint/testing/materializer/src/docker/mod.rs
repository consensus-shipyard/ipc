// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::Context;
use async_trait::async_trait;
use bollard::Docker;
use ethers::{
    core::rand::{rngs::StdRng, SeedableRng},
    types::H160,
};
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_genesis::Collateral;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use tendermint_rpc::Url;

use crate::{
    manifest::Balance,
    materializer::{Materializer, NodeConfig, SubmitConfig, SubnetConfig},
    materials::{DefaultAccount, DefaultDeployment, DefaultGenesis, DefaultSubnet, Materials},
    NodeName, RelayerName, ResourceHash, ResourceName, SubnetName, TestnetName,
};

mod network;
mod node;
mod relayer;

pub use network::DockerNetwork;
pub use node::DockerNode;
pub use relayer::DockerRelayer;

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
    rng: StdRng,
    docker: bollard::Docker,
}

impl DockerMaterializer {
    /// Create a materializer with a directory where all the
    /// testnets can live next to each other.
    pub fn new(dir: &Path, seed: u64) -> anyhow::Result<Self> {
        let docker =
            Docker::connect_with_local_defaults().context("failed to connect to Docker")?;

        Ok(Self {
            dir: dir.into(),
            rng: StdRng::seed_from_u64(seed),
            docker,
        })
    }

    /// Path to a directory based on a resource name.
    fn path<T: AsRef<ResourceName>>(&self, name: T) -> PathBuf {
        let name: &ResourceName = name.as_ref();
        self.dir.join(&name.0)
    }
}

#[async_trait]
impl Materializer<DockerMaterials> for DockerMaterializer {
    async fn create_network(
        &mut self,
        testnet_name: &TestnetName,
    ) -> anyhow::Result<<DockerMaterials as Materials>::Network> {
        DockerNetwork::get_or_create(self.docker.clone(), testnet_name.clone()).await
    }

    /// Create a new key-value pair, or return an existing one.
    fn create_account(
        &mut self,
        account_name: &crate::AccountName,
    ) -> anyhow::Result<DefaultAccount> {
        DefaultAccount::get_or_create(&mut self.rng, &self.dir, account_name)
    }

    async fn fund_from_faucet<'s, 'a>(
        &'s mut self,
        account: &'a DefaultAccount,
        reference: Option<ResourceHash>,
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        todo!("use curl or something to trigger the faucet")
    }

    async fn new_deployment<'s, 'a>(
        &'s mut self,
        subnet_name: &SubnetName,
        deployer: &'a DefaultAccount,
        urls: Vec<Url>,
    ) -> anyhow::Result<DefaultDeployment>
    where
        's: 'a,
    {
        todo!("use the deploy scripts to create a new IPC stack on L1")
    }

    fn existing_deployment(
        &mut self,
        subnet_name: &SubnetName,
        gateway: H160,
        registry: H160,
    ) -> anyhow::Result<DefaultDeployment> {
        Ok(DefaultDeployment {
            name: subnet_name.clone(),
            gateway: EthAddress::from(gateway),
            registry: EthAddress::from(registry),
        })
    }

    fn default_deployment(
        &mut self,
        subnet_name: &SubnetName,
    ) -> anyhow::Result<DefaultDeployment> {
        Ok(DefaultDeployment::builtin(subnet_name.clone()))
    }

    fn create_root_genesis<'a>(
        &mut self,
        subnet_name: &SubnetName,
        validators: BTreeMap<&'a DefaultAccount, Collateral>,
        balances: BTreeMap<&'a DefaultAccount, Balance>,
    ) -> anyhow::Result<DefaultGenesis> {
        todo!("construct an in-memory genesis file, optionally save it to file")
    }

    async fn create_node<'s, 'a>(
        &'s mut self,
        node_name: &NodeName,
        node_config: NodeConfig<'a, DockerMaterials>,
    ) -> anyhow::Result<DockerNode>
    where
        's: 'a,
    {
        todo!("docker-compose template")
    }

    async fn start_node<'s, 'a>(
        &'s mut self,
        node: &'a DockerNode,
        seed_nodes: &'a [&'a DockerNode],
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        todo!("docker-compose up")
    }

    async fn create_subnet<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, DockerMaterials>,
        subnet_name: &SubnetName,
        subnet_config: SubnetConfig<'a, DockerMaterials>,
    ) -> anyhow::Result<DefaultSubnet>
    where
        's: 'a,
    {
        todo!("use the ipc-cli to create a new subnet on the parent")
    }

    async fn fund_subnet<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, DockerMaterials>,
        account: &'a DefaultAccount,
        subnet: &'a DefaultSubnet,
        amount: fvm_shared::econ::TokenAmount,
        reference: Option<ResourceHash>,
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        todo!("use the ipc-cli to fund an existing subnet on the parent")
    }

    async fn join_subnet<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, DockerMaterials>,
        account: &'a DefaultAccount,
        subnet: &'a DefaultSubnet,
        collateral: fendermint_vm_genesis::Collateral,
        balance: Balance,
        reference: Option<ResourceHash>,
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        todo!("use the ipc-cli to join an existing subnet on the parent")
    }

    async fn create_subnet_genesis<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, DockerMaterials>,
        subnet: &'a DefaultSubnet,
    ) -> anyhow::Result<DefaultGenesis>
    where
        's: 'a,
    {
        todo!("use the fendermint CLI to fetch the genesis of a subnet from the parent")
    }

    async fn create_relayer<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, DockerMaterials>,
        relayer_name: &RelayerName,
        subnet: &'a DefaultSubnet,
        submitter: &'a DefaultAccount,
        follow_node: &'a DockerNode,
    ) -> anyhow::Result<DockerRelayer>
    where
        's: 'a,
    {
        todo!("docker run relayer unless it is already running")
    }
}
