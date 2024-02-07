// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, bail, Context};
use async_recursion::async_recursion;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use crate::{
    manifest::{
        BalanceMap, CollateralMap, IpcDeployment, Manifest, Node, NodeMode, Rootnet, Subnet,
    },
    materializer::{Materializer, NodeConfig, SubnetConfig},
    AccountId, AccountName, NodeId, NodeName, RelayerName, SubnetId, SubnetName,
};

/// The `Testnet` parses a [Manifest] and is able to derive the steps
/// necessary to instantiate it with the help of the [Materializer].
///
/// The `Testnet` data structure itself acts as an indexer over the
/// resources created by the [Materializer]. It owns them, and by
/// doing so controls their life cycle. By dropping the `Testnet`
/// or various components from it we are able to free resources.
///
/// Arguably the same could be achieved by keeping the created
/// resources inside the [Materializer] and discarding that as
/// a whole, keeping the `Testnet` completely stateless, but
/// perhaps this way writing a [Materializer] is just a tiny
/// bit simpler.
pub struct Testnet<M: Materializer> {
    accounts: BTreeMap<AccountId, M::Account>,
    deployments: BTreeMap<SubnetName, M::Deployment>,
    genesis: BTreeMap<SubnetName, M::Genesis>,
    subnets: BTreeMap<SubnetName, M::Subnet>,
    nodes: BTreeMap<NodeName, M::Node>,
    relayers: BTreeMap<RelayerName, M::Relayer>,
}

impl<M> Testnet<M>
where
    M: Materializer + Sync + Send,
    M::Account: Ord + Sync + Send,
    M::Genesis: Clone + Sync + Send,
    M::Deployment: Sync + Send,
    M::Subnet: Sync + Send,
    M::Node: Sync + Send,
    M::Relayer: Sync + Send,
{
    pub fn new() -> Self {
        Self {
            accounts: Default::default(),
            deployments: Default::default(),
            genesis: Default::default(),
            subnets: Default::default(),
            nodes: Default::default(),
            relayers: Default::default(),
        }
    }

    /// Set up a testnet from scratch.
    ///
    /// To validate a manifest, we can first create a testnet with a [Materializer]
    /// that only creates symbolic resources.
    pub async fn setup(m: &mut M, manifest: Manifest) -> anyhow::Result<Self> {
        let mut t = Self::new();
        let root_name = SubnetName::root();

        // Create keys for accounts.
        for (account_id, _) in manifest.accounts {
            t.create_account(m, account_id)
        }

        // Create the rootnet.
        t.create_and_start_rootnet(m, &root_name, manifest.rootnet)
            .await
            .context("failed to start rootnet")?;

        // Recursively create and start all subnet nodes.
        for (subnet_id, subnet) in manifest.subnets {
            t.create_and_start_subnet(m, &root_name, &subnet_id, subnet)
                .await
                .context("failed to start subnet")?;
        }

        Ok(t)
    }

    /// Create a cryptographic keypair for an account ID.
    pub fn create_account(&mut self, m: &mut M, id: AccountId) {
        let n = AccountName::new(&id);
        let a = m.create_account(&n);
        self.accounts.insert(id, a);
    }

    /// Get an account by ID.
    pub fn account(&self, id: &AccountId) -> anyhow::Result<&M::Account> {
        self.accounts
            .get(id)
            .ok_or_else(|| anyhow!("account {id} does not exist"))
    }

    /// Get an node by name.
    pub fn node(&self, name: &NodeName) -> anyhow::Result<&M::Node> {
        self.nodes
            .get(name)
            .ok_or_else(|| anyhow!("node {name:?} does not exist"))
    }

    /// Get an subnet by name.
    pub fn subnet(&self, name: &SubnetName) -> anyhow::Result<&M::Subnet> {
        self.subnets
            .get(name)
            .ok_or_else(|| anyhow!("subnet {name:?} does not exist"))
    }

    /// Get an genesis by subnet.
    pub fn genesis(&self, name: &SubnetName) -> anyhow::Result<&M::Genesis> {
        self.genesis
            .get(name)
            .ok_or_else(|| anyhow!("genesis for {name:?} does not exist"))
    }

    /// Get a deployment by subnet.
    pub fn deployment(&self, name: &SubnetName) -> anyhow::Result<&M::Deployment> {
        self.deployments
            .get(name)
            .ok_or_else(|| anyhow!("deployment for {name:?} does not exist"))
    }

    /// List all the nodes in a subnet.
    pub fn nodes_by_subnet(&self, subnet_name: &SubnetName) -> Vec<&M::Node> {
        self.nodes
            .iter()
            .filter(|(node_name, _)| node_name.is_in_subnet(subnet_name))
            .map(|(_, n)| n)
            .collect()
    }

    /// Resolve account IDs in a map to account references.
    fn account_map<T>(
        &self,
        m: BTreeMap<AccountId, T>,
    ) -> anyhow::Result<BTreeMap<&M::Account, T>> {
        m.into_iter()
            .map(|(id, x)| self.account(&id).map(|a| (a, x)))
            .collect()
    }

    /// Create a genesis for the rootnet nodes.
    ///
    /// On the rootnet the validator power comes out of thin air,
    /// ie. the balances don't have to cover it. On subnets this
    /// will be different, the collateral has to be funded.
    fn create_root_genesis(
        &mut self,
        m: &mut M,
        subnet_name: &SubnetName,
        validators: CollateralMap,
        balances: BalanceMap,
    ) -> anyhow::Result<()> {
        let validators = self
            .account_map(validators)
            .context("invalid root collaterals")?;

        let balances = self
            .account_map(balances)
            .context("invalid root balances")?;

        // Remember the genesis so we can potentially create more nodes later.
        let genesis = m.create_root_genesis(subnet_name.clone(), validators, balances)?;

        self.genesis.insert(subnet_name.clone(), genesis);

        Ok(())
    }

    /// Configure and start the nodes of a subnet.
    ///
    /// Fails if the genesis of this subnet hasn't been created yet.
    async fn create_and_start_nodes(
        &mut self,
        m: &mut M,
        subnet_name: &SubnetName,
        nodes: &BTreeMap<NodeId, Node>,
    ) -> anyhow::Result<()> {
        let node_ids = sort_by_seeds(nodes).context("invalid root subnet topology")?;

        for node_id in node_ids.iter() {
            self.create_node(m, subnet_name, node_id, nodes.get(node_id).unwrap())
                .await
                .with_context(|| "failed to create node {node_id} in {subnet_name:?}")?;
        }

        for node_id in node_ids.iter() {
            self.start_node(m, subnet_name, node_id, nodes.get(node_id).unwrap())
                .await
                .with_context(|| "failed to start node {node_id} in {subnet_name:?}")?;
        }

        Ok(())
    }

    /// Create the configuration of a node.
    ///
    /// Fails if the genesis hasn't been created yet.
    async fn create_node(
        &mut self,
        m: &mut M,
        subnet_name: &SubnetName,
        node_id: &NodeId,
        node: &Node,
    ) -> anyhow::Result<()> {
        let genesis = self.genesis(subnet_name)?;
        let node_name = subnet_name.node(node_id);

        let node_config = NodeConfig {
            genesis,
            validator: match &node.mode {
                NodeMode::Full => None,
                NodeMode::Validator(id) => {
                    let validator = self
                        .account(id)
                        .with_context(|| format!("invalid validator in {node_name:?}"))?;
                    Some(validator)
                }
            },
            parent_node: match (subnet_name.parent(), &node.parent_node) {
                (Some(ps), Some(n)) => Some(
                    self.node(&ps.node(n))
                        .with_context(|| format!("invalid parent node in {node_name:?}"))?,
                ),
                (None, Some(_)) => {
                    bail!("node {node_name:?} has parent node, but there is no parent subnet")
                }
                _ => None,
            },
            ethapi: node.ethapi,
        };

        let node = m
            .create_node(&node_name, node_config)
            .await
            .context("failed to create node")?;

        self.nodes.insert(node_name, node);

        Ok(())
    }

    /// Start a node.
    ///
    /// Fails if the node hasn't been created yet.
    async fn start_node(
        &mut self,
        m: &mut M,
        subnet_name: &SubnetName,
        node_id: &NodeId,
        node: &Node,
    ) -> anyhow::Result<()> {
        let node_name = subnet_name.node(node_id);

        let seeds = node
            .seed_nodes
            .iter()
            .map(|s| self.node(&subnet_name.node(s)))
            .collect::<Result<Vec<_>, _>>()
            .with_context(|| format!("failed to collect seeds for {node_name:?}"))?;

        let node = self.node(&node_name)?;

        m.start_node(node, &seeds)
            .await
            .with_context(|| format!("failed to start {node_name:?}"))?;

        Ok(())
    }

    async fn create_and_start_rootnet(
        &mut self,
        m: &mut M,
        root_name: &SubnetName,
        rootnet: Rootnet,
    ) -> anyhow::Result<()> {
        match rootnet {
            Rootnet::External { deployment } => {
                // Establish root contract locations.
                let deployment = match deployment {
                    IpcDeployment::New { deployer } => {
                        let deployer = self.account(&deployer).context("invalid deployer")?;
                        m.new_deployment(root_name, deployer)
                            .await
                            .context("failed to deploy IPC contracts")?
                    }
                    IpcDeployment::Existing { gateway, registry } => {
                        m.existing_deployment(root_name, gateway, registry)
                    }
                };

                self.deployments.insert(root_name.clone(), deployment);

                // Establish balances.
                for a in self.accounts.values() {
                    m.fund_from_faucet(a).await.context("faucet failed")?;
                }
            }
            Rootnet::New {
                validators,
                balances,
                nodes,
            } => {
                let deployment = m.default_deployment(root_name);
                self.deployments.insert(root_name.clone(), deployment);

                self.create_root_genesis(m, &root_name, validators, balances)
                    .context("failed to create root genesis")?;

                self.create_and_start_nodes(m, &root_name, &nodes)
                    .await
                    .context("failed to start root nodes")?;
            }
        }
        Ok(())
    }

    #[async_recursion]
    async fn create_and_start_subnet(
        &mut self,
        m: &mut M,
        parent_subnet_name: &SubnetName,
        subnet_id: &SubnetId,
        subnet: Subnet,
    ) -> anyhow::Result<()> {
        let subnet_name = parent_subnet_name.subnet(subnet_id);

        // Pre-fund the accounts, create the subnet, start the nodes.
        {
            // Assume that all subnets are deployed with the default contracts.
            self.deployments
                .insert(subnet_name.clone(), m.default_deployment(&subnet_name));

            // Where can we reach the gateway and the registry.
            let parent_deployment = self.deployment(parent_subnet_name)?;
            let parent_nodes = self.nodes_by_subnet(parent_subnet_name);

            // Create the subnet on the parent.
            m.create_subnet(
                &subnet_name,
                SubnetConfig {
                    creator: self.account(&subnet.creator).context("invalid creator")?,
                    // Make the number such that the last validator to join activates the subnet.
                    min_validators: subnet.validators.len(),
                },
                &parent_nodes,
                &parent_deployment,
            )
            .await
            .context("failed to create subnet")?;

            let created_subnet = self.subnet(&subnet_name)?;
            let ancestor_hops = subnet_name.ancestor_hops();

            // Fund validator and balances collateral all the way from the root down to the parent.
            for (fund_source, fund_target) in &ancestor_hops {
                let fund_nodes = self.nodes_by_subnet(fund_source);
                let fund_deployment = self.deployment(fund_source)?;
                let fund_subnet = self.subnet(&fund_target)?;

                for (id, amount) in &subnet.validators {
                    let account = self
                        .account(id)
                        .with_context(|| format!("invalid validator in {subnet_name:?}"))?;

                    m.fund_subnet(
                        account,
                        fund_subnet,
                        amount.0.clone(),
                        &fund_nodes,
                        fund_deployment,
                    )
                    .await
                    .with_context(|| format!("failed to fund {id} in {fund_target:?}"))?;
                }

                for (id, amount) in &subnet.balances {
                    let account = self
                        .account(id)
                        .with_context(|| format!("invalid account in {subnet_name:?}"))?;

                    m.fund_subnet(
                        account,
                        fund_subnet,
                        amount.0.clone(),
                        &fund_nodes,
                        fund_deployment,
                    )
                    .await
                    .with_context(|| format!("failed to fund {id} in {fund_target:?}"))?;
                }
            }

            // Join with the validators on the subnet.
            for (id, c) in &subnet.validators {
                let account = self
                    .account(id)
                    .with_context(|| format!("invalid validator in {subnet_name:?}"))?;

                let b = subnet.balances.get(id).cloned().unwrap_or_default();

                m.join_subnet(
                    account,
                    created_subnet,
                    c.clone(),
                    b,
                    &parent_nodes,
                    parent_deployment,
                )
                .await
                .with_context(|| format!("failed to join with {id} in {subnet_name:?}"))?;
            }

            // Create genesis by fetching from the parent.
            let genesis = m
                .create_subnet_genesis(created_subnet, &parent_nodes)
                .context("failed to create subnet genesis")?;

            self.genesis.insert(subnet_name.clone(), genesis);

            // Create and start nodes.
            self.create_and_start_nodes(m, &subnet_name, &subnet.nodes)
                .await
                .context("failed to start subnet nodes")?;
        }

        // Interact with the running subnet .
        {
            let created_subnet = self.subnet(&subnet_name)?;
            let parent_deployment = self.deployment(parent_subnet_name)?;
            let parent_nodes = self.nodes_by_subnet(parent_subnet_name);

            // Fund all non-validator balances (which have been passed to join_validator as a pre-fund request).
            // These could be done as pre-funds if the command is available on its own.
            for (id, b) in &subnet.balances {
                let account = self
                    .account(id)
                    .with_context(|| format!("invalid validator in {subnet_name:?}"))?;

                if subnet.validators.contains_key(id) {
                    continue;
                }

                m.fund_subnet(
                    account,
                    created_subnet,
                    b.0.clone(),
                    &parent_nodes,
                    parent_deployment,
                )
                .await
                .with_context(|| format!("failed to join with {id} in {subnet_name:?}"))?;
            }

            // Create relayers for bottom-up checkpointing.
            let mut relayers = Vec::<(RelayerName, M::Relayer)>::new();
            for (id, relayer) in &subnet.relayers {
                let submitter = self
                    .account(&relayer.submitter)
                    .context("invalid relayer")?;

                let follow_node = self
                    .node(&subnet_name.node(&relayer.follow_node))
                    .context("invalid follow node")?;

                let submit_node = match (subnet_name.parent(), &relayer.submit_node) {
                    (Some(p), Some(s)) => Some(self.node(&p.node(s)).context("invalid submit node")?),
                    (Some(p), None) if p.is_root() => None, // must be an external endpoint
                    (Some(_), None)  => bail!(
                        "invalid relayer {id} in {subnet_name:?}: parent is not root, but submit node is missing"
                    ),
                    (None, _) => bail!(
                        "invalid relayer {id} in {subnet_name:?}: there is no parent subnet to relay to"
                    ),
                };

                let relayer_name = subnet_name.relayer(id);
                let relayer = m
                    .create_relayer(
                        &relayer_name,
                        created_subnet,
                        submitter,
                        follow_node,
                        submit_node,
                        parent_deployment,
                    )
                    .await
                    .context("failed to create relayer")?;

                relayers.push((relayer_name, relayer));
            }
            self.relayers.extend(relayers.into_iter());
        }

        // Recursively create and start all subnet nodes.
        for (subnet_id, subnet) in subnet.subnets {
            self.create_and_start_subnet(m, &subnet_name, &subnet_id, subnet)
                .await
                .with_context(|| {
                    format!("failed to start subnet {subnet_id} in {subnet_name:?}")
                })?;
        }

        Ok(())
    }
}

/// Sort some values in a topological order.
///
/// Cycles can be allowed, in which case it will do its best to order the items
/// with the least amount of dependencies first. This is so we can support nodes
/// mutually be seeded by each other.
fn topo_sort<K, V, F, I>(items: &BTreeMap<K, V>, allow_cycles: bool, f: F) -> anyhow::Result<Vec<K>>
where
    F: Fn(&V) -> I,
    K: Ord + Display + Clone,
    I: IntoIterator<Item = K>,
{
    let mut deps = items
        .iter()
        .map(|(k, v)| (k.clone(), BTreeSet::from_iter(f(v))))
        .collect::<BTreeMap<_, _>>();

    for (k, ds) in deps.iter() {
        for d in ds {
            if !deps.contains_key(d) {
                bail!("non-existing dependency: {d} <- {k}")
            }
        }
    }

    let mut sorted = Vec::new();

    while !deps.is_empty() {
        let leaf: K = match deps.iter().find(|(_, ds)| ds.is_empty()) {
            Some((leaf, _)) => (*leaf).clone(),
            None if allow_cycles => {
                let mut dcs = deps.iter().map(|(k, ds)| (k, ds.len())).collect::<Vec<_>>();
                dcs.sort_by_key(|(_, c)| *c);
                let leaf = dcs.first().unwrap().0;
                (*leaf).clone()
            }
            None => bail!("circular reference in dependencies"),
        };

        deps.remove(&leaf);

        for (_, ds) in deps.iter_mut() {
            ds.remove(&leaf);
        }

        sorted.push(leaf);
    }

    Ok(sorted)
}

/// Sort nodes in a subnet in topological order, so we strive to first
/// start the ones others use as a seed node. However, do allow cycles
/// so that we can have nodes mutually bootstrap from each other.
fn sort_by_seeds(nodes: &BTreeMap<NodeId, Node>) -> anyhow::Result<Vec<NodeId>> {
    topo_sort(nodes, true, |n| {
        BTreeSet::from_iter(n.seed_nodes.iter().cloned())
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::topo_sort;

    #[test]
    fn test_topo_sort() {
        let mut tree = BTreeMap::default();

        tree.insert(1, vec![]);
        tree.insert(2, vec![5]);
        tree.insert(3, vec![1, 5]);
        tree.insert(4, vec![2, 3]);
        tree.insert(5, vec![1]);

        let sorted = topo_sort(&tree, false, |ds| ds.clone()).unwrap();
        assert_eq!(sorted, vec![1, 5, 2, 3, 4]);

        tree.insert(1, vec![5]);

        topo_sort(&tree, false, |ds| ds.clone()).expect_err("shouldn't allow cycles");

        let sorted = topo_sort(&tree, true, |ds| ds.clone()).expect("should allow cycles");
        assert_eq!(sorted.len(), tree.len());
    }
}
