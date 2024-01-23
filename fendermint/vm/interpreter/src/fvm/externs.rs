// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use anyhow::anyhow;
use cid::Cid;
use fendermint_actors::CHAINMETADATA_ACTOR_ID;
use fil_actors_runtime::Array;
use fvm::{
    externs::{Chain, Consensus, Externs, Rand},
    state_tree::StateTree,
};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::CborStore;
use fvm_shared::clock::ChainEpoch;
use std::str::FromStr;

use super::store::ReadOnlyBlockstore;

pub struct FendermintExterns<DB>
where
    DB: Blockstore + 'static,
{
    blockstore: DB,
    state_root: Cid,
}

impl<DB> FendermintExterns<DB>
where
    DB: Blockstore + 'static,
{
    pub fn new(blockstore: DB, state_root: Cid) -> Self {
        Self {
            blockstore,
            state_root,
        }
    }
}

impl<DB> Rand for FendermintExterns<DB>
where
    DB: Blockstore + 'static,
{
    fn get_chain_randomness(&self, _round: ChainEpoch) -> anyhow::Result<[u8; 32]> {
        todo!("might need randomness")
    }

    fn get_beacon_randomness(&self, _round: ChainEpoch) -> anyhow::Result<[u8; 32]> {
        unimplemented!("not expecting to use the beacon")
    }
}

impl<DB> Consensus for FendermintExterns<DB>
where
    DB: Blockstore + 'static,
{
    fn verify_consensus_fault(
        &self,
        _h1: &[u8],
        _h2: &[u8],
        _extra: &[u8],
    ) -> anyhow::Result<(Option<fvm_shared::consensus::ConsensusFault>, i64)> {
        unimplemented!("not expecting to use consensus faults")
    }
}

impl<DB> Chain for FendermintExterns<DB>
where
    DB: Blockstore + Clone + 'static,
{
    // for retreiving the tipset_cid, we load the chain metadata actor state
    // at the given state_root and retrieve the blockhash for the given epoch
    fn get_tipset_cid(&self, epoch: ChainEpoch) -> anyhow::Result<Cid> {
        // create a read only state tree from the state root
        let bstore = ReadOnlyBlockstore::new(&self.blockstore);
        let state_tree = StateTree::new_from_root(&bstore, &self.state_root)?;

        // get the chain metadata actor state cid
        let actor_state_cid = match state_tree.get_actor(CHAINMETADATA_ACTOR_ID) {
            Ok(Some(actor_state)) => actor_state.state,
            Ok(None) => {
                return Err(anyhow!(
                    "chain metadata actor id ({}) not found in state",
                    CHAINMETADATA_ACTOR_ID
                ));
            }
            Err(err) => {
                return Err(anyhow!(
                    "failed to get chain metadata actor ({}) state, error: {}",
                    CHAINMETADATA_ACTOR_ID,
                    err
                ));
            }
        };

        // get the chain metadata actor state from the blockstore
        let actor_state: fendermint_actor_chainmetadata::State =
            match state_tree.store().get_cbor(&actor_state_cid) {
                Ok(Some(v)) => v,
                Ok(None) => {
                    return Err(anyhow!(
                        "chain metadata actor ({}) state not found",
                        CHAINMETADATA_ACTOR_ID
                    ));
                }
                Err(err) => {
                    return Err(anyhow!(
                        "failed to get chain metadata actor ({}) state, error: {}",
                        CHAINMETADATA_ACTOR_ID,
                        err
                    ));
                }
            };

        // load the blockhashe Array from the AMT root cid
        let blockhashes = Array::load(&actor_state.blockhashes, &bstore)?;

        // get the block hash at the given epoch
        let blockhash: &String = match blockhashes.get(epoch as u64).unwrap() {
            Some(v) => v,
            None => {
                return Ok(Cid::default());
            }
        };

        // return the blockhash as a cid, or an error if the cid is invalid
        match Cid::from_str(blockhash.as_str()) {
            Ok(cid) => Ok(cid),
            Err(_) => Err(anyhow!(
                "failed to parse cid, blockhash: {}, epoch: {}",
                blockhash,
                epoch
            )),
        }
    }
}

impl<DB> Externs for FendermintExterns<DB> where DB: Blockstore + Clone + 'static {}
