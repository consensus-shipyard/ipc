// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::cmd::upgrades::CHAIN_ID;
use ethers::abi::Address;
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_interpreter::fvm::state::fevm::ContractCaller;
use fendermint_vm_interpreter::fvm::upgrades::{Upgrade, UpgradeScheduler};
use fvm_ipld_blockstore::Blockstore;
use ipc_actors_abis::ownership_facet::{OwnershipFacet, OwnershipFacetErrors};
use std::str::FromStr;
use tracing::info;

#[allow(dead_code)]
pub(crate) fn transfer_ownership<DB: Blockstore + 'static + Clone>(
    upgrade_scheduler: &mut UpgradeScheduler<DB>,
    block_height: u64,
) -> anyhow::Result<()> {
    // transfer ownership of the gateway to target address
    upgrade_scheduler.add(Upgrade::new_by_id(
        CHAIN_ID.into(),
        block_height,
        None,
        |state| {
            // TODO: update to the actual new owner address
            let new_owner = Address::from_str("0x1A79385eAd0e873FE0C441C034636D3Edf7014cC")?;
            let cur_owner = Address::from_str("0xfF00000000000000000000000000000000000000")?;
            let gateway_addr = Address::from_str("0x77aa40b105843728088c0132e43fc44348881da8")?;

            info!(
                "[Upgrade at height {}] Change gateway ownership",
                state.block_height()
            );

            let ownership = ContractCaller::<_, _, OwnershipFacetErrors>::new(
                EthAddress::from(gateway_addr),
                OwnershipFacet::new,
            );
            ownership.call_with_return(state, |c| {
                let mut call = c.transfer_ownership(new_owner);
                call = call.from(cur_owner);
                call
            })?;
            let owner = ownership.call(state, |c| c.owner())?;
            info!(owner = owner.to_string(), "updated gateway ownership");

            Ok(())
        },
    ))
}
