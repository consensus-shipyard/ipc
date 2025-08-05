// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use super::state::ipc::tokens_to_burn;
use super::state::{ipc::GatewayCaller, FvmExecState};

use crate::fvm::activity::ValidatorActivityTracker;
use anyhow::Context;
use ethers::abi::{AbiEncode, Tokenizable};
use fendermint_vm_genesis::{Power, Validator};
use fvm_ipld_blockstore::Blockstore;
use ipc_actors_abis::checkpointing_facet as checkpoint;
use ipc_actors_abis::checkpointing_facet::{Commitment, FvmAddress, Ipcaddress, SubnetID};
use ipc_actors_abis::gateway_getter_facet::gateway_getter_facet;
use ipc_api::checkpoint::{abi_encode_envelope, abi_encode_envelope_fields};
use ipc_api::merkle::MerkleGen;
use ipc_api::staking::ConfigurationNumber;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tendermint::block::Height;

/// Validator voting power snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerTable(pub Vec<Validator<Power>>);

/// Changes in the power table.
#[derive(Debug, Clone, Default)]
pub struct PowerUpdates(pub Vec<Validator<Power>>);

#[derive(Serialize, Deserialize)]
pub struct MessageBatchCommitment {
    pub total_num_msgs: u64,
    pub msgs_root: [u8; 32],
}

#[derive(Serialize, Deserialize)]
pub struct LightClientCommitments {
    pub msg_batch_commitment: MessageBatchCommitment,
    pub validator_next_configuration_number: u64,
    pub activity_commitment: [u8; 32],
}

pub fn ipc_end_block_hook<DB>(
    gateway: &GatewayCaller<DB>,
    state: &mut FvmExecState<DB>,
) -> anyhow::Result<Option<(LightClientCommitments, PowerUpdates)>>
where
    DB: Blockstore + Sync + Send + Clone + 'static,
{
    // Epoch transitions for checkpointing.
    let height: Height = state
        .block_height()
        .try_into()
        .context("block height is not u64")?;

    let Some(msgs) = should_create_checkpoint(gateway, state, height)? else {
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
    let cross_msg_commitment = MessageBatchCommitment {
        total_num_msgs: msgs_count as u64,
        msgs_root,
    };
    let activity_commitment = state.activity_tracker().commit_activity()?.compressed()?;

    // Figure out the power updates if there was some change in the configuration.
    let power_updates = if next_configuration_number == 0 {
        PowerUpdates(Vec::new())
    } else {
        let (next_power_configuration_number, next_power_table) =
            ipc_power_table(gateway, state).context("failed to get next power table")?;

        debug_assert_eq!(next_power_configuration_number, next_configuration_number);

        power_diff(curr_power_table, next_power_table)
    };

    let commitments = LightClientCommitments {
        msg_batch_commitment: cross_msg_commitment,
        validator_next_configuration_number: next_configuration_number,
        activity_commitment: ethers::utils::keccak256(activity_commitment.encode()),
    };
    Ok(Some((commitments, power_updates)))
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

fn convert_tokenizables<Source: Tokenizable, Target: Tokenizable>(
    tokenizables: Vec<Source>,
) -> anyhow::Result<Vec<Target>> {
    Ok(tokenizables
        .into_iter()
        .map(|t| Target::from_token(t.into_token()))
        .collect::<Result<Vec<_>, _>>()?)
}

fn should_create_checkpoint<DB>(
    gateway: &GatewayCaller<DB>,
    state: &mut FvmExecState<DB>,
    height: Height,
) -> anyhow::Result<Option<Vec<gateway_getter_facet::IpcEnvelope>>>
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

    let msgs = convert_tokenizables(batch.msgs)?;
    Ok(Some(msgs))
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

    use crate::fvm::end_block_hook::{into_power_map, power_diff};

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
