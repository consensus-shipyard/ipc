# Prompt: Fix libp2p listen_addr Binding Issue in IPC

## Problem Statement

There is a critical bug in `ipc-cli node init` that prevents libp2p from binding to network interfaces on cloud VMs, which breaks parent finality voting and top-down message processing (including `cross-msg fund` transactions).

## The Bug

**Location:** `ipc/cli/src/commands/node/peer.rs` lines 95-106

**Current behavior:**
```rust
// Apply Fendermint resolver port configuration
if let Some(resolver_port) = ports.resolver {
    log::info!("Configuring Fendermint resolver port: {}", resolver_port);

    let external_ip = p2p_config.external_ip.as_deref().unwrap_or("127.0.0.1");
    let listen_addr = format!("/ip4/{}/tcp/{}", external_ip, resolver_port);  // BUG: Uses external_ip for listen_addr

    let fendermint_config = FendermintOverrides {
        resolver: Some(ResolverOverrideConfig {
            connection: Some(ConnectionOverrideConfig {
                listen_addr: Some(listen_addr),  // This gets set to the public IP!
                extra: toml::Table::new(),
            }),
            // ...
        }),
        // ...
    };
    // Merges this config, overwriting any fendermint-overrides from node.yaml
}
```

**The issue:**
- The code uses `external_ip` (e.g., `34.73.187.192`) for BOTH `listen_addr` AND `external_addresses`
- On cloud VMs (GCP, AWS, Azure), the public IP is NOT bound to any interface
- The OS can only bind to private IPs or `0.0.0.0`
- This causes libp2p to fail binding with error: `Cannot assign requested address (os error 99)`
- When libp2p can't bind, parent finality vote gossip doesn't work
- Without vote gossip, parent finality cannot commit
- Without parent finality commits, top-down messages (cross-chain transfers) never execute

## The Fix

**Separate concerns:**
1. **`listen_addr`** = Where THIS node binds/listens → Should be `0.0.0.0` or private IP
2. **`external_addresses`** = What THIS node advertises to peers → Should be public IP
3. **`static_addresses`** = Addresses of OTHER nodes to connect to → Should be their public IPs

**Proposed solution:**

```rust
// Apply Fendermint resolver port configuration
if let Some(resolver_port) = ports.resolver {
    log::info!("Configuring Fendermint resolver port: {}", resolver_port);

    // FIXED: Use 0.0.0.0 for listen_addr (can bind on any interface)
    let listen_addr = format!("/ip4/0.0.0.0/tcp/{}", resolver_port);

    // Use external_ip for external_addresses (what we advertise to peers)
    let external_ip = p2p_config.external_ip.as_deref().unwrap_or("127.0.0.1");
    let external_addresses = vec![format!("/ip4/{}/tcp/{}", external_ip, resolver_port)];

    let fendermint_config = FendermintOverrides {
        resolver: Some(ResolverOverrideConfig {
            connection: Some(ConnectionOverrideConfig {
                listen_addr: Some(listen_addr),  // Binds to 0.0.0.0
                external_addresses: Some(external_addresses),  // Advertises public IP
                extra: toml::Table::new(),
            }),
            // ...
        }),
        // ...
    };
    // ...
}
```

**Alternative approach (more flexible):**
Add a separate `listen_ip` field to `P2pConfig` that defaults to `0.0.0.0` but can be overridden for special cases:

```rust
pub struct P2pConfig {
    /// External IP address for peer connections (defaults to "127.0.0.1")
    pub external_ip: Option<String>,
    /// Listen IP for binding (defaults to "0.0.0.0")
    pub listen_ip: Option<String>,
    /// Network port configuration
    pub ports: Option<P2pPortsConfig>,
    /// Peer configuration from various sources
    pub peers: Option<P2pPeersConfig>,
}
```

## Testing

### Manual Testing
1. Deploy a subnet on GCP/AWS with 3 validators
2. Run `ipc-cli node init` on each validator
3. Verify `~/.ipc-node/fendermint/config/default.toml` has:
   ```toml
   [resolver.connection]
   listen_addr = "/ip4/0.0.0.0/tcp/26655"
   external_addresses = ["/ip4/<PUBLIC_IP>/tcp/26655/p2p/<PEER_ID>"]
   ```
4. Start nodes and check libp2p is listening:
   ```bash
   ss -tulpn | grep 26655
   # Should show: 0.0.0.0:26655 (not 127.0.0.1:26655 or <public_ip>:26655)
   ```
5. Check logs for vote gossip:
   ```bash
   grep "parent finality vote gossip loop" ~/.ipc-node/logs/*.log
   grep "PeerVoteReceived" ~/.ipc-node/logs/*.log
   ```
6. Verify parent finality commits:
   ```bash
   grep "ParentFinalityCommitted" ~/.ipc-node/logs/*.log
   ```
7. Test `ipc-cli cross-msg fund` works correctly

### Automated Testing
Add integration test that:
- Initializes multiple nodes with different `external_ip` values
- Verifies `listen_addr` is always `0.0.0.0`
- Verifies `external_addresses` uses the `external_ip`
- Confirms nodes can establish libp2p connections

## Related Code to Review

1. **`ipc/cli/src/commands/node/config.rs`** - P2pConfig struct definition
2. **`ipc/cli/src/commands/node/peer.rs`** - Peer configuration application
3. **Fendermint resolver configuration** - Ensure it respects the `listen_addr` setting
4. **Documentation** - Update `docs/ipc/node-init.md` to explain `external-ip` vs listen binding

## Impact

**High Priority** - This bug prevents parent finality voting on any cloud-deployed subnet, breaking core IPC functionality.

**Affected users:** Anyone deploying IPC subnets on:
- Google Cloud Platform (GCP)
- Amazon Web Services (AWS)
- Microsoft Azure
- Any environment where public IPs are not directly bound to network interfaces

## Workaround (Current)

Users currently need to manually fix `listen_addr` after `ipc-cli node init`:
```bash
sed -i 's|listen_addr = "/ip4/.*/tcp/26655"|listen_addr = "/ip4/0.0.0.0/tcp/26655"|' \
  ~/.ipc-node/fendermint/config/default.toml
```

This workaround is implemented in the community-created `ipc-subnet-manager` script.

## Additional Context

- Issue discovered during troubleshooting why `cross-msg fund` transactions weren't executing
- Root cause identified through systematic debugging of libp2p binding and parent finality voting
- The fix allows inbound libp2p connections, which are required for vote gossip in the parent finality consensus mechanism
- Without this fix, validators can make outbound connections but cannot accept inbound connections, preventing proper P2P mesh formation

---

**Please review this issue and implement the fix in the IPC codebase. The suggested fix ensures libp2p can bind successfully while still advertising the correct public IP to peers.**

