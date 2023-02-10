// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use std::future::Future;
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use cid::Cid;
use fendermint_abci::Application;
use fendermint_vm_interpreter::bytes::BytesMessageApplyRet;
use fendermint_vm_interpreter::chain::ChainMessageApplyRet;
use fendermint_vm_interpreter::fvm::{FvmApplyRet, FvmState};
use fendermint_vm_interpreter::signed::SignedMesssageApplyRet;
use fendermint_vm_interpreter::{Interpreter, Timestamp};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::econ::TokenAmount;
use fvm_shared::event::StampedEvent;
use fvm_shared::version::NetworkVersion;
use tendermint::abci::{request, response, Code, Event};

const VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO: What range should we use for our own error codes? Should we shift FVM errors?

#[repr(u32)]
enum AppError {
    /// Failed to deserialize the transaction.
    InvalidEncoding = 51,
    /// Failed to validate the user signature.
    InvalidSignature = 52,
}

struct AppState {
    block_height: u64,
    state_root: Cid,
    network_version: NetworkVersion,
    base_fee: TokenAmount,
    circ_supply: TokenAmount,
}

/// Handle ABCI requests.
#[derive(Clone)]
pub struct App<DB, I>
where
    DB: Blockstore + 'static,
{
    db: Arc<DB>,
    interpreter: Arc<I>,
    /// State accumulating changes during block execution.
    exec_state: Arc<Mutex<Option<FvmState<DB>>>>,
}

impl<DB, I> App<DB, I>
where
    DB: Blockstore + 'static,
{
    pub fn new(db: DB, interpreter: I) -> Self {
        Self {
            db: Arc::new(db),
            interpreter: Arc::new(interpreter),
            exec_state: Arc::new(Mutex::new(None)),
        }
    }
}

impl<DB, I> App<DB, I>
where
    DB: Blockstore + 'static,
{
    /// Get the last committed state.
    fn committed_state(&self) -> AppState {
        todo!("retrieve state from the DB")
    }

    /// Set the last committed state.
    fn set_committed_state(&self, _state: AppState) {
        todo!("write state to the DB")
    }

    /// Put the execution state during block execution. Has to be empty.
    fn put_exec_state(&self, state: FvmState<DB>) {
        let mut guard = self.exec_state.lock().expect("mutex poisoned");
        assert!(guard.is_some(), "exec state not empty");
        *guard = Some(state);
    }

    /// Take the execution state during block execution. Has to be non-empty.
    fn take_exec_state(&self) -> FvmState<DB> {
        let mut guard = self.exec_state.lock().expect("mutex poisoned");
        guard.take().expect("exec state empty")
    }

    /// Take the execution state, update it, put it back, return the output.
    async fn modify_exec_state<T, F, R>(&self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(FvmState<DB>) -> R,
        R: Future<Output = anyhow::Result<(FvmState<DB>, T)>>,
    {
        let state = self.take_exec_state();
        let (state, ret) = f(state).await?;
        self.put_exec_state(state);
        Ok(ret)
    }
}

// NOTE: The `Application` interface doesn't allow failures at the moment. The protobuf
// of `Response` actually has an `Exception` type, so in theory we could use that, and
// Tendermint would break up the connection. However, before the response could reach it,
// the `tower-abci` library would throw an exception when it tried to convert a
// `Response::Exception` into a `ConensusResponse` for example.
#[async_trait]
impl<DB, I> Application for App<DB, I>
where
    DB: Blockstore + Clone + Send + Sync + 'static,
    I: Interpreter<
        State = FvmState<DB>,
        Message = Vec<u8>,
        BeginOutput = FvmApplyRet,
        DeliverOutput = BytesMessageApplyRet,
        EndOutput = (),
    >,
{
    /// Provide information about the ABCI application.
    async fn info(&self, _request: request::Info) -> response::Info {
        let state = self.committed_state();
        let height =
            tendermint::block::Height::try_from(state.block_height).expect("height too big");
        let app_hash = tendermint::hash::AppHash::try_from(state.state_root.to_bytes())
            .expect("hash can be wrapped");
        response::Info {
            data: "fendermint".to_string(),
            version: VERSION.to_owned(),
            app_version: 1,
            last_block_height: height,
            last_block_app_hash: app_hash,
        }
    }

    /// Called once upon genesis.
    async fn init_chain(&self, _request: request::InitChain) -> response::InitChain {
        Default::default()
    }

    /// Query the application for data at the current or past height.
    async fn query(&self, _request: request::Query) -> response::Query {
        todo!("make a query interpreter")
    }

    /// Check the given transaction before putting it into the local mempool.
    async fn check_tx(&self, _request: request::CheckTx) -> response::CheckTx {
        todo!("make an interpreter for checks, on a projected state")
    }

    /// Signals the beginning of a new block, prior to any `DeliverTx` calls.
    async fn begin_block(&self, request: request::BeginBlock) -> response::BeginBlock {
        let state = self.committed_state();
        let height = request.header.height.into();
        let timestamp = Timestamp(
            request
                .header
                .time
                .unix_timestamp()
                .try_into()
                .expect("negative timestamp"),
        );
        let db = self.db.as_ref().to_owned();

        let state = FvmState::new(
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
                ChainMessageApplyRet::Signed(SignedMesssageApplyRet::InvalidSignature(d)) => {
                    invalid_deliver_tx(AppError::InvalidSignature, d)
                }
                ChainMessageApplyRet::Signed(SignedMesssageApplyRet::Applied(ret)) => {
                    to_deliver_tx(ret)
                }
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
        let state_root = exec_state.commit().expect("failed to commit FVM");

        let mut state = self.committed_state();
        state.state_root = state_root;
        self.set_committed_state(state);

        response::Commit {
            data: state_root.to_bytes().into(),
            // We have to retain blocks until we can support Snapshots.
            retain_height: Default::default(),
        }
    }
}

/// Response to delivery where the input was blatantly invalid.
/// This indicates that the validator who made the block was Byzantine.
fn invalid_deliver_tx(err: AppError, description: String) -> response::DeliverTx {
    response::DeliverTx {
        code: Code::Err(NonZeroU32::try_from(err as u32).expect("error codes are non-zero")),
        info: description,
        ..Default::default()
    }
}

fn to_deliver_tx(ret: FvmApplyRet) -> response::DeliverTx {
    let receipt = ret.apply_ret.msg_receipt;
    let code = if receipt.exit_code.is_success() {
        Code::Ok
    } else {
        Code::Err(
            NonZeroU32::try_from(receipt.exit_code.value()).expect("error codes are non-zero"),
        )
    };

    // Based on the sanity check in the `DefaultExecutor`.
    // gas_cost = gas_fee_cap * gas_limit; this is how much the account is charged up front.
    // &base_fee_burn + &over_estimation_burn + &refund + &miner_tip == gas_cost
    // But that's in tokens. I guess the closes to what we want is the limit.
    let gas_wanted: i64 = ret.gas_limit.try_into().expect("gas wanted not i64");
    let gas_used: i64 = receipt.gas_used.try_into().expect("gas used not i64");

    let data = receipt.return_data.to_vec().into();
    let events = to_events(ret.apply_ret.events);

    response::DeliverTx {
        code,
        data,
        log: Default::default(),
        info: Default::default(),
        gas_wanted,
        gas_used,
        events,
        codespace: Default::default(),
    }
}

/// Map the return values from epoch boundary operations to validator updates.
///
/// (Currently just a placeholder).
fn to_end_block(_ret: ()) -> response::EndBlock {
    response::EndBlock {
        validator_updates: Vec::new(),
        consensus_param_updates: None,
        events: Vec::new(),
    }
}

/// Map the return values from cron operations.
///
/// (Currently just a placeholder).
fn to_begin_block(ret: FvmApplyRet) -> response::BeginBlock {
    let events = to_events(ret.apply_ret.events);

    response::BeginBlock { events }
}

fn to_events(_stamped_events: Vec<StampedEvent>) -> Vec<Event> {
    // TODO: Convert events. This is currently not possible because the event fields are private.
    // I changed that in https://github.com/filecoin-project/ref-fvm/pull/1507 but it's still in review.
    // A possible workaround would be to retrieve the events by their CID, and use a custom type to parse.
    // It will be part of https://github.com/filecoin-project/ref-fvm/pull/1635 :)
    Vec::new()
}
