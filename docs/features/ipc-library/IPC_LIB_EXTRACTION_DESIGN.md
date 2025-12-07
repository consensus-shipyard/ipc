# IPC Library Extraction - Design Document

## Executive Summary

This document outlines a strategy to extract core IPC functionality into a unified `ipc-lib` crate that can be shared between the CLI (`ipc-cli`) and node (`fendermint`), reducing code duplication and creating a cleaner architectural separation.

**Goal:** Create a reusable, well-documented library that encapsulates IPC core functionality, enabling:
- Easier maintenance (single source of truth)
- Better testability
- Third-party integration capabilities
- Clearer architectural boundaries

**Estimated Effort:** 4-6 weeks
**Risk Level:** Medium (requires careful dependency management)

---

## Table of Contents

1. [Current Architecture Analysis](#current-architecture-analysis)
2. [Proposed Architecture](#proposed-architecture)
3. [What Goes Into ipc-lib](#what-goes-into-ipc-lib)
4. [Migration Strategy](#migration-strategy)
5. [Implementation Phases](#implementation-phases)
6. [API Design](#api-design)
7. [Testing Strategy](#testing-strategy)
8. [Backward Compatibility](#backward-compatibility)

---

## 1. Current Architecture Analysis

### 1.1 Existing IPC Crates

| Crate | Lines | Purpose | Used By |
|-------|-------|---------|---------|
| `ipc/api` | ~3,000 | Common types (SubnetID, Checkpoint, Gateway, etc.) | CLI, fendermint (31 files) |
| `ipc/provider` | ~8,000 | Core provider implementation (subnet ops, checkpoints) | CLI, fendermint (11 files) |
| `ipc/wallet` | ~2,000 | Key management (EVM + FVM wallets) | CLI, fendermint |
| `ipc/types` | ~1,500 | Basic types (ethaddr, uints, keys, etc.) | CLI, fendermint |
| `ipc/observability` | ~500 | Tracing and metrics | CLI, fendermint |
| `ipc/cli` | ~15,000 | CLI commands | End users |

**Total IPC functionality:** ~30,000 lines

### 1.2 Current Dependency Flow

```
┌─────────────────────────────────────────────────────────┐
│                    End Users                            │
└──────────────────┬──────────────────────────────────────┘
                   │
         ┌─────────┴──────────┐
         │                    │
┌────────▼────────┐   ┌───────▼────────┐
│   ipc-cli       │   │  fendermint    │
│   (CLI tool)    │   │    (node)      │
└────────┬────────┘   └───────┬────────┘
         │                    │
         │    ┌───────────────┤
         │    │               │
    ┌────▼────▼────┐   ┌──────▼─────────┐
    │ ipc-provider │   │ fendermint/vm  │
    │              │   │ fendermint/app │
    └────┬─────────┘   └──────┬─────────┘
         │                    │
    ┌────▼────────────────────▼────┐
    │        ipc-api                │
    │      ipc-wallet               │
    │      ipc-types                │
    └───────────────────────────────┘
```

**Issues with Current Architecture:**

1. **Tight Coupling:** CLI and fendermint both depend on low-level provider details
2. **Code Duplication:**
   - Both implement similar RPC clients
   - Both handle genesis file parsing
   - Both manage subnet configurations
3. **Unclear Boundaries:** Provider contains business logic mixed with I/O operations
4. **Limited Reusability:** Hard for third parties to integrate IPC functionality

### 1.3 Overlap Analysis

| Functionality | In CLI | In Fendermint | Shared via Provider |
|--------------|---------|---------------|---------------------|
| Subnet operations | ✅ | ✅ | ✅ (partially) |
| Checkpoint management | ✅ | ✅ | ✅ |
| Cross-chain messaging | ✅ | ✅ | ✅ |
| Gateway interactions | ✅ | ✅ | ✅ |
| Genesis handling | ✅ | ✅ | ❌ (duplicated) |
| RPC clients | ✅ | ✅ | ✅ (partially) |
| Config management | ✅ | ✅ | ❌ (duplicated) |
| Wallet operations | ✅ | ✅ | ✅ |
| Contract deployment | ✅ | ✅ | ❌ (duplicated) |
| Ethereum utilities | ✅ | ✅ | ❌ (duplicated) |

**~40% of functionality is duplicated or poorly shared.**

---

## 2. Proposed Architecture

### 2.1 Target Architecture

```
┌──────────────────────────────────────────────────────────┐
│                    End Users                             │
└───────────────────┬──────────────────────────────────────┘
                    │
          ┌─────────┴──────────┐
          │                    │
┌─────────▼────────┐   ┌───────▼────────┐
│   ipc-cli        │   │  fendermint    │
│   (thin shell)   │   │    (thin app)  │
└─────────┬────────┘   └───────┬────────┘
          │                    │
          └────────┬───────────┘
                   │
          ┌────────▼────────┐
          │    ipc-lib      │
          │  (Core Library) │
          └────────┬────────┘
                   │
          ┌────────┴────────────────────┐
          │                             │
    ┌─────▼──────┐             ┌────────▼────────┐
    │  ipc-core  │             │ ipc-contracts   │
    │ (Runtime)  │             │   (Bindings)    │
    └─────┬──────┘             └────────┬────────┘
          │                             │
          └──────────┬──────────────────┘
                     │
            ┌────────▼────────┐
            │   ipc-types     │
            │   ipc-wallet    │
            │ ipc-observability│
            └─────────────────┘
```

### 2.2 New Component Structure

#### `ipc-lib` (NEW - Unified Library)
**Purpose:** High-level API for IPC operations
**Lines:** ~12,000 (consolidates existing code)
**Exports:**
- `SubnetClient` - Interact with subnets
- `CheckpointManager` - Manage checkpoints
- `CrossMessageHandler` - Cross-chain messaging
- `GatewayManager` - Gateway interactions
- `GenesisBuilder` - Genesis file creation
- `ConfigManager` - Configuration management

#### `ipc-core` (REFACTORED from `ipc-provider`)
**Purpose:** Core runtime and business logic
**Lines:** ~6,000
**Exports:**
- Low-level substrate operations
- RPC client abstractions
- Transaction building
- State queries

#### `ipc-contracts` (NEW - from `contract-bindings` + deployer logic)
**Purpose:** Smart contract interactions
**Lines:** ~3,000
**Exports:**
- Contract bindings
- Deployment utilities
- ABI encoders/decoders

---

## 3. What Goes Into ipc-lib

### 3.1 Core Modules

#### **Subnet Module** (`ipc-lib/subnet`)
Consolidates all subnet-related operations:

```rust
// High-level subnet operations
pub mod subnet {
    pub struct SubnetClient {
        provider: Arc<dyn Provider>,
        wallet: Option<Arc<Wallet>>,
    }

    impl SubnetClient {
        // Create new subnet
        pub async fn create(
            &self,
            config: SubnetConfig,
        ) -> Result<SubnetID>;

        // Join existing subnet
        pub async fn join(
            &self,
            subnet_id: SubnetID,
            validator_stake: TokenAmount,
        ) -> Result<()>;

        // Leave subnet
        pub async fn leave(&self, subnet_id: SubnetID) -> Result<()>;

        // Query subnet info
        pub async fn get_info(&self, subnet_id: SubnetID) -> Result<SubnetInfo>;

        // List all subnets
        pub async fn list(&self) -> Result<Vec<SubnetInfo>>;
    }
}
```

**Sources:**
- `ipc-cli/src/commands/subnet/*` (create, join, leave, list)
- `fendermint/app/src/ipc.rs`
- `ipc-provider/src/manager/subnet.rs`

#### **Checkpoint Module** (`ipc-lib/checkpoint`)
Checkpoint creation, validation, and submission:

```rust
pub mod checkpoint {
    pub struct CheckpointManager {
        gateway: GatewayContract,
        provider: Arc<dyn Provider>,
    }

    impl CheckpointManager {
        // Create checkpoint from state
        pub async fn create(
            &self,
            subnet_id: SubnetID,
            height: BlockHeight,
        ) -> Result<Checkpoint>;

        // Submit checkpoint to parent
        pub async fn submit(
            &self,
            checkpoint: Checkpoint,
        ) -> Result<TxHash>;

        // Validate checkpoint
        pub fn validate(&self, checkpoint: &Checkpoint) -> Result<()>;

        // List pending checkpoints
        pub async fn list_pending(
            &self,
            subnet_id: SubnetID,
        ) -> Result<Vec<Checkpoint>>;
    }
}
```

**Sources:**
- `ipc-cli/src/commands/checkpoint/*`
- `ipc-provider/src/checkpoint.rs`
- `fendermint/vm/topdown/src/*`

#### **Cross-Chain Messaging Module** (`ipc-lib/crossmsg`)
Handle cross-subnet message passing:

```rust
pub mod crossmsg {
    pub struct CrossMessageHandler {
        gateway: GatewayContract,
        wallet: Arc<Wallet>,
    }

    impl CrossMessageHandler {
        // Send cross-chain message
        pub async fn send(
            &self,
            target: SubnetID,
            message: CrossMsg,
        ) -> Result<TxHash>;

        // Fund cross-chain message
        pub async fn fund(
            &self,
            subnet_id: SubnetID,
            amount: TokenAmount,
        ) -> Result<TxHash>;

        // Release funds
        pub async fn release(&self, subnet_id: SubnetID) -> Result<TxHash>;

        // Propagate messages
        pub async fn propagate(
            &self,
            messages: Vec<CrossMsg>,
        ) -> Result<Vec<TxHash>>;
    }
}
```

**Sources:**
- `ipc-cli/src/commands/crossmsg/*`
- `fendermint/vm/interpreter/src/fvm/state/ipc.rs`
- `ipc-api/src/cross.rs`

#### **Genesis Module** (`ipc-lib/genesis`)
Genesis file creation and management:

```rust
pub mod genesis {
    pub struct GenesisBuilder {
        chain_name: String,
        validators: Vec<Validator>,
        config: GenesisConfig,
    }

    impl GenesisBuilder {
        pub fn new(chain_name: String) -> Self;

        pub fn add_validator(&mut self, validator: Validator) -> &mut Self;

        pub fn set_accounts(&mut self, accounts: Vec<Account>) -> &mut Self;

        pub fn set_eam_permission_mode(&mut self, mode: PermissionMode) -> &mut Self;

        pub fn build(&self) -> Result<Genesis>;

        pub fn write_to_file(&self, path: &Path) -> Result<()>;
    }

    // Load and parse genesis
    pub fn load_genesis(path: &Path) -> Result<Genesis>;
}
```

**Sources:**
- `ipc-cli/src/commands/subnet/create_genesis.rs`
- `fendermint/app/src/cmd/genesis.rs`
- `fendermint/vm/genesis/src/lib.rs`

#### **Gateway Module** (`ipc-lib/gateway`)
Gateway contract interactions:

```rust
pub mod gateway {
    pub struct GatewayManager {
        contract: GatewayContract,
        provider: Arc<dyn Provider>,
    }

    impl GatewayManager {
        pub async fn deploy(
            provider: Arc<dyn Provider>,
            params: GatewayParams,
        ) -> Result<Self>;

        pub async fn get_subnet(
            &self,
            subnet_id: SubnetID,
        ) -> Result<Option<SubnetInfo>>;

        pub async fn register_subnet(
            &self,
            subnet: SubnetConfig,
        ) -> Result<TxHash>;

        pub async fn fund(&self, subnet_id: SubnetID, amount: TokenAmount) -> Result<TxHash>;
    }
}
```

**Sources:**
- `ipc-cli/src/commands/subnet/*`
- `ipc-api/src/gateway.rs`
- `fendermint/eth/deployer/src/lib.rs`

#### **Configuration Module** (`ipc-lib/config`)
Unified configuration management:

```rust
pub mod config {
    pub struct ConfigManager {
        base_path: PathBuf,
    }

    impl ConfigManager {
        pub fn new(base_path: PathBuf) -> Self;

        // Subnet configuration
        pub fn load_subnet_config(&self, subnet_id: &SubnetID) -> Result<SubnetConfig>;
        pub fn save_subnet_config(&self, config: &SubnetConfig) -> Result<()>;

        // Node configuration
        pub fn load_node_config(&self) -> Result<NodeConfig>;
        pub fn save_node_config(&self, config: &NodeConfig) -> Result<()>;

        // Wallet configuration
        pub fn get_default_wallet(&self) -> Result<Option<Address>>;
        pub fn set_default_wallet(&self, address: Address) -> Result<()>;
    }
}
```

**Sources:**
- `ipc-cli/src/ipc_config_store.rs`
- `ipc-provider/src/config/*`
- `fendermint/app/settings/src/*`

### 3.2 Support Modules

#### **RPC Client Abstraction** (`ipc-lib/rpc`)

```rust
pub mod rpc {
    #[async_trait]
    pub trait Provider: Send + Sync {
        async fn get_block(&self, height: BlockHeight) -> Result<Block>;
        async fn send_transaction(&self, tx: Transaction) -> Result<TxHash>;
        async fn query_state(&self, path: &str) -> Result<Vec<u8>>;
    }

    pub struct EthProvider { /* ... */ }
    pub struct TendermintProvider { /* ... */ }
    pub struct LotusProvider { /* ... */ }
}
```

#### **Contract Utilities** (`ipc-lib/contracts`)

```rust
pub mod contracts {
    pub struct ContractDeployer {
        provider: Arc<dyn Provider>,
        wallet: Arc<Wallet>,
    }

    impl ContractDeployer {
        pub async fn deploy_gateway(
            &self,
            params: GatewayParams,
        ) -> Result<Address>;

        pub async fn deploy_registry(
            &self,
            gateway: Address,
        ) -> Result<Address>;
    }
}
```

---

## 4. Migration Strategy

### 4.1 Dependency Graph

**Current Dependencies:**
```
ipc-cli
  ├─> ipc-provider
  ├─> ipc-api
  ├─> ipc-wallet
  ├─> ipc-types
  └─> fendermint (for genesis, eth deployer)

fendermint
  ├─> ipc-provider (11 files)
  ├─> ipc-api (31 files)
  ├─> ipc-wallet
  └─> ipc-types
```

**Target Dependencies:**
```
ipc-cli
  └─> ipc-lib

fendermint
  ├─> ipc-lib (for subnet operations)
  └─> ipc-core (for low-level runtime)

ipc-lib
  ├─> ipc-core
  ├─> ipc-contracts
  ├─> ipc-api
  ├─> ipc-wallet
  └─> ipc-types
```

### 4.2 What Stays Where

#### **Stays in CLI:**
- Command-line parsing (clap)
- Terminal UI/formatting
- Interactive prompts
- CLI-specific services (comet_runner, daemon mode)

#### **Stays in Fendermint:**
- ABCI application logic
- FVM interpreter
- Tendermint integration
- Actor implementations
- State machine execution
- Block production

#### **Moves to ipc-lib:**
- Subnet operations
- Checkpoint management
- Cross-chain messaging
- Gateway interactions
- Genesis building
- Configuration management
- Contract deployment utilities

#### **Stays in ipc-core:**
- RPC client abstractions
- Transaction building
- Signature creation
- Low-level queries
- Provider implementations (EVM, CometBFT, Lotus)

---

## 5. Implementation Phases

### Phase 1: Setup & Planning (Week 1)
**Goal:** Create library structure and plan API surface

**Tasks:**
1. Create `ipc-lib` crate with module structure
2. Define public API interfaces
3. Audit all CLI and fendermint code for extractable functionality
4. Create migration checklist
5. Set up testing framework

**Deliverables:**
- `ipc-lib/` directory with stub modules
- API documentation (rustdoc)
- Migration plan spreadsheet

**Risk:** Low

---

### Phase 2: Extract Core Types & Utilities (Week 1-2)
**Goal:** Move non-controversial shared code

**Tasks:**
1. Extract RPC client abstractions
2. Move configuration types
3. Extract contract utilities
4. Create common error types
5. Set up observability integration

**Files to Move:**
- `ipc-provider/src/jsonrpc/*` → `ipc-lib/rpc`
- `ipc-provider/src/config/*` → `ipc-lib/config`
- `ipc-cli/src/ipc_config_store.rs` → `ipc-lib/config`

**Deliverables:**
- `ipc-lib::rpc` module
- `ipc-lib::config` module
- `ipc-lib::error` module

**Risk:** Low

---

### Phase 3: Extract Subnet Operations (Week 2-3)
**Goal:** Consolidate subnet management

**Tasks:**
1. Create `SubnetClient` API
2. Move subnet creation logic
3. Move join/leave operations
4. Integrate with provider
5. Add comprehensive tests

**Files to Consolidate:**
- `ipc-cli/src/commands/subnet/*`
- `ipc-provider/src/manager/subnet.rs`
- `fendermint/app/src/ipc.rs`

**Deliverables:**
- `ipc-lib::subnet` module
- Integration tests
- API documentation

**Risk:** Medium (touches multiple systems)

---

### Phase 4: Extract Checkpoint & CrossMsg (Week 3-4)
**Goal:** Consolidate checkpoint and cross-chain messaging

**Tasks:**
1. Create `CheckpointManager` API
2. Create `CrossMessageHandler` API
3. Move checkpoint creation logic
4. Move cross-chain message handling
5. Add validation logic

**Files to Consolidate:**
- `ipc-cli/src/commands/checkpoint/*`
- `ipc-cli/src/commands/crossmsg/*`
- `ipc-provider/src/checkpoint.rs`
- `fendermint/vm/topdown/src/*` (checkpoint parts)

**Deliverables:**
- `ipc-lib::checkpoint` module
- `ipc-lib::crossmsg` module
- Integration tests

**Risk:** Medium-High (consensus-critical code)

---

### Phase 5: Extract Genesis & Gateway (Week 4-5)
**Goal:** Consolidate genesis and gateway management

**Tasks:**
1. Create `GenesisBuilder` API
2. Create `GatewayManager` API
3. Move genesis creation from CLI
4. Move genesis logic from fendermint
5. Extract contract deployment

**Files to Consolidate:**
- `ipc-cli/src/commands/subnet/create_genesis.rs`
- `fendermint/app/src/cmd/genesis.rs`
- `fendermint/vm/genesis/src/lib.rs` (parts)
- `fendermint/eth/deployer/src/lib.rs`

**Deliverables:**
- `ipc-lib::genesis` module
- `ipc-lib::gateway` module
- `ipc-lib::contracts` module

**Risk:** Medium (genesis is critical)

---

### Phase 6: Refactor CLI (Week 5-6)
**Goal:** Update CLI to use ipc-lib

**Tasks:**
1. Replace direct provider calls with ipc-lib
2. Simplify command implementations
3. Remove duplicated code
4. Update error handling
5. Add new examples

**Changes:**
- Rewrite `ipc-cli/src/commands/*` to use ipc-lib APIs
- Remove `fendermint` dependencies from CLI
- Simplify `Cargo.toml`

**Deliverables:**
- Updated CLI using ipc-lib
- Reduced CLI codebase (~30% reduction expected)
- Updated documentation

**Risk:** Low (CLI is leaf dependency)

---

### Phase 7: Refactor Fendermint (Week 6)
**Goal:** Update fendermint to use ipc-lib where appropriate

**Tasks:**
1. Replace subnet operations with ipc-lib calls
2. Use ipc-lib for genesis building
3. Keep low-level operations in fendermint/vm
4. Update integration tests

**Changes:**
- Update `fendermint/app/src/ipc.rs`
- Update `fendermint/app/src/cmd/genesis.rs`
- Simplify topdown module

**Deliverables:**
- Updated fendermint using ipc-lib
- Passing integration tests
- Updated documentation

**Risk:** Medium (node is critical infrastructure)

---

### Phase 8: Documentation & Polish (Ongoing)
**Goal:** Comprehensive documentation and examples

**Tasks:**
1. Write rustdoc for all public APIs
2. Create usage examples
3. Write migration guide
4. Create quickstart guide
5. Add integration examples

**Deliverables:**
- Complete API documentation
- `examples/` directory with working code
- Migration guide for users
- Updated README

**Risk:** Low

---

## 6. API Design

### 6.1 Client Builder Pattern

```rust
use ipc_lib::{IpcClient, NetworkType};

// Create client for existing subnet
let client = IpcClient::builder()
    .network(NetworkType::Calibration)
    .subnet_id("/r314159/t01234")
    .rpc_url("https://api.node.glif.io")
    .wallet_path("~/.ipc/wallet")
    .build()
    .await?;

// Create subnet
let new_subnet = client
    .subnet()
    .create()
    .name("my-subnet")
    .min_validators(3)
    .stake_requirement(TokenAmount::from_fil(10))
    .execute()
    .await?;

// Join subnet as validator
client
    .subnet()
    .join(new_subnet.id)
    .stake(TokenAmount::from_fil(100))
    .public_key(validator_key)
    .execute()
    .await?;
```

### 6.2 High-Level Operations

```rust
// Checkpoint submission
let checkpoint = client
    .checkpoint()
    .create_from_height(subnet_id, height)
    .await?;

let tx_hash = client
    .checkpoint()
    .submit(checkpoint)
    .await?;

// Cross-chain messaging
let msg_hash = client
    .crossmsg()
    .send_to(target_subnet)
    .value(TokenAmount::from_fil(1))
    .data(payload)
    .execute()
    .await?;

// Gateway operations
let gateway = client
    .gateway()
    .deploy()
    .with_params(params)
    .execute()
    .await?;
```

### 6.3 Genesis Builder

```rust
use ipc_lib::genesis::{GenesisBuilder, PermissionMode};

let genesis = GenesisBuilder::new("my-chain")
    .chain_id(123)
    .add_validator(Validator {
        address: addr1,
        power: 100,
    })
    .add_validator(Validator {
        address: addr2,
        power: 100,
    })
    .add_account(Account {
        address: user1,
        balance: TokenAmount::from_fil(1000),
    })
    .eam_permission_mode(PermissionMode::Allowlist)
    .build()?;

genesis.write_to_file("genesis.json")?;
```

### 6.4 Configuration Management

```rust
use ipc_lib::config::ConfigManager;

let config = ConfigManager::new("~/.ipc")?;

// Save subnet configuration
config.save_subnet_config(&SubnetConfig {
    id: subnet_id,
    rpc_url: "https://subnet-rpc.example.com",
    gateway_address: gateway_addr,
})?;

// Load configuration
let subnet_config = config.load_subnet_config(&subnet_id)?;

// Manage default wallet
config.set_default_wallet(my_address)?;
```

---

## 7. Testing Strategy

### 7.1 Unit Tests

Each module must have comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subnet_creation() {
        let mock_provider = MockProvider::new();
        let client = SubnetClient::new(Arc::new(mock_provider), None);

        let result = client.create(SubnetConfig {
            name: "test-subnet".into(),
            min_validators: 1,
            // ...
        }).await;

        assert!(result.is_ok());
    }
}
```

### 7.2 Integration Tests

Test real workflows end-to-end:

```rust
#[tokio::test]
#[ignore] // Requires testnet
async fn test_subnet_lifecycle() {
    let client = IpcClient::builder()
        .network(NetworkType::Testnet)
        .build()
        .await?;

    // Create subnet
    let subnet = client.subnet().create(/* ... */).await?;

    // Join as validator
    client.subnet().join(subnet.id, stake).await?;

    // Verify subnet state
    let info = client.subnet().get_info(subnet.id).await?;
    assert_eq!(info.validators.len(), 1);

    // Leave subnet
    client.subnet().leave(subnet.id).await?;
}
```

### 7.3 Mock Providers

Create mock implementations for testing:

```rust
pub struct MockProvider {
    responses: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl MockProvider {
    pub fn with_response(mut self, key: &str, value: Vec<u8>) -> Self {
        self.responses.lock().unwrap().insert(key.into(), value);
        self
    }
}

#[async_trait]
impl Provider for MockProvider {
    async fn query_state(&self, path: &str) -> Result<Vec<u8>> {
        self.responses
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| anyhow!("not found"))
    }
}
```

### 7.4 Compatibility Tests

Ensure CLI and fendermint work with new library:

```bash
# Run CLI tests against ipc-lib
cargo test -p ipc-cli

# Run fendermint tests
cargo test -p fendermint_app

# Run integration tests
cargo test --test integration_tests
```

---

## 8. Backward Compatibility

### 8.1 Transition Period

Maintain both old and new APIs during transition:

```rust
// Old API (deprecated)
#[deprecated(since = "0.2.0", note = "use ipc_lib::SubnetClient instead")]
pub use ipc_provider::manager::subnet::SubnetManager;

// New API
pub use ipc_lib::subnet::SubnetClient;
```

### 8.2 Feature Flags

Allow gradual adoption:

```toml
[features]
default = ["legacy-api"]
legacy-api = ["ipc-provider"]
new-api = ["ipc-lib"]
```

### 8.3 Migration Path

Provide clear migration guide:

```markdown
# Migrating from ipc-provider to ipc-lib

## Old Code
```rust
use ipc_provider::manager::subnet::SubnetManager;

let manager = SubnetManager::new(provider);
let subnet = manager.create_subnet(params).await?;
```

## New Code
```rust
use ipc_lib::IpcClient;

let client = IpcClient::builder()
    .provider(provider)
    .build()
    .await?;

let subnet = client.subnet().create(params).await?;
```
```

---

## 9. File Structure

### 9.1 New Directory Layout

```
ipc/
├── api/           (existing - types)
├── types/         (existing - basic types)
├── wallet/        (existing - key management)
├── observability/ (existing - tracing)
├── core/          (RENAMED from provider)
│   ├── rpc/       (low-level RPC)
│   ├── provider/  (provider implementations)
│   └── manager/   (business logic)
└── lib/           (NEW - high-level API)
    ├── src/
    │   ├── lib.rs
    │   ├── client.rs       (IpcClient)
    │   ├── subnet.rs       (SubnetClient)
    │   ├── checkpoint.rs   (CheckpointManager)
    │   ├── crossmsg.rs     (CrossMessageHandler)
    │   ├── gateway.rs      (GatewayManager)
    │   ├── genesis.rs      (GenesisBuilder)
    │   ├── config.rs       (ConfigManager)
    │   ├── contracts.rs    (ContractDeployer)
    │   ├── error.rs        (unified errors)
    │   └── prelude.rs      (common imports)
    ├── tests/
    │   ├── subnet_tests.rs
    │   ├── checkpoint_tests.rs
    │   └── integration_tests.rs
    ├── examples/
    │   ├── create_subnet.rs
    │   ├── join_subnet.rs
    │   └── submit_checkpoint.rs
    └── Cargo.toml

ipc-cli/
├── src/
│   ├── main.rs
│   ├── commands/       (simplified)
│   └── cli.rs
└── Cargo.toml          (simpler deps)

fendermint/
└── (unchanged structure, updated imports)
```

---

## 10. Benefits & Trade-offs

### 10.1 Benefits

✅ **Reduced Code Duplication**
- ~40% reduction in duplicated code
- Single source of truth for subnet operations

✅ **Clearer Architecture**
- Well-defined API boundaries
- Separation of concerns (high-level vs low-level)

✅ **Better Testing**
- Mockable interfaces
- Isolated unit tests
- Integration test suite

✅ **Third-Party Integration**
- Clear public API
- Comprehensive documentation
- Example code

✅ **Easier Maintenance**
- Changes in one place
- Consistent error handling
- Unified logging/observability

✅ **Smaller Binaries**
- CLI doesn't need fendermint dependencies
- Can build with only needed features

### 10.2 Trade-offs

⚠️ **Initial Development Cost**
- 4-6 weeks of focused work
- Requires careful API design
- Testing overhead

⚠️ **Migration Complexity**
- Both CLI and fendermint must be updated
- Risk of breaking changes during transition
- Need backward compatibility

⚠️ **Additional Abstraction Layer**
- One more level of indirection
- Potential performance overhead (minimal)

⚠️ **Version Synchronization**
- Need to coordinate releases
- Breaking changes affect multiple components

---

## 11. Success Criteria

### 11.1 Metrics

| Metric | Target |
|--------|--------|
| Code duplication reduction | >35% |
| CLI binary size reduction | >20% |
| Test coverage (ipc-lib) | >80% |
| API documentation completeness | 100% |
| Migration issues | <10 breaking changes |

### 11.2 Acceptance Criteria

- [ ] All CLI commands work with ipc-lib
- [ ] All fendermint operations work with ipc-lib
- [ ] No performance regression
- [ ] All tests passing
- [ ] Complete API documentation
- [ ] At least 5 working examples
- [ ] Migration guide published
- [ ] Backward compatibility maintained for 1 release

---

## 12. Rollout Plan

### 12.1 Alpha Release (Week 4)

**Version:** `0.1.0-alpha`
- Core modules available
- Basic functionality working
- Internal testing only

### 12.2 Beta Release (Week 5)

**Version:** `0.1.0-beta`
- CLI migrated
- Fendermint partially migrated
- External testing with select users

### 12.3 Release Candidate (Week 6)

**Version:** `0.1.0-rc`
- All migrations complete
- Full test suite passing
- Documentation complete

### 12.4 Stable Release (Week 7)

**Version:** `0.1.0`
- Production ready
- Backward compatibility layer
- Deprecation notices for old APIs

### 12.5 Migration Complete (Week 8+)

**Version:** `0.2.0`
- Remove deprecated APIs
- Full ipc-lib adoption
- Performance optimizations

---

## 13. Risk Mitigation

### 13.1 Technical Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking existing functionality | High | Comprehensive test suite, gradual rollout |
| Performance regression | Medium | Benchmarking, profiling |
| API design issues | Medium | Early feedback, iterative design |
| Circular dependencies | Low | Careful dependency planning |

### 13.2 Organizational Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| User migration issues | Medium | Clear documentation, backward compatibility |
| Disruption to development | Medium | Feature freeze during migration |
| Third-party integrations | Low | Version pinning, communication |

---

## 14. Future Enhancements

### Post-1.0 Features

1. **Plugin System**
   - Allow third-party extensions
   - Custom provider implementations

2. **Advanced Query API**
   - GraphQL endpoint
   - Historical queries
   - Real-time subscriptions

3. **Multi-Language Bindings**
   - Python bindings (PyO3)
   - JavaScript/TypeScript (WASM)
   - Go bindings (cgo)

4. **Enhanced Observability**
   - OpenTelemetry integration
   - Distributed tracing
   - Performance metrics

---

## Appendix A: Code Size Estimates

| Component | Current Lines | After Refactor | Change |
|-----------|---------------|----------------|--------|
| ipc-api | ~3,000 | ~3,000 | 0% |
| ipc-provider | ~8,000 | ~6,000 (ipc-core) | -25% |
| ipc-cli | ~15,000 | ~10,000 | -33% |
| fendermint (IPC parts) | ~5,000 | ~3,500 | -30% |
| **ipc-lib (NEW)** | 0 | ~12,000 | +100% |
| **Total** | ~31,000 | ~34,500 | +11% |

**Net Result:** +11% total code, but ~35% reduction in duplication.

---

## Appendix B: Example Migration

### Before (CLI):

```rust
// ipc-cli/src/commands/subnet/create.rs (simplified)
pub async fn create_subnet(args: CreateArgs) -> Result<()> {
    let provider = ipc_provider::manager::evm::manager::EvmSubnetManager::new(
        args.gateway_addr,
        args.registry_addr,
    );

    let config = SubnetConfig {
        name: args.name,
        min_validators: args.min_validators,
        // ... 50 more lines ...
    };

    let subnet_id = provider.create_subnet(config).await?;
    println!("Created subnet: {}", subnet_id);
    Ok(())
}
```

### After (CLI):

```rust
// ipc-cli/src/commands/subnet/create.rs (simplified)
pub async fn create_subnet(args: CreateArgs) -> Result<()> {
    let client = IpcClient::from_env().await?;

    let subnet = client
        .subnet()
        .create()
        .name(args.name)
        .min_validators(args.min_validators)
        .execute()
        .await?;

    println!("Created subnet: {}", subnet.id);
    Ok(())
}
```

**Result:** ~60% reduction in code, clearer intent, easier to test.

---

**Document Version:** 1.0
**Created:** December 4, 2024
**Estimated Completion:** Q1 2025
**Status:** Proposed
