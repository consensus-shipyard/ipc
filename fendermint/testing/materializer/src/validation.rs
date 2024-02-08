use anyhow::{anyhow, bail, Ok};
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use async_trait::async_trait;
use fendermint_vm_genesis::Collateral;
use fvm_shared::{address::Address, bigint::Zero, econ::TokenAmount};
use std::{collections::BTreeMap, fmt::Debug, ops::Add};

use crate::{
    manifest::Balance,
    materializer::{Materializer, NodeConfig, SubmitConfig, SubnetConfig},
    AccountName, NodeName, RelayerName, ResourceHash, ResourceName, SubnetName, TestnetName,
};

const DEFAULT_FAUCET_FIL: u64 = 100;

/// This simple validator just sanity checks that the balance configurations
/// make sense, e.g. we don't try to allocate more collateral than the available
/// funds in the root, or that we do allocate balance to relayers.
#[derive(Clone, Debug, Default)]
pub struct ValidatingMaterializer {
    network: Option<TestnetName>,
    balances: BTreeMap<SubnetName, BTreeMap<AccountName, TokenAmount>>,
}

impl ValidatingMaterializer {
    fn network(&self) -> anyhow::Result<&TestnetName> {
        self.network
            .as_ref()
            .ok_or_else(|| anyhow!("network isn't set"))
    }

    fn ensure_contains<T: AsRef<ResourceName> + Debug>(&self, name: &T) -> anyhow::Result<()> {
        let tn = self.network()?;
        if !tn.contains(name) {
            bail!("{tn:?} does not contain {name:?}");
        }

        Ok(())
    }
}

#[async_trait]
impl Materializer for ValidatingMaterializer {
    type Network = TestnetName;
    type Deployment = SubnetName;
    type Account = AccountName;
    type Genesis = ();
    type Subnet = SubnetName;
    type Node = NodeName;
    type Relayer = RelayerName;

    async fn create_network(
        &mut self,
        testnet_name: &TestnetName,
    ) -> anyhow::Result<Self::Network> {
        self.network = Some(testnet_name.clone());
        Ok(testnet_name.clone())
    }

    fn create_account(&mut self, account_name: &AccountName) -> anyhow::Result<Self::Account> {
        self.ensure_contains(account_name)?;
        Ok(account_name.clone())
    }

    async fn fund_from_faucet(
        &mut self,
        account: &Self::Account,
        reference: Option<ResourceHash>,
    ) -> anyhow::Result<()> {
        let tn = self.network()?;
        let balances = self.balances.entry(tn.root()).or_default();
        let balance = balances.entry(account.clone()).or_default();

        *balance = balance
            .clone()
            .add(TokenAmount::from_whole(DEFAULT_FAUCET_FIL));

        Ok(())
    }

    async fn new_deployment(
        &mut self,
        subnet_name: &SubnetName,
        deployer: &Self::Account,
    ) -> anyhow::Result<Self::Deployment> {
        self.ensure_contains(subnet_name)?;
        Ok(subnet_name.clone())
    }

    fn existing_deployment(
        &mut self,
        subnet_name: &SubnetName,
        gateway: Address,
        registry: Address,
    ) -> anyhow::Result<Self::Deployment> {
        self.ensure_contains(subnet_name)?;
        Ok(subnet_name.clone())
    }

    fn default_deployment(&mut self, subnet_name: &SubnetName) -> anyhow::Result<Self::Deployment> {
        self.ensure_contains(subnet_name)?;
        Ok(subnet_name.clone())
    }

    fn create_root_genesis(
        &mut self,
        subnet_name: &SubnetName,
        validators: BTreeMap<&Self::Account, Collateral>,
        balances: BTreeMap<&Self::Account, Balance>,
    ) -> anyhow::Result<Self::Genesis> {
        self.ensure_contains(subnet_name)?;
        let tn = self.network()?;

        if validators.is_empty() {
            bail!("validators of {subnet_name:?} cannot be empty");
        }

        let root_balances = self.balances.entry(tn.root()).or_default();

        for (n, b) in balances {
            let balance = root_balances.entry(n.clone()).or_default();
            *balance = b.0;
        }

        Ok(())
    }

    async fn create_node<'s, 'a>(
        &'s mut self,
        node_name: &NodeName,
        node_config: NodeConfig<'a, Self>,
    ) -> anyhow::Result<Self::Node>
    where
        's: 'a,
    {
        self.ensure_contains(node_name)?;
        Ok(node_name.clone())
    }

    async fn start_node(
        &mut self,
        _node: &Self::Node,
        _seed_nodes: &[&Self::Node],
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn create_subnet<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, Self>,
        subnet_name: &SubnetName,
        subnet_config: SubnetConfig<'a, Self>,
    ) -> anyhow::Result<Self::Subnet>
    where
        's: 'a,
    {
        self.ensure_contains(subnet_name)?;
        // TODO: Check that creator has balance.
        // TODO: Insert child subnet balances entry.
        Ok(subnet_name.clone())
    }

    async fn fund_subnet<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, Self>,
        account: &Self::Account,
        subnet: &Self::Subnet,
        amount: TokenAmount,
        reference: Option<ResourceHash>,
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        // TODO: Check that the child subnet exists.
        // TODO: Debit parent balance
        // TODO: Credit child balance
    }

    async fn join_subnet<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, Self>,
        account: &Self::Account,
        subnet: &Self::Subnet,
        collateral: Collateral,
        balance: Balance,
        reference: Option<ResourceHash>,
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        // TODO: Check that the child subnet exists.
        // TODO: Debit parent collateral
        // TODO: Debit parent balance
        // TODO: Credit child balance
        todo!()
    }

    async fn create_subnet_genesis<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, Self>,
        subnet: &Self::Subnet,
    ) -> anyhow::Result<Self::Genesis>
    where
        's: 'a,
    {
        todo!()
    }

    async fn create_relayer<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, Self>,
        relayer_name: &RelayerName,
        subnet: &Self::Subnet,
        submitter: &Self::Account,
        follow_node: &Self::Node,
    ) -> anyhow::Result<Self::Relayer>
    where
        's: 'a,
    {
        // TODO: Check that submitter has balance on the parent.
        todo!()
    }
}
