// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use async_trait::async_trait;

pub mod bytes;
pub mod chain;
pub mod fvm;
pub mod signed;

/// Unix timestamp (in seconds) of the current block.
pub struct Timestamp(pub u64);

/// The `Interpreter`applies messages on some state, which is
/// tied to the lifecycle of a block in the ABCI.
///
/// By making it generic, the intention is that interpreters can
/// be stacked, changing the type of message along the way. For
/// example on the outermost layer the input message can be a mix
/// of self-contained messages and CIDs proposed for resolution
/// or execution, while in the innermost layer it's all self-contained.
/// Some interpreters would act like middlewares to resolve CIDs into
/// a concrete message.
///
/// The execution is asynchronous, so that the middleware is allowed
/// to potentially interact with the outside world. If this was restricted
/// to things like scheduling a CID resolution, we could use effects
/// returned from message processing. However, when a node is catching
/// up with the chain others have already committed, they have to do the
/// message resolution synchronously, so it has to be done during
/// message processing. Alternatively we'd have to split the processing
/// into async steps to pre-process the message, then synchronous steps
/// to update the state. But this approach is more flexible, because
/// the middlewares can decide on a message-by-message basis whether
/// to forward the message to the inner layer. Unfortunately block-level
/// pre-processing is not possible, because we are fed the messages
/// one by one through the ABCI.
///
/// There is no separate type for `Error`, only `Output`. The reason
/// is that we'll be calling high level executors internally that
/// already have their internal error handling, returning all domain
/// errors such as `OutOfGas` in their output, and only using the
/// error case for things that are independent of the message itself,
/// signalling unexpected problems there's no recovering from and
/// that should stop the block processing altogether.
#[async_trait]
pub trait Interpreter: Sync + Send {
    type State: Send;
    type Message: Send;
    type BeginOutput;
    type DeliverOutput;
    type EndOutput;

    /// Called once at the beginning of a block.
    ///
    /// This is our chance to to run `cron` jobs for example.
    async fn begin(&self, state: Self::State) -> anyhow::Result<(Self::State, Self::BeginOutput)>;

    /// Apply a message onto the state.
    ///
    /// The state is taken by value, so there's no issue with sharing
    /// mutable references in futures. The modified value should be
    /// returned along with the return value.
    ///
    /// Only return an error case if something truly unexpected happens
    /// that should stop message processing altogether; otherwise use
    /// the output for signalling all execution results.
    async fn deliver(
        &self,
        state: Self::State,
        msg: Self::Message,
    ) -> anyhow::Result<(Self::State, Self::DeliverOutput)>;

    /// Called once at the end of a block.
    ///
    /// This is where we can apply end-of-epoch processing, for example to process staking
    /// requests once every 1000 blocks.
    async fn end(&self, state: Self::State) -> anyhow::Result<(Self::State, Self::EndOutput)>;
}

/// Check if messages can be added to the mempool by performing certain validation
/// over a projected version of the state. Does not execute transactions fully,
/// just does basic validation. The state is updated so that things like nonces
/// and balances are adjusted as if the transaction was executed. This way an
/// account can send multiple messages in a row, not just the next that follows
/// its current nonce.
#[async_trait]
pub trait CheckInterpreter: Sync + Send {
    type State: Send;
    type Message: Send;
    type Output;

    /// Called when a new user transaction is being added to the mempool.
    ///
    /// Returns the updated state, and the check output, which should be
    /// able to describe both the success and failure cases.
    ///
    /// The recheck flags indicates that we are checking the transaction
    /// again because we have seen a new block and the state changed.
    /// As an optimisation, checks that do not depend on state can be skipped.
    async fn check(
        &self,
        state: Self::State,
        msg: Self::Message,
        is_recheck: bool,
    ) -> anyhow::Result<(Self::State, Self::Output)>;
}
