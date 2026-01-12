# Storage Node

Consolidated storage functionality for IPC (InterPlanetary Consensus).

## Overview

This directory contains all storage-node related code, consolidated from the original `recall/` directory and scattered `fendermint/actors/` storage actors. The storage-node provides blob storage, read request handling, and Iroh-based content resolution for the IPC network.

## Structure

```
storage-node/
├── actors/                  # Storage actors (8 actors)
│   ├── adm/                # Autonomous Data Management
│   ├── adm_types/          # ADM type definitions
│   ├── machine/            # Machine base trait
│   ├── blobs/             # Main storage actor (with shared/ and testing/)
│   ├── blob_reader/        # Read-only blob access
│   ├── bucket/             # S3-like object storage
│   ├── timehub/            # Timestamping service
│   └── recall_config/      # Network configuration (with shared/)
│
├── core/                    # Core storage modules
│   ├── executor/           # RecallExecutor for FVM integration
│   ├── kernel/             # Kernel operations (with ops/ subcrate)
│   ├── syscalls/           # System calls
│   ├── sdk/                # Actor development SDK
│   └── ipld/               # IPLD data structures (AMT, HAMT)
│
├── iroh/                    # Iroh network integration
│   ├── manager/            # Iroh network manager
│   └── resolver/           # IPLD resolver with Iroh
│
├── integration/             # Integration API for IPC core
│   └── src/
│       ├── lib.rs          # Public integration API
│       ├── storage_env.rs  # Storage environment (pools, etc.)
│       ├── storage_helpers.rs  # Helper functions
│       └── actor_interface/    # Actor interface re-exports
│
└── contracts/               # Vendored Solidity facades (FVM 4.7)
    └── facade/
```

## Usage

### Enabling Storage Node

Storage-node functionality is **optional** and gated behind the `storage-node` feature flag.

#### In Cargo.toml

```toml
[dependencies]
fendermint = { version = "...", features = ["storage-node"] }
```

#### Via cargo commands

```bash
# Build with storage-node enabled
cargo build --features storage-node

# Run tests with storage-node
cargo test --features storage-node

# Build without storage-node (default)
cargo build
```

### Integration Points

The storage-node integrates with IPC core at the following points:

1. **Message Handling** (`fendermint/vm/interpreter/src/fvm/interpreter.rs`)
   - Handles `IpcMessage::ReadRequestPending` and `IpcMessage::ReadRequestClosed`
   - Feature-gated with `#[cfg(feature = "storage-node")]`

2. **Genesis Initialization** (`fendermint/vm/interpreter/src/genesis.rs`)
   - Initializes storage actors during genesis
   - Conditionally compiled when feature is enabled

3. **Service Startup** (`fendermint/app/src/service/node.rs`)
   - Starts storage services (blob pool, read request pool)
   - Only runs when feature is enabled

## Architecture

### Actors

**Core Actors** (always available when storage-node enabled):
- **blobs**: Main storage actor handling blob uploads, downloads, and lifecycle
- **blob_reader**: Read-only access to blob data
- **recall_config**: Network configuration and settings

**Optional Actors** (may be disabled):
- **adm**: Autonomous Data Management
- **machine**: Machine lifecycle management
- **bucket**: S3-compatible object storage interface
- **timehub**: Timestamping and time-based operations

### Core Modules

- **executor**: `RecallExecutor` - Custom FVM executor with storage operations
- **kernel**: Kernel-level operations for storage
- **syscalls**: System calls for storage operations
- **sdk**: SDK for building storage actors
- **ipld**: IPLD data structures (AMT, HAMT) for efficient storage

### Iroh Integration

- **manager**: Manages Iroh network nodes and connections
- **resolver**: Resolves IPLD content using Iroh's content-addressed storage

### Integration Layer

The `integration/` module provides a clean API for IPC core to interact with storage-node:

```rust
// Public API
use storage_node_integration::{
    BlobPool,
    ReadRequestPool,
    close_read_request,
    read_request_callback,
    set_read_request_pending,
};
```

## Development

### Building

```bash
# Build all storage-node crates
cd storage-node
cargo build --all

# Build specific actor
cargo build -p storage_node_actor_blobs

# Build with release optimizations
cargo build --release --all
```

### Testing

```bash
# Run all tests
cargo test --all

# Run tests for specific crate
cargo test -p storage_node_core_executor

# Run with verbose output
cargo test --all -- --nocapture
```

### Adding New Actors

1. Create actor directory under `actors/`
2. Add to `storage-node/Cargo.toml` workspace members
3. Add to main `Cargo.toml` workspace members
4. Implement actor using `storage_node_sdk`
5. Register actor in genesis initialization (feature-gated)

## Configuration

Storage-node behavior is configured through:

1. **Genesis Configuration**: Initial actor states and parameters
2. **Runtime Configuration**: Via `recall_config` actor
3. **Feature Flags**: Compile-time feature selection

## Dependencies

### External Dependencies
- **iroh**: Content-addressed storage and networking
- **FVM**: Filecoin Virtual Machine (v4.7)
- **fil_actors_runtime**: Filecoin actor runtime

### Internal Dependencies
- **ipc-api**: IPC core API
- **fendermint**: Core consensus and execution

## Versioning

- **FVM Version**: 4.7.4 (upgraded from 4.3)
- **Iroh Version**: 0.34+ (with netwatch socket2 0.5 compatibility patch)

## Migration Notes

This consolidation was performed to:
1. **Simplify structure**: All storage code in one location
2. **Improve modularity**: Clean separation via feature flags
3. **Reduce coupling**: Integration layer provides minimal API surface
4. **Enable flexibility**: Storage-node can be disabled entirely

### Changes from Original Structure

- `recall/` → `storage-node/core/`
- `fendermint/actors/{blobs,blob_reader,etc}` → `storage-node/actors/`
- `recall/iroh_manager` → `storage-node/iroh/manager/`
- `fendermint/vm/iroh_resolver` → `storage-node/iroh/resolver/`
- `recall-contracts/crates/facade` → `storage-node/contracts/facade/`

### Breaking Changes

Crate names have been updated for consistency:
- `recall_ipld` → `storage_node_ipld`
- `recall_actor_sdk` → `storage_node_sdk`
- `iroh_manager` → `storage_node_iroh_manager`
- `fendermint_vm_iroh_resolver` → `storage_node_iroh_resolver`

## Contributing

When contributing to storage-node:

1. **Maintain feature flag discipline**: All storage code should be feature-gated
2. **Update integration layer**: Public API changes go through `integration/`
3. **Test both configurations**: Ensure compilation with and without `storage-node` feature
4. **Document actor changes**: Update this README for new actors or major changes

## License

MIT OR Apache-2.0

## See Also

- [Storage Consolidation Plan](../docs/development/STORAGE_NODE_CONSOLIDATION_PLAN.md)
- [Storage Consolidation Audit](../docs/development/STORAGE_CONSOLIDATION_AUDIT.md)
- [Recall Deployment Guide](../RECALL_DEPLOYMENT_GUIDE.md)

