// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

mod upgrade01;
mod upgrade02;

use fendermint_vm_interpreter::fvm::upgrades::UpgradeScheduler;
use fvm_ipld_blockstore::Blockstore;

const CHAIN_ID: u64 = 901861227013395;

pub fn create_upgrade_scheduler<DB: Blockstore + 'static + Clone>() -> UpgradeScheduler<DB> {
    let mut upgrade_scheduler = UpgradeScheduler::new();

    // upgrade ownership
    upgrade01::new_upgrade(&mut upgrade_scheduler);

    // applied missing validator changes
    upgrade02::new_upgrade(&mut upgrade_scheduler);

    upgrade_scheduler
}
