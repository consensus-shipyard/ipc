// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use std::future::Future;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use cid::Cid;
use fendermint_abci::Application;
use fendermint_storage::{
    Codec, Encode, KVCollection, KVRead, KVReadable, KVStore, KVWritable, KVWrite,
};
use fendermint_vm_genesis::Genesis;
use fendermint_vm_interpreter::bytes::{
    BytesMessageApplyRet, BytesMessageCheckRet, BytesMessageQuery, BytesMessageQueryRet,
};
use fendermint_vm_interpreter::chain::{ChainMessageApplyRet, IllegalMessage};
use fendermint_vm_interpreter::fvm::{
    FvmApplyRet, FvmCheckState, FvmExecState, FvmGenesisState, FvmQueryState,
};
use fendermint_vm_interpreter::signed::InvalidSignature;
use fendermint_vm_interpreter::{CheckInterpreter, Interpreter, QueryInterpreter};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::econ::TokenAmount;
use fvm_shared::version::NetworkVersion;
use serde::{Deserialize, Serialize};
use tendermint::abci::request::CheckTxKind;
use tendermint::abci::{request, response};
use tendermint::block::Height;

use crate::tmconv::*;
use crate::BlockHeight;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize)]
#[repr(u8)]
pub enum AppStoreKey {
    State,
}

// TODO: What range should we use for our own error codes? Should we shift FVM errors?
#[repr(u32)]
pub enum AppError {
    /// Failed to deserialize the transaction.
    InvalidEncoding = 51,
    /// Failed to validate the user signature.
    InvalidSignature = 52,
    /// User sent a message they should not construct.
    IllegalMessage = 53,
}

#[derive(Serialize, Deserialize)]
pub struct AppState {
    /// Last committed block height.
    block_height: BlockHeight,
    /// Last committed state hash.
    state_root: Cid,
    /// Oldest state hash height.
    oldest_state_height: BlockHeight,
    network_version: NetworkVersion,
    base_fee: TokenAmount,
    circ_supply: TokenAmount,
}

impl AppState {
    pub fn app_hash(&self) -> tendermint::hash::AppHash {
        tendermint::hash::AppHash::try_from(self.state_root.to_bytes())
            .expect("hash can be wrapped")
    }
}

/// Handle ABCI requests.
#[derive(Clone)]
pub struct App<DB, S, I>
where
    DB: Blockstore + 'static,
    S: KVStore,
{
    db: Arc<DB>,
    actor_bundle_path: PathBuf,
    /// Namespace to store app state.
    namespace: S::Namespace,
    /// Collection of past state hashes.
    ///
    /// We store the state hash for the height of the block where it was committed,
    /// which is different from how Tendermint Core will refer to it in queries,
    /// shifte by one, because Tendermint Core will use the height where the hash
    /// *appeared*, which is in the block *after* the one which was committed.
    state_hist: KVCollection<S, BlockHeight, Cid>,
    /// Interpreter for block lifecycle events.
    interpreter: Arc<I>,
    /// State accumulating changes during block execution.
    exec_state: Arc<Mutex<Option<FvmExecState<DB>>>>,
    /// Projected partial state accumulating during transaction checks.
    check_state: Arc<tokio::sync::Mutex<Option<FvmCheckState<DB>>>>,
    /// How much history to keep.
    ///
    /// Zero means unlimited.
    state_hist_size: u64,
}

impl<DB, S, I> App<DB, S, I>
where
    S: KVStore + Codec<AppState> + Encode<AppStoreKey> + Encode<BlockHeight> + Codec<Cid>,
    DB: Blockstore + KVWritable<S> + KVReadable<S> + Clone + 'static,
{
    pub fn new(
        db: DB,
        actor_bundle_path: PathBuf,
        app_namespace: S::Namespace,
        hist_namespace: S::Namespace,
        interpreter: I,
    ) -> Self {
        Self {
            db: Arc::new(db),
            actor_bundle_path,
            namespace: app_namespace,
            state_hist: KVCollection::new(hist_namespace),
            interpreter: Arc::new(interpreter),
            exec_state: Arc::new(Mutex::new(None)),
            check_state: Arc::new(tokio::sync::Mutex::new(None)),
            state_hist_size: 24 * 60 * 60,
        }
    }
}

impl<DB, S, I> App<DB, S, I>
where
    S: KVStore + Codec<AppState> + Encode<AppStoreKey> + Encode<BlockHeight> + Codec<Cid>,
    DB: Blockstore + KVWritable<S> + KVReadable<S> + 'static + Clone,
{
    /// Get an owned clone of the database.
    fn clone_db(&self) -> DB {
        self.db.as_ref().clone()
    }
    /// Get the last committed state.
    fn committed_state(&self) -> AppState {
        let tx = self.db.read();
        tx.get(&self.namespace, &AppStoreKey::State)
            .expect("get failed")
            .expect("app state not found") // TODO: Init during setup.
    }

    /// Set the last committed state.
    fn set_committed_state(&self, mut state: AppState) {
        self.db
            .with_write(|tx| {
                // Insert latest state history point.
                self.state_hist
                    .put(tx, &state.block_height, &state.state_root)?;

                // Prune state history.
                if self.state_hist_size > 0 && state.block_height >= self.state_hist_size {
                    let prune_height = state.block_height.saturating_sub(self.state_hist_size);
                    while state.oldest_state_height <= prune_height {
                        self.state_hist.delete(tx, &state.oldest_state_height)?;
                        state.oldest_state_height += 1;
                    }
                }

                // Update the application state.
                tx.put(&self.namespace, &AppStoreKey::State, &state)?;

                Ok(())
            })
            .expect("commit failed");
    }

    /// Put the execution state during block execution. Has to be empty.
    fn put_exec_state(&self, state: FvmExecState<DB>) {
        let mut guard = self.exec_state.lock().expect("mutex poisoned");
        assert!(guard.is_some(), "exec state not empty");
        *guard = Some(state);
    }

    /// Take the execution state during block execution. Has to be non-empty.
    fn take_exec_state(&self) -> FvmExecState<DB> {
        let mut guard = self.exec_state.lock().expect("mutex poisoned");
        guard.take().expect("exec state empty")
    }

    /// Take the execution state, update it, put it back, return the output.
    async fn modify_exec_state<T, F, R>(&self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(FvmExecState<DB>) -> R,
        R: Future<Output = anyhow::Result<(FvmExecState<DB>, T)>>,
    {
        let state = self.take_exec_state();
        let (state, ret) = f(state).await?;
        self.put_exec_state(state);
        Ok(ret)
    }

    /// Look up a past state hash at a particular height Tendermint Core is looking for,
    /// which will be +1 shifted from what we saved. If the height is zero, it means it
    /// wants the latest height.
    ///
    /// Returns the CID and the height of the block which committed it.
    fn state_root_at_height(&self, height: Height) -> (Cid, BlockHeight) {
        if height.value() > 0 {
            let h = height.value() - 1;
            let tx = self.db.read();
            let sh = self
                .state_hist
                .get(&tx, &h)
                .expect("error looking up history");

            if let Some(cid) = sh {
                return (cid, h);
            }
        }
        let state = self.committed_state();
        (state.state_root, state.block_height)
    }
}

// NOTE: The `Application` interface doesn't allow failures at the moment. The protobuf
// of `Response` actually has an `Exception` type, so in theory we could use that, and
// Tendermint would break up the connection. However, before the response could reach it,
// the `tower-abci` library would throw an exception when it tried to convert a
// `Response::Exception` into a `ConensusResponse` for example.
#[async_trait]
impl<DB, S, I> Application for App<DB, S, I>
where
    S: KVStore + Codec<AppState> + Encode<AppStoreKey> + Encode<BlockHeight> + Codec<Cid>,
    S::Namespace: Sync + Send,
    DB: Blockstore + KVWritable<S> + KVReadable<S> + Clone + Send + Sync + 'static,
    I: Interpreter<
        State = FvmExecState<DB>,
        Message = Vec<u8>,
        BeginOutput = FvmApplyRet,
        DeliverOutput = BytesMessageApplyRet,
        EndOutput = (),
    >,
    I: CheckInterpreter<
        State = FvmCheckState<DB>,
        Message = Vec<u8>,
        Output = BytesMessageCheckRet,
    >,
    I: QueryInterpreter<
        State = FvmQueryState<DB>,
        Query = BytesMessageQuery,
        Output = BytesMessageQueryRet,
    >,
{
    /// Provide information about the ABCI application.
    async fn info(&self, _request: request::Info) -> response::Info {
        let state = self.committed_state();

        let height =
            tendermint::block::Height::try_from(state.block_height).expect("height too big");

        response::Info {
            data: "fendermint".to_string(),
            version: VERSION.to_owned(),
            app_version: 1,
            last_block_height: height,
            last_block_app_hash: state.app_hash(),
        }
    }

    /// Called once upon genesis.
    async fn init_chain(&self, request: request::InitChain) -> response::InitChain {
        // TODO (IPC-44): Use the serialized application state instead of `Genesis`.
        let genesis: Genesis =
            parse_genesis(&request.app_state_bytes).expect("failed to parse genesis");

        let height = request.initial_height.into();

        let mut app_state = AppState {
            block_height: height,
            state_root: Default::default(),
            oldest_state_height: height,
            network_version: genesis.network_version,
            base_fee: genesis.base_fee.clone(),
            circ_supply: genesis.circ_supply(),
        };

        let bundle_path = &self.actor_bundle_path;
        let bundle = std::fs::read(bundle_path)
            .unwrap_or_else(|_| panic!("failed to load bundle CAR from {bundle_path:?}"));

        let mut state = FvmGenesisState::new(self.clone_db(), &bundle)
            .await
            .expect("error creating state");

        state
            .create_genesis_actors(&genesis)
            .expect("error creating genesis actors");

        app_state.state_root = state.commit().expect("error committing state");

        let validators = genesis_validators(&genesis).expect("error projecting validators");

        let response = response::InitChain {
            consensus_params: None,
            validators,
            app_hash: app_state.app_hash(),
        };

        self.set_committed_state(app_state);

        response
    }

    /// Query the application for data at the current or past height.
    async fn query(&self, request: request::Query) -> response::Query {
        let db = self.clone_db();
        let (state_root, block_height) = self.state_root_at_height(request.height);
        let state = FvmQueryState::new(db, state_root).expect("error creating query state");
        let qry = (request.path, request.data.to_vec());

        let (_, result) = self
            .interpreter
            .query(state, qry)
            .await
            .expect("error running query");

        match result {
            Err(e) => invalid_query(AppError::InvalidEncoding, e.description),
            Ok(result) => to_query(result, block_height),
        }
    }

    /// Check the given transaction before putting it into the local mempool.
    async fn check_tx(&self, request: request::CheckTx) -> response::CheckTx {
        // Keep the guard through the check, so there can be only one at a time.
        let mut guard = self.check_state.lock().await;

        let state = guard.take().unwrap_or_else(|| {
            let db = self.clone_db();
            let state = self.committed_state();
            FvmCheckState::new(db, state.state_root).expect("error creating check state")
        });

        let (state, result) = self
            .interpreter
            .check(
                state,
                request.tx.to_vec(),
                request.kind == CheckTxKind::Recheck,
            )
            .await
            .expect("error running check");

        // Update the check state.
        *guard = Some(state);

        match result {
            Err(e) => invalid_check_tx(AppError::InvalidEncoding, e.description),
            Ok(result) => match result {
                Err(IllegalMessage) => invalid_check_tx(AppError::IllegalMessage, "".to_owned()),
                Ok(result) => match result {
                    Err(InvalidSignature(d)) => invalid_check_tx(AppError::InvalidSignature, d),
                    Ok(ret) => to_check_tx(ret),
                },
            },
        }
    }

    /// Signals the beginning of a new block, prior to any `DeliverTx` calls.
    async fn begin_block(&self, request: request::BeginBlock) -> response::BeginBlock {
        let db = self.clone_db();
        let state = self.committed_state();
        let height = request.header.height.into();
        let timestamp = to_timestamp(request.header.time);

        let state = FvmExecState::new(
            db,
            height,
            timestamp,
            state.network_version,
            state.state_root,
            state.base_fee,
            state.circ_supply,
        )
        .expect("error creating new state");

        self.put_exec_state(state);

        let ret = self
            .modify_exec_state(|s| self.interpreter.begin(s))
            .await
            .expect("begin failed");

        to_begin_block(ret)
    }

    /// Apply a transaction to the application's state.
    async fn deliver_tx(&self, request: request::DeliverTx) -> response::DeliverTx {
        let msg = request.tx.to_vec();
        let result = self
            .modify_exec_state(|s| self.interpreter.deliver(s, msg))
            .await
            .expect("deliver failed");

        match result {
            Err(e) => invalid_deliver_tx(AppError::InvalidEncoding, e.description),
            Ok(ret) => match ret {
                ChainMessageApplyRet::Signed(Err(InvalidSignature(d))) => {
                    invalid_deliver_tx(AppError::InvalidSignature, d)
                }
                ChainMessageApplyRet::Signed(Ok(ret)) => to_deliver_tx(ret),
            },
        }
    }

    /// Signals the end of a block.
    async fn end_block(&self, _request: request::EndBlock) -> response::EndBlock {
        // TODO: Return events from epoch transitions.
        let ret = self
            .modify_exec_state(|s| self.interpreter.end(s))
            .await
            .expect("end failed");

        to_end_block(ret)
    }

    /// Commit the current state at the current height.
    async fn commit(&self) -> response::Commit {
        let exec_state = self.take_exec_state();
        let block_height = exec_state.block_height();
        let state_root = exec_state.commit().expect("failed to commit FVM");

        let mut state = self.committed_state();
        state.state_root = state_root;
        state.block_height = block_height.try_into().expect("negative height");
        self.set_committed_state(state);

        // Reset check state.
        let mut guard = self.check_state.lock().await;
        *guard = None;

        response::Commit {
            data: state_root.to_bytes().into(),
            // We have to retain blocks until we can support Snapshots.
            retain_height: Default::default(),
        }
    }
}
