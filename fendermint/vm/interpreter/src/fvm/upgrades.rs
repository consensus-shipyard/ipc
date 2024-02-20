// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_ipld_blockstore::Blockstore;

use super::upgrade_scheduler::UpgradeScheduler;

pub fn load_upgrade_scheduler<DB>() -> anyhow::Result<UpgradeScheduler<DB>>
where
    DB: Blockstore + 'static + Clone,
{
    let mut us = UpgradeScheduler::default();

    // Add your upgrades to the upgrade_scheduler here:
    //
    // us.add_upgrade(Upgrade {
    //     block_height: 10,
    //     migration: |_state| Ok(()),
    // })?;

    let upgrade = super::upgrade_scheduler::Upgrade {
        chain_id: 1942764459484029,
        block_height: 10,
        migration: |_state| Ok(()),
    };
    us.add_upgrade(upgrade).unwrap();

    let upgrade = super::upgrade_scheduler::Upgrade {
        chain_id: 1942764459484029,
        block_height: 20,
        migration: |_state| Ok(()),
    };
    us.add_upgrade(upgrade).unwrap();

    Ok(us)
}
