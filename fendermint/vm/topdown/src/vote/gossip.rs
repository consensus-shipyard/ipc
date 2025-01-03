// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::ops::DerefMut;
use std::sync::Arc;
use crate::vote::error::Error;
use crate::vote::payload::Vote;
use async_trait::async_trait;

/// The gossip client communicates with the underlying gossip pub/sub network on various
/// subscribed topics. This client handles the event listening/forwarding and event publication.
#[async_trait]
pub trait GossipClient {
    /// Attempts to poll if there are available vote. This method returns immediately.
    /// If there is no vote, it returns None
    async fn recv_vote(&mut self) -> Result<Vote, Error>;

    async fn publish_vote(&self, vote: Vote) -> Result<(), Error>;
}

#[async_trait]
impl <G: GossipClient + Send + Sync + 'static> GossipClient for Arc<G> {
    async fn recv_vote(&mut self) -> Result<Vote, Error> {
        self.deref_mut().recv_vote().await
    }

    async fn publish_vote(&self, vote: Vote) -> Result<(), Error> {
        self.as_ref().publish_vote(vote).await
    }
}