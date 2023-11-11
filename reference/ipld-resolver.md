---
description: >-
  An ABCI++ application can contact the IPLD resolver & store to read and write
  data so that it is IPLD addressable.
---

# IPLD Resolver

The IPLD resolver is a library that allow users to exchange data between subnets in IPLD format.&#x20;

An ABCI++ application can contact the [IPLD](https://docs.filecoin.io/basics/project-and-community/related-projects#ipld) resolver & store to read and write data so that it is IPLD addressable.&#x20;

The IPLD store is a common store that allows for read/write in IPLD format from the FVM and the resolver.&#x20;

### Usage

Please have a look at the [smoke test](https://github.com/consensus-shipyard/ipc-ipld-resolver/blob/main/tests/smoke.rs) for an example of using the library.

The following snippet demonstrates how one would create a resolver instance and use it:

```
async fn main() {
  let config = Config {
      connection: ConnectionConfig {
          listen_addr: "/ip4/127.0.0.1/tcp/0".parse().unwrap(),
          expected_peer_count: 1000,
          max_incoming: 25,
          max_peers_per_query: 10,
          event_buffer_capacity: 100,
      },
      network: NetworkConfig {
          local_key: Keypair::generate_secp256k1(),
          network_name: "example".to_owned(),
      },
      discovery: DiscoveryConfig {
          static_addresses: vec!["/ip4/95.217.194.97/tcp/8008/p2p/12D3KooWC1EaEEpghwnPdd89LaPTKEweD1PRLz4aRBkJEA9UiUuS".parse().unwrap()]
          target_connections: 50,
          enable_kademlia: true,
      },
      membership: MembershipConfig {
          static_subnets: vec![],
          max_subnets: 10,
          publish_interval: Duration::from_secs(300),
          min_time_between_publish: Duration::from_secs(5),
          max_provider_age: Duration::from_secs(60),
      },
  };

  let store = todo!("implement BitswapStore and a Blockstore");

  let service = Service::new(config, store.clone());
  let client = service.client();

  tokio::task::spawn(async move { service.run().await });

  let cid: Cid = todo!("the CID we want to resolve");
  let subnet_id: SubnetID = todo!("the SubnetID from where the CID can be resolved");

  match client.resolve(cid, subnet_id).await.unwrap() {
    Ok(()) => {
      let _content: MyContent = store.get(cid).unwrap();
    }
    Err(e) => {
      println!("{cid} could not be resolved from {subnet_id}: {e}")
    }
  }
}
```

### Sub-Components

The IPLD Resolver uses libp2p to form a Peer-to-Peer network, using the following protocols:

* [Ping](https://github.com/libp2p/rust-libp2p/tree/v0.50.1/protocols/ping)
* [Identify](https://github.com/libp2p/rust-libp2p/tree/v0.50.1/protocols/ping) is used to learn the listening address of the remote peers
* [Kademlia](https://github.com/libp2p/rust-libp2p/tree/v0.50.1/protocols/kad) is used for peer discovery
* [Gossipsub](https://github.com/libp2p/rust-libp2p/tree/v0.50.1/protocols/gossipsub) is used to announce information about subnets the peers provide data for
* [Bitswap](https://github.com/ipfs-rust/libp2p-bitswap) is used to resolve CIDs to content

See the libp2p [specs](https://github.com/libp2p/specs) and [docs](https://docs.libp2p.io/concepts/fundamentals/protocols/) for details on each protocol, and look [here](https://docs.ipfs.tech/concepts/bitswap/) for Bitswap.

The Resolver is completely agnostic over what content it can resolve, as long as it's based on CIDs; it's not aware of the checkpointing use case above.

The interface with the host system is through a host-provided implementation of the [BitswapStore](https://github.com/ipfs-rust/libp2p-bitswap/blob/7dd9cececda3e4a8f6e14c200a4b457159d8db33/src/behaviour.rs#L55) which the library uses to retrieve and store content. Implementors can make use of the [missing\_blocks](https://github.com/consensus-shipyard/ipc-ipld-resolver/blob/main/src/missing\_blocks.rs) helper method which recursively collects all CIDs from an IPLD `Blockstore`, starting from the root CID we are looking for.

Internally the protocols are wrapped into behaviours that interpret their events and manage their associated state:

* `Discovery` wraps `Kademlia`
* `Membership` wraps `Gossipsub`
* `Content` wraps `Bitswap`

The following diagram shows a typical sequence of events within the IPLD Resolver. For brevity, only one peer is shown in detail; it's counterpart is represented as a single boundary.

[![IPLD Resolver](https://github.com/consensus-shipyard/ipc-ipld-resolver/raw/main/docs/diagrams/ipld\_resolver.png)](https://github.com/consensus-shipyard/ipc-ipld-resolver/blob/main/docs/diagrams/ipld\_resolver.png)

### Diagram Automation

The diagrams in this directory can be rendered with `make diagrams`.

Adding the following script to `.git/hooks/pre-commit` automatically renders and checks in the images when we commit changes to the them. CI should also check that there are no uncommitted changes.

```
#!/usr/bin/env bash

# If any command fails, exit immediately with that command's exit status
set -eo pipefail

# Redirect output to stderr.
exec 1>&2

if git diff --cached --name-only  --diff-filter=d | grep .puml
then
  make diagrams
  git add docs/diagrams/*.png
fi
```
