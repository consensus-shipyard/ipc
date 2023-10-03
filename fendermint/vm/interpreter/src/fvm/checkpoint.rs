// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, Context};
use tendermint::block::Height;
use tendermint_rpc::{endpoint::validators, Client, Paging};

use fvm_ipld_blockstore::Blockstore;
use fvm_shared::{address::Address, chainid::ChainID};

use fendermint_crypto::SecretKey;
use fendermint_vm_actor_interface::ipc::BottomUpCheckpoint;
use fendermint_vm_genesis::{Power, Validator, ValidatorKey};
use fendermint_vm_ipc_actors::gateway_getter_facet as getter;
use fendermint_vm_ipc_actors::gateway_router_facet as router;

use super::{
    broadcast::Broadcaster,
    state::{ipc::GatewayCaller, FvmExecState},
    ValidatorContext,
};

/// Validator voting power snapshot.
#[derive(Debug, Clone)]
pub struct PowerTable(pub Vec<Validator>);

/// Changes in the power table.
#[derive(Debug, Clone, Default)]
pub struct PowerUpdates(pub Vec<Validator>);

/// Construct and store a checkpoint if this is the end of the checkpoint period.
/// Perform end-of-checkpoint-period transitions in the ledger.
///
/// If we are the boundary, return the validators eligible to sign and any updates
/// to the power table, along with the checkpoint that needs to be signed by validators.
pub async fn maybe_create_checkpoint<C, DB>(
    client: &C,
    gateway: &GatewayCaller<DB>,
    state: &mut FvmExecState<DB>,
) -> anyhow::Result<Option<(router::BottomUpCheckpoint, PowerTable, PowerUpdates)>>
where
    C: Client + Sync + Send + 'static,
    DB: Blockstore + Sync + Send + 'static,
{
    // Epoch transitions for checkpointing.
    let height: tendermint::block::Height = state
        .block_height()
        .try_into()
        .context("block height is not u64")?;

    let block_hash = state
        .block_hash()
        .ok_or_else(|| anyhow!("block hash not set"))?;

    match should_create_checkpoint(gateway, state, height)? {
        None => Ok(None),
        Some(subnet_id) => {
            // Get the current power table.
            let power_table = get_power_table(client, height)
                .await
                .context("failed to get the power table")?;

            // TODO #252: Take the next validator changes from the gateway.
            let power_updates = PowerUpdates(Vec::new());

            // TODO: #252: Take the configuration number of the last change,
            // or use 0 to signal that there was no change.
            let next_configuration_number = 0;

            // Retrieve the bottom-up messages so we can put their hash into the checkpoint.
            let cross_messages_hash = gateway
                .bottom_up_msgs_hash(state, height.value())
                .context("failed to retrieve bottom-up messages hash")?;

            // Construct checkpoint.
            let checkpoint = BottomUpCheckpoint {
                subnet_id,
                block_height: height.value(),
                block_hash,
                next_configuration_number,
                cross_messages_hash,
            };

            // Save the checkpoint in the ledger.
            // Pass in the current power table, because these are the validators who can sign this checkpoint.
            gateway
                .create_bottom_up_checkpoint(state, checkpoint.clone(), &power_table.0)
                .context("failed to store checkpoint")?;

            Ok(Some((checkpoint, power_table, power_updates)))
        }
    }
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
    DB: Blockstore + Send + Sync + 'static,
{
    for cp in incomplete_checkpoints {
        let height = Height::try_from(cp.block_height)?;
        let power_table = get_power_table(client, height)
            .await
            .context("failed to get power table")?;

        if let Some(validator) = power_table
            .0
            .iter()
            .find(|v| v.public_key.0 == validator_ctx.public_key)
            .cloned()
        {
            // TODO: Code generation in the ipc-solidity-actors repo should cater for this.
            let checkpoint = router::BottomUpCheckpoint {
                subnet_id: router::SubnetID {
                    root: cp.subnet_id.root,
                    route: cp.subnet_id.route,
                },
                block_height: cp.block_height,
                block_hash: cp.block_hash,
                next_configuration_number: cp.next_configuration_number,
                cross_messages_hash: cp.cross_messages_hash,
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

            tracing::debug!(?height, "submitted checkpoint signature");
        }
    }
    Ok(())
}

/// As a validator, sign the checkpoint and broadcast a transaction to add our signature to the ledger.
pub async fn broadcast_signature<C, DB>(
    broadcaster: &Broadcaster<C>,
    gateway: &GatewayCaller<DB>,
    checkpoint: router::BottomUpCheckpoint,
    power_table: &PowerTable,
    validator: &Validator,
    secret_key: &SecretKey,
    chain_id: ChainID,
) -> anyhow::Result<()>
where
    C: Client + Clone + Send + Sync + 'static,
    DB: Blockstore + Send + Sync + 'static,
{
    let calldata = gateway
        .add_checkpoint_signature_calldata(checkpoint, &power_table.0, validator, secret_key)
        .context("failed to produce checkpoint signature calldata")?;

    broadcaster
        .fevm_invoke(Address::from(gateway.addr()), calldata, chain_id)
        .await
        .context("failed to broadcast signature")?;

    Ok(())
}

fn should_create_checkpoint<DB>(
    gateway: &GatewayCaller<DB>,
    state: &mut FvmExecState<DB>,
    height: Height,
) -> anyhow::Result<Option<router::SubnetID>>
where
    DB: Blockstore,
{
    if gateway.enabled(state)? {
        let id = gateway.subnet_id(state)?;
        let is_root = id.route.is_empty();

        if !is_root && height.value() % gateway.bottom_up_check_period(state)? == 0 {
            let id = router::SubnetID {
                root: id.root,
                route: id.route,
            };
            return Ok(Some(id));
        }
    }
    Ok(None)
}

async fn get_power_table<C>(client: &C, height: Height) -> anyhow::Result<PowerTable>
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
