// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_vm_upgrade_scheduler::upgrade_scheduler::UpgradeScheduler;
use fvm_ipld_blockstore::Blockstore;

pub fn load_upgrade_scheduler<DB>() -> anyhow::Result<UpgradeScheduler<DB>>
where
    DB: Blockstore + 'static + Clone,
{
    let _us = UpgradeScheduler::default();

    // Add your upgrades to the upgrade_scheduler here:
    //
    // us.add_upgrade(Upgrade {
    //     block_height: 10,
    //     migration: |_state| Ok(()),
    // })?;

    Ok(_us)
}
