// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

mod upgrade01;
mod upgrade02;

use fendermint_vm_interpreter::fvm::upgrades::UpgradeScheduler;
use fvm_ipld_blockstore::Blockstore;

// TODO: Update the chain id to your respective chain id
const CHAIN_ID: u64 = 901861227013395;

pub fn create_upgrade_scheduler<DB: Blockstore + 'static + Clone>(
) -> anyhow::Result<UpgradeScheduler<DB>> {
    let mut upgrade_scheduler = UpgradeScheduler::new();

    // applied missing validator changes
    // TODO: update target height
    let target_height = 50;
    upgrade01::store_missing_validator_changes(&mut upgrade_scheduler, target_height)?;

    // upgrade ownership, optional
    // TODO: update target height
    let target_height = 60;
    upgrade02::transfer_ownership(&mut upgrade_scheduler, target_height)?;

    Ok(upgrade_scheduler)
}
