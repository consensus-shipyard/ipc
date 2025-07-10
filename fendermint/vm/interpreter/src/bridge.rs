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

mod eth_light_client {
    use std::collections::HashSet;
    use anyhow::{Result, bail};
    use tendermint::block::Height;
    use ipc_actors_abis::lib_gateway::IpcEnvelope;
    use ipc_api::staking::PowerChangeRequest;
    use crate::fvm::state::FvmExecState;
    use super::{CrossChainBridge};

    const PREFIX_DELIVERED: &[u8] = b"delivered_";
    const PREFIX_HEADER_ROOT: &[u8] = b"trusted_header_";

    pub struct SignedHeader {
        pub height: u64,
        pub header: Header,                 // Includes app_hash, validators_hash, next_validators_hash
        pub commit: Commit,                 // Contains signatures
    }

    pub enum LightClientBridgeUpdate {
        StateSync {
            signed_header: SignedHeader,
            validator_set: EthValidatorChange,    //
        },
        ValidatorChange {
            receipt_root: Vec<u8>,
            height: Height,
            validator_change: PowerChangeRequest,
            event_inclusion_proof: Vec<u8>,
        },
    }

    pub struct LightClientBridge;

    impl CrossChainBridge for LightClientBridge {
        type BridgeStateUpdate = LightClientBridgeUpdate;

        fn update_bridge_state<DB: Blockstore + Clone + 'static>(
            &mut self,
            _: &mut FvmExecState<DB>,
            update: Self::BridgeStateUpdate,
        ) -> Result<()> {
            match update {
                LightClientBridgeUpdate::StateSync { .. } => self.update_parent_blocks(),
                LightClientBridgeUpdate::ValidatorChange { .. } => self.append_validator_inclusion(),
            }
        }

        fn verify_message<DB: Blockstore + Clone + 'static>(
            &self,
            vm_state: &FvmExecState<DB>,
            message: &IpcEnvelope,
            proof: &[u8],
        ) -> Result<bool> {
            let (height, merkle_proof) = parse_proof(proof)?;
            let header = self.get_block_header.get(height)?;
            let message_hash = self.hash_message(message);
            verify_merkle_inclusion(&message_hash, &merkle_proof, &header)?;
        }

        fn execute_message<DB: Blockstore + Clone + 'static>(
            &mut self,
            vm_state: &mut FvmExecState<DB>,
            message: IpcEnvelope,
        ) -> Result<()> {
            self.verify_message(vm_state, message)?;
            self.execute(vm_state, message)
        }

        fn is_delivered<DB: Blockstore + Clone + 'static>(
            &self,
            vm_state: &FvmExecState<DB>,
            message_hash: [u8; 32],
        ) -> Result<bool> {
            vm_state.get_bridge_contract().is_delivered(message_hash)
        }
    }

}
mod validator_set {
    use anyhow::bail;
    use fvm_ipld_blockstore::Blockstore;
    use fendermint_crypto::PublicKey;
    use ipc_api::cross::IpcEnvelope;
    use ipc_api::staking::PowerChangeRequest;
    use crate::bridge::CrossChainBridge;
    use crate::fvm::state::FvmExecState;

    #[derive(Debug, Clone)]
    pub struct ValidatorBridgeUpdate {
        validator_changes: Vec<PowerChangeRequest>,
        xnet_message: Vec<IpcEnvelope>,
        parent_hash: Vec<u8>,
        parent_height: u64,
        signature: Vec<u8>,
        validator: PublicKey,
    }

    pub struct TopdownCrossChainBridge;

    impl CrossChainBridge for TopdownCrossChainBridge
    {
        type BridgeStateUpdate = ValidatorBridgeUpdate;

        fn update_bridge_state<DB: Blockstore + Clone + 'static>(
            &mut self,
            _: &mut FvmExecState<DB>,
            update: Self::BridgeStateUpdate,
        ) -> anyhow::Result<()> {

            let Some(quorum) = self.record_vote(update) else { return Ok(()); } ;
            self.record_quorum(quorum);

            Ok(())
        }

        fn verify_message<DB: Blockstore + Clone + 'static>(&self, _: &mut FvmExecState<DB>, message: IpcEnvelope, _: &[u8]) -> anyhow::Result<()> {
            self.quorums_contain(message)
        }

        fn execute_message<DB: Blockstore + Clone + 'static>(
            &mut self,
            vm_state: &mut FvmExecState<DB>,
            message: IpcEnvelope,
            _: &[u8],
        ) -> anyhow::Result<()>{
            if self.quorums_contain(&message) {
                self.execute(vm_state, message)?;
            } else {
                bail!("not proper message")
            }
        }

        fn is_delivered<DB: Blockstore + Clone + 'static>(&self, vm_state: &mut FvmExecState<DB>, message: &IpcEnvelope) -> anyhow::Result<bool> {
            todo!()
        }
    }
}