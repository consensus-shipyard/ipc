// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::BTreeMap;

use fvm_ipld_blockstore::Blockstore;

use super::state::{snapshot::BlockHeight, FvmExecState};

/// a function type for migration
// TODO: Add missing parameters
pub type MigrationFunc<DB> = fn(state: &mut FvmExecState<DB>) -> anyhow::Result<()>;

pub type ChainID = u64;

/// Upgrade represents a single upgrade to be executed at a given height
#[derive(Clone)]
pub struct Upgrade<DB>
where
    DB: Blockstore + 'static + Clone,
{
    /// the chain id on which the upgrade should be executed
    pub chain_id: ChainID,
    /// the block height at which the upgrade should be executed
    pub block_height: BlockHeight,
    /// the migration function to be executed
    pub migration: MigrationFunc<DB>,
}

/// UpgradeScheduler represents a list of upgrades to be executed at given heights
/// During each block height we check if there is an upgrade scheduled at that
/// height, and if so the migration for that upgrade is performed.
#[derive(Clone)]
pub struct UpgradeScheduler<DB>
where
    DB: Blockstore + 'static + Clone,
{
    pub upgrades: BTreeMap<(ChainID, BlockHeight), Upgrade<DB>>,
}

impl<DB> Default for UpgradeScheduler<DB>
where
    DB: Blockstore + 'static + Clone,
{
    fn default() -> Self {
        Self {
            upgrades: BTreeMap::new(),
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
            .contains_key(&(upgrade.chain_id, upgrade.block_height))
        {
            return Err(anyhow::anyhow!(
                "Upgrade already exists at block height {}",
                upgrade.block_height
            ));
        }

        self.upgrades
            .insert((upgrade.chain_id, upgrade.block_height), upgrade);

        Ok(())
    }

    // check if there is an upgrade scheduled for the given chain_id at a given height
    pub fn get(&self, chain_id: ChainID, height: BlockHeight) -> Option<&Upgrade<DB>> {
        self.upgrades.get(&(chain_id, height))
    }
}

#[test]
fn test_validate_upgrade_schedule() {
    let mut us: UpgradeScheduler<fvm_ipld_blockstore::MemoryBlockstore> =
        UpgradeScheduler::default();

    let upgrade = Upgrade {
        chain_id: 1,
        block_height: 10,
        migration: |_state| Ok(()),
    };
    us.add(upgrade).unwrap();

    let upgrade = Upgrade {
        chain_id: 1,
        block_height: 20,
        migration: |_state| Ok(()),
    };
    us.add(upgrade).unwrap();

    // adding an upgrade at the same height should fail
    let upgrade = Upgrade {
        chain_id: 1,
        block_height: 20,
        migration: |_state| Ok(()),
    };
    let res = us.add(upgrade);
    assert!(res.is_err());

    assert!(us.get(1, 9).is_none());
    assert!(us.get(1, 10).is_some());
}
