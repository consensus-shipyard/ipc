// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::fvm::state::FvmExecState;
use fvm_ipld_blockstore::Blockstore;
use ipc_api::cross::IpcEnvelope;
use std::fmt::Debug;

/// The `CrossChainBridge` trait defines the core interface for a generic, verifiable, and stateful
/// cross-chain bridge system.
///
/// This trait is designed to support different types of bridging backends—such as validator-based voting,
/// light clients, or potentially ZK-proof bridges—by abstracting over the **state update**, **message verification**,
/// **execution**, and **delivery tracking** processes.
///
/// ## Key Concepts
/// - **Stateful**: Implementations should persist internal trust state (e.g., validator sets or synced headers)
///   and track delivered messages to prevent replay attacks.
/// - **Modular**: `BridgeStateUpdate` allows you to define custom update formats for different proof models.
///
/// ## Usage Flow
/// A typical relayer or node should:
/// 1. Update the bridge's state with `update_bridge_state`
/// 2. Users can call `verify_message()` with a `IpcEnvelope` and its proof to check if the message is valid.
/// 3. If valid and not yet delivered, call `execute_message()` to apply it.
/// 4. Optionally to track if message has been executed
pub trait CrossChainBridge {
    /// An implementation-specific update payload for modifying the bridge's internal trusted state.
    type BridgeStateUpdate: Debug;

    /// Applies a bridge state update such as:
    /// - Changing validator sets (for quorum-based bridges)
    /// - Syncing a new block header + validator changes (for light client bridges)
    ///
    /// This method should validate the update (if necessary) and modify internal state accordingly.
    fn update_bridge_state<DB: Blockstore + Clone + 'static>(
        &mut self,
        vm_state: &mut FvmExecState<DB>,
        update: Self::BridgeStateUpdate,
    ) -> anyhow::Result<()>;

    /// Verifies the validity of an incoming cross-chain message and its accompanying proof.
    ///
    /// This method **must not modify bridge state**. It is purely for checking the correctness of the proof
    /// under the current bridge state.
    ///
    /// Note that `vm_state` has to be &mut due to FvmExecState does not expose `&` for read only execution
    fn verify_message<DB: Blockstore + Clone + 'static>(
        &self,
        vm_state: &mut FvmExecState<DB>,
        message: IpcEnvelope,
        proof: &[u8],
    ) -> anyhow::Result<()>;

    /// Executes a validated cross-chain message.
    ///
    /// This method should:
    /// - Apply the message payload (e.g., route it to the recipient contract or module)
    /// - Record the message hash as delivered to prevent future replays
    fn execute_message<DB: Blockstore + Clone + 'static>(
        &mut self,
        vm_state: &mut FvmExecState<DB>,
        message: IpcEnvelope,
        proof: &[u8],
    ) -> anyhow::Result<()>;

    /// Checks whether a given message has already been executed on this chain.
    ///
    /// Prevents message replay by tracking processed message hashes.
    fn is_delivered<DB: Blockstore + Clone + 'static>(
        &self,
        vm_state: &mut FvmExecState<DB>,
        message: &IpcEnvelope,
    ) -> anyhow::Result<bool>;
}
