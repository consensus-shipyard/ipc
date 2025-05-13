// Copyright 2025 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

use cid::multihash::Multihash;
use cid::Cid;
use fil_actors_runtime::runtime::{Primitives, RuntimePolicy};
use fil_actors_runtime::{
    runtime::{fvm::FvmRuntime, Runtime},
    test_blockstores::MemoryBlockstore,
};
use fvm_ipld_encoding::{Cbor, CborStore, CBOR};

use crate::actor::Actor;

use super::*;

struct DummyRuntime {
    blockstore: Arc<MemoryBlockstore>,
    root: Arc<Mutex<Cid>>,
    // state: Arc<Mutex<State>>,
}

impl DummyRuntime {
    /// Setup the runtime with a genesis state
    fn init_genesis() -> Self {
        let mut blockstore = Arc::new(MemoryBlockstore::new());

        // genesis state
        let mut state = State::new(&blockstore, 2).unwrap();
        let root = blockstore
            .put_cbor(&state, cid::multihash::Code::Blake2b256)
            .unwrap();
        let root = Arc::new(Mutex::new(root));

        Self {
            root,
            blockstore,
            // state,
        }
    }
}

impl RuntimePolicy for DummyRuntime {
    fn policy(&self) -> &fil_actors_runtime::runtime::Policy {
        todo!()
    }
}

impl Primitives for DummyRuntime {
    fn batch_verify_seals(
        &self,
        batch: &[fvm_shared::sector::SealVerifyInfo],
    ) -> anyhow::Result<Vec<bool>> {
        todo!()
    }
    fn compute_unsealed_sector_cid(
        &self,
        proof_type: fvm_shared::sector::RegisteredSealProof,
        pieces: &[fvm_shared::piece::PieceInfo],
    ) -> Result<cid::Cid, anyhow::Error> {
        todo!()
    }
    fn hash(&self, hasher: fvm_shared::crypto::hash::SupportedHashes, data: &[u8]) -> Vec<u8> {
        todo!()
    }
    fn hash_64(
        &self,
        hasher: fvm_shared::crypto::hash::SupportedHashes,
        data: &[u8],
    ) -> ([u8; 64], usize) {
        todo!()
    }
    fn hash_blake2b(&self, data: &[u8]) -> [u8; 32] {
        todo!()
    }
    fn recover_secp_public_key(
        &self,
        hash: &[u8; fvm_shared::crypto::signature::SECP_SIG_MESSAGE_HASH_SIZE],
        signature: &[u8; fvm_shared::crypto::signature::SECP_SIG_LEN],
    ) -> Result<[u8; fvm_shared::crypto::signature::SECP_PUB_LEN], anyhow::Error> {
        todo!()
    }
    fn verify_aggregate_seals(
        &self,
        aggregate: &fvm_shared::sector::AggregateSealVerifyProofAndInfos,
    ) -> Result<(), anyhow::Error> {
        todo!()
    }
    fn verify_consensus_fault(
        &self,
        h1: &[u8],
        h2: &[u8],
        extra: &[u8],
    ) -> Result<Option<fvm_shared::consensus::ConsensusFault>, anyhow::Error> {
        todo!()
    }
    fn verify_post(
        &self,
        verify_info: &fvm_shared::sector::WindowPoStVerifyInfo,
    ) -> Result<(), anyhow::Error> {
        todo!()
    }
    fn verify_replica_update(
        &self,
        replica: &fvm_shared::sector::ReplicaUpdateInfo,
    ) -> Result<(), anyhow::Error> {
        todo!()
    }
    fn verify_signature(
        &self,
        signature: &fvm_shared::crypto::signature::Signature,
        signer: &fvm_shared::address::Address,
        plaintext: &[u8],
    ) -> Result<(), anyhow::Error> {
        todo!()
    }
}

impl Runtime for DummyRuntime {
    type Blockstore = Arc<MemoryBlockstore>;
    fn transaction<S, RT, F>(&self, f: F) -> Result<RT, fil_actors_runtime::ActorError>
    where
        S: serde::Serialize + serde::de::DeserializeOwned,
        F: FnOnce(&mut S, &Self) -> Result<RT, fil_actors_runtime::ActorError>,
    {
        let mut root = self.root.lock().unwrap();
        dbg!(*root);
        let mut state = self.blockstore.get_cbor::<S>(&*root).unwrap().unwrap();
        let rt = f(&mut state, self)?;

        let new_root = self
            .blockstore
            .put_cbor(&state, cid::multihash::Code::Blake2b256)
            .unwrap();
        dbg!(&new_root);
        *root = new_root;
        Ok(rt)
    }

    fn validate_immediate_caller_is<'a, I>(
        &self,
        addresses: I,
    ) -> Result<(), fil_actors_runtime::ActorError>
    where
        I: IntoIterator<Item = &'a fvm_shared::address::Address>,
    {
        Ok(())
    }

    fn actor_balance(&self, id: fvm_shared::ActorID) -> Option<fvm_shared::econ::TokenAmount> {
        todo!()
    }

    fn base_fee(&self) -> fvm_shared::econ::TokenAmount {
        todo!()
    }

    fn chain_id(&self) -> fvm_shared::chainid::ChainID {
        todo!()
    }
    fn charge_gas(&self, name: &'static str, compute: i64) {
        todo!()
    }
    fn create<T: serde::Serialize>(&self, obj: &T) -> Result<(), fil_actors_runtime::ActorError> {
        Ok(())
    }
    fn create_actor(
        &self,
        code_id: cid::Cid,
        actor_id: fvm_shared::ActorID,
        predictable_address: Option<fvm_shared::address::Address>,
    ) -> Result<(), fil_actors_runtime::ActorError> {
        todo!()
    }
    fn curr_epoch(&self) -> fvm_shared::clock::ChainEpoch {
        1 as _
    }
    fn current_balance(&self) -> fvm_shared::econ::TokenAmount {
        todo!()
    }
    fn delete_actor(&self) -> Result<(), fil_actors_runtime::ActorError> {
        todo!()
    }
    fn emit_event(
        &self,
        event: &fvm_shared::event::ActorEvent,
    ) -> Result<(), fil_actors_runtime::ActorError> {
        todo!()
    }
    fn gas_available(&self) -> u64 {
        todo!()
    }
    fn get_actor_code_cid(&self, id: &fvm_shared::ActorID) -> Option<cid::Cid> {
        todo!()
    }
    fn get_beacon_randomness(
        &self,
        rand_epoch: fvm_shared::clock::ChainEpoch,
    ) -> Result<[u8; fvm_shared::randomness::RANDOMNESS_LENGTH], fil_actors_runtime::ActorError>
    {
        todo!()
    }
    fn get_code_cid_for_type(&self, typ: fil_actors_runtime::runtime::builtins::Type) -> cid::Cid {
        todo!()
    }
    fn get_randomness_from_beacon(
        &self,
        personalization: fil_actors_runtime::runtime::DomainSeparationTag,
        rand_epoch: fvm_shared::clock::ChainEpoch,
        entropy: &[u8],
    ) -> Result<[u8; fvm_shared::randomness::RANDOMNESS_LENGTH], fil_actors_runtime::ActorError>
    {
        todo!()
    }
    fn get_randomness_from_tickets(
        &self,
        personalization: fil_actors_runtime::runtime::DomainSeparationTag,
        rand_epoch: fvm_shared::clock::ChainEpoch,
        entropy: &[u8],
    ) -> Result<[u8; fvm_shared::randomness::RANDOMNESS_LENGTH], fil_actors_runtime::ActorError>
    {
        todo!()
    }
    fn get_state_root(&self) -> Result<cid::Cid, fil_actors_runtime::ActorError> {
        Ok(self.root.lock().unwrap().clone())
    }
    fn lookup_delegated_address(
        &self,
        id: fvm_shared::ActorID,
    ) -> Option<fvm_shared::address::Address> {
        todo!()
    }
    fn message(&self) -> &dyn fil_actors_runtime::runtime::MessageInfo {
        todo!()
    }
    fn network_version(&self) -> fvm_shared::version::NetworkVersion {
        todo!()
    }
    fn new_actor_address(
        &self,
    ) -> Result<fvm_shared::address::Address, fil_actors_runtime::ActorError> {
        todo!()
    }
    fn read_only(&self) -> bool {
        todo!()
    }
    fn resolve_address(
        &self,
        address: &fvm_shared::address::Address,
    ) -> Option<fvm_shared::ActorID> {
        todo!()
    }
    fn resolve_builtin_actor_type(
        &self,
        code_id: &cid::Cid,
    ) -> Option<fil_actors_runtime::runtime::builtins::Type> {
        todo!()
    }
    fn send(
        &self,
        to: &fvm_shared::address::Address,
        method: fvm_shared::MethodNum,
        params: Option<fvm_ipld_encoding::ipld_block::IpldBlock>,
        value: fvm_shared::econ::TokenAmount,
        gas_limit: Option<u64>,
        flags: fvm_shared::sys::SendFlags,
    ) -> Result<fvm_shared::Response, fil_actors_runtime::SendError> {
        todo!()
    }
    fn send_simple(
        &self,
        to: &fvm_shared::address::Address,
        method: fvm_shared::MethodNum,
        params: Option<fvm_ipld_encoding::ipld_block::IpldBlock>,
        value: fvm_shared::econ::TokenAmount,
    ) -> Result<fvm_shared::Response, fil_actors_runtime::SendError> {
        todo!()
    }
    fn set_state_root(&self, root: &cid::Cid) -> Result<(), fil_actors_runtime::ActorError> {
        todo!()
    }
    fn state<T: serde::de::DeserializeOwned>(&self) -> Result<T, fil_actors_runtime::ActorError> {
        todo!()
    }
    fn store(&self) -> &Self::Blockstore {
        &self.blockstore
    }
    fn tipset_cid(&self, epoch: i64) -> Result<cid::Cid, fil_actors_runtime::ActorError> {
        todo!()
    }
    fn tipset_timestamp(&self) -> u64 {
        todo!()
    }
    fn total_fil_circ_supply(&self) -> fvm_shared::econ::TokenAmount {
        todo!()
    }
    fn validate_immediate_caller_accept_any(&self) -> Result<(), fil_actors_runtime::ActorError> {
        todo!()
    }
    fn validate_immediate_caller_namespace<I>(
        &self,
        namespace_manager_addresses: I,
    ) -> Result<(), fil_actors_runtime::ActorError>
    where
        I: IntoIterator<Item = u64>,
    {
        todo!()
    }
    fn validate_immediate_caller_type<'a, I>(
        &self,
        types: I,
    ) -> Result<(), fil_actors_runtime::ActorError>
    where
        I: IntoIterator<Item = &'a fil_actors_runtime::runtime::builtins::Type>,
    {
        todo!()
    }
}

#[test]
fn fvm_actor_ordering_stays_identical() {
    let mut rt = DummyRuntime::init_genesis();

    Actor::constructor(&mut rt, ConstructorParams { lookback_len: 2 }).unwrap();
    Actor::push_block_hash(
        &mut rt,
        PushBlockParams {
            block: [0; 32],
            epoch: 1,
        },
    )
    .unwrap();
    Actor::push_block_hash(
        &mut rt,
        PushBlockParams {
            block: [1; 32],
            epoch: 1,
        },
    )
    .unwrap();
    Actor::push_block_hash(
        &mut rt,
        PushBlockParams {
            block: [2; 32],
            epoch: 1,
        },
    )
    .unwrap();
    Actor::push_block_hash(
        &mut rt,
        PushBlockParams {
            block: [3; 32],
            epoch: 1,
        },
    )
    .unwrap();
}
