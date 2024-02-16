// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::collections::BTreeMap;

use fendermint_vm_interpreter::fvm::state::{snapshot::BlockHeight, FvmExecState};
use fvm_ipld_blockstore::Blockstore;

/// a function type for migration
// TODO: Add missing parameters
pub type MigrationFunc<DB> = fn(state: &FvmExecState<DB>) -> anyhow::Result<()>;

/// Upgrade represents a single upgrade to be executed at a given height
#[derive(Clone)]
pub struct Upgrade<DB>
where
    DB: Blockstore + 'static + Clone,
{
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
    pub upgrades: BTreeMap<BlockHeight, Upgrade<DB>>,
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
    pub fn add_upgrade(&mut self, upgrade: Upgrade<DB>) -> anyhow::Result<()> {
        if self.upgrades.contains_key(&upgrade.block_height) {
            return Err(anyhow::anyhow!(
                "Upgrade already exists at block height {}",
                upgrade.block_height
            ));
        }

        self.upgrades.insert(upgrade.block_height, upgrade);

        Ok(())
    }

    pub fn get_upgrade(&self, block_height: BlockHeight) -> Option<&Upgrade<DB>> {
        self.upgrades.get(&block_height)
    }
}

#[test]
fn test_validate_upgrade_schedule() {
    let mut us: UpgradeScheduler<fvm_ipld_blockstore::MemoryBlockstore> =
        UpgradeScheduler::default();

    let upgrade = Upgrade {
        block_height: 10,
        migration: |_state| Ok(()),
    };
    us.add_upgrade(upgrade).unwrap();

    let upgrade = Upgrade {
        block_height: 20,
        migration: |_state| Ok(()),
    };
    us.add_upgrade(upgrade).unwrap();

    // adding an upgrade at the same height should fail
    let upgrade = Upgrade {
        block_height: 20,
        migration: |_state| Ok(()),
    };
    let res = us.add_upgrade(upgrade);
    assert!(res.is_err());

    assert!(us.get_upgrade(9).is_none());
    assert!(us.get_upgrade(10).is_some());
}
