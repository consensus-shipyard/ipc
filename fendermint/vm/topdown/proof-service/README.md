# F3 Proof Generator Service

Pre-generates cryptographic proofs for F3 certificates from the parent chain, caching them for instant use by block proposers.

## Features

✅ **Full Cryptographic Validation**
- BLS signature verification
- Quorum checks (>2/3 power)
- Chain continuity validation
- Power table verification

✅ **F3 Light Client Integration**
- Direct F3 RPC access
- Sequential certificate validation
- Stateful power table tracking

✅ **High Performance**
- Pre-generates proofs ahead of time
- In-memory cache with RocksDB persistence
- Multi-provider failover for reliability

## Architecture

```
┌──────────────┐
│ F3 RPC       │ (Parent chain F3 endpoint)
└──────┬───────┘
       │
       ↓ Fetch certificates
┌──────────────────────────────────┐
│ F3 Light Client                  │
│ - Cryptographic validation       │
│ - BLS signature verification     │
│ - Quorum + continuity checks     │
└──────┬───────────────────────────┘
       │
       ↓ Validated certificates
┌──────────────────────────────────┐
│ Proof Generator Service          │
│ 1. Generate proofs               │
│ 2. Cache (memory + RocksDB)      │
│ 3. Serve to proposers            │
└──────────────────────────────────┘
```

## Components

### F3Client (`src/f3_client.rs`)
Wraps the F3 light client to provide:
- Certificate fetching from F3 RPC
- Full cryptographic validation
- Sequential state management
- Fallback to Lotus RPC if needed

### ProofAssembler (`src/assembler.rs`)
Generates cryptographic proofs using the ipc-filecoin-proofs library:
- Storage proofs (contract state)
- Event proofs (emitted events)
- Merkle proofs for parent chain data

### ProofCache (`src/cache.rs`)
Thread-safe cache with:
- In-memory BTreeMap for fast access
- Optional RocksDB persistence
- Lookahead and retention policies
- Sequential instance ordering

### ProofGeneratorService (`src/service.rs`)
Background service that:
- Polls for new F3 certificates
- Validates them cryptographically
- Generates and caches proofs
- Enforces sequential processing

## Usage

### In Fendermint Application

```rust
use fendermint_vm_topdown_proof_service::{launch_service, ProofServiceConfig};

// Configuration
let config = ProofServiceConfig {
    enabled: true,
    parent_rpc_url: "https://api.calibration.node.glif.io/rpc/v1".to_string(),
    parent_subnet_id: "/r314159".to_string(),
    gateway_actor_id: Some(1001),
    subnet_id: Some("my-subnet".to_string()),
    lookahead_instances: 5,
    polling_interval: Duration::from_secs(30),
    ..Default::default()
};

// Launch service
let initial_instance = 100; // From F3CertManager actor
let (cache, handle) = launch_service(config, initial_instance).await?;

// Query cache for next proof
if let Some(entry) = cache.get_next_uncommitted() {
    // Use entry.proof_bundle_bytes for block proposal
    // Use entry.actor_certificate for on-chain submission
}
```

### Standalone Testing

```bash
# Build the test binary
cargo build --package fendermint_vm_topdown_proof_service --features cli --bin proof-cache-test

# Run against Calibration testnet
./target/debug/proof-cache-test \
    --rpc https://api.calibration.node.glif.io/rpc/v1 \
    --subnet-id /r314159 \
    --gateway-id 1001 \
    --start-instance 0
```

## Configuration

See `src/config.rs` for all options:

- `enabled` - Enable/disable the service
- `parent_rpc_url` - F3 RPC endpoint URL
- `fallback_rpc_urls` - Backup RPC endpoints
- `parent_subnet_id` - Parent subnet ID (e.g., "/r314159")
- `gateway_actor_id` - Gateway actor ID for proofs
- `subnet_id` - Current subnet ID
- `lookahead_instances` - How many instances to cache ahead
- `retention_instances` - How many old instances to keep
- `polling_interval` - How often to check for new certificates
- `max_cache_size_bytes` - Maximum cache size (0 = unlimited)
- `persistence_path` - Optional RocksDB path for persistence

## Certificate Validation

The service performs **full cryptographic validation** via the F3 light client:

1. **BLS Signature Verification**
   - Validates aggregated BLS signatures
   - Checks signature against power table

2. **Quorum Validation**
   - Ensures >2/3 of power has signed
   - Validates signer bitmap

3. **Chain Continuity**
   - Ensures sequential F3 instances
   - Validates EC chain linkage

4. **Power Table Validation**
   - Validates power table CIDs
   - Applies power table deltas

## Data Flow

1. **Fetch** - Light client fetches certificate from F3 RPC
2. **Validate** - Full cryptographic validation (BLS, quorum, continuity)
3. **Convert** - Convert to Lotus format for proof generation
4. **Generate** - Create cryptographic proofs for parent chain data
5. **Cache** - Store in memory + RocksDB
6. **Serve** - Proposers query cache for instant proofs

## Testing

```bash
# Unit tests
cargo test --package fendermint_vm_topdown_proof_service

# Integration tests (requires live network)
cargo test --package fendermint_vm_topdown_proof_service --test integration -- --ignored
```

## Dependencies

- `filecoin-f3-lightclient` - F3 light client with crypto validation
- `filecoin-f3-certs` - F3 certificate types
- `ipc-filecoin-proofs` - Proof generation library
- `rocksdb` - Optional persistence layer

## Documentation

- `ARCHITECTURE.md` - Detailed architecture and design decisions
- `BLS_ISSUE.md` - BLS dependency analysis (resolved!)
- `F3_LIGHTCLIENT_FIX_NEEDED.md` - Fix applied to moshababo/rust-f3

## Related Code

- IPC Provider: `ipc/provider/src/lotus/message/f3.rs` - Lotus F3 types
- F3CertManager Actor: `fendermint/actors/f3-cert-manager` - On-chain certificate storage
- Fendermint App: Uses this service for topdown finality proofs
