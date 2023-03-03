// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use libipld::store::StoreParams;
use libp2p::{
    identify,
    identity::{Keypair, PublicKey},
    ping,
    swarm::NetworkBehaviour,
    PeerId,
};
use libp2p_bitswap::Bitswap;

mod discovery;
mod membership;

pub struct NetworkConfig {
    /// Cryptographic key used to sign messages.
    pub local_key: Keypair,
    /// Network name to be differentiate this peer group.
    pub network_name: String,
}

impl NetworkConfig {
    pub fn local_public_key(&self) -> PublicKey {
        self.local_key.public()
    }
    pub fn local_peer_id(&self) -> PeerId {
        self.local_public_key().to_peer_id()
    }
}

/// Libp2p behaviour to manage content resolution from other subnets, using:
///
/// * Kademlia for peer discovery
/// * Gossipsub to advertise subnet membership
/// * Bitswap to resolve CIDs
#[derive(NetworkBehaviour)]
pub struct IpldResolver<P: StoreParams> {
    ping: ping::Behaviour,
    identify: identify::Behaviour,
    discovery: discovery::Behaviour,
    membership: membership::Behaviour,
    bitswap: Bitswap<P>, // TODO (IPC-36): Wrap
}

// Unfortunately by using `#[derive(NetworkBehaviour)]` we cannot easily inspects events
// from the inner behaviours, e.g. we cannot poll a behaviour and if it returns something
// of interest then call a method on another behaviour. We can do this in another wrapper
// where we manually implement `NetworkBehaviour`, or the outer service where we drive the
// Swarm; there we are free to call any of the behaviours.
