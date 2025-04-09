// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! IPC related execution

use crate::app::{AppState, AppStoreKey};
use crate::{App, BlockHeight};
use anyhow::anyhow;
use async_trait::async_trait;
use fendermint_storage::{Codec, Encode, KVReadable, KVStore, KVWritable};
use fendermint_vm_interpreter::fvm::state::ipc::GatewayCaller;
use fendermint_vm_interpreter::fvm::state::{FvmExecState, FvmStateParams};
use fendermint_vm_interpreter::fvm::store::ReadOnlyBlockstore;
use fendermint_vm_interpreter::fvm::Broadcaster;
use fendermint_vm_interpreter::MessagesInterpreter;
use fendermint_vm_message::chain::{ChainMessage, ValidatorMessage};
use fendermint_vm_topdown::cache::ParentViewPayload;
use fendermint_vm_topdown::sync::FendermintStateQuery;
use fendermint_vm_topdown::{ParentState, TopdownVoter};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::Address;
use fvm_shared::chainid::ChainID;
use fvm_shared::clock::ChainEpoch;
use ipc_api::checkpoint::TopdownCheckpoint;
use ipc_api::evm::payload_to_evm_address;
use std::sync::Arc;

pub struct AppTopdownVoter<SS>
where
    SS: Blockstore + Clone + 'static + Send + Sync,
{
    gateway_caller: GatewayCaller<ReadOnlyBlockstore<Arc<SS>>>,
    broadcaster: Broadcaster<tendermint_rpc::HttpClient>,
}

/// Queries the LATEST COMMITTED parent finality from the storage
pub struct AppParentFinalityQuery<DB, SS, S, I>
where
    SS: Blockstore + Clone + 'static + Send + Sync,
    S: KVStore,
    I: MessagesInterpreter<SS> + Send + Sync,
{
    /// The app to get state
    app: App<DB, SS, S, I>,
    gateway_caller: GatewayCaller<ReadOnlyBlockstore<Arc<SS>>>,
}

impl<SS: Blockstore + Clone + 'static + Send + Sync> AppTopdownVoter<SS> {
    pub fn new(broadcaster: Broadcaster<tendermint_rpc::HttpClient>) -> Self {
        Self {
            gateway_caller: Default::default(),
            broadcaster,
        }
    }
}

impl<DB, SS, S, I> AppParentFinalityQuery<DB, SS, S, I>
where
    S: KVStore
        + Codec<AppState>
        + Encode<AppStoreKey>
        + Encode<BlockHeight>
        + Codec<FvmStateParams>,
    DB: KVWritable<S> + KVReadable<S> + 'static + Clone,
    SS: Blockstore + Clone + 'static + Send + Sync,
    I: MessagesInterpreter<SS> + Send + Sync,
{
    pub fn new(app: App<DB, SS, S, I>) -> Self {
        Self {
            app,
            gateway_caller: GatewayCaller::default(),
        }
    }

    fn with_exec_state<F, T>(&self, f: F) -> anyhow::Result<Option<T>>
    where
        F: FnOnce(FvmExecState<ReadOnlyBlockstore<Arc<SS>>>) -> anyhow::Result<T>,
    {
        match self.app.read_only_view(None)? {
            Some(s) => f(s).map(Some),
            None => Ok(None),
        }
    }
}

impl<DB, SS, S, I> FendermintStateQuery for AppParentFinalityQuery<DB, SS, S, I>
where
    S: KVStore
        + Codec<AppState>
        + Encode<AppStoreKey>
        + Encode<BlockHeight>
        + Codec<FvmStateParams>,
    DB: KVWritable<S> + KVReadable<S> + 'static + Clone,
    SS: Blockstore + Clone + 'static + Send + Sync,
    I: MessagesInterpreter<SS> + Send + Sync,
{
    fn get_latest_topdown_parent_state(&self) -> anyhow::Result<Option<ParentState>> {
        self.with_exec_state(|mut exec_state| {
            self.gateway_caller
                .get_latest_topdown_parent_state(&mut exec_state)
        })
    }

    fn has_voted(&self, validator: &Address) -> anyhow::Result<bool> {
        let eth_address = payload_to_evm_address(validator.payload())?;
        self.with_exec_state(|mut s| self.gateway_caller.has_voted(&mut s, eth_address))?
            .ok_or_else(|| anyhow!("app is not up"))
    }

    fn get_chain_id(&self) -> anyhow::Result<ChainID> {
        let r = self.with_exec_state(|s| Ok(s.chain_id()))?;
        r.ok_or_else(|| anyhow!("chain id not available as app is not up"))
    }
}

#[async_trait]
impl<SS> TopdownVoter for AppTopdownVoter<SS>
where
    SS: Blockstore + Clone + 'static + Send + Sync,
{
    async fn vote(
        &self,
        chain_id: ChainID,
        height: BlockHeight,
        parent_view_payload: ParentViewPayload,
    ) -> anyhow::Result<()> {
        let cp = TopdownCheckpoint {
            parent_height: height as ChainEpoch,
            parent_block_hash: parent_view_payload.0,
            xnet_msgs: parent_view_payload.2,
            power_changes: parent_view_payload.1,
        };
        let calldata = self.gateway_caller.propose_calldata(cp.try_into()?)?;
        self.broadcaster
            .fevm_invoke(
                Address::from(self.gateway_caller.addr()),
                calldata,
                chain_id,
                |s| ChainMessage::Validator(ValidatorMessage::TopdownPropose(s)),
            )
            .await?;
        Ok(())
    }
}
