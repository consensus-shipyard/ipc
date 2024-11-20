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
use std::time::Duration;

register_metrics! {
    IPLD_RESOLVER_PING_RTT: Histogram =
        register_histogram!("ipld_resolver_ping_rtt", "Ping roundtrip time");

    IPLD_RESOLVER_PING_TIMEOUT: IntCounter =
        register_int_counter!("ipld_resolver_ping_timeouts", "Number of timed out pings");

    IPLD_RESOLVER_PING_FAILURE: IntCounter =
        register_int_counter!("ipld_resolver_ping_failure", "Number of failed pings");

    IPLD_RESOLVER_PING_SUCCESS: IntCounter =
        register_int_counter!("ipld_resolver_ping_success", "Number of successful pings");

    IPLD_RESOLVER_IDENTIFY_FAILURE: IntCounter =
        register_int_counter!("ipld_resolver_identify_failure", "Number of Identify errors");

    IPLD_RESOLVER_IDENTIFY_RECEIVED: IntCounter =
        register_int_counter!("ipld_resolver_identify_received", "Number of Identify infos received");

    IPLD_RESOLVER_DISCOVERY_BACKGROUND_LOOKUP: IntCounter =
        register_int_counter!("ipld_resolver_discovery_background_lookup", "Number of background lookups started");

    IPLD_RESOLVER_DISCOVERY_CONNECTED_PEERS: IntGauge =
        register_int_gauge!("ipld_resolver_discovery_connected_peers", "Number of connections");

    IPLD_RESOLVER_MEMBERSHIP_SKIPPED_PEERS: IntCounter =
        register_int_counter!("ipld_resolver_membership_skipped_peers", "Number of providers skipped");

    IPLD_RESOLVER_MEMBERSHIP_ROUTABLE_PEERS: IntGauge =
        register_int_gauge!("ipld_resolver_membership_routable_peers", "Number of routable peers");

    IPLD_RESOLVER_MEMBERSHIP_PROVIDER_PEERS: IntGauge =
        register_int_gauge!("ipld_resolver_membership_provider_peers", "Number of unique providers");

    IPLD_RESOLVER_MEMBERSHIP_UNKNOWN_TOPIC: IntCounter =
        register_int_counter!("ipld_resolver_membership_unknown_topic", "Number of messages with unknown topic");

    IPLD_RESOLVER_MEMBERSHIP_INVALID_MESSAGE: IntCounter =
        register_int_counter!("ipld_resolver_membership_invalid_message", "Number of invalid messages received");

    IPLD_RESOLVER_MEMBERSHIP_PUBLISH_SUCCESS: IntCounter =
        register_int_counter!("ipld_resolver_membership_publish_total", "Number of published messages");

    IPLD_RESOLVER_MEMBERSHIP_PUBLISH_FAILURE: IntCounter =
        register_int_counter!("ipld_resolver_membership_publish_failure", "Number of failed publish attempts");

    IPLD_RESOLVER_CONTENT_RESOLVE_RUNNING: IntGauge =
        register_int_gauge!("ipld_resolver_content_resolve_running", "Number of currently running content resolutions");

    IPLD_RESOLVER_CONTENT_RESOLVE_NO_PEERS: IntCounter =
        register_int_counter!("ipld_resolver_content_resolve_no_peers", "Number of resolutions with no known peers");

    IPLD_RESOLVER_CONTENT_RESOLVE_SUCCESS: IntCounter =
        register_int_counter!("ipld_resolver_content_resolve_success", "Number of successful resolutions");

    IPLD_RESOLVER_CONTENT_RESOLVE_FAILURE: IntCounter =
        register_int_counter!("ipld_resolver_content_resolve_failure", "Number of failed resolutions");

    IPLD_RESOLVER_CONTENT_RESOLVE_FALLBACK: IntCounter =
        register_int_counter!("ipld_resolver_content_resolve_fallback", "Number of resolutions that fall back on secondary peers");

    IPLD_RESOLVER_CONTENT_RESOLVE_PEERS: Histogram =
        register_histogram!("ipld_resolver_content_resolve_peers", "Number of peers found for resolution from a subnet");

    IPLD_RESOLVER_CONTENT_CONNECTED_PEERS: Histogram =
        register_histogram!("ipld_resolver_content_connected_peers", "Number of connected peers in a resolution");

    IPLD_RESOLVER_CONTENT_RATE_LIMITED: IntCounter =
        register_int_counter!("ipld_resolver_content_rate_limited", "Number of rate limited requests");
}

impl_traceables!(TraceLevel::Info, "IPLD_Resolver/Ping", PingEvent);
impl_traceables!(TraceLevel::Warn, "IPLD_Resolver/Ping", PingFailureEvent);
impl_traceables!(TraceLevel::Info, "IPLD_Resolver/Identify", IdentifyEvent);
impl_traceables!(TraceLevel::Warn, "IPLD_Resolver/Identify", IdentifyFailureEvent);
impl_traceables!(TraceLevel::Info, "IPLD_Resolver/Discovery", DiscoveryEvent);
impl_traceables!(TraceLevel::Info, "IPLD_Resolver/Membership", MembershipEvent);
impl_traceables!(TraceLevel::Warn, "IPLD_Resolver/Membership", MembershipFailureEvent);
impl_traceables!(TraceLevel::Info, "IPLD_Resolver/Content", ResolveEvent);
impl_traceables!(TraceLevel::Warn, "IPLD_Resolver/Content", ResolveFailureEvent);

#[derive(Debug)]
#[allow(dead_code)]
pub enum PingEvent {
    Success(PeerId, Duration),
}

impl Recordable for PingEvent {
    fn record_metrics(&self) {
        match self {
            Self::Success(_, rtt) => {
                IPLD_RESOLVER_PING_SUCCESS.inc();
                IPLD_RESOLVER_PING_RTT.observe(rtt.as_millis() as f64);
            }
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum PingFailureEvent {
    Timeout(PeerId),
    Failure(PeerId, String),
}

impl Recordable for PingFailureEvent {
    fn record_metrics(&self) {
        match self {
            Self::Failure(_, _) => IPLD_RESOLVER_PING_FAILURE.inc(),
            Self::Timeout(_) => IPLD_RESOLVER_PING_TIMEOUT.inc(),
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
            Self::Received(_) => IPLD_RESOLVER_IDENTIFY_RECEIVED.inc(),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum IdentifyFailureEvent {
    Failure(PeerId, String),
}

impl Recordable for IdentifyFailureEvent {
    fn record_metrics(&self) {
        match self {
            Self::Failure(_, _) => IPLD_RESOLVER_IDENTIFY_FAILURE.inc(),
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
            Self::BackgroundLookup(_) => IPLD_RESOLVER_DISCOVERY_BACKGROUND_LOOKUP.inc(),
            Self::ConnectionEstablished(_) => IPLD_RESOLVER_DISCOVERY_CONNECTED_PEERS.inc(),
            Self::ConnectionClosed(_) => IPLD_RESOLVER_DISCOVERY_CONNECTED_PEERS.dec(),
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
            Self::Added(_) => IPLD_RESOLVER_MEMBERSHIP_PROVIDER_PEERS.inc(),
            Self::Removed(_) => IPLD_RESOLVER_MEMBERSHIP_PROVIDER_PEERS.dec(),
            Self::Skipped(_) => IPLD_RESOLVER_MEMBERSHIP_SKIPPED_PEERS.inc(),
            Self::PublishSuccess => IPLD_RESOLVER_MEMBERSHIP_PUBLISH_SUCCESS.inc(),
            Self::RoutablePeers(num_routable) => {
                IPLD_RESOLVER_MEMBERSHIP_ROUTABLE_PEERS.set(*num_routable)
            }
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum MembershipFailureEvent {
    PublishFailure(String),
    GossipInvalidProviderRecord(Option<PeerId>, String),
    GossipInvalidVoteRecord(Option<PeerId>, String),
    GossipUnknownTopic(Option<PeerId>, TopicHash),
}

impl Recordable for MembershipFailureEvent {
    fn record_metrics(&self) {
        match self {
            Self::PublishFailure(_) => IPLD_RESOLVER_MEMBERSHIP_PUBLISH_FAILURE.inc(),
            Self::GossipInvalidProviderRecord(_, _) => {
                IPLD_RESOLVER_MEMBERSHIP_INVALID_MESSAGE.inc()
            }
            Self::GossipInvalidVoteRecord(_, _) => IPLD_RESOLVER_MEMBERSHIP_INVALID_MESSAGE.inc(),
            Self::GossipUnknownTopic(_, _) => IPLD_RESOLVER_MEMBERSHIP_UNKNOWN_TOPIC.inc(),
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
            Self::Started(_) => IPLD_RESOLVER_CONTENT_RESOLVE_RUNNING.inc(),
            Self::Success(_) => IPLD_RESOLVER_CONTENT_RESOLVE_SUCCESS.inc(),
            Self::Completed => IPLD_RESOLVER_CONTENT_RESOLVE_RUNNING.dec(),
            Self::Peers(num) => IPLD_RESOLVER_CONTENT_RESOLVE_PEERS.observe(*num as f64),
            Self::NoPeers => IPLD_RESOLVER_CONTENT_RESOLVE_NO_PEERS.inc(),
            Self::ConnectedPeers(num) => IPLD_RESOLVER_CONTENT_CONNECTED_PEERS.observe(*num as f64),
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
            Self::Failure(_) => IPLD_RESOLVER_CONTENT_RESOLVE_FAILURE.inc(),
            Self::Fallback(_) => IPLD_RESOLVER_CONTENT_RESOLVE_FALLBACK.inc(),
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
        let err_str = "err".to_string();
        let cid = Cid::default();

        emit(PingEvent::Success(peer_id, rtt));
        emit(PingFailureEvent::Timeout(peer_id));
        emit(PingFailureEvent::Failure(peer_id, err_str.clone()));
        emit(IdentifyEvent::Received(peer_id));
        emit(IdentifyFailureEvent::Failure(peer_id, err_str.clone()));
        emit(DiscoveryEvent::BackgroundLookup(peer_id));
        emit(DiscoveryEvent::ConnectionEstablished(peer_id));
        emit(DiscoveryEvent::ConnectionClosed(peer_id));
        emit(MembershipEvent::Added(peer_id));
        emit(MembershipEvent::Removed(peer_id));
        emit(MembershipEvent::Skipped(peer_id));
        emit(MembershipEvent::PublishSuccess);
        emit(MembershipEvent::RoutablePeers(Default::default()));
        emit(MembershipFailureEvent::PublishFailure(err_str.clone()));
        emit(MembershipFailureEvent::GossipInvalidProviderRecord(
            Some(peer_id),
            err_str.clone(),
        ));
        emit(MembershipFailureEvent::GossipInvalidVoteRecord(
            Some(peer_id),
            err_str.clone(),
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
