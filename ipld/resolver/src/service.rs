// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::time::Duration;

use anyhow::anyhow;
use bloom::{BloomFilter, ASMS};
use ipc_api::subnet_id::SubnetID;
use iroh::blobs::Hash;
use iroh::client::blobs::ReadAtLen;
use iroh::client::Iroh;
use iroh::net::NodeAddr;
use libipld::store::StoreParams;
use libipld::Cid;
use libp2p::futures::StreamExt;
use libp2p::swarm::SwarmEvent;
use libp2p::{
    core::{muxing::StreamMuxerBox, transport::Boxed},
    identity::Keypair,
    noise, yamux, Multiaddr, PeerId, Swarm, Transport,
};
use libp2p::{identify, ping};
use libp2p_bitswap::{BitswapResponse, BitswapStore};
use libp2p_mplex::MplexConfig;
use log::{debug, error, info, trace, warn};
use maybe_iroh::MaybeIroh;
use prometheus::Registry;
use rand::seq::SliceRandom;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::oneshot::Sender;

use crate::behaviour::{
    content, discovery, membership, Behaviour, BehaviourEvent, ConfigError, ContentConfig,
    DiscoveryConfig, MembershipConfig, NetworkConfig,
};
use crate::client::Client;
use crate::stats;
use crate::vote_record::{SignedVoteRecord, VoteRecord};

/// Result of attempting to resolve a CID.
pub type ResolveResult = anyhow::Result<()>;

/// Result of attempting to resolve a read request.
pub type ResolveReadRequestResult = anyhow::Result<bytes::Bytes>;

/// Channel to complete the results with.
type ResponseChannel = Sender<ResolveResult>;

/// Channel to complete the read request with.
type ReadRequestResponseChannel = Sender<anyhow::Result<bytes::Bytes>>;

/// State of a query. The fallback peers can be used
/// if the current attempt fails.
struct Query {
    cid: Cid,
    subnet_id: SubnetID,
    fallback_peer_ids: Vec<PeerId>,
    response_channel: ResponseChannel,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct QueryId(pub u64);

impl std::fmt::Display for QueryId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Keeps track of where to send query responses to.
type QueryMap = HashMap<QueryId, Query>;

/// Error returned when we tried to get a CID from a subnet for
/// which we currently have no peers to contact
#[derive(thiserror::Error, Debug)]
#[error("No known peers for subnet {0}")]
pub struct NoKnownPeers(SubnetID);

#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// The address where we will listen to incoming connections.
    pub listen_addr: Multiaddr,
    /// A list of known external addresses this node is reachable on.
    pub external_addresses: Vec<Multiaddr>,
    /// Maximum number of incoming connections.
    pub max_incoming: u32,
    /// Expected number of peers, for sizing the Bloom filter.
    pub expected_peer_count: u32,
    /// Maximum number of peers to send Bitswap requests to in a single attempt.
    pub max_peers_per_query: u32,
    /// Maximum number of events in the push-based broadcast channel before a slow
    /// consumer gets an error because it's falling behind.
    pub event_buffer_capacity: u32,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub network: NetworkConfig,
    pub discovery: DiscoveryConfig,
    pub membership: MembershipConfig,
    pub connection: ConnectionConfig,
    pub content: ContentConfig,
    pub iroh_addr: Option<String>,
}

/// Internal requests to enqueue to the [`Service`]
pub(crate) enum Request<V> {
    SetProvidedSubnets(Vec<SubnetID>),
    AddProvidedSubnet(SubnetID),
    RemoveProvidedSubnet(SubnetID),
    // (publish vote is a broadcast)
    PublishVote(Box<SignedVoteRecord<V>>),
    PublishPreemptive(SubnetID, Vec<u8>),
    PinSubnet(SubnetID),
    UnpinSubnet(SubnetID),
    Resolve(Cid, SubnetID, ResponseChannel),
    ResolveIroh(Hash, NodeAddr, ResponseChannel),
    ResolveReadRequest(Hash, ReadRequestResponseChannel),
    RateLimitUsed(PeerId, usize),
    UpdateRateLimit(u32),
}

/// Events that arise from the subnets, pushed to the clients,
/// not part of a request-response action.
#[derive(Clone, Debug)]
pub enum Event<V> {
    /// Received a vote about in a subnet about a CID.
    ReceivedVote(Box<VoteRecord<V>>),
    /// Received raw pre-emptive data published to a pinned subnet.
    ReceivedPreemptive(SubnetID, Vec<u8>),
}

/// The `Service` handles P2P communication to resolve IPLD content by wrapping and driving a number of `libp2p` behaviours.
pub struct Service<P, V>
where
    P: StoreParams,
    V: Serialize + DeserializeOwned + Send + 'static,
{
    peer_id: PeerId,
    listen_addr: Multiaddr,
    swarm: Swarm<Behaviour<P, V>>,
    /// To match finished queries to response channels.
    queries: QueryMap,
    /// For receiving requests from the clients and self.
    request_rx: mpsc::UnboundedReceiver<Request<V>>,
    /// For creating new clients and sending messages to self.
    request_tx: mpsc::UnboundedSender<Request<V>>,
    /// For broadcasting events to all clients.
    event_tx: broadcast::Sender<Event<V>>,
    /// To avoid looking up the same peer over and over.
    background_lookup_filter: BloomFilter,
    /// To limit the number of peers contacted in a Bitswap resolution attempt.
    max_peers_per_query: usize,
    /// Iroh client
    iroh: MaybeIroh,
}

impl<P, V> Service<P, V>
where
    P: StoreParams,
    V: Serialize + DeserializeOwned + Clone + Send + 'static,
{
    /// Build a [`Service`] and a [`Client`] with the default `tokio` transport.
    pub async fn new<S>(config: Config, store: S) -> Result<Self, ConfigError>
    where
        S: BitswapStore<Params = P>,
    {
        Self::new_with_transport(config, store, build_transport).await
    }

    /// Build a [`Service`] and a [`Client`] by passing in a transport factory function.
    ///
    /// The main goal is to be facilitate testing with a [`MemoryTransport`].
    pub async fn new_with_transport<S, F>(
        config: Config,
        store: S,
        transport: F,
    ) -> Result<Self, ConfigError>
    where
        S: BitswapStore<Params = P>,
        F: FnOnce(Keypair) -> Boxed<(PeerId, StreamMuxerBox)>,
    {
        let peer_id = config.network.local_peer_id();
        let transport = transport(config.network.local_key.clone());
        let behaviour = Behaviour::new(
            config.network,
            config.discovery,
            config.membership,
            config.content,
            store,
        )?;

        // NOTE: Hardcoded values from Forest. Will leave them as is until we know we need to change.

        // TODO: Where this these go? Used to be `SwarmBuilder::connection_limits`
        // let _limits = ConnectionLimits::default()
        //     .with_max_pending_incoming(Some(10))
        //     .with_max_pending_outgoing(Some(30))
        //     .with_max_established_incoming(Some(config.connection.max_incoming))
        //     .with_max_established_outgoing(None) // Allow bitswap to connect to subnets we did not anticipate when we started.
        //     .with_max_established_per_peer(Some(5));

        //.connection_limits(limits)
        //.notify_handler_buffer_size(std::num::NonZeroUsize::new(20).expect("Not zero"))
        //.connection_event_buffer_size(64)
        //.build();

        let mut swarm = Swarm::new(
            transport,
            behaviour,
            peer_id,
            libp2p::swarm::Config::with_tokio_executor(),
        );

        for addr in config.connection.external_addresses {
            swarm.add_external_address(addr)
        }

        let (request_tx, request_rx) = mpsc::unbounded_channel();
        let (event_tx, _) = broadcast::channel(config.connection.event_buffer_capacity as usize);

        let service = Self {
            peer_id,
            listen_addr: config.connection.listen_addr,
            swarm,
            queries: Default::default(),
            request_rx,
            request_tx,
            event_tx,
            background_lookup_filter: BloomFilter::with_rate(
                0.1,
                config.connection.expected_peer_count,
            ),
            max_peers_per_query: config.connection.max_peers_per_query as usize,
            iroh: MaybeIroh::maybe_addr(config.iroh_addr),
        };

        Ok(service)
    }

    /// Create a new [`Client`] instance bound to this `Service`.
    ///
    /// The [`Client`] is geared towards request-response interactions,
    /// while the `Receiver` returned by `subscribe` is used for events
    /// which weren't initiated by the `Client`.
    pub fn client(&self) -> Client<V> {
        Client::new(self.request_tx.clone())
    }

    /// Create a new [`broadcast::Receiver`] instance bound to this `Service`,
    /// which will be notified upon each event coming from any of the subnets
    /// the `Service` is subscribed to.
    ///
    /// The consumers are expected to process events quick enough to be within
    /// the configured capacity of the broadcast channel, or otherwise be able
    /// to deal with message loss if they fall behind.
    ///
    /// # Notes
    ///
    /// This is not part of the [`Client`] because `Receiver::recv` takes
    /// a mutable reference and it would prevent the [`Client`] being used
    /// for anything else.
    ///
    /// One alternative design would be to accept an interface similar to
    /// [`BitswapStore`] that we can pass events to. In that case we would
    /// have to create an internal event queue to stand in front of it,
    /// and because these events arrive from the outside, it would still
    /// have to have limited capacity.
    ///
    /// Because the channel has limited capacity, we have to take care not
    /// to use it for signaling critical events that we want to await upon.
    /// For example if we used this to signal the readiness of bootstrapping,
    /// we should make sure we have not yet subscribed to external events
    /// which could drown it out.
    ///
    /// One way to achieve this is for the consumer of the events to redistribute
    /// them into priorities event queues, some bounded, some unbounded.
    pub fn subscribe(&self) -> broadcast::Receiver<Event<V>> {
        self.event_tx.subscribe()
    }

    /// Register Prometheus metrics.
    pub fn register_metrics(&mut self, registry: &Registry) -> anyhow::Result<()> {
        self.content_mut().register_metrics(registry)?;
        stats::register_metrics(registry)?;
        Ok(())
    }

    /// Start the swarm listening for incoming connections and drive the events forward.
    pub async fn run(mut self) -> anyhow::Result<()> {
        // Start the swarm.
        info!("running service on {}", self.listen_addr);
        Swarm::listen_on(&mut self.swarm, self.listen_addr.clone())?;

        loop {
            select! {
                swarm_event = self.swarm.next() => match swarm_event {
                    // Events raised by our behaviours.
                    Some(SwarmEvent::Behaviour(event)) => {
                        self.handle_behaviour_event(event)
                    },
                    // Connection events are handled by the behaviours, passed directly from the Swarm.
                    Some(_) => { },
                    // The connection is closed.
                    None => { break; },
                },
                request = self.request_rx.recv() => match request {
                    // A Client sent us a request.
                    Some(req) => self.handle_request(req),
                    // This shouldn't happen because the service has a copy of the sender.
                    // All Client instances have been dropped.
                    None => { break; }
                }
            }
        }
        Ok(())
    }

    /// Handle events that the [`NetworkBehaviour`] macro generated for our [`Behaviour`], one for each field.
    fn handle_behaviour_event(&mut self, event: BehaviourEvent<P, V>) {
        match event {
            BehaviourEvent::Ping(e) => self.handle_ping_event(e),
            BehaviourEvent::Identify(e) => self.handle_identify_event(e),
            BehaviourEvent::Discovery(e) => self.handle_discovery_event(e),
            BehaviourEvent::Membership(e) => self.handle_membership_event(e),
            BehaviourEvent::Content(e) => self.handle_content_event(e),
        }
    }

    // Copied from Forest.
    fn handle_ping_event(&mut self, event: ping::Event) {
        let peer_id = event.peer.to_base58();
        match event.result {
            Ok(rtt) => {
                stats::PING_SUCCESS.inc();
                stats::PING_RTT.observe(rtt.as_millis() as f64);
                trace!(
                    "PingSuccess::Ping rtt to {} from {} is {} ms",
                    peer_id,
                    self.peer_id,
                    rtt.as_millis()
                );
            }
            Err(ping::Failure::Timeout) => {
                stats::PING_TIMEOUT.inc();
                debug!("PingFailure::Timeout from {peer_id} to {}", self.peer_id);
            }
            Err(ping::Failure::Other { error }) => {
                stats::PING_FAILURE.inc();
                warn!(
                    "PingFailure::Other from {peer_id} to {}: {error}",
                    self.peer_id
                );
            }
            Err(ping::Failure::Unsupported) => {
                warn!("Should ban peer {peer_id} due to protocol error");
                // TODO: How do we ban peers in 0.53 ?
                // see https://github.com/libp2p/rust-libp2p/pull/3590/files
                // self.swarm.ban_peer_id(event.peer);
            }
        }
    }

    fn handle_identify_event(&mut self, event: identify::Event) {
        if let identify::Event::Error { peer_id, error } = event {
            stats::IDENTIFY_FAILURE.inc();
            warn!("Error identifying {peer_id}: {error}")
        } else if let identify::Event::Received { peer_id, info } = event {
            stats::IDENTIFY_RECEIVED.inc();
            debug!("protocols supported by {peer_id}: {:?}", info.protocols);
            debug!("adding identified address of {peer_id} to {}", self.peer_id);
            self.discovery_mut().add_identified(&peer_id, info);
        }
    }

    fn handle_discovery_event(&mut self, event: discovery::Event) {
        match event {
            discovery::Event::Added(peer_id) => {
                debug!("adding routable peer {peer_id} to {}", self.peer_id);
                self.membership_mut().set_routable(peer_id)
            }
            discovery::Event::Removed(peer_id) => {
                debug!("removing unroutable peer {peer_id} from {}", self.peer_id);
                self.membership_mut().set_unroutable(peer_id)
            }
        }
    }

    fn handle_membership_event(&mut self, event: membership::Event<V>) {
        match event {
            membership::Event::Skipped(peer_id) => {
                debug!("skipped adding provider {peer_id} to {}", self.peer_id);
                // Don't repeatedly look up peers we can't add to the routing table.
                if self.background_lookup_filter.insert(&peer_id) {
                    debug!(
                        "triggering background lookup of {peer_id} on {}",
                        self.peer_id
                    );
                    self.discovery_mut().background_lookup(peer_id)
                }
            }
            membership::Event::Updated(p, delta) => {
                debug!("peer updated: {} with {:?}", p, delta.added);
            }
            membership::Event::Removed(p) => {
                debug!("removed peer {}", p);
            }
            membership::Event::ReceivedVote(vote) => {
                let event = Event::ReceivedVote(vote);
                if self.event_tx.send(event).is_err() {
                    debug!("dropped received vote because there are no subscribers")
                }
            }
            membership::Event::ReceivedPreemptive(subnet_id, data) => {
                let event = Event::ReceivedPreemptive(subnet_id, data);
                if self.event_tx.send(event).is_err() {
                    debug!("dropped received preemptive data because there are no subscribers")
                }
            }
        }
    }

    /// Handle Bitswap lookup result.
    fn handle_content_event(&mut self, event: content::Event) {
        match event {
            content::Event::Complete(query_id, result) => {
                if let Some(query) = self.queries.remove(&query_id) {
                    self.resolve_query(query, result);
                } else {
                    warn!("query ID not found");
                }
            }
            content::Event::BitswapForward {
                peer_id,
                response_rx,
                response_tx,
            } => {
                let request_tx = self.request_tx.clone();
                tokio::task::spawn(async move {
                    if let Ok(res) = response_rx.await {
                        if let BitswapResponse::Block(bz) = &res {
                            let _ = request_tx.send(Request::RateLimitUsed(peer_id, bz.len()));
                        }
                        // Forward, if the listener is still open.
                        let _ = response_tx.send(res);
                    }
                });
            }
        }
    }

    /// Handle an internal request coming from a [`Client`].
    fn handle_request(&mut self, request: Request<V>) {
        match request {
            Request::SetProvidedSubnets(ids) => {
                if let Err(e) = self.membership_mut().set_provided_subnets(ids) {
                    warn!("failed to publish set provided subnets: {e}")
                }
            }
            Request::AddProvidedSubnet(id) => {
                if let Err(e) = self.membership_mut().add_provided_subnet(id) {
                    warn!("failed to publish added provided subnet: {e}")
                }
            }
            Request::RemoveProvidedSubnet(id) => {
                if let Err(e) = self.membership_mut().remove_provided_subnet(id) {
                    warn!("failed to publish removed provided subnet: {e}")
                }
            }
            Request::PublishVote(vote) => {
                let listen_addr = self.listen_addr.clone();
                eprintln!("=====>>>> publishing vote in node: {:?}", listen_addr);

                if let Err(e) = self.membership_mut().publish_vote(*vote) {
                    warn!("failed to publish vote: {e}")
                }
            }
            Request::PublishPreemptive(subnet_id, data) => {
                if let Err(e) = self.membership_mut().publish_preemptive(subnet_id, data) {
                    warn!("failed to publish pre-emptive data: {e}")
                }
            }
            Request::PinSubnet(id) => {
                if let Err(e) = self.membership_mut().pin_subnet(id) {
                    warn!("error pinning subnet: {e}")
                }
            }
            Request::UnpinSubnet(id) => {
                if let Err(e) = self.membership_mut().unpin_subnet(&id) {
                    warn!("error unpinning subnet: {e}")
                }
            }
            Request::Resolve(cid, subnet_id, response_channel) => {
                self.start_query(cid, subnet_id, response_channel)
            }
            Request::ResolveIroh(hash, node_addr, response_channel) => {
                self.start_iroh_query(hash, node_addr, response_channel)
            }
            Request::ResolveReadRequest(hash, response_channel) => {
                eprintln!(
                    "====>>>>> (handle_request) starting resolve read request: {:?}",
                    hash
                );
                self.start_iroh_read_request(hash, response_channel)
            }
            Request::RateLimitUsed(peer_id, bytes) => {
                self.content_mut().rate_limit_used(peer_id, bytes)
            }
            Request::UpdateRateLimit(bytes) => self.content_mut().update_rate_limit(bytes),
        }
    }

    /// Start a CID resolution.
    fn start_query(&mut self, cid: Cid, subnet_id: SubnetID, response_channel: ResponseChannel) {
        let mut peers = self.membership_mut().providers_of_subnet(&subnet_id);

        stats::CONTENT_RESOLVE_PEERS.observe(peers.len() as f64);

        if peers.is_empty() {
            stats::CONTENT_RESOLVE_NO_PEERS.inc();
            send_resolve_result(response_channel, Err(anyhow!(NoKnownPeers(subnet_id))));
        } else {
            // Connect to them in a random order, so as not to overwhelm any specific peer.
            peers.shuffle(&mut rand::thread_rng());

            // Prioritize peers we already have an established connection with.
            let (connected, known) = peers
                .into_iter()
                .partition::<Vec<_>, _>(|id| self.swarm.is_connected(id));

            stats::CONTENT_CONNECTED_PEERS.observe(connected.len() as f64);

            let peers = [connected, known].into_iter().flatten().collect();
            let (peers, fallback) = self.split_peers_for_query(peers);

            let query = Query {
                cid,
                subnet_id,
                response_channel,
                fallback_peer_ids: fallback,
            };

            let query_id = self.content_mut().resolve(cid, peers);

            self.queries.insert(query_id, query);
        }
    }

    /// Start a CID resolution using iroh.
    fn start_iroh_query(
        &mut self,
        hash: Hash,
        node_addr: NodeAddr,
        response_channel: ResponseChannel,
    ) {
        let listen_addr = self.listen_addr.clone();
        eprintln!("===>>>> (start_iroh_query) in node: {:?}", listen_addr);

        let mut iroh = self.iroh.clone();
        tokio::spawn(async move {
            match iroh.client().await {
                Ok(client) => {
                    let res = download_blob(client, hash, node_addr).await;
                    match res {
                        Ok(_) => send_resolve_result(response_channel, Ok(())),
                        Err(e) => send_resolve_result(response_channel, Err(anyhow!(e))),
                    }
                }
                Err(e) => warn!(
                    "cannot resolve {}; failed to create iroh client ({})",
                    hash, e
                ),
            }
        });
    }

    /// Start a read request resolution using iorh.
    fn start_iroh_read_request(
        &mut self,
        hash: Hash,
        response_channel: ReadRequestResponseChannel,
    ) {
        let mut iroh = self.iroh.clone();
        tokio::spawn(async move {
            eprintln!(
                "====>>>>> (start_iroh_read_request) starting resolve read request: {:?}",
                hash
            );
            match iroh.client().await {
                Ok(client) => {
                    let offset = 0;
                    let len = 1024 * 1024;

                    // put a dummy blob in iroh corresponding to the blob hash
                    let data_hex = r#"a46022bf66ff011abaedd2556bd2e0c21d2367cb430692ea6a1e84ec9d4639c2bb79c46629e9c5c76759b2aadf906df8ad45b7da458290be731e043b9dacbefcb931ba4da810f0673ec06f8d9dc05e6dd20e232765e78e301c7ffa5c0ebf4579c8d20096cbfe6f80044ffff274343ad7735eae3e12e2322e9cc8473f80e50c2893b284e619fcee70ad59a735520dbb1263be54c4cffa9bb914284c835de4550ba4c40cc6f733406aebc281aaa42e4ff44db953647d6d3737493fe9f7936a8dbf91fd381be144201ea5d9b376789279b2c5c6357b29d814dcbed9f8d8f4dd4802b9206a391645b8c712943ca2ae8a78611570bd96fe0175d524dfe75a373639435d7ea8125f53605b774ab96708b60a8e8805b795c92d8b49f238fa3e0d714f0db15f1c02abe081f25a544a9127cbf383dc52c8a88752fc0dc2c03692e22e8c0f6e1c5adcf4de18fa3a821ddd151a7e6707e1f0ea559105f87eb7ea3d0dd62815ef0b21a0d5f8bdf4abfcd5a16764a9120220db8c2dbe207892cb2b20a404f21b901b5272b5af27a63deca7641a160bd8c7903a40131d1ed84aa6bde91a55f156042e0cf2f0d738095c741e8913e5cdf8d40cb346f4e6793ac8502a7aa0873ed96c0929ee1a0fab035a5c12c76e92bfe844fd96f92cbe104c41c8ffe0e1bdb8878f7158f64ba44ac08178411ca46d0557a6e7cc1ade13f69bd6751bd3023558e0ca33f13af5afa70f3d3cce094bc51510475f1e6f4a5b0cb9f1120229ceacf220f919024410f3c06c607a823aa47c10879a655b9d3a96433dc0ad64e0d8c80fbf7a6e6e5adfdf7e8e4b68335325b5a57c735dbf72b256bf6dd89816dcded29eabdf7f6379f354b81cab2464d43462b99e0a6093d0b1368cfd1563c0c2ee9bef7106c0727ce204405044ae11926e534dda81a2e082c680a76bbbbccd92b44d44773b27cf1388359d73facae0545754155b5e63172088067bcb9d7779d6e8d49b331f0593800bcf4b96f33ccc2e68a9e7272472966c9c0285e9260e88a3698be07fae1f728bd0dc4103eb13cda16657dd6e8de6ef300467c5ed52f1409be7a0065fbe45d90a64f15d44980b91ce40155430a3eadeced2653ad089656f24e1a5213f25cef0a43f6ee829f22c3213c16db8f71e37923617a79661ece1c305390c81d696aa1788f5182d9ef8cf0d1ef0d4da223d0934a53f16af46af75e145675d0cf6b4f49d414d0c916cd48a4fa87d7e22fd1131c21ab521ae245b819c4ea2cf7b6539a49877c5a87894ba49744723657d996413c0a77029b401397461cd100f715fa299ec6200cb7c17898d8e80e52c7ab5555dd6858592c6d7dc7f2ae289cc5e266f592a389e3d87ece818592bc3096e18afb7e95cd0520542af073dc49e817ffe778bdbdcf71c52ecb7e0bc8d9624fa9dd833fcb19c7c1467029b9165bfa1af5bba"#;
                    let data = hex::decode(data_hex).unwrap();
                    client.blobs().add_bytes(data).await.unwrap();

                    // -----

                    let res = read_blob(client, hash, offset, len).await;
                    match res {
                        Ok(bytes) => send_read_request_result(response_channel, Ok(bytes)),
                        Err(e) => send_read_request_result(response_channel, Err(anyhow!(e))),
                    }
                }
                Err(e) => warn!(
                    "cannot resolve read request {}; failed to create iroh client ({})",
                    hash, e
                ),
            }
        });
    }

    /// Handle the results from a resolve attempt. If it succeeded, notify the
    /// listener. Otherwise if we have fallback peers to try, start another
    /// query and send the result to them. By default these are the peers
    /// we know support the subnet, but weren't connected to when the we
    /// first attempted the resolution.
    fn resolve_query(&mut self, mut query: Query, result: ResolveResult) {
        match result {
            Ok(_) => {
                stats::CONTENT_RESOLVE_SUCCESS.inc();
                send_resolve_result(query.response_channel, result)
            }
            Err(_) if query.fallback_peer_ids.is_empty() => {
                stats::CONTENT_RESOLVE_FAILURE.inc();
                send_resolve_result(query.response_channel, result)
            }
            Err(e) => {
                stats::CONTENT_RESOLVE_FALLBACK.inc();
                debug!(
                    "resolving {} from {} failed with {}, but there are {} fallback peers to try",
                    query.cid,
                    query.subnet_id,
                    e,
                    query.fallback_peer_ids.len()
                );

                // Try to resolve from the next batch of peers.
                let peers = std::mem::take(&mut query.fallback_peer_ids);
                let (peers, fallback) = self.split_peers_for_query(peers);
                let query_id = self.content_mut().resolve(query.cid, peers);

                // Leave the rest for later.
                query.fallback_peer_ids = fallback;

                self.queries.insert(query_id, query);
            }
        }
    }

    /// Split peers into a group we query now and a group we fall back on if the current batch fails.
    fn split_peers_for_query(&self, mut peers: Vec<PeerId>) -> (Vec<PeerId>, Vec<PeerId>) {
        let size = std::cmp::min(self.max_peers_per_query, peers.len());
        let fallback = peers.split_off(size);
        (peers, fallback)
    }

    // The following are helper functions because Rust Analyzer has trouble with recognising that `swarm.behaviour_mut()` is a legal call.

    fn discovery_mut(&mut self) -> &mut discovery::Behaviour {
        self.swarm.behaviour_mut().discovery_mut()
    }
    fn membership_mut(&mut self) -> &mut membership::Behaviour<V> {
        self.swarm.behaviour_mut().membership_mut()
    }
    fn content_mut(&mut self) -> &mut content::Behaviour<P> {
        self.swarm.behaviour_mut().content_mut()
    }
}

/// Respond to the sender of the query, if they are still listening.
fn send_resolve_result(tx: Sender<ResolveResult>, res: ResolveResult) {
    if tx.send(res).is_err() {
        error!("error sending resolve result; listener closed")
    }
}

fn send_read_request_result(
    tx: Sender<anyhow::Result<bytes::Bytes>>,
    res: anyhow::Result<bytes::Bytes>,
) {
    if tx.send(res).is_err() {
        error!("error sending read request result; listener closed")
    }
}

/// Builds the transport stack that libp2p will communicate over.
///
/// Based on the equivalent in Forest.
pub fn build_transport(local_key: Keypair) -> Boxed<(PeerId, StreamMuxerBox)> {
    let tcp_transport =
        || libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::new().nodelay(true));
    let transport = libp2p::dns::tokio::Transport::system(tcp_transport()).unwrap();
    let auth_config = noise::Config::new(&local_key).expect("Noise key generation failed");

    let mplex_config = {
        let mut mplex_config = MplexConfig::new();
        mplex_config.set_max_buffer_size(usize::MAX);

        // FIXME: Yamux will end up beaing deprecated.
        let yamux_config = yamux::Config::default();
        // yamux_config.set_window_update_mode(WindowUpdateMode::OnRead);
        libp2p::core::upgrade::SelectUpgrade::new(yamux_config, mplex_config)
    };

    transport
        .upgrade(libp2p::core::upgrade::Version::V1)
        .authenticate(auth_config)
        .multiplex(mplex_config)
        .timeout(Duration::from_secs(20))
        .boxed()
}

async fn download_blob(iroh: Iroh, hash: Hash, node_addr: NodeAddr) -> anyhow::Result<()> {
    // Use an explicit tag so we can keep track of it
    // TODO: this needs to be tagged with a "user id"
    let tag = iroh::blobs::Tag(format!("stored-{hash}").into());
    let res = iroh
        .blobs()
        .download_with_opts(
            hash,
            iroh::client::blobs::DownloadOptions {
                format: iroh::blobs::BlobFormat::Raw,
                nodes: vec![node_addr],
                tag: iroh::blobs::util::SetTagOption::Named(tag),
                mode: iroh::client::blobs::DownloadMode::Queued,
            },
        )
        .await?
        .await?;

    debug!("downloaded blob {}: {:?}", hash, res);

    // Delete the temporary tag (this might fail as not all nodes will have one).
    // TODO: this needs to be tagged with a "user id"
    let tag = iroh::blobs::Tag(format!("temp-{hash}").into());
    iroh.tags().delete(tag).await.ok();

    Ok(())
}

async fn read_blob(iroh: Iroh, hash: Hash, offset: u64, len: u64) -> anyhow::Result<bytes::Bytes> {
    eprintln!(
        "====>>>>> (read_blob) starting read blob: {:?} offset: {} len: {}",
        hash, offset, len
    );
    let len = ReadAtLen::AtMost(len);
    let res = iroh.blobs().read_at_to_bytes(hash, offset, len).await?;
    debug!("read blob {}: {:?}", hash, res);
    Ok(res)
}
