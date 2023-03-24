// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, Context};
use cid::{multihash::Code, Cid};
use fendermint_vm_actor_interface::{account, init, multisig};
use fendermint_vm_genesis::{Account, Multisig};
use fvm::{
    machine::Manifest,
    state_tree::{ActorState, StateTree},
};
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_car::load_car_unchecked;
use fvm_ipld_encoding::CborStore;
use fvm_shared::{clock::ChainEpoch, econ::TokenAmount, state::StateTreeVersion, ActorID};
use num_traits::Zero;
use serde::Serialize;

/// A state we create for the execution of genesis initialisation.
pub struct FvmGenesisState<DB>
where
    DB: Blockstore,
{
    pub manifest_data_cid: Cid,
    pub manifest: Manifest,
    state_tree: StateTree<DB>,
}

impl<DB> FvmGenesisState<DB>
where
    DB: Blockstore,
{
    pub async fn new(store: DB, bundle: &[u8]) -> anyhow::Result<Self> {
        // Load the actor bundle.
        let bundle_roots = load_car_unchecked(&store, bundle).await?;
        let bundle_root = match bundle_roots.as_slice() {
            [root] => root,
            roots => {
                return Err(anyhow!(
                    "expected one root in actor bundle; got {}",
                    roots.len()
                ))
            }
        };

        let (manifest_version, manifest_data_cid): (u32, Cid) = match store.get_cbor(bundle_root)? {
            Some(vd) => vd,
            None => {
                return Err(anyhow!(
                    "no manifest information in bundle root {}",
                    bundle_root
                ))
            }
        };
        let manifest = Manifest::load(&store, &manifest_data_cid, manifest_version)?;

        // Create an empty state tree.
        let state_tree = StateTree::new(store, StateTreeVersion::V5)?;

        let state = Self {
            manifest_data_cid,
            manifest,
            state_tree,
        };

        Ok(state)
    }

    /// Flush the data to the block store.
    pub fn commit(mut self) -> anyhow::Result<Cid> {
        let root = self.state_tree.flush()?;
        Ok(root)
    }

    /// Creates an actor using code specified in the manifest.
    pub fn create_actor(
        &mut self,
        code_id: u32,
        id: ActorID,
        state: &impl Serialize,
        balance: TokenAmount,
    ) -> anyhow::Result<()> {
        // Retrieve the CID of the actor code by the numeric ID.
        let code_cid = *self
            .manifest
            .code_by_id(code_id)
            .ok_or_else(|| anyhow!("can't find {code_id} in the manifest"))?;

        let state_cid = self.put_state(state)?;

        let actor_state = ActorState {
            code: code_cid,
            state: state_cid,
            sequence: 0,
            balance,
            delegated_address: None,
        };

        self.state_tree.set_actor(id, actor_state);

        Ok(())
    }

    pub fn create_account_actor(
        &mut self,
        acct: Account,
        balance: TokenAmount,
        ids: &init::AddressMap,
    ) -> anyhow::Result<()> {
        let owner = acct.owner.0;
        let state = account::State { address: owner };

        let id = ids
            .get(&owner)
            .ok_or_else(|| anyhow!("can't find ID for owner {owner}"))?;

        self.create_actor(account::ACCOUNT_ACTOR_CODE_ID, *id, &state, balance)
    }

    pub fn create_multisig_actor(
        &mut self,
        ms: Multisig,
        balance: TokenAmount,
        ids: &init::AddressMap,
        next_id: ActorID,
    ) -> anyhow::Result<()> {
        let mut signers = Vec::new();

        // Make sure every signer has their own account.
        for signer in ms.signers {
            let id = ids
                .get(&signer.0)
                .ok_or_else(|| anyhow!("can't find ID for signer {}", signer.0))?;

            if self.state_tree.get_actor(*id)?.is_none() {
                self.create_account_actor(Account { owner: signer }, TokenAmount::zero(), ids)?;
            }

            signers.push(*id)
        }

        // Now create a multisig actor that manages group transactions.
        let state = multisig::State::new(
            self.state_tree.store(),
            signers,
            ms.threshold,
            ms.vesting_start as ChainEpoch,
            ms.vesting_duration as ChainEpoch,
            balance.clone(),
        )?;

        self.create_actor(multisig::MULTISIG_ACTOR_CODE_ID, next_id, &state, balance)
    }

    pub fn store(&self) -> &DB {
        self.state_tree.store()
    }

    fn put_state(&mut self, state: impl Serialize) -> anyhow::Result<Cid> {
        self.state_tree
            .store()
            .put_cbor(&state, Code::Blake2b256)
            .context("failed to store actor state")
    }
}
