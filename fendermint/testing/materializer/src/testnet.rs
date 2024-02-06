// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, bail, Context};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
};

use crate::{
    manifest::{BalanceMap, CollateralMap, IpcDeployment, Manifest, Node, NodeMode, Rootnet},
    materializer::{Materializer, NodeConfig},
    AccountId, AccountName, NodeId, NodeName, SubnetName,
};

/// The `Testnet` parses a [Manifest] and is able to derive the steps
/// necessary to instantiate it with the help of the materializer.
pub struct Testnet<M: Materializer> {
    accounts: BTreeMap<AccountId, M::Account>,
    nodes: BTreeMap<NodeName, M::Node>,
    genesis: BTreeMap<SubnetName, M::Genesis>,
}

impl<M> Testnet<M>
where
    M: Materializer,
    M::Account: Ord,
    M::Genesis: Clone,
{
    pub fn new() -> Self {
        Self {
            accounts: Default::default(),
            nodes: Default::default(),
            genesis: Default::default(),
        }
    }

    /// Set up a testnet from scratch.
    ///
    /// To validate a manifest, we can first create a testnet with a [Materializer]
    /// that only creates symbolic resources.
    pub async fn setup(m: &mut M, manifest: Manifest) -> anyhow::Result<Self> {
        let mut t = Self::new();
        let root_name = SubnetName::root();

        for (account_id, _) in manifest.accounts {
            t.create_account(m, account_id)
        }

        match manifest.rootnet {
            Rootnet::External {
                deployment: IpcDeployment::New { deployer },
            } => {
                t.deploy_root_ipc(m, deployer).await?;
            }
            Rootnet::External {
                deployment: IpcDeployment::Existing { gateway, registry },
            } => {
                m.set_root_ipc(gateway, registry).await?;
            }
            Rootnet::New {
                validators,
                balances,
                nodes,
            } => {
                t.create_root_genesis(m, root_name.clone(), validators, balances)
                    .context("failed to create root genesis")?;

                let node_ids = sort_by_seeds(&nodes).context("invalid root subnet topology")?;

                for node_id in node_ids.iter() {
                    let node = nodes.get(node_id).unwrap();
                    t.create_node(m, &root_name, &node_id, &node)
                        .await
                        .context("failed to create root node")?;
                }
            }
        }

        Ok(t)
    }

    /// Create a cryptographic keypair for an account ID.
    pub fn create_account(&mut self, m: &mut M, id: AccountId) {
        let n = AccountName::new(&id);
        let a = m.create_account(n);
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

    /// Get an genesis by subnet.
    pub fn genesis(&self, name: &SubnetName) -> anyhow::Result<&M::Genesis> {
        self.genesis
            .get(name)
            .ok_or_else(|| anyhow!("genesis for {name:?} does not exist"))
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

    /// Deploy the IPC contracts to the rootnet.
    async fn deploy_root_ipc(&mut self, m: &mut M, deployer: AccountId) -> anyhow::Result<()> {
        let d = self.account(&deployer).context("invalid deployer")?;
        m.deploy_root_ipc(d)
            .await
            .context("failed to deploy IPC to root")?;
        Ok(())
    }

    /// Create a genesis for the rootnet nodes.
    ///
    /// On the rootnet the validator power comes out of thin air,
    /// ie. the balances don't have to cover it. On subnets this
    /// will be different, the collateral has to be funded.
    fn create_root_genesis(
        &mut self,
        m: &mut M,
        subnet_name: SubnetName,
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
        self.genesis.insert(subnet_name, genesis);

        Ok(())
    }

    /// Create a node, but don't start it
    async fn create_node(
        &mut self,
        m: &mut M,
        subnet_name: &SubnetName,
        node_id: &NodeId,
        node: &Node,
    ) -> anyhow::Result<()> {
        let genesis = self.genesis(subnet_name)?;
        let node_name = subnet_name.node(&node_id);

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
            .create_node(node_name.clone(), node_config)
            .context("failed to create node")?;

        self.nodes.insert(node_name, node);

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
