# Future Implementation: Custom RPC Client with ParentClient Integration

## Goal

Enable the F3 light client to use our `ParentClient` for multi-provider failover and reliability, while maintaining full cryptographic validation.

## Current Limitation

The F3 light client uses `jsonrpsee` directly with a single endpoint:

```rust
// In filecoin-f3-lightclient
pub struct LightClient {
    rpc: RPCClient,  // Single endpoint only
    network: NetworkName,
    verifier: BLSVerifier,
}
```

Our `ParentClient` provides:

- ✅ Multi-provider failover
- ✅ Health tracking
- ✅ Automatic recovery
- ❌ Can't be used with F3 light client (API incompatible)

## Solution: Add Custom RPC Client Support to rust-f3

### Step 1: Define RPC Trait in rust-f3

**File:** `rust-f3-fork/rpc/src/trait.rs` (NEW)

```rust
use async_trait::async_trait;
use crate::{FinalityCertificate, PowerEntry};
use anyhow::Result;

/// Abstract RPC client trait for F3 operations
#[async_trait]
pub trait F3RpcClient: Send + Sync {
    /// Fetch F3 certificate by instance ID
    async fn get_certificate(&self, instance: u64) -> Result<FinalityCertificate>;

    /// Fetch power table by instance ID
    async fn get_power_table(&self, instance: u64) -> Result<Vec<PowerEntry>>;

    /// Get latest F3 certificate
    async fn get_latest_certificate(&self) -> Result<Option<FinalityCertificate>>;
}
```

### Step 2: Update LightClient to Accept Custom Client

**File:** `rust-f3-fork/lightclient/src/lib.rs`

```rust
pub struct LightClient<C: F3RpcClient = RPCClient> {
    rpc: C,  // Generic over RPC client!
    network: NetworkName,
    verifier: BLSVerifier,
}

impl<C: F3RpcClient> LightClient<C> {
    pub fn new_with_client(client: C, network_name: &str) -> Result<Self> {
        Ok(Self {
            rpc: client,
            network: network_name.parse()?,
            verifier: BLSVerifier::new(),
        })
    }

    pub async fn get_certificate(&self, instance: u64) -> Result<certs::FinalityCertificate> {
        let rpc_cert = self.rpc.get_certificate(instance).await?;
        rpc_to_internal::convert_certificate(rpc_cert)
    }

    // ... other methods use self.rpc
}

// Keep existing constructor for default client
impl LightClient<RPCClient> {
    pub fn new(endpoint: &str, network_name: &str) -> Result<Self> {
        Self::new_with_client(RPCClient::new(endpoint)?, network_name)
    }
}
```

### Step 3: Implement Trait for ParentClient

**File:** `fendermint/vm/topdown/proof-service/src/parent_client.rs`

```rust
use async_trait::async_trait;
use filecoin_f3_rpc::{F3RpcClient, FinalityCertificate, PowerEntry};

#[async_trait]
impl F3RpcClient for ParentClient {
    async fn get_certificate(&self, instance: u64) -> Result<FinalityCertificate> {
        // Fetch from Lotus with multi-provider failover
        let lotus_cert = self.fetch_certificate(instance).await?
            .context("Certificate not available")?;

        // Convert Lotus → F3 RPC format
        let json = serde_json::to_value(&lotus_cert)?;
        let f3_cert = serde_json::from_value(json)?;

        Ok(f3_cert)
    }

    async fn get_power_table(&self, instance: u64) -> Result<Vec<PowerEntry>> {
        // Fetch from Lotus with failover
        let lotus_power = self.fetch_power_table(instance).await?;

        // Convert to F3 format
        lotus_power.into_iter()
            .map(|entry| PowerEntry {
                id: entry.id,
                power: entry.power.parse()?,
                pub_key: base64::decode(&entry.pub_key)?,
            })
            .collect()
    }

    async fn get_latest_certificate(&self) -> Result<Option<FinalityCertificate>> {
        // Use primary provider, fallback on failure
        match self.fetch_latest_certificate().await? {
            Some(lotus_cert) => {
                let json = serde_json::to_value(&lotus_cert)?;
                Ok(Some(serde_json::from_value(json)?))
            }
            None => Ok(None),
        }
    }
}
```

### Step 4: Update F3Client to Use Custom Client

**File:** `fendermint/vm/topdown/proof-service/src/f3_client.rs`

```rust
pub struct F3Client {
    light_client: Arc<Mutex<LightClient<ParentClient>>>,  // Use our client!
    state: Arc<Mutex<LightClientState>>,
}

impl F3Client {
    pub fn new(
        parent_client: Arc<ParentClient>,  // Inject our multi-provider client
        network_name: &str,
        initial_instance: u64,
        power_table: PowerEntries,
    ) -> Result<Self> {
        // Create light client with our ParentClient
        let light_client = LightClient::new_with_client(
            (*parent_client).clone(),  // Clone the client
            network_name,
        )?;

        let state = LightClientState {
            instance: initial_instance,
            chain: None,
            power_table,
        };

        Ok(Self {
            light_client: Arc::new(Mutex::new(light_client)),
            state: Arc::new(Mutex::new(state)),
        })
    }
}
```

### Step 5: Update Service to Use Integrated Client

**File:** `fendermint/vm/topdown/proof-service/src/service.rs`

```rust
// Create parent client with multi-provider support
let parent_client = Arc::new(ParentClient::new(parent_client_config)?);

// Create F3 client using ParentClient as RPC backend
let f3_client = Arc::new(F3Client::new(
    parent_client.clone(),  // Multi-provider backend!
    "calibrationnet",
    initial_instance,
    power_table,
)?);
```

## Benefits

**Combining F3 Validation + Multi-Provider Reliability:**

```
ParentClient (multi-provider failover)
     ↓ (implements F3RpcClient trait)
F3 Light Client (crypto validation)
     ↓
Validated Certificates
```

✅ Multi-provider failover (from ParentClient)
✅ Health tracking and recovery (from ParentClient)
✅ Full cryptographic validation (from F3 Light Client)
✅ Best of both worlds!

## Implementation Checklist

### In rust-f3-fork:

- [ ] Create `rpc/src/trait.rs` with `F3RpcClient` trait
- [ ] Add `async-trait` dependency
- [ ] Make `LightClient` generic: `LightClient<C: F3RpcClient = RPCClient>`
- [ ] Add `new_with_client(client: C)` constructor
- [ ] Implement trait for existing `RPCClient`
- [ ] Update all methods to use `self.rpc` generically
- [ ] Test with both default and custom clients
- [ ] Submit PR to moshababo/rust-f3

### In IPC project:

- [ ] Add `async-trait` to parent_client dependencies
- [ ] Implement `F3RpcClient` trait for `ParentClient`
- [ ] Add methods: `fetch_power_table()`, `fetch_latest_certificate()`
- [ ] Update `F3Client` to use `LightClient<ParentClient>`
- [ ] Update service to pass `ParentClient` to `F3Client::new()`
- [ ] Remove `new_from_rpc()` test-only constructor
- [ ] Test failover scenarios
- [ ] Verify health checks work correctly

## Why Keep ParentClient

**Current:** Only used for health checks (minimal)
**Future:** Will be the RPC backend for F3 light client, providing:

- Multi-endpoint failover
- Health tracking
- Automatic recovery
- Production-grade reliability

**Status:** Keep ParentClient in codebase for this future integration.

## Files

### rust-f3-fork:

1. `rpc/src/trait.rs` (NEW) - F3RpcClient trait
2. `rpc/src/lib.rs` - Export trait
3. `rpc/Cargo.toml` - Add async-trait
4. `lightclient/src/lib.rs` - Generic LightClient

### IPC project:

1. `src/parent_client.rs` - Implement F3RpcClient, add missing methods
2. `src/f3_client.rs` - Use LightClient<ParentClient>
3. `src/service.rs` - Pass ParentClient to F3Client

## Timeline

**Phase 1:** ✅ BLS fix submitted to rust-f3 (done!)
**Phase 2:** ⏳ Wait for BLS fix merge
**Phase 3:** 📋 Implement custom RPC client trait (this plan)
**Phase 4:** 🚀 Submit custom RPC client PR
**Phase 5:** 🎉 Use integrated solution in production
