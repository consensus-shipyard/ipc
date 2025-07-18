// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use super::observe::{
    CheckpointCreated, CheckpointFinalized, CheckpointSigned, CheckpointSignedRole,
};
use super::state::ipc::tokens_to_burn;
use super::{
    broadcast::Broadcaster,
    state::{ipc::GatewayCaller, FvmExecState},
    ValidatorContext,
};

use crate::fvm::activity::ValidatorActivityTracker;
use crate::types::BlockEndEvents;
use anyhow::{anyhow, Context};
use ethers::abi::Tokenizable;
use fendermint_crypto::PublicKey;
use fendermint_crypto::SecretKey;
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_actor_interface::ipc::BottomUpCheckpoint;
use fendermint_vm_genesis::{Power, Validator, ValidatorKey};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{address::Address, chainid::ChainID};
use ipc_actors_abis::checkpointing_facet as checkpoint;
use ipc_actors_abis::checkpointing_facet::{Commitment, FvmAddress, Ipcaddress, SubnetID};
use ipc_actors_abis::gateway_getter_facet as getter;
use ipc_actors_abis::gateway_getter_facet::gateway_getter_facet;
use ipc_api::checkpoint::{abi_encode_envelope, abi_encode_envelope_fields};
use ipc_api::merkle::MerkleGen;
use ipc_api::staking::ConfigurationNumber;
use ipc_observability::{emit, observe::TracingError, serde::HexEncodableBlockHash, Traceable};
use std::collections::HashMap;
use std::time::Duration;
use tendermint::block::Height;
use tendermint_rpc::endpoint::commit;
use tendermint_rpc::{endpoint::validators, Client, Paging};

/// Validator voting power snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerTable(pub Vec<Validator<Power>>);

/// Changes in the power table.
#[derive(Debug, Clone, Default)]
pub struct PowerUpdates(pub Vec<Validator<Power>>);

pub struct CheckpointOutcome {
    pub checkpoint: ipc_actors_abis::checkpointing_facet::BottomUpCheckpoint,
    pub power_updates: PowerUpdates,
    pub block_end_events: BlockEndEvents,
}

#[derive(Clone)]
pub struct BottomUpManager<DB, C>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
    C: Client + Clone + Send + Sync + 'static,
{
    /// Tendermint client for querying the RPC.
    tendermint_client: C,
    /// If this is a validator node, this should be the key we can use to sign transactions.
    validator_ctx: Option<ValidatorContext<C>>,

    // Gateway caller for IPC gateway interactions
    gateway_caller: GatewayCaller<DB>,
}

impl<DB, C> BottomUpManager<DB, C>
where
    DB: Blockstore + Clone + 'static + Send + Sync,
    C: Client + Clone + Send + Sync + 'static,
{
    pub fn new(tendermint_client: C, validator_ctx: Option<ValidatorContext<C>>) -> Self {
        Self {
            tendermint_client,
            validator_ctx,
            gateway_caller: GatewayCaller::default(),
        }
    }

    pub fn create_checkpoint_if_needed(
        &self,
        state: &mut FvmExecState<DB>,
    ) -> anyhow::Result<Option<CheckpointOutcome>> {
        let mut block_end_events = BlockEndEvents::default();

        // Emit trace; errors here are logged but not fatal.
        let _ = emit_trace_if_check_checkpoint_finalized(&self.gateway_caller, state).inspect_err(
            |e| {
                emit(TracingError {
                    affected_event: CheckpointFinalized::name(),
                    reason: e.to_string(),
                });
            },
        );

        let maybe_result =
            maybe_create_checkpoint(&self.gateway_caller, state, &mut block_end_events)
                .context("failed to create checkpoint")?;

        if let Some((checkpoint, power_updates)) = maybe_result {
            Ok(Some(CheckpointOutcome {
                checkpoint,
                power_updates,
                block_end_events,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn cast_validator_signatures_for_incomplete_checkpoints(
        &self,
        current_checkpoint: ipc_actors_abis::checkpointing_facet::BottomUpCheckpoint,
        state: &mut FvmExecState<DB>,
    ) -> anyhow::Result<()> {
        // Exit early if there's no validator context.
        let validator_ctx = match self.validator_ctx.as_ref() {
            Some(ctx) => ctx,
            None => return Ok(()),
        };

        // If we're currently syncing, do not resend past signatures.
        if self.syncing().await {
            return Ok(());
        }

        // Retrieve incomplete checkpoints synchronously (state cannot be shared across threads).
        let incomplete_checkpoints =
            unsigned_checkpoints(&self.gateway_caller, state, validator_ctx.public_key)
                .context("failed to fetch incomplete checkpoints")?;

        // Ensure that the current checkpoint exists among the incomplete ones.
        debug_assert!(
            incomplete_checkpoints.iter().any(|checkpoint| {
                checkpoint.block_height == current_checkpoint.block_height
                    && checkpoint.block_hash == current_checkpoint.block_hash
            }),
            "the current checkpoint is incomplete"
        );

        // Clone the necessary values to move into the asynchronous task.
        let client = self.tendermint_client.clone();
        let gateway = self.gateway_caller.clone();
        let chain_id = state.chain_id();
        let height = current_checkpoint.block_height;
        let validator_ctx = validator_ctx.clone();

        // Spawn an asynchronous task to broadcast incomplete checkpoint signatures.
        tokio::spawn(async move {
            if let Err(e) = broadcast_incomplete_signatures(
                &client,
                &validator_ctx,
                &gateway,
                chain_id,
                incomplete_checkpoints,
            )
            .await
            {
                tracing::error!(error = ?e, height = height.as_u64(), "error broadcasting checkpoint signature");
            }
        });

        Ok(())
    }

    /// Indicate that the node is syncing with the rest of the network and hasn't caught up with the tip yet.
    async fn syncing(&self) -> bool {
        match self.tendermint_client.status().await {
            Ok(status) => status.sync_info.catching_up,
            Err(e) => {
                // CometBFT often takes a long time to boot, e.g. while it's replaying blocks it won't
                // respond to JSON-RPC calls. Let's treat this as an indication that we are syncing.
                tracing::warn!(error =? e, "failed to get CometBFT sync status");
                true
            }
        }
    }
}

/// Construct and store a checkpoint if this is the end of the checkpoint period.
/// Perform end-of-checkpoint-period transitions in the ledger.
///
/// If we are the boundary, return the validators eligible to sign and any updates
/// to the power table, along with the checkpoint that needs to be signed by validators.
pub fn maybe_create_checkpoint<DB>(
    gateway: &GatewayCaller<DB>,
    state: &mut FvmExecState<DB>,
    event_tracker: &mut BlockEndEvents,
) -> anyhow::Result<Option<(checkpoint::BottomUpCheckpoint, PowerUpdates)>>
where
    DB: Blockstore + Sync + Send + Clone + 'static,
{
    // Epoch transitions for checkpointing.
    let height: tendermint::block::Height = state
        .block_height()
        .try_into()
        .context("block height is not u64")?;

    let block_hash = state
        .block_hash()
        .ok_or_else(|| anyhow!("block hash not set"))?;

    let Some((msgs, subnet_id)) = should_create_checkpoint(gateway, state, height)? else {
        return Ok(None);
    };

    // Get the current power table from the ledger, not CometBFT.
    let (_, curr_power_table) =
        ipc_power_table(gateway, state).context("failed to get the current power table")?;

    // Apply any validator set transitions.
    let next_configuration_number = gateway
        .apply_validator_changes(state)
        .context("failed to apply validator changes")?;

    // Sum up the value leaving the subnet as part of the bottom-up messages.
    let burnt_tokens = tokens_to_burn(&msgs);

    // NOTE: Unlike when we minted tokens for the gateway by modifying its balance,
    // we don't have to burn them here, because it's already being done in
    // https://github.com/consensus-shipyard/ipc-solidity-actors/pull/263
    // by sending the funds to the BURNTFUNDS_ACTOR.
    // Ostensibly we could opt _not_ to decrease the circ supply here, but rather
    // look up the burnt funds balance at the beginning of each block and subtract
    // it from the monotonically increasing supply, in which case it could reflect
    // a wider range of burning activity than just IPC.
    // It might still be inconsistent if someone uses another address for burning tokens.
    // By decreasing here, at least `circ_supply` is consistent with IPC.
    state.update_circ_supply(|circ_supply| {
        *circ_supply -= burnt_tokens;
    });

    let msgs = convert_envelopes(msgs);
    let msgs_count = msgs.len();

    let mut msgs_root = [0u8; 32];
    if msgs_count > 0 {
        msgs_root = MerkleGen::new(
            abi_encode_envelope,
            msgs.as_slice(),
            &abi_encode_envelope_fields(),
        )?
        .root()
        .to_fixed_bytes()
    }

    let full_activity_rollup = state.activity_tracker().commit_activity()?;

    // Construct checkpoint.
    let checkpoint = BottomUpCheckpoint {
        subnet_id,
        block_height: ethers::types::U256::from(height.value()),
        block_hash,
        next_configuration_number,
        msgs: Commitment {
            total_num_msgs: msgs_count as u64,
            msgs_root,
        },
        activity: full_activity_rollup.compressed()?,
    };

    // Save the checkpoint in the ledger.
    // Pass in the current power table, because these are the validators who can sign this checkpoint.
    let ret = gateway
        .create_bottom_up_checkpoint(
            state,
            checkpoint.clone(),
            &curr_power_table.0,
            msgs,
            full_activity_rollup.into_inner(),
        )
        .context("failed to store checkpoint")?;
    event_tracker.push((ret.apply_ret.events, ret.emitters));

    // Figure out the power updates if there was some change in the configuration.
    let power_updates = if next_configuration_number == 0 {
        PowerUpdates(Vec::new())
    } else {
        let (next_power_configuration_number, next_power_table) =
            ipc_power_table(gateway, state).context("failed to get next power table")?;

        debug_assert_eq!(next_power_configuration_number, next_configuration_number);

        power_diff(curr_power_table, next_power_table)
    };

    emit(CheckpointCreated {
        height: height.value(),
        hash: HexEncodableBlockHash(block_hash.to_vec()),
        msg_count: msgs_count,
        config_number: next_configuration_number,
    });

    Ok(Some((checkpoint, power_updates)))
}

/// Wait until CometBFT has reached a specific block height.
///
/// This is used so we can wait for the next block where the ledger changes
/// we have done durign execution has been committed.
async fn wait_for_commit<C>(
    client: &C,
    block_height: u64,
    retry_delay: Duration,
) -> anyhow::Result<()>
where
    C: Client + Clone + Send + Sync + 'static,
{
    loop {
        let res: commit::Response = client
            .latest_commit()
            .await
            .context("failed to fetch latest commit")?;

        if res.signed_header.header().height.value() >= block_height {
            return Ok(());
        }

        tokio::time::sleep(retry_delay).await;
    }
}

/// Collect incomplete signatures from the ledger which this validator hasn't signed yet.
///
/// It doesn't check whether the validator should have signed it, that's done inside
/// [broadcast_incomplete_signatures] at the moment. The goal is rather to avoid double
/// signing for those who have already done it.
pub fn unsigned_checkpoints<DB>(
    gateway: &GatewayCaller<DB>,
    state: &mut FvmExecState<DB>,
    validator_key: PublicKey,
) -> anyhow::Result<Vec<getter::BottomUpCheckpoint>>
where
    DB: Blockstore + Send + Sync + Clone + 'static,
{
    let mut unsigned_checkpoints = Vec::new();
    let validator_addr = EthAddress::from(validator_key);

    let incomplete_checkpoints = gateway
        .incomplete_checkpoints(state)
        .context("failed to fetch incomplete checkpoints")?;

    for cp in incomplete_checkpoints {
        let signatories = gateway
            .checkpoint_signatories(state, cp.block_height.as_u64())
            .context("failed to get checkpoint signatories")?;

        if !signatories.contains(&validator_addr) {
            unsigned_checkpoints.push(cp);
        }
    }

    Ok(unsigned_checkpoints)
}

/// Sign the current and any incomplete checkpoints.
pub async fn broadcast_incomplete_signatures<C, DB>(
    client: &C,
    validator_ctx: &ValidatorContext<C>,
    gateway: &GatewayCaller<DB>,
    chain_id: ChainID,
    incomplete_checkpoints: Vec<getter::BottomUpCheckpoint>,
) -> anyhow::Result<()>
where
    C: Client + Clone + Send + Sync + 'static,
    DB: Blockstore + Send + Sync + Clone + 'static,
{
    // Make sure that these had time to be added to the ledger.
    if let Some(highest) = incomplete_checkpoints
        .iter()
        .map(|cp| cp.block_height)
        .max()
    {
        wait_for_commit(
            client,
            highest.as_u64() + 1,
            validator_ctx.broadcaster.retry_delay(),
        )
        .await
        .context("failed to wait for commit")?;
    }

    for cp in incomplete_checkpoints {
        let height = Height::try_from(cp.block_height.as_u64())?;
        // Getting the power table from CometBFT where the history is available.
        let power_table = bft_power_table(client, height)
            .await
            .context("failed to get power table")?;

        if let Some(validator) = power_table
            .0
            .iter()
            .find(|v| v.public_key.0 == validator_ctx.public_key)
            .cloned()
        {
            // TODO: Code generation in the ipc-solidity-actors repo should cater for this.
            let checkpoint = checkpoint::BottomUpCheckpoint {
                subnet_id: checkpoint::SubnetID {
                    root: cp.subnet_id.root,
                    route: cp.subnet_id.route,
                },
                block_height: cp.block_height,
                block_hash: cp.block_hash,
                next_configuration_number: cp.next_configuration_number,
                msgs: Commitment {
                    total_num_msgs: cp.msgs.total_num_msgs,
                    msgs_root: cp.msgs.msgs_root,
                },
                activity: checkpoint::CompressedActivityRollup {
                    consensus: checkpoint::CompressedSummary {
                        stats: checkpoint::AggregatedStats {
                            total_active_validators: cp
                                .activity
                                .consensus
                                .stats
                                .total_active_validators,
                            total_num_blocks_committed: cp
                                .activity
                                .consensus
                                .stats
                                .total_num_blocks_committed,
                        },
                        data_root_commitment: cp.activity.consensus.data_root_commitment,
                    },
                },
            };

            // We mustn't do these in parallel because of how nonces are fetched.
            broadcast_signature(
                &validator_ctx.broadcaster,
                gateway,
                checkpoint,
                &power_table,
                &validator,
                &validator_ctx.secret_key,
                chain_id,
            )
            .await
            .context("failed to broadcast checkpoint signature")?;

            emit(CheckpointSigned {
                role: CheckpointSignedRole::Own,
                height: height.value(),
                hash: HexEncodableBlockHash(cp.block_hash.to_vec()),
                validator: validator_ctx.addr,
            });

            tracing::debug!(?height, "submitted checkpoint signature");
        }
    }
    Ok(())
}

/// As a validator, sign the checkpoint and broadcast a transaction to add our signature to the ledger.
pub async fn broadcast_signature<C, DB>(
    broadcaster: &Broadcaster<C>,
    gateway: &GatewayCaller<DB>,
    checkpoint: checkpoint::BottomUpCheckpoint,
    power_table: &PowerTable,
    validator: &Validator<Power>,
    secret_key: &SecretKey,
    chain_id: ChainID,
) -> anyhow::Result<()>
where
    C: Client + Clone + Send + Sync + 'static,
    DB: Blockstore + Send + Sync + Clone + 'static,
{
    let calldata = gateway
        .add_checkpoint_signature_calldata(checkpoint, &power_table.0, validator, secret_key)
        .context("failed to produce checkpoint signature calldata")?;

    let tx_hash = broadcaster
        .fevm_invoke(Address::from(gateway.addr()), calldata, chain_id)
        .await
        .context("failed to broadcast signature")?;

    // The transaction should be in the mempool now.
    tracing::info!(tx_hash = tx_hash.to_string(), "broadcasted signature");

    Ok(())
}

// Emit a CheckpointFinalized trace event if a checkpoint has been finalized on the current block.
pub fn emit_trace_if_check_checkpoint_finalized<DB>(
    gateway: &GatewayCaller<DB>,
    state: &mut FvmExecState<DB>,
) -> anyhow::Result<()>
where
    DB: Blockstore + Clone,
{
    if !gateway.is_anchored(state)? {
        return Ok(());
    }

    let block_height = state.block_height();
    let block_hash = state
        .block_hash()
        .ok_or_else(|| anyhow!("block hash not set"))?;

    // Check if the checkpoint has been finalized.
    // If no checkpoint was emitted at this height, the QuorumInfo struct will carry zero values,
    // including reached=false.
    let checkpoint_quorum = gateway.checkpoint_info(state, block_height)?;

    if checkpoint_quorum.reached {
        emit(CheckpointFinalized {
            height: block_height,
            hash: HexEncodableBlockHash(block_hash.to_vec()),
        })
    }

    Ok(())
}

fn convert_tokenizables<Source: Tokenizable, Target: Tokenizable>(
    tokenizables: Vec<Source>,
) -> anyhow::Result<Vec<Target>> {
    Ok(tokenizables
        .into_iter()
        .map(|t| Target::from_token(t.into_token()))
        .collect::<Result<Vec<_>, _>>()?)
}

fn convert_envelopes(msgs: Vec<gateway_getter_facet::IpcEnvelope>) -> Vec<checkpoint::IpcEnvelope> {
    msgs.into_iter()
        .map(|m| checkpoint::IpcEnvelope {
            kind: m.kind,
            local_nonce: m.local_nonce,
            from: Ipcaddress {
                subnet_id: SubnetID {
                    root: m.from.subnet_id.root,
                    route: m.from.subnet_id.route,
                },
                raw_address: FvmAddress {
                    addr_type: m.from.raw_address.addr_type,
                    payload: m.from.raw_address.payload,
                },
            },
            to: Ipcaddress {
                subnet_id: SubnetID {
                    root: m.to.subnet_id.root,
                    route: m.to.subnet_id.route,
                },
                raw_address: FvmAddress {
                    addr_type: m.to.raw_address.addr_type,
                    payload: m.to.raw_address.payload,
                },
            },
            value: m.value,
            original_nonce: m.original_nonce,
            message: m.message,
        })
        .collect()
}

fn should_create_checkpoint<DB>(
    gateway: &GatewayCaller<DB>,
    state: &mut FvmExecState<DB>,
    height: Height,
) -> anyhow::Result<Option<(Vec<gateway_getter_facet::IpcEnvelope>, checkpoint::SubnetID)>>
where
    DB: Blockstore + Clone,
{
    let id = gateway.subnet_id(state)?;
    let is_root = id.route.is_empty();

    if is_root {
        return Ok(None);
    }

    let batch = gateway.bottom_up_msg_batch(state, height.into())?;

    if batch.block_height.as_u64() != 0 {
        tracing::debug!(
            height = height.value(),
            "bottom up msg batch exists at height"
        );
    } else if height.value() % gateway.bottom_up_check_period(state)? == 0 {
        tracing::debug!(
            height = height.value(),
            "bottom up checkpoint period reached height"
        );
    } else {
        return Ok(None);
    }

    let id = checkpoint::SubnetID {
        root: id.root,
        route: id.route,
    };
    let msgs = convert_tokenizables(batch.msgs)?;
    Ok(Some((msgs, id)))
}

/// Get the power table from CometBFT.
///
/// This is prone to failing, e.g. one theory is that CometBFT is trying to restart
/// the application, and while doing that it does not open up its HTTP services,
/// leading to a chicken-and-egg problem of failing to start.
async fn bft_power_table<C>(client: &C, height: Height) -> anyhow::Result<PowerTable>
where
    C: Client + Sync + Send + 'static,
{
    let mut power_table = Vec::new();
    let validators: validators::Response = client.validators(height, Paging::All).await?;

    for v in validators.validators {
        power_table.push(Validator {
            public_key: ValidatorKey::try_from(v.pub_key)?,
            power: Power(v.power()),
        });
    }

    Ok(PowerTable(power_table))
}

/// Get the current power table from the Gateway actor.
fn ipc_power_table<DB>(
    gateway: &GatewayCaller<DB>,
    state: &mut FvmExecState<DB>,
) -> anyhow::Result<(ConfigurationNumber, PowerTable)>
where
    DB: Blockstore + Sync + Send + Clone + 'static,
{
    gateway
        .current_power_table(state)
        .context("failed to get current power table")
        .map(|(cn, pt)| (cn, PowerTable(pt)))
}

/// Calculate the difference between the current and the next power table, to return to CometBFT only what changed:
/// * include any new validator, or validators whose power has been updated
/// * include validators to be removed with a power of 0, as [expected](https://github.com/informalsystems/tendermint-rs/blob/bcc0b377812b8e53a02dff156988569c5b3c81a2/rpc/src/dialect/end_block.rs#L12-L14) by CometBFT
fn power_diff(current: PowerTable, next: PowerTable) -> PowerUpdates {
    let current = into_power_map(current);
    let next = into_power_map(next);

    let mut diff = Vec::new();

    // Validators in `current` but not in `next` should be removed.
    for (k, v) in current.iter() {
        if !next.contains_key(k) {
            let delete = Validator {
                public_key: v.public_key.clone(),
                power: Power(0),
            };
            diff.push(delete);
        }
    }

    // Validators in `next` that differ from `current` should be updated.
    for (k, v) in next.into_iter() {
        let insert = match current.get(&k) {
            Some(w) if *w == v => None,
            _ => Some(v),
        };
        if let Some(insert) = insert {
            diff.push(insert);
        }
    }

    PowerUpdates(diff)
}

/// Convert the power list to a `HashMap` to support lookups by the public key.
///
/// Unfortunately in their raw format the [`PublicKey`] does not implement `Hash`,
/// so we have to use the serialized format.
fn into_power_map(value: PowerTable) -> HashMap<[u8; 65], Validator<Power>> {
    value
        .0
        .into_iter()
        .map(|v| {
            let k = v.public_key.0.serialize();
            (k, v)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use fendermint_vm_genesis::{Power, Validator};
    use quickcheck_macros::quickcheck;

    use crate::fvm::bottomup::{into_power_map, power_diff};

    use super::{PowerTable, PowerUpdates};

    fn power_update(current: PowerTable, updates: PowerUpdates) -> PowerTable {
        let mut current = into_power_map(current);

        for v in updates.0 {
            let k = v.public_key.0.serialize();
            if v.power.0 == 0 {
                current.remove(&k);
            } else {
                current.insert(k, v);
            }
        }

        PowerTable(current.into_values().collect())
    }

    #[derive(Debug, Clone)]
    struct TestPowerTables {
        current: PowerTable,
        next: PowerTable,
    }

    impl quickcheck::Arbitrary for TestPowerTables {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let v = 1 + usize::arbitrary(g) % 10;
            let c = 1 + usize::arbitrary(g) % v;
            let n = 1 + usize::arbitrary(g) % v;

            let vs = (0..v).map(|_| Validator::arbitrary(g)).collect::<Vec<_>>();
            let cvs = vs.iter().take(c).cloned().collect();
            let nvs = vs
                .into_iter()
                .skip(v - n)
                .map(|mut v| {
                    v.power = Power::arbitrary(g);
                    v
                })
                .collect();

            TestPowerTables {
                current: PowerTable(cvs),
                next: PowerTable(nvs),
            }
        }
    }

    #[quickcheck]
    fn prop_power_diff_update(powers: TestPowerTables) {
        let diff = power_diff(powers.current.clone(), powers.next.clone());
        let next = power_update(powers.current, diff);

        // Order shouldn't matter.
        let next = into_power_map(next);
        let expected = into_power_map(powers.next);

        assert_eq!(next, expected)
    }

    #[quickcheck]
    fn prop_power_diff_nochange(v1: Validator<Power>, v2: Validator<Power>) {
        let current = PowerTable(vec![v1.clone(), v2.clone()]);
        let next = PowerTable(vec![v2, v1]);
        assert!(power_diff(current, next).0.is_empty());
    }
}
