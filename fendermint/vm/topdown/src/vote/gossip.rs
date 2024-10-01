// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::vote::error::Error;
use crate::vote::payload::Vote;
use async_trait::async_trait;

/// The gossip client communicates with the underlying gossip pub/sub network on various
/// subscribed topics. This client handles the event listening/forwarding and event publication.
#[async_trait]
pub trait GossipClient {
    /// Attempts to poll if there are available vote. This method returns immediately.
    /// If there is no vote, it returns None
    fn try_poll_vote(&mut self) -> Result<Option<Vote>, Error>;

    async fn publish_vote(&self, vote: Vote) -> Result<(), Error>;
}
