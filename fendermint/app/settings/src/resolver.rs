// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::{net::{SocketAddr, SocketAddrV4, SocketAddrV6}, path::PathBuf, time::Duration};

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DurationSeconds};

use ipc_api::subnet_id::SubnetID;
use multiaddr::Multiaddr;

use crate::{home_relative, IsHumanReadable};

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResolverSettings {
    /// Time to wait between attempts to resolve a CID, in seconds.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub retry_delay: Duration,
    pub network: NetworkSettings,
    pub discovery: DiscoverySettings,
    pub membership: MembershipSettings,
    pub connection: ConnectionSettings,
    pub content: ContentSettings,
    pub iroh_resolver_config: IrohResolverSettings,
}

impl Default for ResolverSettings {
    fn default() -> Self {
        Self {
            retry_delay: Duration::from_secs(10),
            network: Default::default(),
            discovery: Default::default(),
            membership: Default::default(),
            connection: Default::default(),
            content: Default::default(),
            iroh_resolver_config: Default::default(),
        }
    }
}

/// Settings describing the subnet hierarchy, not the physical network.
///
/// For physical network settings see [ConnectionSettings].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NetworkSettings {
    /// Cryptographic key used to sign messages.
    ///
    /// This is the name of a Secp256k1 private key file,
    /// relative to the `home_dir`.
    local_key: PathBuf,
    /// Network name to differentiate this peer group.
    pub network_name: String,
}

home_relative!(NetworkSettings { local_key });

impl Default for NetworkSettings {
    fn default() -> Self {
        Self {
            local_key: PathBuf::from("keys/network.sk"),
            network_name: "".into(),
        }
    }
}

/// Configuration for [`discovery::Behaviour`].
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiscoverySettings {
    /// Custom nodes which never expire, e.g. bootstrap or reserved nodes.
    ///
    /// The addresses must end with a `/p2p/<peer-id>` part.
    pub static_addresses: Vec<Multiaddr>,
    /// Number of connections at which point we pause further discovery lookups.
    pub target_connections: usize,
    /// Option to disable Kademlia, for example in a fixed static network.
    pub enable_kademlia: bool,
}

impl Default for DiscoverySettings {
    fn default() -> Self {
        Self {
            static_addresses: Vec::new(),
            target_connections: 50,
            enable_kademlia: true,
        }
    }
}

/// Configuration for [`membership::Behaviour`].
#[serde_as]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MembershipSettings {
    /// User defined list of subnets which will never be pruned from the cache.
    #[serde_as(as = "Vec<IsHumanReadable>")]
    pub static_subnets: Vec<SubnetID>,

    /// Maximum number of subnets to track in the cache.
    pub max_subnets: usize,

    /// Publish interval for supported subnets.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub publish_interval: Duration,

    /// Minimum time between publishing own provider record in reaction to new joiners.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub min_time_between_publish: Duration,

    /// Maximum age of provider records before the peer is removed without an update.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub max_provider_age: Duration,
}

impl Default for MembershipSettings {
    fn default() -> Self {
        Self {
            static_subnets: Vec::new(),
            max_subnets: 100,
            publish_interval: Duration::from_secs(60),
            min_time_between_publish: Duration::from_secs(5),
            max_provider_age: Duration::from_secs(300),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConnectionSettings {
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

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            listen_addr: Multiaddr::empty(),
            external_addresses: Vec::new(),
            max_incoming: 30,
            expected_peer_count: 10000,
            max_peers_per_query: 5,
            event_buffer_capacity: 100,
        }
    }
}

/// Configuration for [`content::Behaviour`].
#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContentSettings {
    /// Number of bytes that can be consumed by remote peers in a time period.
    ///
    /// 0 means no limit.
    pub rate_limit_bytes: u32,
    /// Length of the time period at which the consumption limit fills.
    ///
    /// 0 means no limit.
    #[serde_as(as = "DurationSeconds<u64>")]
    pub rate_limit_period: Duration,
}

impl Default for ContentSettings {
    fn default() -> Self {
        Self {
            rate_limit_bytes: 0,
            rate_limit_period: Duration::from_secs(0),
        }
    }
}

/// Configuration for Iroh blob storage and transfer
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct IrohResolverSettings {
    /// IPv4 address for Iroh node
    pub v4_addr: Option<SocketAddrV4>,
    /// IPv6 address for Iroh node
    pub v6_addr: Option<SocketAddrV6>,
    /// Data directory for Iroh
    pub iroh_data_dir: PathBuf,
    /// RPC address for Iroh
    pub rpc_addr: SocketAddr,
}

impl Default for IrohResolverSettings {
    fn default() -> Self {
        Self {
            v4_addr: None,
            v6_addr: None,
            iroh_data_dir: PathBuf::from("data/iroh_resolver"),
            rpc_addr: "127.0.0.1:4444".parse().unwrap(),
        }
    }
}
