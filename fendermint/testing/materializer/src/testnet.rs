// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, bail, Context};
use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    fmt::Display,
};

use crate::{
    manifest::{
        AccountId, BalanceMap, CollateralMap, IpcDeployment, Manifest, Node, NodeId, ResourceName,
        Rootnet,
    },
    materializer::{AccountName, GenesisName, Materializer},
};

/// The `Testnet` parses a [Manifest] and is able to derive the steps
/// necessary to instantiate it with the help of the materializer.
pub struct Testnet<'a, M> {
    materializer: &'a mut M,
    accounts: BTreeMap<AccountId, AccountName>,
    root_genesis: Option<GenesisName>,
}

impl<'a, M> Testnet<'a, M>
where
    M: Materializer,
{
    pub fn new(materializer: &'a mut M) -> Self {
        Self {
            materializer,
            accounts: Default::default(),
            root_genesis: None,
        }
    }

    /// Set up a testnet from scratch.
    ///
    /// To validate a manifest, we can first create a testnet with a [Materializer]
    /// that only creates symbolic resources.
    pub async fn setup(materializer: &'a mut M, m: Manifest) -> anyhow::Result<Self> {
        let mut t = Self::new(materializer);

        for (account_id, _) in m.accounts {
            t.create_account(account_id)
        }

        match m.rootnet {
            Rootnet::External {
                deployment: IpcDeployment::New { deployer },
            } => {
                t.deploy_ipc(deployer).await?;
            }
            Rootnet::New {
                validators,
                balances,
                nodes,
            } => {
                t.create_root_genesis(validators, balances)?;
                for (node_id, node) in nodes {
                    t.create_root_node(node_id, node).await?;
                }
            }
        }

        Ok(t)
    }

    /// Create a cryptographic keypair for an account ID.
    pub fn create_account(&mut self, account_id: AccountId) {
        let name = self.materializer.create_account(account_id.clone());
        self.accounts.insert(account_id, name);
    }

    /// Get the name of an account.
    ///
    /// Returns an error if the account doesn't exist yet.
    pub fn account_name(&self, account_id: &AccountId) -> anyhow::Result<AccountName> {
        self.accounts
            .get(account_id)
            .cloned()
            .ok_or_else(|| anyhow!("account {account_id} does not exist"))
    }

    /// Resolve account IDs in a map to resource names.
    fn account_map<T>(
        &self,
        m: BTreeMap<AccountId, T>,
    ) -> anyhow::Result<BTreeMap<AccountName, T>> {
        m.into_iter()
            .map(|(id, x)| self.account_name(&id).map(|a| (a, x)))
            .collect()
    }

    /// Deploy the IPC contracts to the rootnet.
    async fn deploy_ipc(&mut self, deployer: AccountId) -> anyhow::Result<()> {
        let d = self.account_name(&deployer).context("invalid deployer")?;
        self.materializer
            .deploy_ipc(d)
            .await
            .context("failed to deploy IPC to root")?;
        Ok(())
    }

    /// Create a genesis file for the rootnet nodes.
    ///
    /// On the rootnet the validator power comes out of thin air,
    /// ie. the balances don't have to cover it. On subnets this
    /// will be different, the collateral has to be funded.
    fn create_root_genesis(
        &self,
        validators: CollateralMap,
        balances: BalanceMap,
    ) -> anyhow::Result<()> {
        let validators = self
            .account_map(validators)
            .context("invalid root collaterals")?;

        let balances = self
            .account_map(balances)
            .context("invalid root balances")?;

        self.root_genesis = Some(self.materializer.create_genesis(validators, balances)?);

        Ok(())
    }

    /// Create a node on the rootnet.
    ///
    /// Fails if the root genesis or the validator hasn't been created yet.
    async fn create_root_node(&self, node_id: NodeId, node: Node) -> anyhow::Result<()> {
        // TODO:
        // * validatate that seed nodes exist
        // * sort nodes in topological order by seed ID
        // * create a node, refer to seed nodes by name
        // INSTEAD:
        // * create node should merely create the configuration of a node but not start it
        // * that way we can pre-allocate the hostnames or whatever,
        // * then we can sort the nodes in a way that we start seeds first, and this is where
        //   we can make final touches to the configuration to inject the address of seeds by name
        todo!()
    }
}

/// Sort some values in a topological order.
///
/// Cycles can be allowed, in which case it will do its best to order the items
/// with the least amount of dependencies first. This is so we can support nodes
/// mutually be seeded by each other.
fn topo_sort<K, V, F>(
    mut items: BTreeMap<K, V>,
    allow_cycles: bool,
    f: F,
) -> anyhow::Result<Vec<(K, V)>>
where
    F: Fn(&V) -> BTreeSet<K>,
    K: Ord + Display,
{
    let mut deps = items
        .iter()
        .map(|(k, v)| (k, f(v)))
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
        let leaf = match deps.iter().find(|(k, ds)| ds.is_empty()) {
            Some((k, _)) => k,
            None if allow_cycles => {
                let mut dcs = deps.iter().map(|(k, ds)| (k, ds.len())).collect::<Vec<_>>();
                dcs.sort_by_key(|(_, c)| c);
                dcs.first().unwrap().0
            }
            None => bail!("circular reference in dependencies"),
        };

        deps.remove(leaf);

        for (_, ds) in deps.iter_mut() {
            ds.remove(leaf);
        }

        if let Some((k, v)) = items.remove_entry(leaf) {
            sorted.push((k, v));
        }
    }

    Ok(sorted)
}
