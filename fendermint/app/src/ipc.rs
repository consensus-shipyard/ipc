// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
//! IPC related execution

use crate::{App, AppStore};
use fendermint_storage::{KVReadable, KVWritable};
use fendermint_vm_genesis::{Power, Validator};
use fendermint_vm_interpreter::fvm::state::ipc::GatewayCaller;
use fendermint_vm_interpreter::fvm::state::FvmExecState;
use fendermint_vm_interpreter::fvm::store::ReadOnlyBlockstore;
use fendermint_vm_topdown::sync::ParentFinalityStateQuery;
use fendermint_vm_topdown::IPCParentFinality;
use fvm_ipld_blockstore::Blockstore;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// All the things that can be voted on in a subnet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppVote {
    /// The validator considers a certain block final on the parent chain.
    ParentFinality(IPCParentFinality),
}

/// Queries the LATEST COMMITTED parent finality from the storage
pub struct AppParentFinalityQuery<DB, BS, I>
where
    BS: Blockstore + Clone + 'static,
{
    /// The app to get state
    app: App<DB, BS, I>,
    gateway_caller: GatewayCaller<ReadOnlyBlockstore<Arc<BS>>>,
}

impl<DB, SS, I> AppParentFinalityQuery<DB, SS, I>
where
    DB: KVWritable<AppStore> + KVReadable<AppStore> + 'static + Clone,
    SS: Blockstore + 'static + Clone,
{
    pub fn new(app: App<DB, SS, I>) -> Self {
        Self {
            app,
            gateway_caller: GatewayCaller::default(),
        }
    }

    fn with_exec_state<F, T>(&self, f: F) -> anyhow::Result<Option<T>>
    where
        F: FnOnce(FvmExecState<ReadOnlyBlockstore<Arc<SS>>>) -> anyhow::Result<T>,
    {
        match self.app.new_read_only_exec_state()? {
            Some(s) => f(s).map(Some),
            None => Ok(None),
        }
    }
}

impl<DB, SS, I> ParentFinalityStateQuery for AppParentFinalityQuery<DB, SS, I>
where
    DB: KVWritable<AppStore> + KVReadable<AppStore> + 'static + Clone,
    SS: Blockstore + 'static + Clone,
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
