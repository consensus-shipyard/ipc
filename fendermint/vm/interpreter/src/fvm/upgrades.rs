// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::BTreeMap;

use anyhow::bail;
use fendermint_vm_core::chainid;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::chainid::ChainID;
use std::collections::btree_map::Entry::{Occupied, Vacant};

use super::state::{snapshot::BlockHeight, FvmExecState};
use super::DefaultModule;

#[derive(PartialEq, Eq, Clone)]
struct UpgradeKey(ChainID, BlockHeight);

impl PartialOrd for UpgradeKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UpgradeKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.0 == other.0 {
            self.1.cmp(&other.1)
        } else {
            let chain_id: u64 = self.0.into();
            chain_id.cmp(&other.0.into())
        }
    }
}

/// a function type for migration
///
/// This is now generic over the module type M, allowing migrations to work with any module bundle.
/// Note: The ModuleBundle bound is enforced at usage sites rather than in the type alias
/// (Rust doesn't support where clauses on type aliases).
pub type MigrationFunc<DB, M = crate::fvm::DefaultModule> = fn(state: &mut FvmExecState<DB, M>) -> anyhow::Result<()>;

/// Upgrade represents a single upgrade to be executed at a given height
#[derive(Clone)]
pub struct Upgrade<DB, M = crate::fvm::DefaultModule>
where
    DB: Blockstore + 'static + Clone,
    M: fendermint_module::ModuleBundle,
{
    /// the chain_id should match the chain_id from the network configuration
    chain_id: ChainID,
    /// the block height at which the upgrade should be executed
    block_height: BlockHeight,
    /// the application version after the upgrade (or None if not affected)
    new_app_version: Option<u64>,
    /// the migration function to be executed
    migration: MigrationFunc<DB, M>,
}

impl<DB, M> Upgrade<DB, M>
where
    DB: Blockstore + 'static + Clone,
    M: fendermint_module::ModuleBundle,
{
    pub fn new(
        chain_name: impl ToString,
        block_height: BlockHeight,
        new_app_version: Option<u64>,
        migration: MigrationFunc<DB, M>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            chain_id: chainid::from_str_hashed(&chain_name.to_string())?,
            block_height,
            new_app_version,
            migration,
        })
    }

    pub fn new_by_id(
        chain_id: ChainID,
        block_height: BlockHeight,
        new_app_version: Option<u64>,
        migration: MigrationFunc<DB, M>,
    ) -> Self {
        Self {
            chain_id,
            block_height,
            new_app_version,
            migration,
        }
    }

    pub fn execute(&self, state: &mut FvmExecState<DB, M>) -> anyhow::Result<Option<u64>> {
        (self.migration)(state)?;

        Ok(self.new_app_version)
    }
}

/// UpgradeScheduler represents a list of upgrades to be executed at given heights
/// During each block height we check if there is an upgrade scheduled at that
/// height, and if so the migration for that upgrade is performed.
#[derive(Clone)]
pub struct UpgradeScheduler<DB, M = crate::fvm::DefaultModule>
where
    DB: Blockstore + 'static + Clone,
    M: fendermint_module::ModuleBundle,
{
    upgrades: BTreeMap<UpgradeKey, Upgrade<DB, M>>,
}

impl<DB, M> Default for UpgradeScheduler<DB, M>
where
    DB: Blockstore + 'static + Clone,
    M: fendermint_module::ModuleBundle,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<DB, M> UpgradeScheduler<DB, M>
where
    DB: Blockstore + 'static + Clone,
    M: fendermint_module::ModuleBundle,
{
    pub fn new() -> Self {
        Self {
            upgrades: BTreeMap::new(),
        }
    }
}

impl<DB, M> UpgradeScheduler<DB, M>
where
    DB: Blockstore + 'static + Clone,
    M: fendermint_module::ModuleBundle,
{
    // add a new upgrade to the schedule
    pub fn add(&mut self, upgrade: Upgrade<DB, M>) -> anyhow::Result<()> {
        match self
            .upgrades
            .entry(UpgradeKey(upgrade.chain_id, upgrade.block_height))
        {
            Vacant(entry) => {
                entry.insert(upgrade);
                Ok(())
            }
            Occupied(_) => {
                bail!("Upgrade already exists");
            }
        }
    }

    // check if there is an upgrade scheduled for the given chain_id at a given height
    pub fn get(&self, chain_id: ChainID, height: BlockHeight) -> Option<&Upgrade<DB, M>> {
        self.upgrades.get(&UpgradeKey(chain_id, height))
    }
}

#[test]
fn test_validate_upgrade_schedule() {
    use crate::fvm::store::memory::MemoryBlockstore;

    let mut upgrade_scheduler: UpgradeScheduler<MemoryBlockstore> = UpgradeScheduler::new();

    let upgrade = Upgrade::new("mychain", 10, None, |_state| Ok(())).unwrap();
    upgrade_scheduler.add(upgrade).unwrap();

    let upgrade = Upgrade::new("mychain", 20, None, |_state| Ok(())).unwrap();
    upgrade_scheduler.add(upgrade).unwrap();

    // adding an upgrade with the same chain_id and height should fail
    let upgrade = Upgrade::new("mychain", 20, None, |_state| Ok(())).unwrap();
    let res = upgrade_scheduler.add(upgrade);
    assert!(res.is_err());

    let mychain_id = chainid::from_str_hashed("mychain").unwrap();
    let otherhain_id = chainid::from_str_hashed("otherchain").unwrap();

    assert!(upgrade_scheduler.get(mychain_id, 9).is_none());
    assert!(upgrade_scheduler.get(mychain_id, 10).is_some());
    assert!(upgrade_scheduler.get(otherhain_id, 10).is_none());
}
