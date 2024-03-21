// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_rocksdb::blockstore::NamespaceBlockstore;
use fendermint_vm_interpreter::fvm::upgrades::{Upgrade, UpgradeScheduler};
use fvm_shared::chainid::ChainID;
use patch_actor_state::patch_actor_state_func;
use upgrade_wasm_actor::upgrade_wasm_actor_func;

pub mod patch_actor_state;
pub mod upgrade_wasm_actor;

pub fn get_example_upgrade_scheduler() -> UpgradeScheduler<NamespaceBlockstore> {
    let chain_id = ChainID::from(1942764459484029);

    let mut scheduler = UpgradeScheduler::new();

    // At block height 5, we patch the chainmetadata actor to only save previous 16 blockhashes (instead of 256)
    scheduler
        .add(Upgrade::new_by_id(
            chain_id,
            5,
            None,
            patch_actor_state_func,
        ))
        .unwrap();

    // At block height 10, we replace the chainmetadata actor with a new version which returns the current number
    // of blockhashes saved. Starting from this height, we can check the number of blockhashes saved by the
    // chainmetadata actor and they should go from 10, 11, .. 16 and stay there.
    scheduler
        .add(Upgrade::new_by_id(
            chain_id,
            10,
            None,
            upgrade_wasm_actor_func,
        ))
        .unwrap();

    scheduler
}
