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
use ipc_gateway::TopDownCheckpoint;
use ipc_sdk::subnet_id::SubnetID;
use tokio::sync::Notify;

use crate::config::Subnet;
use crate::constants::GATEWAY_ACTOR_ADDRESS;
use crate::jsonrpc::JsonRpcClient;
use crate::lotus::client::LotusJsonRPCClient;
use crate::lotus::message::mpool::MpoolPushMessage;
use crate::lotus::LotusClient;
use crate::manager::checkpoint::{wait_next_iteration, CHAIN_HEAD_REQUEST_PERIOD};

pub async fn manage_topdown_checkpoints(
    (child, parent): (Subnet, Subnet),
    stop_notify: Arc<Notify>,
) -> Result<()> {
    log::info!(
        "Starting top-down checkpoint manager for (child, parent) subnet pair ({}, {})",
        child.id,
        parent.id
    );

    let child_client = LotusJsonRPCClient::from_subnet(&child);
    let parent_client = LotusJsonRPCClient::from_subnet(&parent);

    let result: Result<()> = try {
        // Read the child's chain head and obtain the tip set CID.
        log::debug!("Getting child tipset and starting top-down checkpointing manager");
        let child_head = child_client.chain_head().await?;
        let cid_map = child_head.cids.first().unwrap().clone();
        let child_tip_set = Cid::try_from(cid_map)?;

        // Read the child's chain head and obtain the topdown checkpoint period
        // and genesis epoch.
        let state = child_client.ipc_read_gateway_state(child_tip_set).await?;
        let period = state.top_down_check_period;

        loop {
            // get current epoch in the parent and tipset
            log::debug!("Get current epoch in parent tipset");
            let parent_head = parent_client.chain_head().await?;
            let curr_epoch: ChainEpoch = ChainEpoch::try_from(parent_head.height)?;
            let cid_map = parent_head.cids.first().unwrap().clone();
            let parent_tip_set = Cid::try_from(cid_map)?;

            // get child gateway state to determine the last executed checkpoint
            // and compute the submission epoch
            log::debug!("Get submission epoch for checkpoint from child");
            let child_head = child_client.chain_head().await?;
            let cid_map = child_head.cids.first().unwrap().clone();
            let child_tip_set = Cid::try_from(cid_map)?;
            let child_gw_state = child_client.ipc_read_gateway_state(child_tip_set).await?;
            let last_exec = child_gw_state
                .top_down_checkpoint_voting
                .last_voting_executed;
            let submission_epoch = last_exec + period;

            // wait for the subnet to be initialized before submitting.
            // if it is time to execute a checkpoint...
            if child_gw_state.initialized && curr_epoch >= submission_epoch {
                // We check which accounts are in the validator set. This is done by reading
                // the parent's chain head and requesting the state at that tip set.
                let subnet_actor_state = parent_client
                    .ipc_read_subnet_actor_state(&child.id, parent_tip_set)
                    .await
                    .map_err(|e| {
                        log::error!("error getting subnet actor state for {:?}", &child.id);
                        e
                    })?;

                let mut validator_set: HashSet<Address, RandomState> = HashSet::new();
                match subnet_actor_state.validator_set.validators {
                    None => {}
                    Some(validators) => {
                        for v in validators {
                            validator_set.insert(Address::from_str(v.addr.deref())?);
                        }
                    }
                };

                // For each account that we manage that is in the validator set, we submit a topdown
                // checkpoint.
                for account in child.accounts.iter() {
                    log::debug!("Getting list of validators from subnet");
                    if validator_set.contains(account) {
                        // check if the validator already voted the top-down checkpoint
                        // in the child.
                        let has_voted = child_client
                            .ipc_validator_has_voted_topdown(
                                // FIXME: Do not use the default, use the one configured
                                // for the subnet
                                &Address::from_str(GATEWAY_ACTOR_ADDRESS)?,
                                submission_epoch,
                                account,
                            )
                            .await
                            .map_err(|e| {
                                log::error!(
                                    "error checking if validator has voted in subnet: {:?}",
                                    &child.id
                                );
                                e
                            })?;

                        if !has_voted {
                            // submitting the checkpoint synchronously and waiting to be committed.
                            submit_topdown_checkpoint(
                                submission_epoch,
                                parent_tip_set,
                                child_tip_set,
                                account,
                                child.id.clone(),
                                &child_client,
                                &parent_client,
                            )
                            .await?;

                            // loop to submit the lagging checkpoints as many as possible
                            loop {
                                // check if by any chance we have the opportunity to submit any outstanding checkpoint we may be
                                // missing in case the previous one was executed successfully.
                                // - we get the up to date head of the parent and the child.
                                // - check the last executed checkpoint for the subnet
                                // - And if we still have the info, submit a new checkpoint
                                let parent_head = parent_client.chain_head().await?;
                                let curr_epoch: ChainEpoch =
                                    ChainEpoch::try_from(parent_head.height)?;
                                let cid_map = parent_head.cids.first().unwrap().clone();
                                let parent_tip_set = Cid::try_from(cid_map)?;
                                let child_head = child_client.chain_head().await?;
                                let cid_map = child_head.cids.first().unwrap().clone();
                                let child_tip_set = Cid::try_from(cid_map)?;
                                let child_gw_state =
                                    child_client.ipc_read_gateway_state(child_tip_set).await?;
                                let last_exec = child_gw_state
                                    .top_down_checkpoint_voting
                                    .last_voting_executed;
                                let submission_epoch = last_exec + period;
                                if curr_epoch >= submission_epoch {
                                    submit_topdown_checkpoint(
                                        submission_epoch,
                                        parent_tip_set,
                                        child_tip_set,
                                        account,
                                        child.id.clone(),
                                        &child_client,
                                        &parent_client,
                                    )
                                    .await?;
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
        "error in manage_topdown_checkpoints() for subnet pair ({}, {})",
        parent.id, child.id
    ))
}

// Prototype function for submitting topdown messages. This function is supposed to be called each
// Nth epoch of a parent subnet. It reads the topdown messages from the parent subnet and submits
// them to the child subnet.
async fn submit_topdown_checkpoint<T: JsonRpcClient + Send + Sync>(
    submission_epoch: ChainEpoch,
    curr_parent_tip_set: Cid,
    curr_child_tip_set: Cid,
    account: &Address,
    child_subnet: SubnetID,
    child_client: &LotusJsonRPCClient<T>,
    parent_client: &LotusJsonRPCClient<T>,
) -> Result<()> {
    log::info!("Submitting topdown checkpoint for account {}", account);
    // First, we read from the child subnet the nonce of the last topdown message executed
    // after the last executed checkpoint. We
    // increment the result by one to obtain the nonce of the first topdown message we want to
    // submit to the child subnet.
    let state = child_client
        .ipc_read_gateway_state(curr_child_tip_set)
        .await?;

    let nonce = state.applied_topdown_nonce;

    // Then, we get the top-down messages from the latest nonce at the specific submission epoch.
    // This ensures that all validators will provide deterministically the top-down messages
    // to be included in all checkpoints.
    // FIXME: We probably shouldn't use the default one here and use the one
    // deployed in the subnet
    log::debug!("Get tipset by height at submission_epoch: {submission_epoch}");
    let gateway_addr = Address::from_str(GATEWAY_ACTOR_ADDRESS)?;
    let submission_tip_set = parent_client
        .get_tipset_by_height(submission_epoch, curr_parent_tip_set)
        .await?;
    let cid_map = submission_tip_set.cids.first().unwrap().clone();
    let submission_tip_set = Cid::try_from(cid_map)?;
    log::debug!(
        "Get top down messages for {:?} and nonce {:?}",
        &child_subnet,
        nonce
    );
    let top_down_msgs = parent_client
        .ipc_get_topdown_msgs(&child_subnet, gateway_addr, submission_tip_set, nonce)
        .await?;

    // Finally, we submit the topdown messages to the child subnet.
    let to = gateway_addr;
    let from = *account;
    let topdown_checkpoint = TopDownCheckpoint {
        epoch: submission_epoch,
        top_down_msgs,
    };
    let message = MpoolPushMessage::new(
        to,
        from,
        ipc_gateway::Method::SubmitTopDownCheckpoint as MethodNum,
        cbor::serialize(&topdown_checkpoint, "topdown_checkpoint")?.to_vec(),
    );
    let mem_push_response = child_client.mpool_push_message(message)
        .await
        .map_err(|e| {
            log::error!(
                "error submitting top-down checkpoint for epoch {submission_epoch:} in subnet: {:?}",
                &child_subnet
            );
            e
        })?;

    // wait for the checkpoint to be committed before moving on.
    let message_cid = mem_push_response.cid()?;
    log::debug!("top-down checkpoint message published with cid: {message_cid:?}");
    log::info!("waiting for top-down checkpoint for epoch {submission_epoch:} to be committed");
    child_client.state_wait_msg(message_cid).await?;
    log::info!(
        "successfully published top-down checkpoint submission for epoch {submission_epoch:}"
    );

    Ok(())
}
