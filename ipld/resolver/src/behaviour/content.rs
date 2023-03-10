// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use std::task::{Context, Poll};

use libipld::{store::StoreParams, Cid};
use libp2p::{
    swarm::{
        derive_prelude::{ConnectionId, FromSwarm},
        ConnectionHandler, IntoConnectionHandler, NetworkBehaviour, NetworkBehaviourAction,
        PollParameters,
    },
    Multiaddr, PeerId,
};
use libp2p_bitswap::{Bitswap, BitswapConfig, BitswapEvent, BitswapStore};
use prometheus::Registry;

use crate::stats;

pub type QueryId = libp2p_bitswap::QueryId;

// Not much to do here, just hiding the `Progress` event as I don't think we'll need it.
// We can't really turn it into anything more meaningful; the outer Service, which drives
// the Swarm events, will have to store the `QueryId` and figure out which CID it was about
// (there could be multiple queries running over the same CID) and how to respond to the
// original requestor (e.g. by completing a channel).
#[derive(Debug)]
pub enum Event {
    /// Event raised when a resolution request is finished.
    ///
    /// The result will indicate either success, or arbitrary failure.
    /// If it is a success, the CID can be found in the [`BitswapStore`]
    /// instance the behaviour was created with.
    ///
    /// Note that it is possible that the synchronization completed
    /// partially, but some recursive constituent is missing. The
    /// caller can use the [`missing_blocks`] function to check
    /// whether a retry is necessary.
    Complete(QueryId, anyhow::Result<()>),
}

/// Behaviour built on [`Bitswap`] to resolve IPLD content from [`Cid`] to raw bytes.
pub struct Behaviour<P: StoreParams> {
    inner: Bitswap<P>,
}

impl<P: StoreParams> Behaviour<P> {
    pub fn new<S>(store: S) -> Self
    where
        S: BitswapStore<Params = P>,
    {
        let bitswap = Bitswap::new(BitswapConfig::default(), store);
        Self { inner: bitswap }
    }

    /// Register Prometheus metrics.
    pub fn register_metrics(&self, registry: &Registry) -> anyhow::Result<()> {
        self.inner.register_metrics(registry)
    }

    /// Recursively resolve a [`Cid`] and all underlying CIDs into blocks.
    ///
    /// The [`Bitswap`] behaviour will call the [`BitswapStore`] to ask for
    /// blocks which are missing, ie. find CIDs which aren't available locally.
    /// It is up to the store implementation to decide which links need to be
    /// followed.
    ///
    /// It is also up to the store implementation to decide which CIDs requests
    /// to responds to, e.g. if we only want to resolve certain type of content,
    /// then the store can look up in a restricted collection, rather than the
    /// full IPLD store.
    ///
    /// Resolution will be attempted from the peers passed to the method,
    /// starting with the first one with `WANT-BLOCK`, then whoever responds
    /// positively to `WANT-HAVE` requests. The caller should talk to the
    /// `membership::Behaviour` first to find suitable peers, and then
    /// prioritise peers which are connected.
    ///
    /// The underlying [`libp2p_request_response::RequestResponse`] behaviour
    /// will initiate connections to the peers which aren't connected at the moment.
    pub fn resolve(&mut self, cid: Cid, peers: Vec<PeerId>) -> QueryId {
        stats::CONTENT_RESOLVE_RUNNING.inc();
        // Not passing any missing items, which will result in a call to `BitswapStore::missing_blocks`.
        self.inner.sync(cid, peers, [].into_iter())
    }
}

impl<P: StoreParams> NetworkBehaviour for Behaviour<P> {
    type ConnectionHandler = <Bitswap<P> as NetworkBehaviour>::ConnectionHandler;
    type OutEvent = Event;

    fn new_handler(&mut self) -> Self::ConnectionHandler {
        self.inner.new_handler()
    }

    fn addresses_of_peer(&mut self, peer_id: &PeerId) -> Vec<Multiaddr> {
        self.inner.addresses_of_peer(peer_id)
    }

    fn on_swarm_event(&mut self, event: FromSwarm<Self::ConnectionHandler>) {
        self.inner.on_swarm_event(event)
    }

    fn on_connection_handler_event(
        &mut self,
        peer_id: PeerId,
        connection_id: ConnectionId,
        event: <<Self::ConnectionHandler as IntoConnectionHandler>::Handler as ConnectionHandler>::OutEvent,
    ) {
        self.inner
            .on_connection_handler_event(peer_id, connection_id, event)
    }

    fn poll(
        &mut self,
        cx: &mut Context<'_>,
        params: &mut impl PollParameters,
    ) -> Poll<NetworkBehaviourAction<Self::OutEvent, Self::ConnectionHandler>> {
        while let Poll::Ready(ev) = self.inner.poll(cx, params) {
            match ev {
                NetworkBehaviourAction::GenerateEvent(ev) => match ev {
                    BitswapEvent::Progress(_, _) => {}
                    BitswapEvent::Complete(id, result) => {
                        stats::CONTENT_RESOLVE_RUNNING.dec();
                        let out = Event::Complete(id, result);
                        return Poll::Ready(NetworkBehaviourAction::GenerateEvent(out));
                    }
                },
                other => {
                    return Poll::Ready(other.map_out(|_| unreachable!("already handled")));
                }
            }
        }

        Poll::Pending
    }
}
