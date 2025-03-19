// Copyright 2025 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::time::Duration;

use crate::observe::{
    BlobsFinalityVotingFailure, BlobsFinalityVotingSuccess, ReadRequestsCloseVoting,
};
use async_stm::{atomically, atomically_or_err, queues::TQueueLike};
use fendermint_vm_topdown::voting::VoteTally;
use ipc_api::subnet_id::SubnetID;
use ipc_ipld_resolver::{Client, ResolverIroh, ResolverIrohReadRequest, ValidatorKey, VoteRecord};
use ipc_observability::emit;

use iroh::blobs::Hash;
use libp2p::identity::Keypair;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::pool::{ResolveKey, ResolveQueue, ResolveResults, ResolveTask, TaskType};

/// The iroh Resolver takes resolution tasks from the [ResolvePool] and
/// uses the [ipc_ipld_resolver] to fetch the content from the local iroh node.
pub struct IrohResolver<V> {
    client: Client<V>,
    queue: ResolveQueue,
    retry_delay: Duration,
    vote_tally: VoteTally,
    key: Keypair,
    subnet_id: SubnetID,
    to_vote: fn(Hash, bool) -> V,
    results: ResolveResults,
}

impl<V> IrohResolver<V>
where
    V: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        client: Client<V>,
        queue: ResolveQueue,
        retry_delay: Duration,
        vote_tally: VoteTally,
        key: Keypair,
        subnet_id: SubnetID,
        to_vote: fn(Hash, bool) -> V,
        results: ResolveResults,
    ) -> Self {
        Self {
            client,
            queue,
            retry_delay,
            vote_tally,
            key,
            subnet_id,
            to_vote,
            results,
        }
    }

    /// Start taking tasks from the resolver pool and resolving them using the iroh Resolver.
    pub async fn run(self) {
        loop {
            let task = atomically(|| {
                let task = self.queue.read()?;
                Ok(task)
            })
            .await;

            start_resolve(
                task,
                self.client.clone(),
                self.queue.clone(),
                self.retry_delay,
                self.vote_tally.clone(),
                self.key.clone(),
                self.subnet_id.clone(),
                self.to_vote,
                self.results.clone(),
            );
        }
    }
}

/// Run task resolution in the background, so as not to block items from other
/// subnets being tried.
#[allow(clippy::too_many_arguments)]
fn start_resolve<V>(
    task: ResolveTask,
    client: Client<V>,
    queue: ResolveQueue,
    retry_delay: Duration,
    vote_tally: VoteTally,
    key: Keypair,
    subnet_id: SubnetID,
    to_vote: fn(Hash, bool) -> V,
    results: ResolveResults,
) where
    V: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    tokio::spawn(async move {
        tracing::debug!(hash = %task.hash(), "starting iroh blob resolve");
        match task.task_type() {
            TaskType::ResolveBlob { source, size } => {
                match client
                    .resolve_iroh(task.hash(), size, source.id.into())
                    .await
                {
                    Err(e) => {
                        tracing::error!(
                            error = e.to_string(),
                            "failed to submit iroh resolution task"
                        );
                        // The service is no longer listening, we might as well stop taking new tasks from the queue.
                        // By not quitting, we should see this error every time there is a new task, which is at least a constant reminder.
                    }
                    Ok(Ok(())) => {
                        tracing::debug!(hash = %task.hash(), "iroh blob resolved");
                        atomically(|| task.set_resolved()).await;
                        if add_own_vote(
                            task.hash(),
                            client,
                            vote_tally,
                            key,
                            subnet_id,
                            true,
                            to_vote,
                        )
                        .await
                        {
                            emit(BlobsFinalityVotingSuccess {
                                blob_hash: Some(task.hash().to_string()),
                            });
                        }
                    }
                    Ok(Err(e)) => {
                        tracing::error!(
                            hash = ?task.hash(),
                            error = e.to_string(),
                            "iroh blob resolution failed, attempting retry"
                        );
                        // If we fail to re-enqueue the task, cast a "failure" vote.
                        // And emit a failure event.
                        if !reenqueue(task.clone(), queue, retry_delay).await
                            && add_own_vote(
                                task.hash(),
                                client,
                                vote_tally,
                                key,
                                subnet_id,
                                false,
                                to_vote,
                            )
                            .await
                        {
                            emit(BlobsFinalityVotingFailure {
                                blob_hash: Some(task.hash().to_string()),
                            });
                        }
                    }
                };
            }
            TaskType::CloseReadRequest {
                blob_hash,
                offset,
                len,
            } => {
                match client.close_read_request(blob_hash, offset, len).await {
                    Err(e) => {
                        tracing::error!(
                            error = e.to_string(),
                            "failed to submit iroh resolution task"
                        );
                        // The service is no longer listening, we might as well stop taking new tasks from the queue.
                        // By not quitting, we should see this error every time there is a new task, which is at least a constant reminder.
                    }
                    Ok(Ok(response)) => {
                        let hash = task.hash();
                        tracing::debug!(hash = ?hash, "iroh read request resolved");

                        atomically(|| task.set_resolved()).await;
                        atomically(|| {
                            results.update(|mut results| {
                                results.insert(ResolveKey { hash }, response.to_vec());
                                results
                            })
                        })
                        .await;

                        // Extend task hash with response data to use as the vote hash.
                        // This ensures that the all validators are voting
                        // on the same response from IROH.
                        let mut task_id = task.hash().as_bytes().to_vec();
                        task_id.extend(response.to_vec());
                        let vote_hash = Hash::new(task_id);
                        if add_own_vote(
                            vote_hash, client, vote_tally, key, subnet_id, true, to_vote,
                        )
                        .await
                        {
                            emit(ReadRequestsCloseVoting {
                                read_request_id: Some(vote_hash.to_string()),
                            });
                        }
                    }
                    Ok(Err(e)) => {
                        tracing::error!(
                            hash = ?task.hash(),
                            error = e.to_string(),
                            "iroh read request failed"
                        );
                        if !reenqueue(task.clone(), queue, retry_delay).await {
                            tracing::error!(
                                hash = ?task.hash(),
                                "failed to re-enqueue read request"
                            );
                        }
                    }
                };
            }
        };
    });
}

async fn add_own_vote<V>(
    vote_hash: Hash,
    client: Client<V>,
    vote_tally: VoteTally,
    key: Keypair,
    subnet_id: SubnetID,
    resolved: bool,
    to_vote: fn(Hash, bool) -> V,
) -> bool
where
    V: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    let vote = to_vote(vote_hash, resolved);
    match VoteRecord::signed(&key, subnet_id, vote) {
        Ok(vote) => {
            let validator_key = ValidatorKey::from(key.public());
            let res = atomically_or_err(|| {
                vote_tally.add_blob_vote(
                    validator_key.clone(),
                    vote_hash.as_bytes().to_vec(),
                    resolved,
                )
            })
            .await;

            match res {
                Ok(added) => {
                    if added {
                        // Send our own vote to peers
                        if let Err(e) = client.publish_vote(vote) {
                            tracing::error!(error = e.to_string(), "failed to publish vote");
                            return false;
                        }
                    }
                    true
                }
                Err(e) => {
                    tracing::error!(error = e.to_string(), "failed to handle own vote");
                    false
                }
            }
        }
        Err(e) => {
            tracing::error!(error = e.to_string(), "failed to sign vote");
            false
        }
    }
}

async fn reenqueue(task: ResolveTask, queue: ResolveQueue, retry_delay: Duration) -> bool {
    if atomically(|| task.add_attempt()).await {
        tracing::error!(
            hash = %task.hash(),
            "iroh blob resolution failed; retrying later"
        );
        schedule_retry(task, queue, retry_delay).await;
        true
    } else {
        tracing::error!(
            hash = ?task.hash(),
            "iroh blob resolution failed; no attempts remaining"
        );
        atomically(|| task.add_failure()).await;
        false
    }
}

/// Part of error handling.
///
/// In our case, we added the task from transaction processing,
/// which will not happen again, so there is no point further
/// propagating this error back to the sender to deal with.
/// Rather, we should retry until we can conclude whether it will
/// ever complete. Some errors raised by the service are transitive,
/// such as having no peers currently, but that might change.
///
/// For now, let's retry the same task later.
async fn schedule_retry(task: ResolveTask, queue: ResolveQueue, retry_delay: Duration) {
    tokio::spawn(async move {
        tokio::time::sleep(retry_delay).await;
        tracing::debug!(hash = ?task.hash(), "retrying blob resolution after sleep");
        atomically(|| queue.write(task.clone())).await;
    });
}
