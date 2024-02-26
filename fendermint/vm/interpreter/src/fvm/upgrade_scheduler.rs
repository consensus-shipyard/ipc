// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::bail;
use fendermint_vm_core::chainid;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::chainid::ChainID;

use super::state::{snapshot::BlockHeight, FvmExecState};

/// a function type for migration
// TODO: Add missing parameters
pub type MigrationFunc<DB> = fn(state: &mut FvmExecState<DB>) -> anyhow::Result<()>;

/// Upgrade represents a single upgrade to be executed at a given height
#[derive(Clone)]
pub struct Upgrade<DB>
where
    DB: Blockstore + 'static + Clone,
{
    /// the chain id on which the upgrade should be executed
    chain_id: ChainID,
    /// the block height at which the upgrade should be executed
    block_height: BlockHeight,
    /// the migration function to be executed
    migration: MigrationFunc<DB>,

    /// the chain name is never read after initialization
    _chain_name: String,
}

impl<DB> Upgrade<DB>
where
    DB: Blockstore + 'static + Clone,
{
    pub fn new(
        chain_name: String,
        block_height: BlockHeight,
        migration: MigrationFunc<DB>,
    ) -> anyhow::Result<Self> {
        let chain_id = chainid::from_str_hashed(&chain_name)?;

        let me = Self {
            chain_id,
            block_height,
            migration,
            _chain_name: chain_name,
        };

        Ok(me)
    }

    pub fn execute(&self, state: &mut FvmExecState<DB>) -> anyhow::Result<()> {
        (self.migration)(state)
    }
}

/// UpgradeScheduler represents a list of upgrades to be executed at given heights
/// During each block height we check if there is an upgrade scheduled at that
/// height, and if so the migration for that upgrade is performed.
#[derive(Clone)]
pub struct UpgradeScheduler<DB>
where
    DB: Blockstore + 'static + Clone,
{
    // TODO: Consider using Set
    pub upgrades: Vec<Upgrade<DB>>,
}

impl<DB> Default for UpgradeScheduler<DB>
where
    DB: Blockstore + 'static + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<DB> UpgradeScheduler<DB>
where
    DB: Blockstore + 'static + Clone,
{
    pub fn new() -> Self {
        Self {
            upgrades: Vec::new(),
        }
    }
}

impl<DB> UpgradeScheduler<DB>
where
    DB: Blockstore + 'static + Clone,
{
    // add a new upgrade to the schedule
    pub fn add(&mut self, upgrade: Upgrade<DB>) -> anyhow::Result<()> {
        if self
            .upgrades
            .iter()
            .any(|u| u.chain_id == upgrade.chain_id && u.block_height == upgrade.block_height)
        {
            bail!(
                "Upgrade already exists at block height {}",
                upgrade.block_height
            );
        }

        self.upgrades.push(upgrade);

        Ok(())
    }

    // check if there is an upgrade scheduled for the given chain_id at a given height
    pub fn get(&self, chain_id: ChainID, height: BlockHeight) -> Option<&Upgrade<DB>> {
        self.upgrades
            .iter()
            .find(|u| u.chain_id == chain_id && u.block_height == height)
    }
}

#[test]
fn test_validate_upgrade_schedule() {
    use crate::fvm::store::memory::MemoryBlockstore;

    let mut upgrade_scheduler: UpgradeScheduler<MemoryBlockstore> = UpgradeScheduler::new();

    let upgrade = Upgrade::new("mychain".to_string(), 10, |_state| Ok(())).unwrap();
    upgrade_scheduler.add(upgrade).unwrap();

    let upgrade = Upgrade::new("mychain".to_string(), 20, |_state| Ok(())).unwrap();
    upgrade_scheduler.add(upgrade).unwrap();

    // adding an upgrade with the same chain_id and height should fail
    let upgrade = Upgrade::new("mychain".to_string(), 20, |_state| Ok(())).unwrap();
    let res = upgrade_scheduler.add(upgrade);
    assert!(res.is_err());

    let mychain_id = chainid::from_str_hashed("mychain").unwrap();
    let otherhain_id = chainid::from_str_hashed("otherchain").unwrap();

    assert!(upgrade_scheduler.get(mychain_id, 9).is_none());
    assert!(upgrade_scheduler.get(mychain_id, 10).is_some());
    assert!(upgrade_scheduler.get(otherhain_id, 10).is_none());
}
