// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Context, Result};
use cid::Cid;
use fil_actors_runtime::cbor;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::MethodNum;
use ipc_gateway::BottomUpCheckpoint;
use primitives::TCid;
use tokio::sync::Notify;

use crate::config::Subnet;
use crate::jsonrpc::JsonRpcClient;
use crate::lotus::client::LotusJsonRPCClient;
use crate::lotus::message::mpool::MpoolPushMessage;
use crate::lotus::LotusClient;
use crate::manager::checkpoint::{wait_next_iteration, CHAIN_HEAD_REQUEST_PERIOD};

/// Monitors a subnet `child` for checkpoint blocks. It emits an event for every new checkpoint block.
pub async fn manage_bottomup_checkpoints(
    (child, parent): (Subnet, Subnet),
    stop_notify: Arc<Notify>,
) -> Result<()> {
    log::info!(
        "Starting bottom-up checkpoint manager for (child, parent) subnet pair ({}, {})",
        child.id,
        parent.id
    );

    let child_client = LotusJsonRPCClient::from_subnet(&child);
    let parent_client = LotusJsonRPCClient::from_subnet(&parent);

    let result: Result<()> = try {
        // Read the parent's chain head and obtain the tip set CID.
        log::debug!("Getting parent tipset");
        let parent_head = parent_client.chain_head().await?;
        let cid_map = parent_head.cids.first().unwrap().clone();
        let parent_tip_set = Cid::try_from(cid_map)?;

        // Extract the checkpoint period from the state of the subnet actor in the parent.
        log::debug!("Get checkpointing period from subnet actor in parent");
        let state = parent_client
            .ipc_read_subnet_actor_state(&child.id, parent_tip_set)
            .await
            .map_err(|e| {
                log::error!("error getting subnet actor state for {:?}", &child.id);
                e
            })?;
        let period = state.bottom_up_check_period;

        // We can now start looping. In each loop we read the child subnet's chain head and check if
        // it is time to submit a new checkpoint. If it is, we construct and submit a checkpoint.
        loop {
            let child_head = child_client.chain_head().await?;
            let curr_epoch: ChainEpoch = ChainEpoch::try_from(child_head.height)?;
            // get child gateway state to see if the subnet is initialized
            let cid_map = child_head.cids.first().unwrap().clone();
            let child_tip_set = Cid::try_from(cid_map)?;
            let child_gw_state = child_client.ipc_read_gateway_state(child_tip_set).await?;

            // get parent chain head
            let parent_head = parent_client.chain_head().await?;
            // A key assumption we make now is that each block has exactly one tip set. We panic
            // if this is not the case as it violates our assumption.
            // TODO: update this logic once the assumption changes (i.e., mainnet)
            assert_eq!(parent_head.cids.len(), 1);
            let cid_map = parent_head.cids.first().unwrap().clone();
            let parent_tip_set = Cid::try_from(cid_map)?;
            // get subnet actor state and last checkpoint executed
            let subnet_actor_state = parent_client
                .ipc_read_subnet_actor_state(&child.id, parent_tip_set)
                .await?;
            let last_exec = subnet_actor_state
                .bottom_up_checkpoint_voting
                .last_voting_executed;
            let submission_epoch = last_exec + period;

            // wait for the subnet to be initialized before submitting.
            // if it is time to execute a checkpoint...
            if child_gw_state.initialized && curr_epoch >= submission_epoch {
                // First, we check which accounts are in the validator set. This is done by reading
                // the parent's chain head and requesting the state at that tip set.
                let mut validator_set: HashSet<Address, RandomState> = HashSet::new();
                match subnet_actor_state.validator_set.validators {
                    None => {}
                    Some(validators) => {
                        for v in validators {
                            validator_set.insert(Address::from_str(v.addr.deref())?);
                        }
                    }
                };

                // Now, for each account defined in the `child` subnet that is in the validator set, we
                // submit a checkpoint on its behalf.
                assert_eq!(child_head.cids.len(), 1); // Again, check key assumption
                let child_tip_set = Cid::try_from(child_head.cids.first().unwrap().clone())?;
                for account in child.accounts.iter() {
                    if validator_set.contains(account) {
                        // check if the validator already voted
                        // let has_voted = parent_client
                        //     .ipc_validator_has_voted_bottomup(&child.id, submission_epoch, account)
                        //     .await
                        //     .map_err(|e| {
                        //         log::error!(
                        //             "error checking if validator has voted in subnet: {:?}",
                        //             &child.id
                        //         );
                        //         e
                        //     })?;
                        // FIXME: There is a nasty bug in the de-serialization of EpochVoteSubmissions in
                        // the actor due to the fact that we are using Cids and nodes can't be load.
                        // commenting for now, but needs to be fixed in actors.
                        let has_voted = false;
                        if !has_voted {
                            // submitting the checkpoint synchronously and waiting to be committed.
                            let r = submit_checkpoint(
                                child_tip_set,
                                submission_epoch,
                                account,
                                &child,
                                &child_client,
                                &parent_client,
                            )
                            .await;
                            if r.is_err() {
                                log::warn!("error submitting bottom-up checkpoint, waiting to next iteration: {:?}", r);
                                if !wait_next_iteration(&stop_notify, CHAIN_HEAD_REQUEST_PERIOD)
                                    .await?
                                {
                                    return Ok(());
                                }
                                continue;
                            }

                            loop {
                                // check if by any chance we have the opportunity to submit any outstanding checkpoint we may be
                                // missing in case the previous one was executed successfully.
                                // - we get the up to date head of the parent and the child.
                                // - check the last executed checkpoint for the subnet
                                // - And if we still have the info, submit a new checkpoint
                                let child_head = child_client.chain_head().await?;
                                let curr_epoch: ChainEpoch =
                                    ChainEpoch::try_from(child_head.height)?;
                                let parent_head = parent_client.chain_head().await?;
                                let cid_map = parent_head.cids.first().unwrap().clone();
                                let parent_tip_set = Cid::try_from(cid_map)?;
                                let subnet_actor_state = parent_client
                                    .ipc_read_subnet_actor_state(&child.id, parent_tip_set)
                                    .await?;
                                let last_exec = subnet_actor_state
                                    .bottom_up_checkpoint_voting
                                    .last_voting_executed;
                                let submission_epoch = last_exec + period;
                                if curr_epoch >= submission_epoch {
                                    let r = submit_checkpoint(
                                        child_tip_set,
                                        submission_epoch,
                                        account,
                                        &child,
                                        &child_client,
                                        &parent_client,
                                    )
                                    .await;
                                    if r.is_err() {
                                        log::warn!("error submitting bottom-up checkpoint, waiting to next iteration: {:?}", r);
                                        if !wait_next_iteration(
                                            &stop_notify,
                                            CHAIN_HEAD_REQUEST_PERIOD,
                                        )
                                        .await?
                                        {
                                            return Ok(());
                                        }
                                        break;
                                    }
                                } else {
                                    // if no checkpoint lagging we can wait for the
                                    // next iteration.
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            // Sleep for an appropriate amount of time before checking the chain head again or return
            // if a stop notification is received.
            if !wait_next_iteration(&stop_notify, CHAIN_HEAD_REQUEST_PERIOD).await? {
                return Ok(());
            }
        }
    };
    result.context(format!(
        "error in subnet pair ({}, {})",
        parent.id, child.id
    ))
}

/// Submits a checkpoint for `epoch` on behalf of `account` to the subnet actor of `child_subnet`
/// deployed on the parent subnet.
async fn submit_checkpoint<T: JsonRpcClient + Send + Sync>(
    child_tip_set: Cid,
    epoch: ChainEpoch,
    account: &Address,
    child_subnet: &Subnet,
    child_client: &LotusJsonRPCClient<T>,
    parent_client: &LotusJsonRPCClient<T>,
) -> Result<()> {
    log::info!(
        "Submitting checkpoint bottom-up for account {} and epoch {} from child {}",
        account,
        epoch,
        child_subnet.id,
    );
    let mut checkpoint = BottomUpCheckpoint::new(child_subnet.id.clone(), epoch);

    // From the template on the gateway actor of the child subnet, we get the children checkpoints
    // and the bottom-up cross-net messages.
    log::debug!(
        "Getting checkpoint bottom-up template for {epoch:} in subnet: {:?}",
        &child_subnet.id
    );
    let template = child_client
        .ipc_get_checkpoint_template(epoch)
        .await
        .map_err(|e| {
            log::error!(
                "error getting bottom-up checkpoint template for epoch:{epoch:} in subnet: {:?}",
                &child_subnet.id
            );
            e
        })?;
    checkpoint.data.children = template.data.children;
    checkpoint.data.cross_msgs = template.data.cross_msgs;

    log::info!(
        "checkpoint at epoch {:} contains {:} number of cross messages",
        checkpoint.data.epoch,
        checkpoint
            .data
            .cross_msgs
            .cross_msgs
            .as_ref()
            .map(|s| s.len())
            .unwrap_or_default()
    );

    // Get the CID of previous checkpoint of the child subnet from the gateway actor of the parent
    // subnet.
    log::debug!(
        "Getting previous checkpoint bottom-up from parent gateway for {epoch:} in subnet: {:?}",
        &child_subnet.id
    );
    let response = parent_client
        .ipc_get_prev_checkpoint_for_child(child_subnet.id.clone())
        .await
        .map_err(|e| {
            log::error!(
                "error getting previous bottom-up checkpoint for epoch:{epoch:} in subnet: {:?}",
                &child_subnet.id
            );
            e
        })?;

    // if previous checkpoint is set
    if response.is_some() {
        let cid = Cid::try_from(response.unwrap())?;
        checkpoint.data.prev_check = TCid::from(cid);
    }
    checkpoint.data.proof = child_tip_set.to_bytes();

    // The checkpoint is constructed. Now we call the `submit_checkpoint` method on the subnet actor
    // of the child subnet that is deployed on the parent subnet.
    log::debug!(
        "Pushing bottom-up checkpoint submission message for {epoch:} in subnet: {:?}",
        &child_subnet.id
    );
    let to = child_subnet.id.subnet_actor();
    let from = *account;
    let message = MpoolPushMessage::new(
        to,
        from,
        ipc_subnet_actor::Method::SubmitCheckpoint as MethodNum,
        cbor::serialize(&checkpoint, "checkpoint")?.to_vec(),
    );
    let mem_push_response = parent_client
        .mpool_push_message(message)
        .await
        .map_err(|e| {
            log::error!(
                "error submitting bottom-up checkpoint for epoch {epoch:} in subnet: {:?}",
                &child_subnet.id
            );
            e
        })?;

    // wait for the checkpoint to be committed before moving on.
    let message_cid = mem_push_response.cid()?;
    log::debug!("bottom-up checkpoint message published with cid: {message_cid:?}");
    log::info!("waiting bottom-up for checkpoint for epoch {epoch:} to be committed");
    parent_client.state_wait_msg(message_cid).await?;
    log::info!("successfully published bottom-up checkpoint submission for epoch {epoch:}");

    Ok(())
}
