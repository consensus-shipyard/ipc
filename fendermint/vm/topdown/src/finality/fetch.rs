// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::finality::null::FinalityWithNull;
use crate::finality::ParentViewPayload;
use crate::proxy::ParentQueryProxy;
use crate::{
    handle_null_round, BlockHash, BlockHeight, Config, Error, IPCParentFinality,
    ParentFinalityProvider, ParentViewProvider,
};
use async_stm::{Stm, StmResult};
use ipc_sdk::cross::CrossMsg;
use ipc_sdk::staking::StakingChangeRequest;
use std::sync::Arc;

/// The finality provider that performs io to the parent if not found in cache
#[derive(Clone)]
pub struct CachedFinalityProvider<T> {
    inner: FinalityWithNull,
    config: Config,
    /// The ipc client proxy that works as a back up if cache miss
    parent_client: Arc<T>,
}

/// Exponential backoff for futures
macro_rules! retry {
    ($wait:expr, $retires:expr, $f:expr) => {{
        let mut retries = $retires;
        let mut wait = $wait;

        loop {
            let res = $f;
            if let Err(e) = &res {
                // there is no point in retrying if the current block is null round
                if crate::is_null_round_str(&e.to_string()) {
                    tracing::warn!(
                        "cannot query ipc parent_client due to null round, skip retry"
                    );
                    break res;
                }

                tracing::warn!(
                    error = e.to_string(),
                    retries,
                    wait = ?wait,
                    "cannot query ipc parent_client"
                );

                if retries > 0 {
                    retries -= 1;

                    tokio::time::sleep(wait).await;

                    wait *= 2;
                    continue;
                }
            }

            break res;
        }
    }};
}

#[async_trait::async_trait]
impl<T: ParentQueryProxy + Send + Sync + 'static> ParentViewProvider for CachedFinalityProvider<T> {
    fn genesis_epoch(&self) -> anyhow::Result<BlockHeight> {
        self.inner.genesis_epoch()
    }

    async fn validator_changes_from(
        &self,
        from: BlockHeight,
        to: BlockHeight,
    ) -> anyhow::Result<Vec<StakingChangeRequest>> {
        let mut v = vec![];
        for h in from..=to {
            let mut r = self.validator_changes(h).await?;
            tracing::debug!(
                number_of_messages = r.len(),
                height = h,
                "obtained validator change set",
            );
            v.append(&mut r);
        }

        Ok(v)
    }

    async fn validator_changes(
        &self,
        height: BlockHeight,
    ) -> anyhow::Result<Vec<StakingChangeRequest>> {
        let r = self.inner.validator_changes(height).await?;

        if let Some(v) = r {
            return Ok(v);
        }

        let r = retry!(
            self.config.exponential_back_off,
            self.config.exponential_retry_limit,
            self.parent_client
                .get_validator_changes(height)
                .await
                .map(|r| r.value)
        );

        handle_null_round(r, Vec::new)
    }

    /// Should always return the top down messages, only when ipc parent_client is down after exponential
    /// retries
    async fn top_down_msgs(
        &self,
        height: BlockHeight,
        block_hash: &BlockHash,
    ) -> anyhow::Result<Vec<CrossMsg>> {
        let r = self.inner.top_down_msgs(height).await?;

        if let Some(v) = r {
            return Ok(v);
        }

        let r = retry!(
            self.config.exponential_back_off,
            self.config.exponential_retry_limit,
            self.parent_client
                .get_top_down_msgs_with_hash(height, block_hash)
                .await
        );

        handle_null_round(r, Vec::new)
    }

    async fn top_down_msgs_from(
        &self,
        from: BlockHeight,
        to: BlockHeight,
        block_hash: &BlockHash,
    ) -> anyhow::Result<Vec<CrossMsg>> {
        let mut v = vec![];
        for h in from..=to {
            let mut r = self.top_down_msgs(h, block_hash).await?;
            tracing::debug!(
                number_of_top_down_messages = r.len(),
                height = h,
                "obtained topdown messages",
            );
            v.append(&mut r);
        }
        Ok(v)
    }
}

impl<T: ParentQueryProxy + Send + Sync + 'static> ParentFinalityProvider
    for CachedFinalityProvider<T>
{
    fn next_proposal(&self) -> Stm<Option<IPCParentFinality>> {
        self.inner.next_proposal()
    }

    fn check_proposal(&self, proposal: &IPCParentFinality) -> Stm<bool> {
        self.inner.check_proposal(proposal)
    }

    fn set_new_finality(
        &self,
        finality: IPCParentFinality,
        previous_finality: Option<IPCParentFinality>,
    ) -> Stm<()> {
        self.inner.set_new_finality(finality, previous_finality)
    }
}

impl<T: ParentQueryProxy + Send + Sync + 'static> CachedFinalityProvider<T> {
    /// Creates an uninitialized provider
    /// We need this because `fendermint` has yet to be initialized and might
    /// not be able to provide an existing finality from the storage. This provider requires an
    /// existing committed finality. Providing the finality will enable other functionalities.
    pub async fn uninitialized(config: Config, parent_client: Arc<T>) -> anyhow::Result<Self> {
        let genesis = parent_client.get_genesis_epoch().await?;
        Ok(Self::new(config, genesis, None, parent_client))
    }
}

impl<T> CachedFinalityProvider<T> {
    pub(crate) fn new(
        config: Config,
        genesis_epoch: BlockHeight,
        committed_finality: Option<IPCParentFinality>,
        parent_client: Arc<T>,
    ) -> Self {
        let inner = FinalityWithNull::new(genesis_epoch, committed_finality);
        Self {
            inner,
            config,
            parent_client,
        }
    }

    pub fn block_hash(&self, height: BlockHeight) -> Stm<Option<BlockHash>> {
        self.inner.block_hash_at_height(height)
    }

    pub fn first_non_null_parent_hash(&self, height: BlockHeight) -> Stm<Option<BlockHash>> {
        self.inner.first_non_null_parent_hash(height)
    }

    pub fn latest_height(&self) -> Stm<Option<BlockHeight>> {
        self.inner.latest_height()
    }

    pub fn last_committed_finality(&self) -> Stm<Option<IPCParentFinality>> {
        self.inner.last_committed_finality()
    }

    /// Clear the cache and set the committed finality to the provided value
    pub fn reset(&self, finality: IPCParentFinality) -> Stm<()> {
        self.inner.reset(finality)
    }

    pub fn new_parent_view(
        &self,
        height: BlockHeight,
        maybe_payload: Option<ParentViewPayload>,
    ) -> StmResult<(), Error> {
        self.inner.new_parent_view(height, maybe_payload)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    #[tokio::test]
    async fn test_retry() {
        struct Test {
            nums_run: AtomicUsize,
        }

        impl Test {
            async fn run(&self) -> Result<(), &'static str> {
                self.nums_run.fetch_add(1, Ordering::SeqCst);
                Err("mocked error")
            }
        }

        let t = Test {
            nums_run: AtomicUsize::new(0),
        };

        let res = retry!(Duration::from_secs(1), 2, t.run().await);
        assert!(res.is_err());
        // execute the first time, retries twice
        assert_eq!(t.nums_run.load(Ordering::SeqCst), 3);
    }
}
