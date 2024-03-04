// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use async_trait::async_trait;
use ethers::types::H160;
use fendermint_vm_genesis::Collateral;
use fvm_shared::econ::TokenAmount;
use std::{collections::BTreeMap, fmt::Display};
use tendermint_rpc::Url;

use crate::{
    manifest::Balance,
    materializer::{Materializer, NodeConfig, SubmitConfig, SubnetConfig},
    materials::Materials,
    AccountName, NodeName, RelayerName, ResourceHash, SubnetName, TestnetName,
};

/// Simple in-memory logging to help debug manifests.
pub struct LoggingMaterializer<R> {
    tag: String,
    inner: R,
}

impl<R> LoggingMaterializer<R> {
    pub fn new(inner: R, tag: String) -> Self {
        Self { inner, tag }
    }
}

#[async_trait]
impl<M, R> Materializer<M> for LoggingMaterializer<R>
where
    M: Materials + Send + Sync + 'static,
    R: Materializer<M> + Send + Sync,
    M::Network: Display,
    M::Deployment: Display,
    M::Account: Display,
    M::Genesis: Display,
    M::Subnet: Display,
    M::Node: Display,
    M::Relayer: Display,
{
    async fn create_network(&mut self, testnet_name: &TestnetName) -> anyhow::Result<M::Network> {
        tracing::info!(self.tag, %testnet_name, "create_network");
        self.inner.create_network(testnet_name).await
    }

    fn create_account(&mut self, account_name: &AccountName) -> anyhow::Result<M::Account> {
        tracing::info!(self.tag, %account_name, "create_account");
        self.inner.create_account(account_name)
    }

    async fn fund_from_faucet<'s, 'a>(
        &'s mut self,
        account: &'a M::Account,
        reference: Option<ResourceHash>,
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        tracing::info!(self.tag, %account, "fund_from_faucet");
        self.inner.fund_from_faucet(account, reference).await
    }

    async fn new_deployment<'s, 'a>(
        &'s mut self,
        subnet_name: &SubnetName,
        deployer: &'a M::Account,
        urls: Vec<Url>,
    ) -> anyhow::Result<M::Deployment>
    where
        's: 'a,
    {
        tracing::info!(self.tag, %subnet_name, %deployer, "new_deployment");
        self.inner.new_deployment(subnet_name, deployer, urls).await
    }

    fn existing_deployment(
        &mut self,
        subnet_name: &SubnetName,
        gateway: H160,
        registry: H160,
    ) -> anyhow::Result<M::Deployment> {
        tracing::info!(self.tag, %subnet_name, "existing_deployment");
        self.inner
            .existing_deployment(subnet_name, gateway, registry)
    }

    fn default_deployment(&mut self, subnet_name: &SubnetName) -> anyhow::Result<M::Deployment> {
        tracing::info!(self.tag, %subnet_name, "default_deployment");
        self.inner.default_deployment(subnet_name)
    }

    fn create_root_genesis<'a>(
        &mut self,
        subnet_name: &SubnetName,
        validators: BTreeMap<&'a M::Account, Collateral>,
        balances: BTreeMap<&'a M::Account, Balance>,
    ) -> anyhow::Result<M::Genesis> {
        tracing::info!(self.tag, %subnet_name, "create_root_genesis");
        self.inner
            .create_root_genesis(subnet_name, validators, balances)
    }

    async fn create_node<'s, 'a>(
        &'s mut self,
        node_name: &NodeName,
        node_config: NodeConfig<'a, M>,
    ) -> anyhow::Result<M::Node>
    where
        's: 'a,
    {
        tracing::info!(self.tag, %node_name, "create_node");
        self.inner.create_node(node_name, node_config).await
    }

    async fn start_node<'s, 'a>(
        &'s mut self,
        node: &'a M::Node,
        seed_nodes: &'a [&'a M::Node],
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        tracing::info!(self.tag, %node, "start_node");
        self.inner.start_node(node, seed_nodes).await
    }

    async fn create_subnet<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, M>,
        subnet_name: &SubnetName,
        subnet_config: SubnetConfig<'a, M>,
    ) -> anyhow::Result<M::Subnet>
    where
        's: 'a,
    {
        tracing::info!(self.tag, %subnet_name, "create_subnet");
        self.inner
            .create_subnet(parent_submit_config, subnet_name, subnet_config)
            .await
    }

    async fn fund_subnet<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, M>,
        account: &'a M::Account,
        subnet: &'a M::Subnet,
        amount: TokenAmount,
        reference: Option<ResourceHash>,
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        tracing::info!(self.tag, %subnet, %account, "fund_subnet");
        self.inner
            .fund_subnet(parent_submit_config, account, subnet, amount, reference)
            .await
    }

    async fn join_subnet<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, M>,
        account: &'a M::Account,
        subnet: &'a M::Subnet,
        collateral: Collateral,
        balance: Balance,
        reference: Option<ResourceHash>,
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        tracing::info!(self.tag, %subnet, %account, "join_subnet");
        self.inner
            .join_subnet(
                parent_submit_config,
                account,
                subnet,
                collateral,
                balance,
                reference,
            )
            .await
    }

    async fn create_subnet_genesis<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, M>,
        subnet: &'a M::Subnet,
    ) -> anyhow::Result<M::Genesis>
    where
        's: 'a,
    {
        tracing::info!(self.tag, %subnet, "create_subnet_genesis");
        self.inner
            .create_subnet_genesis(parent_submit_config, subnet)
            .await
    }

    async fn create_relayer<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, M>,
        relayer_name: &RelayerName,
        subnet: &'a M::Subnet,
        submitter: &'a M::Account,
        follow_node: &'a M::Node,
    ) -> anyhow::Result<M::Relayer>
    where
        's: 'a,
    {
        tracing::info!(self.tag, %relayer_name, "create_relayer");
        self.inner
            .create_relayer(
                parent_submit_config,
                relayer_name,
                subnet,
                submitter,
                follow_node,
            )
            .await
    }
}
