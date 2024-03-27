// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::anyhow;
use cid::multihash::Code;
use fendermint_rocksdb::blockstore::NamespaceBlockstore;
use fendermint_vm_actor_interface::chainmetadata::CHAINMETADATA_ACTOR_ID;
use fendermint_vm_interpreter::fvm::state::FvmExecState;
use fvm::state_tree::ActorState;
use fvm_ipld_blockstore::Block;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::IPLD_RAW;

static WASM_BIN: &[u8] = include_bytes!("../output/fendermint_actor_chainmetadata_v2.wasm");

pub fn upgrade_wasm_actor_func(
    state: &mut FvmExecState<NamespaceBlockstore>,
) -> anyhow::Result<()> {
    let state_tree = state.state_tree_mut();

    // get the ActorState from the state tree
    //
    let actor_state = match state_tree.get_actor(CHAINMETADATA_ACTOR_ID)? {
        Some(actor) => actor,
        None => {
            return Err(anyhow!("chainmetadata actor not found"));
        }
    };
    println!(
        "chainmetadata code_cid: {:?}, state_cid: {:?}",
        actor_state.code, actor_state.state
    );

    // store the new wasm code in the blockstore and get the new code cid
    //
    let new_code_cid = state_tree.store().put(
        Code::Blake2b256,
        &Block {
            codec: IPLD_RAW,
            data: WASM_BIN,
        },
    )?;
    println!("new chainmetadata code_cid: {:?}", new_code_cid);

    // next we update the actor state in the state tree
    //
    state_tree.set_actor(
        CHAINMETADATA_ACTOR_ID,
        ActorState {
            code: new_code_cid,
            state: actor_state.state,
            sequence: actor_state.sequence,
            balance: actor_state.balance,
            delegated_address: actor_state.delegated_address,
        },
    );

    Ok(())
}
