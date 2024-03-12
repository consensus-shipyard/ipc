// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::{collections::BTreeMap, path::Path};

use anyhow::{bail, Context};
use fvm_ipld_blockstore::Blockstore;
use serde::{Deserialize, Serialize};
use std::collections::btree_map::Entry::{Occupied, Vacant};

use super::state::{snapshot::BlockHeight, FvmExecState};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpgradeInfo {
    /// the block height at which the upgrade should be executed
    pub block_height: BlockHeight,
    /// the fendermint app version version this Upgrade will migrate to
    pub new_app_version: u64,
    /// the required cometbft version for the upgrade
    pub cometbft_version: String,
    /// whether the upgrade is backwards compatible or not. In case a
    /// non-backwards compatible upgrade is scheduled where we don't have
    /// the corresponding Upgrade defined to migrate to that version, then
    /// fendermint will freeze and not process any more blocks.
    pub backwards_compatible: bool,
}

impl UpgradeInfo {
    pub fn new(
        block_height: BlockHeight,
        cometbft_version: impl ToString,
        new_app_version: u64,
        backwards_compatible: bool,
    ) -> Self {
        Self {
            block_height,
            cometbft_version: cometbft_version.to_string(),
            new_app_version,
            backwards_compatible,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpgradeSchedule {
    // schedule of upgrades to be executed at given height
    schedule: BTreeMap<u64, UpgradeInfo>,
}

impl UpgradeSchedule {
    pub fn new() -> Self {
        Self {
            schedule: BTreeMap::new(),
        }
    }

    pub fn from_file(path: impl AsRef<Path>) -> anyhow::Result<UpgradeSchedule> {
        let json = std::fs::read_to_string(path).context("failed to read upgrade_info file")?;
        let schedules = serde_json::from_str::<Vec<UpgradeInfo>>(&json)
            .context("failed to parse upgrade_info")?;

        let mut us = UpgradeSchedule::new();
        for upgrade_info in schedules {
            us.add(upgrade_info)?;
        }

        Ok(us)
    }

    pub fn to_file(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let values = self
            .schedule
            .values()
            .cloned()
            .collect::<Vec<UpgradeInfo>>();
        let json =
            serde_json::to_string_pretty(&values).context("failed to serialize upgrade_info")?;
        std::fs::write(path, &json)?;

        Ok(())
    }

    pub fn add(&mut self, upgrade_info: UpgradeInfo) -> anyhow::Result<()> {
        match self.schedule.entry(upgrade_info.block_height) {
            Vacant(entry) => {
                entry.insert(upgrade_info);
                Ok(())
            }
            Occupied(_) => {
                bail!("Upgrade schedule already exists");
            }
        }
    }

    pub fn get(&self, block_height: u64) -> Option<&UpgradeInfo> {
        self.schedule.get(&block_height)
    }

    pub fn schedule(&self) -> &BTreeMap<u64, UpgradeInfo> {
        &self.schedule
    }
}

/// a function type for migration
// TODO: Add missing parameters
pub type MigrationFunc<DB> = fn(state: &mut FvmExecState<DB>) -> anyhow::Result<()>;

/// Upgrade implements a migration function to be executed on the fendermint app state which
/// will then upgrade the fendermint version to new_app_version after successful execution
#[derive(Clone)]
pub struct Upgrade<DB>
where
    DB: Blockstore + 'static + Clone,
{
    /// the fendermint app version version this Upgrade will migrate to
    new_app_version: u64,
    /// the migration function to be executed
    migration: MigrationFunc<DB>,
}

impl<DB> Upgrade<DB>
where
    DB: Blockstore + 'static + Clone,
{
    pub fn new(new_app_version: u64, migration: MigrationFunc<DB>) -> Self {
        Self {
            new_app_version,
            migration,
        }
    }

    pub fn execute(&self, state: &mut FvmExecState<DB>) -> anyhow::Result<u64> {
        (self.migration)(state)?;

        Ok(self.new_app_version)
    }
}

// Upgrades is a collection of all the available upgrades
#[derive(Clone)]
pub struct Upgrades<DB>
where
    DB: Blockstore + 'static + Clone,
{
    /// a map of all the available hardcoded upgrades
    upgrades: BTreeMap<u64, Upgrade<DB>>,
}

impl<DB> Upgrades<DB>
where
    DB: Blockstore + 'static + Clone,
{
    pub fn new() -> Self {
        Self {
            upgrades: BTreeMap::new(),
        }
    }
}

impl<DB> Upgrades<DB>
where
    DB: Blockstore + 'static + Clone,
{
    pub fn add(&mut self, upgrade: Upgrade<DB>) -> anyhow::Result<()> {
        match self.upgrades.entry(upgrade.new_app_version) {
            Vacant(entry) => {
                entry.insert(upgrade);
                Ok(())
            }
            Occupied(_) => {
                bail!("Upgrade already exists");
            }
        }
    }

    pub fn get(&self, new_app_version: u64) -> Option<&Upgrade<DB>> {
        self.upgrades.get(&new_app_version)
    }
}

#[test]
fn test_validate_upgrade_schedule() {
    use crate::fvm::store::memory::MemoryBlockstore;

    let mut upgrades: Upgrades<MemoryBlockstore> = Upgrades::new();
    upgrades.add(Upgrade::new(1, |_state| Ok(()))).unwrap();
    upgrades.add(Upgrade::new(2, |_state| Ok(()))).unwrap();

    // adding an upgrade with the same chain_id and height should fail
    let res = upgrades.add(Upgrade::new(2, |_state| Ok(())));
    assert!(res.is_err());

    assert!(upgrades.get(0).is_none());
    assert!(upgrades.get(1).is_some());
}
