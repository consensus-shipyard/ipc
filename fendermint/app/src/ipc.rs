// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! IPC related execution

use crate::app::{AppState, AppStoreKey};
use crate::{App, BlockHeight};
use fendermint_storage::{Codec, Encode, KVReadable, KVStore, KVWritable};
use fendermint_vm_genesis::{Power, Validator};
use fendermint_vm_interpreter::fvm::state::ipc::GatewayCaller;
use fendermint_vm_interpreter::fvm::state::{FvmExecState, FvmStateParams};
use fendermint_vm_interpreter::fvm::store::ReadOnlyBlockstore;
use fendermint_vm_topdown::sync::ParentFinalityStateQuery;
use fendermint_vm_topdown::IPCParentFinality;
use fvm_ipld_blockstore::Blockstore;
use std::sync::Arc;

use tendermint_rpc::Client as TendermintClient;

use serde::{Deserialize, Serialize};

/// All the things that can be voted on in a subnet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppVote {
    /// The validator considers a certain block final on the parent chain.
    ParentFinality(IPCParentFinality),
}

/// Queries the LATEST COMMITTED parent finality from the storage
pub struct AppParentFinalityQuery<DB, SS, S, TC>
where
    SS: Blockstore + Clone + 'static + Send + Sync,
    TC: TendermintClient + Clone + Send + Sync + 'static,
    S: KVStore,
{
    /// The app to get state
    app: App<DB, SS, S, TC>,
    gateway_caller: GatewayCaller<ReadOnlyBlockstore<Arc<SS>>>,
}

impl<DB, SS, S, TC> AppParentFinalityQuery<DB, SS, S, TC>
where
    S: KVStore
        + Codec<AppState>
        + Encode<AppStoreKey>
        + Encode<BlockHeight>
        + Codec<FvmStateParams>,
    DB: KVWritable<S> + KVReadable<S> + 'static + Clone,
    SS: Blockstore + Clone + 'static + Send + Sync,
    TC: TendermintClient + Clone + Send + Sync + 'static,
{
    pub fn new(app: App<DB, SS, S, TC>) -> Self {
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

impl<DB, SS, S, TC> ParentFinalityStateQuery for AppParentFinalityQuery<DB, SS, S, TC>
where
    S: KVStore
        + Codec<AppState>
        + Encode<AppStoreKey>
        + Encode<BlockHeight>
        + Codec<FvmStateParams>,
    DB: KVWritable<S> + KVReadable<S> + 'static + Clone,
    SS: Blockstore + Clone + 'static + Send + Sync,
    TC: TendermintClient + Clone + Send + Sync + 'static,
{
    fn get_latest_committed_finality(&self) -> anyhow::Result<Option<IPCParentFinality>> {
        self.with_exec_state(|mut exec_state| {
            self.gateway_caller
                .get_latest_parent_finality(&mut exec_state)
        })
    }

    fn get_power_table(&self) -> anyhow::Result<Option<Vec<Validator<Power>>>> {
        self.with_exec_state(|mut exec_state| {
            self.gateway_caller
                .current_power_table(&mut exec_state)
                .map(|(_, pt)| pt)
        })
    }
}
