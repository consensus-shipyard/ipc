// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::blobs::BlobRequest;
use fendermint_actor_blobs_shared::{
    blobs::{
        BlobStatus, FinalizeBlobParams, GetAddedBlobsParams, GetBlobStatusParams,
        GetPendingBlobsParams, SetBlobPendingParams,
    },
    credit::{Credit, GasAllowance, GetGasAllowanceParams, UpdateGasAllowanceParams},
    operators::{
        GetActiveOperatorsReturn, GetOperatorInfoParams, OperatorInfo, RegisterNodeOperatorParams,
    },
};
use fendermint_actor_ipc_storage_config_shared::get_config;
use fil_actors_runtime::{runtime::Runtime, ActorError, SYSTEM_ACTOR_ADDR};
use fvm_shared::error::ExitCode;
use ipc_storage_actor_sdk::{
    caller::{Caller, CallerOption},
    evm::emit_evm_event,
};
use num_traits::Zero;

use crate::{
    actor::{delete_from_disc, BlobsActor},
    sol_facade::{blobs as sol_blobs, credit::CreditDebited},
    state::blobs::{FinalizeBlobStateParams, SetPendingBlobStateParams},
    State,
};

impl BlobsActor {
    /// Returns the gas allowance from a credit purchase for an address.
    ///
    /// This method is called by the ipc_storage executor, and as such, cannot fail.
    pub fn get_gas_allowance(
        rt: &impl Runtime,
        params: GetGasAllowanceParams,
    ) -> Result<GasAllowance, ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let from_caller = match Caller::new(rt, params.0, None, CallerOption::None) {
            Ok(caller) => caller,
            Err(e) => {
                return if e.exit_code() == ExitCode::USR_FORBIDDEN {
                    // Disallowed actor type (this is called by all txns so we can't error)
                    Ok(GasAllowance::default())
                } else {
                    Err(e)
                };
            }
        };

        let allowance = rt.state::<State>()?.get_gas_allowance(
            rt.store(),
            from_caller.state_address(),
            rt.curr_epoch(),
        )?;

        Ok(allowance)
    }

    /// Updates gas allowance for the `from` address.
    ///
    /// The allowance update is applied to `sponsor` if it exists.
    /// The `from` address must have an approval from `sponsor`.
    /// The `from` address can be any actor, including those without delegated addresses.
    /// This method is called by the ipc_storage executor, and as such, cannot fail.
    pub fn update_gas_allowance(
        rt: &impl Runtime,
        params: UpdateGasAllowanceParams,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let caller = Caller::new(rt, params.from, params.sponsor, CallerOption::None)?;

        rt.transaction(|st: &mut State, rt| {
            st.update_gas_allowance(
                rt.store(),
                caller.state_address(),
                caller.sponsor_state_address(),
                params.add_amount,
                rt.curr_epoch(),
            )
        })
    }

    /// Returns the current [`BlobStatus`] for a blob by hash.
    pub fn get_blob_status(
        rt: &impl Runtime,
        params: GetBlobStatusParams,
    ) -> Result<Option<BlobStatus>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let caller = Caller::new(rt, params.subscriber, None, CallerOption::None)?;

        rt.state::<State>()?.get_blob_status(
            rt.store(),
            caller.state_address(),
            params.hash,
            params.id,
        )
    }

    /// Returns a list of [`BlobRequest`]s that are currently in the [`BlobStatus::Added`] state.
    ///
    /// All blobs that have been added but have not yet been picked up by validators for download
    /// are in the [`BlobStatus::Added`] state.
    pub fn get_added_blobs(
        rt: &impl Runtime,
        params: GetAddedBlobsParams,
    ) -> Result<Vec<BlobRequest>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state::<State>()?.get_added_blobs(rt.store(), params.0)
    }

    /// Returns a list of [`BlobRequest`]s that are currently in the [`BlobStatus::Pending`] state.
    ///
    /// All blobs that have been added and picked up by validators for download are in the
    /// [`BlobStatus::Pending`] state.
    /// These are the blobs that validators are currently coordinating to download. They will
    /// vote on the final status ([`BlobStatus::Resolved`] or [`BlobStatus::Failed`]), which is
    /// recorded on-chain with the `finalize_blob` method.
    pub fn get_pending_blobs(
        rt: &impl Runtime,
        params: GetPendingBlobsParams,
    ) -> Result<Vec<BlobRequest>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;
        rt.state::<State>()?.get_pending_blobs(rt.store(), params.0)
    }

    /// Sets a blob to the [`BlobStatus::Pending`] state.
    ///
    /// The `subscriber` address must be delegated (only delegated addresses can use credit).
    pub fn set_blob_pending(
        rt: &impl Runtime,
        params: SetBlobPendingParams,
    ) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let caller = Caller::new_delegated(rt, params.subscriber, None, CallerOption::None)?;

        rt.transaction(|st: &mut State, rt| {
            st.set_blob_pending(
                rt.store(),
                caller.state_address(),
                SetPendingBlobStateParams::from_actor_params(params.clone()),
            )
        })?;

        emit_evm_event(
            rt,
            sol_blobs::BlobPending {
                subscriber: caller.event_address(),
                hash: &params.hash,
                source: &params.source,
            },
        )
    }

    /// Finalizes a blob to the [`BlobStatus::Resolved`] or [`BlobStatus::Failed`] state.
    ///
    /// This is the final protocol step to add a blob, which is controlled by node operator consensus.
    /// The [`BlobStatus::Resolved`] state means that a quorum of operators was able to download the blob.
    /// The [`BlobStatus::Failed`] state means that a quorum of operators was not able to download the blob.
    ///
    /// # BLS Signature Verification
    /// This method verifies the aggregated BLS signature from node operators to ensure:
    /// 1. At least 2/3+ of operators signed the blob hash
    /// 2. The aggregated signature is valid for the blob hash
    pub fn finalize_blob(rt: &impl Runtime, params: FinalizeBlobParams) -> Result<(), ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let caller = Caller::new(rt, params.subscriber, None, CallerOption::None)?;

        // Get current blob status from state
        let current_status = rt.state::<State>()?.get_blob_status(
            rt.store(),
            caller.state_address(),
            params.hash,
            params.id.clone(),
        )?;

        // Only finalize blobs that are in Added or Pending status
        // (Resolved blobs are already finalized, Failed blobs cannot be retried)
        if !matches!(
            current_status,
            Some(BlobStatus::Added) | Some(BlobStatus::Pending)
        ) {
            return Ok(());
        }

        Self::verify_blob_signatures(rt, &params)?;

        let event_resolved = matches!(params.status, BlobStatus::Resolved);

        rt.transaction(|st: &mut State, rt| {
            st.finalize_blob(
                rt.store(),
                caller.state_address(),
                FinalizeBlobStateParams::from_actor_params(params.clone(), rt.curr_epoch()),
            )
        })?;

        emit_evm_event(
            rt,
            sol_blobs::BlobFinalized {
                subscriber: caller.event_address(),
                hash: &params.hash,
                resolved: event_resolved,
            },
        )
    }

    /// Verify aggregated BLS signatures for blob finalization.
    ///
    /// Only operators assigned to store shards of this blob are required to sign.
    /// The set of assigned operators is computed deterministically from the blob hash,
    /// blob size, encoding parameters (data_shards, parity_shards), and the active
    /// operator list. The quorum threshold is 2/3+ of the assigned operators.
    fn verify_blob_signatures(
        rt: &impl Runtime,
        params: &FinalizeBlobParams,
    ) -> Result<(), ActorError> {
        use bls_signatures::{
            verify_messages, PublicKey as BlsPublicKey, Serialize as BlsSerialize,
            Signature as BlsSignature,
        };

        // Parse aggregated signature
        let aggregated_sig = BlsSignature::from_bytes(&params.aggregated_signature)
            .map_err(|e| ActorError::illegal_argument(format!("Invalid BLS signature: {:?}", e)))?;

        // Get active operators from state
        let state = rt.state::<State>()?;
        let active_operators = state.operators.get_active_operators();
        let total_operators = active_operators.len();

        if total_operators == 0 {
            return Err(ActorError::illegal_state(
                "No active operators registered".into(),
            ));
        }

        // Look up blob to get encoding parameters
        let blob = state
            .get_blob(rt.store(), params.hash)?
            .ok_or_else(|| ActorError::not_found(format!("Blob {} not found", params.hash)))?;

        let data_shards = blob.data_shards as usize;
        let parity_shards = blob.parity_shards as usize;
        let shards_per_chunk = data_shards + parity_shards;

        // Compute number of chunks from blob size
        const MAX_CHUNK_SIZE: u64 = 16 * 1024 * 1024; // 16 MiB, matches erasure-encoding
        let num_chunks = if params.size == 0 {
            1
        } else {
            params.size.div_ceil(MAX_CHUNK_SIZE) as usize
        };

        // Compute the set of unique assigned operator indices using the deterministic
        // assignment formula from erasure-encoding:
        //   rotation_offset = blob_hash (big-endian) % num_nodes
        //   node_index = (chunk_index * shards_per_chunk + shard_index + rotation_offset) % num_nodes
        let rotation_offset = {
            let mut remainder: u64 = 0;
            for &byte in &params.hash.0 {
                remainder = (remainder * 256 + byte as u64) % total_operators as u64;
            }
            remainder as usize
        };

        let mut assigned_indices = std::collections::BTreeSet::new();
        for chunk_idx in 0..num_chunks {
            for shard_idx in 0..shards_per_chunk {
                let shard_global = chunk_idx * shards_per_chunk + shard_idx;
                let node_index = (shard_global + rotation_offset) % total_operators;
                assigned_indices.insert(node_index);
            }
        }

        let assigned_count = assigned_indices.len();
        if assigned_count == 0 {
            return Err(ActorError::illegal_state(
                "No operators assigned to blob".into(),
            ));
        }

        // Extract signer indices from bitmap, only counting assigned operators
        let mut signer_pubkeys = Vec::new();
        let mut signer_count = 0;

        for (index, operator_addr) in active_operators.iter().enumerate() {
            if index >= 128 {
                break; // u128 bitmap can only hold 128 operators
            }

            // Check if this operator signed (bit is set in bitmap)
            if (params.signer_bitmap & (1u128 << index)) != 0 {
                // Only count signers that are in the assigned set
                if !assigned_indices.contains(&index) {
                    continue;
                }

                signer_count += 1;

                // Get operator info to retrieve BLS public key
                let operator_info =
                    state
                        .operators
                        .get(rt.store(), operator_addr)?
                        .ok_or_else(|| {
                            ActorError::illegal_state(format!(
                                "Operator {} not found in state",
                                operator_addr
                            ))
                        })?;

                // Parse BLS public key
                let pubkey = BlsPublicKey::from_bytes(&operator_info.bls_pubkey).map_err(|e| {
                    ActorError::illegal_state(format!(
                        "Invalid BLS public key for operator {}: {:?}",
                        operator_addr, e
                    ))
                })?;

                signer_pubkeys.push(pubkey);
            }
        }

        // Check threshold: need at least 2/3+ of assigned operators
        let threshold = (assigned_count * 2 + 2) / 3; // Ceiling of 2/3
        if signer_count < threshold {
            return Err(ActorError::illegal_argument(format!(
                "Insufficient signatures: got {}, need {} out of {} assigned operators",
                signer_count, threshold, assigned_count
            )));
        }

        if signer_pubkeys.is_empty() {
            return Err(ActorError::illegal_state("No signer public keys".into()));
        }

        // All operators signed the same message (the blob hash)
        let hash_bytes = params.hash.0.as_slice();

        // Create a vector of the message repeated for each signer
        let messages: Vec<&[u8]> = vec![hash_bytes; signer_count];

        // Verify the aggregated signature using verify_messages
        let verification_result = verify_messages(&aggregated_sig, &messages, &signer_pubkeys);

        if !verification_result {
            return Err(ActorError::illegal_argument(
                "BLS signature verification failed".into(),
            ));
        }

        log::info!(
            "BLS signature verified: {}/{} assigned operators signed (threshold: {}, total operators: {})",
            signer_count,
            assigned_count,
            threshold,
            total_operators
        );

        Ok(())
    }

    /// Debits accounts for current blob usage.
    ///
    /// This is called by the system actor every X blocks, where X is set in the ipc_storage config actor.
    pub fn debit_accounts(rt: &impl Runtime) -> Result<(), ActorError> {
        rt.validate_immediate_caller_is(std::iter::once(&SYSTEM_ACTOR_ADDR))?;

        let config = get_config(rt)?;

        let mut credit_debited = Credit::zero();
        let (deletes, num_accounts, more_accounts) = rt.transaction(|st: &mut State, rt| {
            let initial_credit_debited = st.credits.credit_debited.clone();
            let (deletes, more_accounts) =
                st.debit_accounts(rt.store(), &config, rt.curr_epoch())?;
            credit_debited = &st.credits.credit_debited - initial_credit_debited;
            let num_accounts = st.accounts.len();
            Ok((deletes, num_accounts, more_accounts))
        })?;

        for hash in deletes {
            delete_from_disc(hash)?;
        }

        emit_evm_event(
            rt,
            CreditDebited {
                amount: credit_debited,
                num_accounts,
                more_accounts,
            },
        )?;

        Ok(())
    }

    /// Register a new node operator with BLS public key and RPC URL
    ///
    /// The caller's address will be registered as the operator address.
    /// This method can be called by anyone who wants to become a node operator.
    pub fn register_node_operator(
        rt: &impl Runtime,
        params: RegisterNodeOperatorParams,
    ) -> Result<usize, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        // Validate BLS public key length (must be 48 bytes)
        if params.bls_pubkey.len() != 48 {
            return Err(ActorError::illegal_argument(
                "BLS public key must be exactly 48 bytes".into(),
            ));
        }

        // Validate RPC URL is not empty
        if params.rpc_url.is_empty() {
            return Err(ActorError::illegal_argument(
                "RPC URL cannot be empty".into(),
            ));
        }

        let operator_address = rt.message().caller();

        let index = rt.transaction(|st: &mut State, rt| {
            let node_operator_info = crate::state::operators::NodeOperatorInfo {
                bls_pubkey: params.bls_pubkey,
                rpc_url: params.rpc_url,
                registered_epoch: rt.curr_epoch(),
                active: true,
            };

            st.operators
                .register(rt.store(), operator_address, node_operator_info)
        })?;

        Ok(index)
    }

    /// Get information about a specific node operator
    pub fn get_operator_info(
        rt: &impl Runtime,
        params: GetOperatorInfoParams,
    ) -> Result<Option<OperatorInfo>, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let state = rt.state::<State>()?;
        let info = state.operators.get(rt.store(), &params.address)?;

        Ok(info.map(|i| OperatorInfo {
            bls_pubkey: i.bls_pubkey,
            rpc_url: i.rpc_url,
            active: i.active,
        }))
    }

    /// Get the ordered list of all active node operators
    ///
    /// The order of addresses in the returned list corresponds to the bit positions
    /// in the signature bitmap used for BLS signature aggregation.
    pub fn get_active_operators(rt: &impl Runtime) -> Result<GetActiveOperatorsReturn, ActorError> {
        rt.validate_immediate_caller_accept_any()?;

        let state = rt.state::<State>()?;
        let operators = state.operators.get_active_operators();

        Ok(GetActiveOperatorsReturn { operators })
    }
}
