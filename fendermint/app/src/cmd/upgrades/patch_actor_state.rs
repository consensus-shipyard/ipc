// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::anyhow;
use cid::multihash::Code;
use fendermint_actor_chainmetadata::State;
use fendermint_rocksdb::blockstore::NamespaceBlockstore;
use fendermint_vm_actor_interface::chainmetadata::CHAINMETADATA_ACTOR_ID;
use fendermint_vm_interpreter::fvm::upgrades::Upgrade;
use fvm::state_tree::ActorState;
use fvm_ipld_encoding::CborStore;
use lazy_static::lazy_static;

use crate::cmd::upgrades::EXAMPLE_CHAIN_ID;

lazy_static! {
    // This example upgrade shows how we can patch the state of an actor
    //
    pub static ref UPGRADE_EXAMPLE_PATCH_STATE: Upgrade<NamespaceBlockstore> = Upgrade::new_by_id(
        *EXAMPLE_CHAIN_ID,
        100,  // the upgrade block height
        None, // this upgrade does not change the application version as its not consensus breaking
        |state| {
            let state_tree = state.state_tree_mut();

            // get the ActorState from the state tree
            //
            let actor_state = match state_tree.get_actor(CHAINMETADATA_ACTOR_ID)? {
                Some(actor) => actor,
                None => {
                    return Err(anyhow!("chainmetadata actor not found"));
                }
            };
            println!("chainmetadata code_cid: {:?}, state_cid: {:?}", actor_state.code, actor_state.state);

            // retrieve the chainmetadata actor state from the blockstore
            //
            let mut chainmetadata_state: State = match state_tree.store().get_cbor(&actor_state.state)? {
                Some(v) => v,
                None => return Err(anyhow!("chain metadata actor state not found")),
            };
            println!("chainmetadata lookback length: {}", chainmetadata_state.lookback_len);

            // lets patch the state, here we update the lookback_len from the default (256) to 1024
            //
            chainmetadata_state.lookback_len = 1024;

            // store the updated state back to the blockstore and get the new state cid
            //
            let new_state_cid = state_tree
                .store()
                .put_cbor(&chainmetadata_state, Code::Blake2b256)
                .map_err(|e| anyhow!("failed to put chain metadata actor state: {}", e))?;
            println!("new chainmetadata state_cid: {:?}", new_state_cid);

            // next we update the actor state in the state tree
            //
            state_tree.set_actor(CHAINMETADATA_ACTOR_ID, ActorState {
                code: actor_state.code,
                state: new_state_cid,
                sequence: actor_state.sequence,
                balance: actor_state.balance,
                delegated_address: actor_state.delegated_address,
            });

            Ok(())
        },
    );
}
