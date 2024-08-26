// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::time::Duration;

use async_stm::{atomically, atomically_or_err, queues::TQueueLike};
use fendermint_vm_topdown::voting::VoteTally;
use ipc_api::subnet_id::SubnetID;
use ipc_ipld_resolver::{Client, ResolverIroh, ValidatorKey, VoteRecord};
use iroh::blobs::Hash;
use libp2p::identity::Keypair;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::pool::{ResolveQueue, ResolveTask};

/// The iroh Resolver takes resolution tasks from the [ResolvePool] and
/// uses the [ipc_ipld_resolver] to fetch the content from the local iroh node.
pub struct IrohResolver<V> {
    client: Client<V>,
    queue: ResolveQueue,
    retry_delay: Duration,
    vote_tally: VoteTally,
    key: Keypair,
    subnet_id: SubnetID,
    to_vote: fn(Hash) -> V,
}

impl<V> IrohResolver<V>
where
    V: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    pub fn new(
        client: Client<V>,
        queue: ResolveQueue,
        retry_delay: Duration,
        vote_tally: VoteTally,
        key: Keypair,
        subnet_id: SubnetID,
        to_vote: fn(Hash) -> V,
    ) -> Self {
        Self {
            client,
            queue,
            retry_delay,
            vote_tally,
            key,
            subnet_id,
            to_vote,
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
    to_vote: fn(Hash) -> V,
) where
    V: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
{
    tokio::spawn(async move {
        tracing::debug!(hash = ?task.hash(), "starting iroh blob resolve");
        let res = client.resolve_iroh(task.hash(), task.node_addr()).await;

        let err = match res {
            Err(e) => {
                tracing::error!(
                    error = e.to_string(),
                    "failed to submit iroh resolution task"
                );
                // The service is no longer listening, we might as well stop taking new tasks from the queue.
                // By not quitting, we should see this error every time there is a new task, which is at least a constant reminder.
                return;
            }
            Ok(Ok(())) => None,
            Ok(Err(e)) => Some(e),
        };

        match err {
            None => {
                tracing::debug!(hash = ?task.hash(), "iroh blob resolved");

                // Mark task as resolved
                atomically(|| task.set_resolved()).await;

                let vote = to_vote(task.hash());
                match VoteRecord::signed(&key, subnet_id, vote) {
                    Ok(vote) => {
                        // Add our own vote
                        let validator_key = ValidatorKey::from(key.public());
                        let res = atomically_or_err(|| {
                            vote_tally.add_blob_vote(
                                validator_key.clone(),
                                task.hash().as_bytes().to_vec(),
                            )
                        })
                        .await;

                        match res {
                            Ok(added) => {
                                if added {
                                    // Send our own vote to peers
                                    if let Err(e) = client.publish_vote(vote) {
                                        tracing::error!(
                                            error = e.to_string(),
                                            "failed to publish vote"
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!(error = e.to_string(), "failed to handle own vote");
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!(error = e.to_string(), "failed to sign vote");
                    }
                }
            }
            Some(e) => {
                tracing::error!(
                    hash = ?task.hash(),
                    error = e.to_string(),
                    "iroh blob resolution failed; retrying later"
                );
                schedule_retry(task, queue, retry_delay);
            }
        }
    });
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
fn schedule_retry(task: ResolveTask, queue: ResolveQueue, retry_delay: Duration) {
    tokio::spawn(async move {
        if atomically(|| task.add_attempt()).await {
            tokio::time::sleep(retry_delay).await;
            tracing::debug!(hash = ?task.hash(), "retrying blob resolution");
            atomically(|| queue.write(task.clone())).await;
        } else {
            tracing::warn!(hash = ?task.hash(), "no attempts remaining; blob resolution failed");
            atomically(|| task.set_failed()).await;
        }
    });
}
