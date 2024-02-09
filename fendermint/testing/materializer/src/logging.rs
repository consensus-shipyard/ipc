// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use async_trait::async_trait;
use fendermint_vm_genesis::Collateral;
use fvm_shared::{address::Address, econ::TokenAmount};
use std::collections::BTreeMap;
use std::fmt::Debug;

use crate::{
    manifest::Balance,
    materializer::{Materializer, Materials, NodeConfig, SubmitConfig, SubnetConfig},
    AccountName, NodeName, RelayerName, ResourceHash, SubnetName, TestnetName,
};

/// Simple in-memory logging to help debug manifests.
pub struct LoggingMaterializer<R> {
    inner: R,
}

impl<R> LoggingMaterializer<R> {
    pub fn new(inner: R) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<M, R> Materializer<M> for LoggingMaterializer<R>
where
    M: Materials + Send + Sync + 'static,
    R: Materializer<M> + Send + Sync,
    M::Network: Debug,
    M::Deployment: Debug,
    M::Account: Debug,
    M::Genesis: Debug,
    M::Subnet: Debug,
    M::Node: Debug,
    M::Relayer: Debug,
{
    async fn create_network<'s>(
        &'s mut self,
        testnet_name: &TestnetName,
    ) -> anyhow::Result<M::Network> {
        tracing::info!(?testnet_name, "create_network");
        self.inner.create_network(testnet_name).await
    }

    fn create_account<'s>(&'s mut self, account_name: &AccountName) -> anyhow::Result<M::Account> {
        tracing::info!(?account_name, "create_account");
        self.inner.create_account(account_name)
    }

    async fn fund_from_faucet<'s, 'a>(
        &'s mut self,
        account: &'a M::Account,
        reference: Option<ResourceHash>,
    ) -> anyhow::Result<()> {
        tracing::info!(?account, "fund_from_faucet");
        self.inner.fund_from_faucet(account, reference).await
    }

    async fn new_deployment<'s, 'a>(
        &'s mut self,
        subnet_name: &SubnetName,
        deployer: &'a M::Account,
    ) -> anyhow::Result<M::Deployment> {
        tracing::info!(?subnet_name, ?deployer, "new_deployment");
        self.inner.new_deployment(subnet_name, deployer).await
    }

    fn existing_deployment<'s>(
        &'s mut self,
        subnet_name: &SubnetName,
        gateway: Address,
        registry: Address,
    ) -> anyhow::Result<M::Deployment> {
        tracing::info!(?subnet_name, "existing_deployment");
        self.inner
            .existing_deployment(subnet_name, gateway, registry)
    }

    fn default_deployment<'s, 'a>(
        &'s mut self,
        subnet_name: &SubnetName,
    ) -> anyhow::Result<M::Deployment> {
        tracing::info!(?subnet_name, "default_deployment");
        self.inner.default_deployment(subnet_name)
    }

    fn create_root_genesis<'s, 'a>(
        &'s mut self,
        subnet_name: &SubnetName,
        validators: BTreeMap<&'a M::Account, Collateral>,
        balances: BTreeMap<&'a M::Account, Balance>,
    ) -> anyhow::Result<M::Genesis> {
        tracing::info!(?subnet_name, "create_root_genesis");
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
        tracing::info!(?node_name, "create_node");
        self.inner.create_node(node_name, node_config).await
    }

    async fn start_node<'s, 'a>(
        &'s mut self,
        node: &'a M::Node,
        seed_nodes: &'a [&'a M::Node],
    ) -> anyhow::Result<()> {
        tracing::info!(?node, "start_node");
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
        tracing::info!(?subnet_name, "create_subnet");
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
        tracing::info!(?subnet, ?account, "fund_subnet");
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
        tracing::info!(?subnet, ?account, "join_subnet");
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
        tracing::info!(?subnet, "create_subnet_genesis");
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
        tracing::info!(?relayer_name, "create_relayer");
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
