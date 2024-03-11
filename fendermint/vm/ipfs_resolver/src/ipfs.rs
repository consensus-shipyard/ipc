// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::time::Duration;

use async_stm::{atomically, queues::TQueueLike};
use ipc_ipld_resolver::IpfsResolver as Resolver;

use crate::pool::{ResolveQueue, ResolveTask};

/// The IPLD Resolver takes resolution tasks from the [ResolvePool] and
/// uses the [ipc_ipld_resolver] to fetch the content from subnets.
pub struct IpfsResolver<C> {
    client: C,
    queue: ResolveQueue,
    retry_delay: Duration,
}

impl<C> IpfsResolver<C>
where
    C: Resolver + Clone + Send + 'static,
{
    pub fn new(client: C, queue: ResolveQueue, retry_delay: Duration) -> Self {
        Self {
            client,
            queue,
            retry_delay,
        }
    }

    /// Start taking tasks from the resolver pool and resolving them using the IPLD Resolver.
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
            );
        }
    }
}

/// Run task resolution in the background, so as not to block items from other
/// subnets being tried.
fn start_resolve<C>(task: ResolveTask, client: C, queue: ResolveQueue, retry_delay: Duration)
where
    C: Resolver + Send + 'static,
{
    tokio::spawn(async move {
        let res = client.resolve_ipfs(task.cid()).await;

        let err = match res {
            Err(e) => {
                tracing::error!(
                    error = e.to_string(),
                    "failed to submit ipfs resolution task"
                );
                // The service is no longer listening, we might as well stop taking new tasks from the queue.
                // By not quitting we should see this error every time there is a new task, which is at least is a constant reminder.
                return;
            }
            Ok(Ok(())) => None,
            Ok(Err(e)) => Some(e),
        };

        match err {
            None => {
                tracing::info!(cid = ?task.cid(), "ipfs content resolved");
                atomically(|| task.set_resolved()).await;
            }
            Some(e) => {
                tracing::error!(
                    cid = ?task.cid(),
                    error = e.to_string(),
                    "ipfs content resolution failed; retrying later"
                );
                schedule_retry(task, queue, retry_delay);
            }
        }
    });
}

/// Part of error handling.
///
/// In our case we enqueued the task from transaction processing,
/// which will not happen again, so there is no point further
/// propagating this error back to the sender to deal with.
/// Rather, we should retry until we can conclude whether it will
/// ever complete. Some errors raised by the service are transitive,
/// such as having no peers currently, but that might change.
///
/// For now, let's retry the same task later.
fn schedule_retry(task: ResolveTask, queue: ResolveQueue, retry_delay: Duration) {
    tokio::spawn(async move {
        tokio::time::sleep(retry_delay).await;
        tracing::debug!(cid = ?task.cid(), "retrying content resolution");
        atomically(move || queue.write(task.clone())).await;
    });
}
