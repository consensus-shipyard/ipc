// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT
use ipc_observability::{
    impl_traceable, impl_traceables, register_metrics, Recordable, TraceLevel, Traceable,
};
use lazy_static::lazy_static;
use libipld::cid::Cid;
use libp2p::gossipsub::TopicHash;
use libp2p::PeerId;
use prometheus::{
    register_histogram, register_int_counter, register_int_gauge, Histogram, IntCounter, IntGauge,
    Registry,
};
use std::error::Error;
use std::time::Duration;

register_metrics! {
    PING_RTT: Histogram =
        register_histogram!("ping_rtt", "Ping roundtrip time");

    PING_TIMEOUT: IntCounter =
        register_int_counter!("ping_timeouts", "Number of timed out pings");

    PING_FAILURE: IntCounter =
        register_int_counter!("ping_failure", "Number of failed pings");

    PING_SUCCESS: IntCounter =
        register_int_counter!("ping_success", "Number of successful pings");

    IDENTIFY_FAILURE: IntCounter =
        register_int_counter!("identify_failure", "Number of Identify errors");

    IDENTIFY_RECEIVED: IntCounter =
        register_int_counter!("identify_received", "Number of Identify infos received");

    DISCOVERY_BACKGROUND_LOOKUP: IntCounter =
        register_int_counter!("discovery_background_lookup", "Number of background lookups started");

    DISCOVERY_CONNECTED_PEERS: IntGauge =
        register_int_gauge!("discovery_connected_peers", "Number of connections");

    MEMBERSHIP_SKIPPED_PEERS: IntCounter =
        register_int_counter!("membership_skipped_peers", "Number of providers skipped");

    MEMBERSHIP_ROUTABLE_PEERS: IntGauge =
        register_int_gauge!("membership_routable_peers", "Number of routable peers");

    MEMBERSHIP_PROVIDER_PEERS: IntGauge =
        register_int_gauge!("membership_provider_peers", "Number of unique providers");

    MEMBERSHIP_UNKNOWN_TOPIC: IntCounter =
        register_int_counter!("membership_unknown_topic", "Number of messages with unknown topic");

    MEMBERSHIP_INVALID_MESSAGE: IntCounter =
        register_int_counter!("membership_invalid_message", "Number of invalid messages received");

    MEMBERSHIP_PUBLISH_SUCCESS: IntCounter =
        register_int_counter!("membership_publish_total", "Number of published messages");

    MEMBERSHIP_PUBLISH_FAILURE: IntCounter =
        register_int_counter!("membership_publish_failure", "Number of failed publish attempts");

    CONTENT_RESOLVE_RUNNING: IntGauge =
        register_int_gauge!("content_resolve_running", "Number of currently running content resolutions");

    CONTENT_RESOLVE_NO_PEERS: IntCounter =
        register_int_counter!("content_resolve_no_peers", "Number of resolutions with no known peers");

    CONTENT_RESOLVE_SUCCESS: IntCounter =
        register_int_counter!("content_resolve_success", "Number of successful resolutions");

    CONTENT_RESOLVE_FAILURE: IntCounter =
        register_int_counter!("content_resolve_failure", "Number of failed resolutions");

    CONTENT_RESOLVE_FALLBACK: IntCounter =
        register_int_counter!("content_resolve_fallback", "Number of resolutions that fall back on secondary peers");

    CONTENT_RESOLVE_PEERS: Histogram =
        register_histogram!("content_resolve_peers", "Number of peers found for resolution from a subnet");

    CONTENT_CONNECTED_PEERS: Histogram =
        register_histogram!("content_connected_peers", "Number of connected peers in a resolution");

    CONTENT_RATE_LIMITED: IntCounter =
        register_int_counter!("content_rate_limited", "Number of rate limited requests");
}

impl_traceables!(TraceLevel::Info, "Ping", PingEvent);
impl_traceables!(TraceLevel::Warn, "Ping", PingFailureEvent);
impl_traceables!(TraceLevel::Info, "Identify", IdentifyEvent);
impl_traceables!(TraceLevel::Warn, "Identify", IdentifyFailureEvent);
impl_traceables!(TraceLevel::Info, "Discovery", DiscoveryEvent);
impl_traceables!(TraceLevel::Info, "Membership", MembershipEvent);
impl_traceables!(TraceLevel::Warn, "Membership", MembershipFailureEvent);
impl_traceables!(TraceLevel::Info, "Content", ResolveEvent);
impl_traceables!(TraceLevel::Warn, "Content", ResolveFailureEvent);

#[derive(Debug)]
#[allow(dead_code)]
pub enum PingEvent {
    Success(PeerId, Duration),
}

impl Recordable for PingEvent {
    fn record_metrics(&self) {
        match self {
            Self::Success(_, rtt) => {
                PING_SUCCESS.inc();
                PING_RTT.observe(rtt.as_millis() as f64);
            }
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum PingFailureEvent {
    Timeout(PeerId),
    Failure(PeerId, Box<dyn Error>),
}

impl Recordable for PingFailureEvent {
    fn record_metrics(&self) {
        match self {
            Self::Failure(_, _) => PING_FAILURE.inc(),
            Self::Timeout(_) => PING_TIMEOUT.inc(),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum IdentifyEvent {
    Received(PeerId),
}

impl Recordable for IdentifyEvent {
    fn record_metrics(&self) {
        match self {
            Self::Received(_) => IDENTIFY_RECEIVED.inc(),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum IdentifyFailureEvent {
    Failure(PeerId, Box<dyn Error>),
}

impl Recordable for IdentifyFailureEvent {
    fn record_metrics(&self) {
        match self {
            Self::Failure(_, _) => IDENTIFY_FAILURE.inc(),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum DiscoveryEvent {
    BackgroundLookup(PeerId),
    ConnectionEstablished(PeerId),
    ConnectionClosed(PeerId),
}

impl Recordable for DiscoveryEvent {
    fn record_metrics(&self) {
        match self {
            Self::BackgroundLookup(_) => DISCOVERY_BACKGROUND_LOOKUP.inc(),
            Self::ConnectionEstablished(_) => DISCOVERY_CONNECTED_PEERS.inc(),
            Self::ConnectionClosed(_) => DISCOVERY_CONNECTED_PEERS.dec(),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum MembershipEvent {
    Added(PeerId),
    Removed(PeerId),
    Skipped(PeerId),
    PublishSuccess,
    RoutablePeers(i64),
}

impl Recordable for MembershipEvent {
    fn record_metrics(&self) {
        match self {
            Self::Added(_) => MEMBERSHIP_PROVIDER_PEERS.inc(),
            Self::Removed(_) => MEMBERSHIP_PROVIDER_PEERS.dec(),
            Self::Skipped(_) => MEMBERSHIP_SKIPPED_PEERS.inc(),
            Self::PublishSuccess => MEMBERSHIP_PUBLISH_SUCCESS.inc(),
            Self::RoutablePeers(num_routable) => MEMBERSHIP_ROUTABLE_PEERS.set(*num_routable),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum MembershipFailureEvent {
    PublishFailure(Box<dyn Error>),
    GossipInvalidProviderRecord(Option<PeerId>, Box<dyn Error>),
    GossipInvalidVoteRecord(Option<PeerId>, Box<dyn Error>),
    GossipUnknownTopic(Option<PeerId>, TopicHash),
}

impl Recordable for MembershipFailureEvent {
    fn record_metrics(&self) {
        match self {
            Self::PublishFailure(_) => MEMBERSHIP_PUBLISH_FAILURE.inc(),
            Self::GossipInvalidProviderRecord(_, _) => MEMBERSHIP_INVALID_MESSAGE.inc(),
            Self::GossipInvalidVoteRecord(_, _) => MEMBERSHIP_INVALID_MESSAGE.inc(),
            Self::GossipUnknownTopic(_, _) => MEMBERSHIP_UNKNOWN_TOPIC.inc(),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ResolveEvent {
    Started(Cid),
    Success(Cid),
    Completed,
    Peers(usize),
    NoPeers,
    ConnectedPeers(usize),
}

impl Recordable for ResolveEvent {
    fn record_metrics(&self) {
        match self {
            Self::Started(_) => CONTENT_RESOLVE_RUNNING.inc(),
            Self::Success(_) => CONTENT_RESOLVE_SUCCESS.inc(),
            Self::Completed => CONTENT_RESOLVE_RUNNING.dec(),
            Self::Peers(num) => CONTENT_RESOLVE_PEERS.observe(*num as f64),
            Self::NoPeers => CONTENT_RESOLVE_NO_PEERS.inc(),
            Self::ConnectedPeers(num) => CONTENT_CONNECTED_PEERS.observe(*num as f64),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum ResolveFailureEvent {
    Failure(Cid),
    Fallback(Cid),
}

impl Recordable for ResolveFailureEvent {
    fn record_metrics(&self) {
        match self {
            Self::Failure(_) => CONTENT_RESOLVE_FAILURE.inc(),
            Self::Fallback(_) => CONTENT_RESOLVE_FALLBACK.inc(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ipc_observability::emit;

    #[test]
    fn test_metrics() {
        let registry = Registry::new();
        register_metrics(&registry).unwrap();
    }

    #[test]
    fn test_emit() {
        let peer_id = PeerId::random();
        let rtt: Duration = Duration::from_millis(500);
        let error = Box::new(std::fmt::Error);
        let cid = Cid::default();

        emit(PingEvent::Success(peer_id, rtt));
        emit(PingFailureEvent::Timeout(peer_id));
        emit(PingFailureEvent::Failure(peer_id, error.clone()));
        emit(IdentifyEvent::Received(peer_id));
        emit(IdentifyFailureEvent::Failure(peer_id, error.clone()));
        emit(DiscoveryEvent::BackgroundLookup(peer_id));
        emit(DiscoveryEvent::ConnectionEstablished(peer_id));
        emit(DiscoveryEvent::ConnectionClosed(peer_id));
        emit(MembershipEvent::Added(peer_id));
        emit(MembershipEvent::Removed(peer_id));
        emit(MembershipEvent::Skipped(peer_id));
        emit(MembershipEvent::PublishSuccess);
        emit(MembershipEvent::RoutablePeers(Default::default()));
        emit(MembershipFailureEvent::PublishFailure(error.clone()));
        emit(MembershipFailureEvent::GossipInvalidProviderRecord(
            Some(peer_id),
            error.clone(),
        ));
        emit(MembershipFailureEvent::GossipInvalidVoteRecord(
            Some(peer_id),
            error.clone(),
        ));
        emit(MembershipFailureEvent::GossipUnknownTopic(
            Some(peer_id),
            TopicHash::from_raw("topic".to_string()),
        ));
        emit(ResolveEvent::Started(cid));
        emit(ResolveEvent::Success(cid));
        emit(ResolveEvent::Completed);
        emit(ResolveEvent::Peers(Default::default()));
        emit(ResolveEvent::NoPeers);
        emit(ResolveEvent::ConnectedPeers(Default::default()));
        emit(ResolveFailureEvent::Failure(cid));
        emit(ResolveFailureEvent::Fallback(cid));
    }
}
