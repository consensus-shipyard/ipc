# F3 Proof Generator Service

Pre-generates cryptographic proofs for F3 certificates from the parent chain, caching them for instant use by block proposers.

## Overview

This service provides production-ready proof generation for IPC subnets using F3 finality from the parent chain. It combines:
- **F3 Light Client** for cryptographic validation (BLS signatures, quorum, chain continuity)
- **Proof Generation** using the `ipc-filecoin-proofs` library
- **High-Performance Caching** with RocksDB persistence
- **Prometheus Metrics** for production monitoring

## Features

### Full Cryptographic Validation
- BLS signature verification using F3 light client
- Quorum checks (>2/3 power)
- Chain continuity validation (sequential instances)
- Power table verification and tracking

### High Performance
- Pre-generates proofs ahead of time (configurable lookahead)
- In-memory cache with optional RocksDB persistence
- Sequential processing ensures F3 state consistency
- ~15KB proof bundles with 15-20 witness blocks

### Production Ready
- Prometheus metrics for monitoring
- Structured logging with tracing
- Configuration validation on startup
- Graceful error handling with detailed context
- Supports recent F3 instances (RPC lookback limit: ~16.7 hours)

## Architecture

```
┌──────────────┐
│ Parent Chain │
│   F3 RPC     │
└──────┬───────┘
       │ Fetch certificates
       ↓
┌──────────────────────────────────┐
│ F3 Light Client                  │
│ - Fetch from F3 RPC              │
│ - BLS signature verification     │
│ - Quorum validation (>2/3 power) │
│ - Chain continuity checks        │
└──────┬───────────────────────────┘
       │ Validated certificates
       ↓
┌──────────────────────────────────┐
│ Proof Assembler                  │
│ - Fetch parent/child tipsets     │
│ - Generate storage proofs        │
│ - Generate event proofs          │
│ - Build witness blocks           │
└──────┬───────────────────────────┘
       │ Proof bundles
       ↓
┌──────────────────────────────────┐
│ Proof Cache (Memory + RocksDB)   │
│ - Sequential storage             │
│ - Lookahead window               │
│ - Retention policy               │
└──────┬───────────────────────────┘
       │
       ↓ Query by proposers
┌──────────────────────────────────┐
│ Block Proposer                   │
│ - Get next uncommitted proof     │
│ - Include in block               │
│ - Mark as committed              │
└──────────────────────────────────┘
```

## Components

### F3Client (`src/f3_client.rs`)
Wraps the F3 light client to provide:
- Certificate fetching from F3 RPC
- Full cryptographic validation (BLS, quorum, continuity)
- Sequential state management (prevents instance skipping)
- Power table tracking and updates

**Key Methods:**
- `new(rpc, network, instance, power_table)` - Production constructor with power table from F3CertManager
- `new_from_rpc(rpc, network, instance)` - Testing constructor that fetches power table from RPC
- `fetch_and_validate(instance)` - Fetch and cryptographically validate a certificate

### ProofAssembler (`src/assembler.rs`)
Generates cryptographic proofs using the `ipc-filecoin-proofs` library:
- Fetches parent and child tipsets from Lotus RPC
- Generates storage proofs for Gateway contract state (`subnet.topDownNonce`)
- Generates event proofs for `NewTopDownMessage` events
- Creates minimal Merkle witness blocks for verification

**Proof Specs:**
- Storage: `subnets[subnetKey].topDownNonce` (Gateway contract)
- Events: `NewTopDownMessage(address,IpcEnvelope,bytes32)` (Gateway contract)

### ProofCache (`src/cache.rs`)
Thread-safe cache with:
- In-memory BTreeMap for O(log n) ordered access
- Optional RocksDB persistence for crash recovery
- Lookahead window (pre-generate N instances ahead)
- Retention policy (keep M instances after commitment)
- Prometheus metrics for hits/misses and cache size

### ProofGeneratorService (`src/service.rs`)
Background service that:
- Polls F3 RPC at configured intervals
- Validates certificates cryptographically
- Generates and caches proofs sequentially
- Handles errors gracefully with retries
- Emits Prometheus metrics

## Usage

### In Fendermint Application

```rust
use fendermint_vm_topdown_proof_service::{launch_service, ProofServiceConfig};
use filecoin_f3_gpbft::PowerEntries;
use std::time::Duration;

// Get initial state from F3CertManager actor
let initial_instance = f3_cert_manager.last_committed_instance();
let power_table = f3_cert_manager.power_table();

// Configuration
let config = ProofServiceConfig {
    enabled: true,
    parent_rpc_url: "http://api.calibration.node.glif.io/rpc/v1".to_string(),
    parent_subnet_id: "/r314159".to_string(),
    f3_network_name: "calibrationnet".to_string(),
    gateway_actor_id: Some(1001),
    subnet_id: Some("my-subnet".to_string()),
    lookahead_instances: 5,
    retention_instances: 2,
    polling_interval: Duration::from_secs(30),
    max_cache_size_bytes: 100 * 1024 * 1024, // 100 MB
    fallback_rpc_urls: vec![],
};

// Launch service with optional persistence
let db_path = Some(PathBuf::from("/var/lib/fendermint/proof-cache"));
let (cache, handle) = launch_service(config, initial_instance, power_table, db_path).await?;

// Query cache in block proposer
if let Some(entry) = cache.get_next_uncommitted() {
    // Use entry.proof_bundle for verification
    // Use entry.certificate for F3 certificate
    propose_block_with_proof(entry);
}

// After block execution, mark instance as committed
cache.mark_committed(entry.instance_id);
```

### CLI Tools

Inspect cache contents:
```bash
ipc-cli proof-cache inspect --db-path /var/lib/fendermint/proof-cache
```

Show cache statistics:
```bash
ipc-cli proof-cache stats --db-path /var/lib/fendermint/proof-cache
```

Get specific proof:
```bash
ipc-cli proof-cache get --db-path /var/lib/fendermint/proof-cache --instance-id 12345
```

### Standalone Testing

```bash
# Build the test binary
cargo build --package fendermint_vm_topdown_proof_service --features cli --bin proof-cache-test

# Get current F3 instance
LATEST=$(curl -s -X POST -H "Content-Type: application/json" \
  --data '{"jsonrpc":"2.0","method":"Filecoin.F3GetLatestCertificate","params":[],"id":1}' \
  http://api.calibration.node.glif.io/rpc/v1 | jq -r '.result.GPBFTInstance')

# Start from recent instance (within RPC lookback limit of ~16.7 hours)
START=$((LATEST - 5))

# Run against Calibration testnet
./target/debug/proof-cache-test run \
    --rpc-url "http://api.calibration.node.glif.io/rpc/v1" \
    --initial-instance $START \
    --gateway-actor-id 176609 \
    --subnet-id "calib-subnet-1" \
    --poll-interval 10 \
    --lookahead 3 \
    --db-path /tmp/proof-cache-test.db
```

## Configuration

All configuration options in `ProofServiceConfig`:

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | bool | Yes | Enable/disable the service |
| `parent_rpc_url` | String | Yes | F3 RPC endpoint URL (HTTP or HTTPS) |
| `parent_subnet_id` | String | Yes | Parent subnet ID (e.g., "/r314159") |
| `f3_network_name` | String | Yes | F3 network name ("calibrationnet", "mainnet") |
| `gateway_actor_id` | Option<u64> | Yes | Gateway actor ID on parent chain |
| `subnet_id` | Option<String> | Yes | Current subnet ID for event filtering |
| `lookahead_instances` | u64 | Yes | How many instances to pre-generate (must be > 0) |
| `retention_instances` | u64 | Yes | How many old instances to keep (must be > 0) |
| `polling_interval` | Duration | Yes | How often to check for new certificates |
| `max_cache_size_bytes` | usize | No | Maximum cache size (0 = unlimited) |
| `fallback_rpc_urls` | Vec<String> | No | Backup RPC endpoints for failover |

## Observability

### Prometheus Metrics

**F3 Certificate Operations:**
- `f3_cert_fetch_total{status}` - Certificate fetch attempts (success/failure)
- `f3_cert_fetch_latency_secs{status}` - Fetch latency histogram
- `f3_cert_validation_total{status}` - Validation attempts (success/failure)
- `f3_cert_validation_latency_secs{status}` - Validation latency histogram
- `f3_current_instance` - Current F3 instance in light client state

**Proof Generation:**
- `proof_generation_total{status}` - Proof generation attempts
- `proof_generation_latency_secs{status}` - Generation latency histogram
- `proof_bundle_size_bytes{type}` - Proof bundle size distribution

**Cache Operations:**
- `proof_cache_size` - Number of proofs in cache
- `proof_cache_last_committed` - Last committed F3 instance
- `proof_cache_highest_cached` - Highest cached F3 instance
- `proof_cache_hit_total{result}` - Cache hits and misses

### Structured Logging

The service uses `tracing` for structured logging with appropriate levels:
- `ERROR` - Validation failures, proof generation errors, RPC failures
- `WARN` - Configuration issues, deprecated features
- `INFO` - Certificate validated, proof generated, cache updates
- `DEBUG` - Detailed operation flow, state transitions

Set log level with `RUST_LOG`:
```bash
RUST_LOG=info,fendermint_vm_topdown_proof_service=debug fendermint run
```

## Data Flow

### Certificate Validation Flow
1. **Poll** - Service checks for new F3 instances (every polling_interval)
2. **Fetch** - Light client fetches certificate from F3 RPC
3. **Validate** - Full cryptographic validation:
   - Verify BLS aggregated signature
   - Check quorum (>2/3 of power signed)
   - Verify chain continuity (sequential instances)
   - Apply power table deltas
4. **Update State** - Light client state advances to next instance

### Proof Generation Flow
1. **Extract Epoch** - Get highest finalized epoch from validated certificate
2. **Fetch Tipsets** - Get parent and child tipsets from Lotus RPC
3. **Generate Proofs** - Using `ipc-filecoin-proofs` library:
   - Storage proof for `subnets[subnetKey].topDownNonce`
   - Event proofs for `NewTopDownMessage` emissions
   - Minimal Merkle witness blocks
4. **Cache** - Store in memory and optionally persist to RocksDB

### Cache Entry Structure
```rust
pub struct CacheEntry {
    pub instance_id: u64,
    pub finalized_epochs: Vec<ChainEpoch>,
    pub proof_bundle: UnifiedProofBundle,  // Typed proof bundle
    pub certificate: SerializableF3Certificate,  // For on-chain submission
    pub generated_at: SystemTime,
    pub source_rpc: String,
}
```

## Troubleshooting

### Common Issues

**1. "lookbacks of more than 16h40m0s are disallowed"**

The Lotus RPC endpoint won't serve tipsets older than ~16.7 hours. 

**Solution**: Start from a recent F3 instance:
```bash
# Get latest instance
LATEST=$(curl -s -X POST ... | jq -r '.result.GPBFTInstance')
# Start from 5-10 instances back
START=$((LATEST - 5))
```

**2. "expected instance X, found instance Y"**

The F3 light client requires sequential validation. If proof generation fails, the state advances but the proof isn't cached, causing retry failures.

**Solution**: The service automatically handles this by checking if F3 state is past an instance before retrying.

**3. "Failed to fetch certificate from F3 RPC"**

Network connectivity issue or invalid RPC endpoint.

**Solution**: 
- Verify RPC endpoint is accessible
- Use HTTP instead of HTTPS if TLS issues occur
- Check `fallback_rpc_urls` configuration

**4. macOS system-configuration panic**

Older issue with reqwest library on macOS (now fixed in upstream).

**Solution**: Already fixed in upstream `ipc-filecoin-proofs` (uses `.no_proxy()`)

## Testing

### Unit Tests
```bash
cargo test --package fendermint_vm_topdown_proof_service --lib
```

**Test Coverage:**
- F3 client creation and state management
- Cache operations (insert, get, cleanup)
- Persistence layer (RocksDB save/load)
- Configuration parsing
- Metrics registration

### Integration Tests
```bash
# Requires live Calibration network
cargo test --package fendermint_vm_topdown_proof_service --test integration -- --ignored
```

### End-to-End Testing

1. **Deploy Test Contract** (optional - for testing with TopdownMessenger):
```bash
cd /path/to/proofs/topdown-messenger
forge create --rpc-url http://api.calibration.node.glif.io/rpc/v1 \
    --private-key $PRIVATE_KEY \
    src/TopdownMessenger.sol:TopdownMessenger
```

2. **Run Proof Service**:
```bash
./target/debug/proof-cache-test run \
    --rpc-url "http://api.calibration.node.glif.io/rpc/v1" \
    --initial-instance <RECENT_INSTANCE> \
    --gateway-actor-id <GATEWAY_ACTOR_ID> \
    --subnet-id "your-subnet-id" \
    --poll-interval 10 \
    --lookahead 3 \
    --db-path /tmp/proof-cache-test.db
```

3. **Inspect Results**:
```bash
# After stopping the service
./target/debug/proof-cache-test inspect --db-path /tmp/proof-cache-test.db
./target/debug/proof-cache-test get --db-path /tmp/proof-cache-test.db --instance-id <INSTANCE>
```

## Dependencies

### Core Dependencies
- `filecoin-f3-lightclient` - F3 light client with BLS validation
- `filecoin-f3-certs` - F3 certificate types and validation
- `filecoin-f3-gpbft` - GPBFT consensus types (power tables)
- `proofs` - IPC proof generation library (`ipc-filecoin-proofs`)
- `rocksdb` - Optional persistence layer
- `ipc-observability` - Metrics and tracing

### Repository Links
- F3 Light Client: https://github.com/moshababo/rust-f3/tree/bdn_agg
- Proofs Library: https://github.com/consensus-shipyard/ipc-filecoin-proofs/tree/proofs

## Module Documentation

### `f3_client.rs` - F3 Certificate Handling
Wraps `filecoin-f3-lightclient` to provide certificate fetching and validation.

**Production Mode:**
```rust
F3Client::new(rpc_url, network, instance, power_table)
```
Uses power table from F3CertManager actor on-chain.

**Testing Mode:**
```rust
F3Client::new_from_rpc(rpc_url, network, instance).await
```
Fetches power table from F3 RPC (for testing/development).

### `assembler.rs` - Proof Generation
Uses `ipc-filecoin-proofs` to generate cryptographic proofs.

**Proof Targets (Real Gateway Contract):**
- **Storage Proof**: `subnets[subnetKey].topDownNonce` (slot offset 3)
- **Event Proof**: `NewTopDownMessage(address indexed subnet, IpcEnvelope message, bytes32 indexed id)`

Creates `LotusClient` on-demand (not `Send`, so created per-request).

### `cache.rs` - Proof Caching
Thread-safe cache using `Arc<RwLock<BTreeMap>>`.

**Features:**
- Sequential instance ordering (BTreeMap)
- Lookahead enforcement (can't cache beyond window)
- Automatic cleanup (retention policy)
- Optional RocksDB persistence
- Prometheus metrics

### `service.rs` - Background Service
Main service loop that:
1. Polls at configured interval
2. Generates proofs for instances within lookahead window
3. Skips already-cached instances
4. Emits metrics on success/failure
5. Retries on errors

**Critical**: Processes F3 instances sequentially - never skips!

### `observe.rs` - Observability
Prometheus metrics and structured events using `ipc-observability`.

**Metrics Registration:**
```rust
use fendermint_vm_topdown_proof_service::observe::register_metrics;
register_metrics(&prometheus_registry)?;
```

### `persistence.rs` - RocksDB Storage
Persistent storage for proof cache using RocksDB.

**Schema:**
- `meta:last_committed` - Last committed instance ID
- `meta:schema_version` - Database schema version
- `entry:{instance_id}` - Serialized cache entries

### `verifier.rs` - Proof Verification
Deterministic verification of proof bundles against F3 certificates.

**Usage:**
```rust
use fendermint_vm_topdown_proof_service::verify_proof_bundle;
verify_proof_bundle(&bundle, &certificate)?;
```

Validates storage proofs and event proofs using `ipc-filecoin-proofs` verifier.

## Performance Characteristics

### Typical Proof Bundle
- **Size**: 15-17 KB
- **Storage Proofs**: 1 (for topDownNonce)
- **Event Proofs**: 0-N (depends on messages in that instance)
- **Witness Blocks**: 15-21 Merkle tree blocks
- **Generation Time**: ~1-2 seconds (network dependent)
- **Validation Time**: ~10ms (BLS signature check)

### Cache Efficiency
- **Memory**: ~20 KB per cached instance
- **Lookahead=5**: ~100 KB memory
- **RocksDB**: Similar disk usage + metadata overhead
- **Hit Rate**: >95% for sequential block production

## Known Limitations

1. **RPC Lookback Limit**: Can only generate proofs for instances within ~16.7 hours (Lotus RPC limitation)
2. **Sequential Processing**: Must validate instances in order (F3 light client requirement)
3. **Single RPC Endpoint**: Currently uses single endpoint (multi-provider support in future plan)
4. **No Batch Fetching**: Fetches certificates one at a time (could be optimized)

## Future Improvements

See Cursor plan "Custom RPC Client Integration" for:
- Multi-provider failover using custom RPC client trait
- Health tracking and automatic recovery
- Integration with ParentClient for reliability

## Related Code

- **F3CertManager Actor**: `fendermint/actors/f3-cert-manager` - On-chain certificate storage
- **Gateway Contract**: `contracts/contracts/gateway` - Parent chain gateway
- **IPC Provider**: `ipc/provider` - Lotus RPC client
- **Fendermint App**: Integrates this service for topdown finality

## License

MIT OR Apache-2.0 - Protocol Labs
