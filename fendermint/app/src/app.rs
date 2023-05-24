// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use std::future::Future;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use cid::Cid;
use fendermint_abci::{AbciResult, Application};
use fendermint_storage::{
    Codec, Encode, KVCollection, KVRead, KVReadable, KVStore, KVWritable, KVWrite,
};
use fendermint_vm_core::Timestamp;
use fendermint_vm_interpreter::bytes::{
    BytesMessageApplyRet, BytesMessageCheckRet, BytesMessageQuery, BytesMessageQueryRet,
};
use fendermint_vm_interpreter::chain::{ChainMessageApplyRet, IllegalMessage};
use fendermint_vm_interpreter::fvm::state::{
    empty_state_tree, FvmCheckState, FvmExecState, FvmGenesisState, FvmQueryState, FvmStateParams,
};
use fendermint_vm_interpreter::fvm::{FvmApplyRet, FvmGenesisOutput};
use fendermint_vm_interpreter::signed::InvalidSignature;
use fendermint_vm_interpreter::{
    CheckInterpreter, ExecInterpreter, GenesisInterpreter, QueryInterpreter,
};
use fvm::engine::MultiEngine;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::chainid::ChainID;
use fvm_shared::econ::TokenAmount;
use fvm_shared::version::NetworkVersion;
use num_traits::Zero;
use serde::{Deserialize, Serialize};
use tendermint::abci::request::CheckTxKind;
use tendermint::abci::{request, response};
use tendermint::block::Height;

use crate::{tmconv::*, VERSION};
use crate::{BlockHeight, APP_VERSION};

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
    /// The genesis block hasn't been initialized yet.
    NotInitialized = 54,
}

#[derive(Serialize, Deserialize)]
pub struct AppState {
    /// Last committed block height.
    block_height: BlockHeight,
    /// Oldest state hash height.
    oldest_state_height: BlockHeight,
    /// Last committed version of the evolving state of the FVM.
    state_params: FvmStateParams,
}

impl AppState {
    pub fn state_root(&self) -> Cid {
        self.state_params.state_root
    }
    pub fn chain_id(&self) -> ChainID {
        ChainID::from(self.state_params.chain_id)
    }
    pub fn app_hash(&self) -> tendermint::hash::AppHash {
        tendermint::hash::AppHash::try_from(self.state_root().to_bytes())
            .expect("hash can be wrapped")
    }
}

/// Handle ABCI requests.
#[derive(Clone)]
pub struct App<DB, SS, S, I>
where
    SS: Blockstore + 'static,
    S: KVStore,
{
    /// Database backing all key-value operations.
    db: Arc<DB>,
    /// State store, backing all the smart contracts.
    ///
    /// Must be kept separate from storage that can be influenced by network operations such as Bitswap;
    /// nodes must be able to run transactions deterministically. By contrast the Bitswap store should
    /// be able to read its own storage area as well as state storage, to serve content from both.
    state_store: Arc<SS>,
    /// Wasm engine cache.
    multi_engine: Arc<MultiEngine>,
    /// Path to the Wasm bundle.
    ///
    /// Only loaded once during genesis; later comes from the [`StateTree`].
    actor_bundle_path: PathBuf,
    /// Namespace to store app state.
    namespace: S::Namespace,
    /// Collection of past state parameters.
    ///
    /// We store the state hash for the height of the block where it was committed,
    /// which is different from how Tendermint Core will refer to it in queries,
    /// shifted by one, because Tendermint Core will use the height where the hash
    /// *appeared*, which is in the block *after* the one which was committed.
    ///
    /// The state also contains things like timestamp and the network version,
    /// so that we can retrospectively execute FVM messages at past block heights
    /// in read-only mode.
    state_hist: KVCollection<S, BlockHeight, FvmStateParams>,
    /// Interpreter for block lifecycle events.
    interpreter: Arc<I>,
    /// State accumulating changes during block execution.
    exec_state: Arc<Mutex<Option<FvmExecState<SS>>>>,
    /// Projected partial state accumulating during transaction checks.
    check_state: Arc<tokio::sync::Mutex<Option<FvmCheckState<SS>>>>,
    /// How much history to keep.
    ///
    /// Zero means unlimited.
    state_hist_size: u64,
}

impl<DB, SS, S, I> App<DB, SS, S, I>
where
    S: KVStore
        + Codec<AppState>
        + Encode<AppStoreKey>
        + Encode<BlockHeight>
        + Codec<FvmStateParams>,
    DB: KVWritable<S> + KVReadable<S> + Clone + 'static,
    SS: Blockstore + Clone + 'static,
{
    pub fn new(
        db: DB,
        state_store: SS,
        actor_bundle_path: PathBuf,
        app_namespace: S::Namespace,
        state_hist_namespace: S::Namespace,
        state_hist_size: u64,
        interpreter: I,
    ) -> Result<Self> {
        let app = Self {
            db: Arc::new(db),
            state_store: Arc::new(state_store),
            multi_engine: Arc::new(MultiEngine::new(1)),
            actor_bundle_path,
            namespace: app_namespace,
            state_hist: KVCollection::new(state_hist_namespace),
            state_hist_size,
            interpreter: Arc::new(interpreter),
            exec_state: Arc::new(Mutex::new(None)),
            check_state: Arc::new(tokio::sync::Mutex::new(None)),
        };
        app.init_committed_state()?;
        Ok(app)
    }
}

impl<DB, SS, S, I> App<DB, SS, S, I>
where
    S: KVStore
        + Codec<AppState>
        + Encode<AppStoreKey>
        + Encode<BlockHeight>
        + Codec<FvmStateParams>,
    DB: KVWritable<S> + KVReadable<S> + 'static + Clone,
    SS: Blockstore + 'static + Clone,
{
    /// Get an owned clone of the state store.
    fn state_store_clone(&self) -> SS {
        self.state_store.as_ref().clone()
    }

    /// Ensure the store has some initial state.
    fn init_committed_state(&self) -> Result<()> {
        if self.get_committed_state()?.is_none() {
            // We need to be careful never to run a query on this.
            let mut state_tree = empty_state_tree(self.state_store_clone())
                .context("failed to create empty state tree")?;
            let state_root = state_tree.flush()?;
            let state = AppState {
                block_height: 0,
                oldest_state_height: 0,
                state_params: FvmStateParams {
                    timestamp: Timestamp(0),
                    state_root,
                    network_version: NetworkVersion::MAX,
                    base_fee: TokenAmount::zero(),
                    circ_supply: TokenAmount::zero(),
                    chain_id: 0,
                },
            };
            self.set_committed_state(state)?;
        }
        Ok(())
    }

    /// Get the last committed state, if exists.
    fn get_committed_state(&self) -> Result<Option<AppState>> {
        let tx = self.db.read();
        tx.get(&self.namespace, &AppStoreKey::State)
            .context("get failed")
    }

    /// Get the last committed state; return error if it doesn't exist.
    fn committed_state(&self) -> Result<AppState> {
        match self.get_committed_state()? {
            Some(state) => Ok(state),
            None => Err(anyhow!("app state not found")),
        }
    }

    /// Set the last committed state.
    fn set_committed_state(&self, mut state: AppState) -> Result<()> {
        self.db
            .with_write(|tx| {
                // Insert latest state history point.
                self.state_hist
                    .put(tx, &state.block_height, &state.state_params)?;

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
            .context("commit failed")
    }

    /// Put the execution state during block execution. Has to be empty.
    fn put_exec_state(&self, state: FvmExecState<SS>) {
        let mut guard = self.exec_state.lock().expect("mutex poisoned");
        assert!(guard.is_none(), "exec state not empty");
        *guard = Some(state);
    }

    /// Take the execution state during block execution. Has to be non-empty.
    fn take_exec_state(&self) -> FvmExecState<SS> {
        let mut guard = self.exec_state.lock().expect("mutex poisoned");
        guard.take().expect("exec state empty")
    }

    /// Take the execution state, update it, put it back, return the output.
    async fn modify_exec_state<T, F, R>(&self, f: F) -> Result<T>
    where
        F: FnOnce(FvmExecState<SS>) -> R,
        R: Future<Output = Result<(FvmExecState<SS>, T)>>,
    {
        let state = self.take_exec_state();
        let (state, ret) = f(state).await?;
        self.put_exec_state(state);
        Ok(ret)
    }

    /// Look up a past state at a particular height Tendermint Core is looking for.
    ///
    /// A height of zero means we are looking for the latest state.
    /// The genesis block state is saved under height 1.
    /// Under height 0 we saved the empty state, which we must not query,
    /// because it doesn't contain any initialized state for the actors.
    ///
    /// Returns the state params and the height of the block which committed it.
    fn state_params_at_height(&self, height: Height) -> Result<(FvmStateParams, BlockHeight)> {
        let h = height.value();
        if h > 0 {
            let tx = self.db.read();
            let sh = self
                .state_hist
                .get(&tx, &h)
                .context("error looking up history")?;

            if let Some(p) = sh {
                return Ok((p, h));
            }
        }
        let state = self.committed_state()?;
        Ok((state.state_params, state.block_height))
    }
}

// NOTE: The `Application` interface doesn't allow failures at the moment. The protobuf
// of `Response` actually has an `Exception` type, so in theory we could use that, and
// Tendermint would break up the connection. However, before the response could reach it,
// the `tower-abci` library would throw an exception when it tried to convert a
// `Response::Exception` into a `ConsensusResponse` for example.
#[async_trait]
impl<DB, SS, S, I> Application for App<DB, SS, S, I>
where
    S: KVStore
        + Codec<AppState>
        + Encode<AppStoreKey>
        + Encode<BlockHeight>
        + Codec<FvmStateParams>,
    S::Namespace: Sync + Send,
    DB: KVWritable<S> + KVReadable<S> + Clone + Send + Sync + 'static,
    SS: Blockstore + Clone + Send + Sync + 'static,
    I: ExecInterpreter<
        State = FvmExecState<SS>,
        Message = Vec<u8>,
        BeginOutput = FvmApplyRet,
        DeliverOutput = BytesMessageApplyRet,
        EndOutput = (),
    >,
    I: CheckInterpreter<
        State = FvmCheckState<SS>,
        Message = Vec<u8>,
        Output = BytesMessageCheckRet,
    >,
    I: QueryInterpreter<
        State = FvmQueryState<SS>,
        Query = BytesMessageQuery,
        Output = BytesMessageQueryRet,
    >,
    I: GenesisInterpreter<
        State = FvmGenesisState<SS>,
        Genesis = Vec<u8>,
        Output = FvmGenesisOutput,
    >,
{
    /// Provide information about the ABCI application.
    async fn info(&self, _request: request::Info) -> AbciResult<response::Info> {
        let state = self.committed_state()?;

        let height = tendermint::block::Height::try_from(state.block_height)?;

        let info = response::Info {
            data: "fendermint".to_string(),
            version: VERSION.to_owned(),
            app_version: APP_VERSION,
            last_block_height: height,
            last_block_app_hash: state.app_hash(),
        };
        Ok(info)
    }

    /// Called once upon genesis.
    async fn init_chain(&self, request: request::InitChain) -> AbciResult<response::InitChain> {
        let bundle_path = &self.actor_bundle_path;
        let bundle = std::fs::read(bundle_path)
            .map_err(|e| anyhow!("failed to load bundle CAR from {bundle_path:?}: {e}"))?;

        let state = FvmGenesisState::new(self.state_store_clone(), &bundle)
            .await
            .context("failed to create genesis state")?;

        tracing::info!(
            manifest_root = format!("{}", state.manifest_data_cid),
            "pre-genesis state created"
        );

        let (state, out) = self
            .interpreter
            .init(state, request.app_state_bytes.to_vec())
            .await
            .context("failed to init from genesis")?;

        let state_root = state.commit().context("failed to commit genesis state")?;
        let height = request.initial_height.into();
        let validators =
            to_validator_updates(out.validators).context("failed to convert validators")?;

        let app_state = AppState {
            block_height: height,
            oldest_state_height: height,
            state_params: FvmStateParams {
                state_root,
                timestamp: out.timestamp,
                network_version: out.network_version,
                base_fee: out.base_fee,
                circ_supply: out.circ_supply,
                chain_id: out.chain_id.into(),
            },
        };

        let response = response::InitChain {
            consensus_params: None,
            validators,
            app_hash: app_state.app_hash(),
        };

        tracing::info!(
            state_root = format!("{}", app_state.state_root()),
            app_hash = format!("{}", app_state.app_hash()),
            "init chain"
        );

        self.set_committed_state(app_state)?;

        Ok(response)
    }

    /// Query the application for data at the current or past height.
    async fn query(&self, request: request::Query) -> AbciResult<response::Query> {
        let db = self.state_store_clone();
        let (state_params, block_height) = self.state_params_at_height(request.height)?;

        tracing::info!(
            query_height = request.height.value(),
            block_height,
            state_root = state_params.state_root.to_string(),
            "running query"
        );

        // Don't run queries on the empty state, they won't work.
        if block_height == 0 {
            return Ok(invalid_query(
                AppError::NotInitialized,
                "The app hasn't been initialized yet.".to_owned(),
            ));
        }

        let state = FvmQueryState::new(
            db,
            self.multi_engine.clone(),
            block_height.try_into()?,
            state_params,
        )
        .context("error creating query state")?;

        let qry = (request.path, request.data.to_vec());

        let (_, result) = self
            .interpreter
            .query(state, qry)
            .await
            .context("error running query")?;

        let response = match result {
            Err(e) => invalid_query(AppError::InvalidEncoding, e.description),
            Ok(result) => to_query(result, block_height)?,
        };
        Ok(response)
    }

    /// Check the given transaction before putting it into the local mempool.
    async fn check_tx(&self, request: request::CheckTx) -> AbciResult<response::CheckTx> {
        // Keep the guard through the check, so there can be only one at a time.
        let mut guard = self.check_state.lock().await;

        let state = match guard.take() {
            Some(state) => state,
            None => {
                let db = self.state_store_clone();
                let state = self.committed_state()?;
                FvmCheckState::new(db, state.state_root(), state.chain_id())
                    .context("error creating check state")?
            }
        };

        let (state, result) = self
            .interpreter
            .check(
                state,
                request.tx.to_vec(),
                request.kind == CheckTxKind::Recheck,
            )
            .await
            .context("error running check")?;

        // Update the check state.
        *guard = Some(state);

        let response = match result {
            Err(e) => invalid_check_tx(AppError::InvalidEncoding, e.description),
            Ok(result) => match result {
                Err(IllegalMessage) => invalid_check_tx(AppError::IllegalMessage, "".to_owned()),
                Ok(result) => match result {
                    Err(InvalidSignature(d)) => invalid_check_tx(AppError::InvalidSignature, d),
                    Ok(ret) => to_check_tx(ret),
                },
            },
        };

        Ok(response)
    }

    /// Signals the beginning of a new block, prior to any `DeliverTx` calls.
    async fn begin_block(&self, request: request::BeginBlock) -> AbciResult<response::BeginBlock> {
        let db = self.state_store_clone();
        let height = request.header.height.into();
        let state = self.committed_state()?;
        let mut state_params = state.state_params.clone();
        state_params.timestamp = to_timestamp(request.header.time);

        tracing::debug!(height, "begin block");

        let state = FvmExecState::new(db, self.multi_engine.as_ref(), height, state_params)
            .context("error creating new state")?;

        tracing::debug!("initialized exec state");

        self.put_exec_state(state);

        let ret = self
            .modify_exec_state(|s| self.interpreter.begin(s))
            .await
            .context("begin failed")?;

        Ok(to_begin_block(ret))
    }

    /// Apply a transaction to the application's state.
    async fn deliver_tx(&self, request: request::DeliverTx) -> AbciResult<response::DeliverTx> {
        let msg = request.tx.to_vec();
        let result = self
            .modify_exec_state(|s| self.interpreter.deliver(s, msg))
            .await
            .context("deliver failed")?;

        let response = match result {
            Err(e) => invalid_deliver_tx(AppError::InvalidEncoding, e.description),
            Ok(ret) => match ret {
                ChainMessageApplyRet::Signed(Err(InvalidSignature(d))) => {
                    invalid_deliver_tx(AppError::InvalidSignature, d)
                }
                ChainMessageApplyRet::Signed(Ok(ret)) => {
                    tracing::info!(
                        from = ret.from.to_string(),
                        to = ret.to.to_string(),
                        method_num = ret.method_num,
                        "tx delivered"
                    );
                    to_deliver_tx(ret)
                }
            },
        };

        Ok(response)
    }

    /// Signals the end of a block.
    async fn end_block(&self, request: request::EndBlock) -> AbciResult<response::EndBlock> {
        tracing::debug!(height = request.height, "end block");

        // TODO: Return events from epoch transitions.
        let ret = self
            .modify_exec_state(|s| self.interpreter.end(s))
            .await
            .context("end failed")?;

        Ok(to_end_block(ret))
    }

    /// Commit the current state at the current height.
    async fn commit(&self) -> AbciResult<response::Commit> {
        let exec_state = self.take_exec_state();

        let mut state = self.committed_state()?;
        state.block_height = exec_state.block_height().try_into()?;
        state.state_params.timestamp = exec_state.timestamp();
        state.state_params.state_root = exec_state.commit().context("failed to commit FVM")?;

        let state_root = state.state_root();

        tracing::debug!(
            state_root = state_root.to_string(),
            timestamp = state.state_params.timestamp.0,
            "commit state"
        );

        self.set_committed_state(state)?;

        // Reset check state.
        let mut guard = self.check_state.lock().await;
        *guard = None;

        tracing::debug!("committed state");

        let response = response::Commit {
            data: state_root.to_bytes().into(),
            // We have to retain blocks until we can support Snapshots.
            retain_height: Default::default(),
        };
        Ok(response)
    }
}
