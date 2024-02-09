use anyhow::{anyhow, bail, Ok};
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use async_trait::async_trait;
use ethers::types::H160;
use fendermint_vm_genesis::Collateral;
use fvm_shared::econ::TokenAmount;
use std::{
    collections::{BTreeMap, HashSet},
    fmt::Debug,
    ops::{Add, Sub},
};
use tendermint_rpc::Url;

use crate::{
    manifest::{Balance, Manifest},
    materializer::{Materializer, NodeConfig, SubmitConfig, SubnetConfig},
    testnet::Testnet,
    AccountName, NodeName, RelayerName, ResourceHash, ResourceName, SubnetName, TestnetId,
    TestnetName,
};

const DEFAULT_FAUCET_FIL: u64 = 100;

/// Do simple sanity checks on the manifest, e.g.:
/// * we are not over allocating the balances
/// * relayers have balances on the parent to submit transactions
/// * subnet creators have balances on the parent to submit transactions
pub async fn validate_manifest(id: &TestnetId, manifest: &Manifest) -> anyhow::Result<()> {
    let mut m = ValidatingMaterializer::default();
    let _ = Testnet::setup(&mut m, id, manifest).await?;
    // We could check here that all subnets have enough validators for a quorum.
    Ok(())
}

#[derive(Clone, Debug, Default)]
struct ValidatingMaterializer {
    network: Option<TestnetName>,
    balances: BTreeMap<SubnetName, BTreeMap<AccountName, TokenAmount>>,
    references: BTreeMap<SubnetName, HashSet<ResourceHash>>,
}

impl ValidatingMaterializer {
    fn network(&self) -> anyhow::Result<TestnetName> {
        self.network
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow!("network isn't set"))
    }

    /// Check that a name is within the subnet. This should trivially be true by construction, but still.
    fn ensure_contains<T: AsRef<ResourceName> + Debug>(&self, name: &T) -> anyhow::Result<()> {
        let tn = self.network()?;
        if !tn.contains(name) {
            bail!("{tn:?} does not contain {name:?}");
        }
        Ok(())
    }

    /// Ensure we aren't reusing references.
    fn ensure_unique(
        &mut self,
        subnet: &SubnetName,
        reference: Option<ResourceHash>,
    ) -> anyhow::Result<()> {
        if let Some(r) = reference {
            let rs = self.references.entry(subnet.clone()).or_default();
            if !rs.insert(r) {
                bail!("a reference is reused in {subnet:?}");
            }
        }
        Ok(())
    }

    /// Check that an account has a positive balance on a subnet
    fn ensure_balance(&self, subnet: &SubnetName, account: &AccountName) -> anyhow::Result<()> {
        match self.balances.get(subnet) {
            None => bail!("{subnet:?} has not been created"),
            Some(bs) => match bs.get(account) {
                None => bail!("{account:?} has no balance on {subnet:?}"),
                Some(b) if b.is_zero() => bail!("{account:?} has zero balance on {subnet:?}"),
                Some(_) => Ok(()),
            },
        }
    }

    /// Get the parent of a subnet, or fail if it doesn't have one.
    fn parent_name(subnet: &SubnetName) -> anyhow::Result<SubnetName> {
        subnet
            .parent()
            .ok_or_else(|| anyhow!("{subnet:?} has no parent"))
    }

    /// Check that the subnet has been created already.
    fn ensure_subnet_exists(&self, subnet: &SubnetName) -> anyhow::Result<()> {
        if !self.balances.contains_key(subnet) {
            bail!("{subnet:?} has not been created");
        }
        Ok(())
    }

    /// Move funds of an account from the parent to the child subnet.
    ///
    /// Fails if either:
    /// * the parent doesn't exist
    /// * the child doesn't exist
    /// * the account doesn't have the funds
    fn fund_from_parent(
        &mut self,
        subnet: &SubnetName,
        account: &AccountName,
        amount: TokenAmount,
        credit_child: bool,
    ) -> anyhow::Result<()> {
        let parent = Self::parent_name(subnet)?;
        self.ensure_subnet_exists(&parent)?;
        self.ensure_subnet_exists(subnet)?;
        self.ensure_balance(&parent, account)?;

        let pbs = self.balances.get_mut(&parent).unwrap();
        let pb = pbs.get_mut(account).unwrap();

        if *pb < amount {
            bail!("{account:?} has less than {amount} on {parent:?}, cannot fund {subnet:?}");
        }
        *pb = pb.clone().sub(amount.clone());

        if credit_child {
            let cbs = self.balances.get_mut(subnet).unwrap();
            let cb = cbs.entry(account.clone()).or_default();
            *cb = cb.clone().add(amount);
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
        self.ensure_unique(&tn.root(), reference)?;
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
        urls: Vec<Url>,
    ) -> anyhow::Result<Self::Deployment> {
        self.ensure_contains(subnet_name)?;
        self.ensure_balance(subnet_name, deployer)?;
        Ok(subnet_name.clone())
    }

    fn existing_deployment(
        &mut self,
        subnet_name: &SubnetName,
        gateway: H160,
        registry: H160,
    ) -> anyhow::Result<Self::Deployment> {
        self.ensure_contains(subnet_name)?;

        if gateway == registry {
            bail!("gateway and registry addresses are the same in {subnet_name:?}: {gateway} == {registry}");
        }

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
        _node_config: NodeConfig<'a, Self>,
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
        _parent_submit_config: &SubmitConfig<'a, Self>,
        subnet_name: &SubnetName,
        subnet_config: SubnetConfig<'a, Self>,
    ) -> anyhow::Result<Self::Subnet>
    where
        's: 'a,
    {
        self.ensure_contains(subnet_name)?;
        // Check that the submitter has balance on the parent subnet to create the child.
        let parent = Self::parent_name(subnet_name)?;
        self.ensure_balance(&parent, subnet_config.creator)?;
        // Insert child subnet balances entry.
        self.balances
            .insert(subnet_name.clone(), Default::default());
        Ok(subnet_name.clone())
    }

    async fn fund_subnet<'s, 'a>(
        &'s mut self,
        _parent_submit_config: &SubmitConfig<'a, Self>,
        account: &Self::Account,
        subnet: &Self::Subnet,
        amount: TokenAmount,
        reference: Option<ResourceHash>,
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        // Debit parent balance; Credit child balance
        self.fund_from_parent(subnet, account, amount, true)?;
        self.ensure_unique(&subnet.parent().unwrap(), reference)?;
        Ok(())
    }

    async fn join_subnet<'s, 'a>(
        &'s mut self,
        _parent_submit_config: &SubmitConfig<'a, Self>,
        account: &Self::Account,
        subnet: &Self::Subnet,
        collateral: Collateral,
        balance: Balance,
        reference: Option<ResourceHash>,
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        // Debit parent balance, but do not make the funds available in the child
        self.fund_from_parent(subnet, account, collateral.0, false)?;
        // Debit parent balance; Credit child balance
        self.fund_from_parent(subnet, account, balance.0, true)?;
        self.ensure_unique(&subnet.parent().unwrap(), reference)?;
        Ok(())
    }

    async fn create_subnet_genesis<'s, 'a>(
        &'s mut self,
        _parent_submit_config: &SubmitConfig<'a, Self>,
        _subnet: &Self::Subnet,
    ) -> anyhow::Result<Self::Genesis>
    where
        's: 'a,
    {
        // We're supposed to fetch the data from the parent, there's nothing to check.
        Ok(())
    }

    async fn create_relayer<'s, 'a>(
        &'s mut self,
        _parent_submit_config: &SubmitConfig<'a, Self>,
        relayer_name: &RelayerName,
        subnet: &Self::Subnet,
        submitter: &Self::Account,
        _follow_node: &Self::Node,
    ) -> anyhow::Result<Self::Relayer>
    where
        's: 'a,
    {
        self.ensure_contains(relayer_name)?;
        // Check that submitter has balance on the parent.
        let parent = Self::parent_name(subnet)?;
        self.ensure_balance(&parent, submitter)?;
        Ok(relayer_name.clone())
    }
}

#[cfg(test)]
mod tests {

    use crate::{manifest::Manifest, validation::validate_manifest, TestnetId};

    /// Check that the random manifests we generate would pass validation.
    #[quickcheck_async::tokio]
    async fn prop_validation(id: TestnetId, manifest: Manifest) -> anyhow::Result<()> {
        validate_manifest(&id, &manifest).await
    }
}
