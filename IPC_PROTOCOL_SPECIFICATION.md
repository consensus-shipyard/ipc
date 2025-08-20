# InterPlanetary Consensus (IPC) Protocol Specification

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Architecture Overview](#architecture-overview)
3. [Core Components](#core-components)
4. [Addressing and Identity](#addressing-and-identity)
5. [Subnet Lifecycle](#subnet-lifecycle)
6. [Cross-Chain Communication](#cross-chain-communication)
7. [Consensus and Validation](#consensus-and-validation)
8. [Security Model](#security-model)
9. [Network Protocols](#network-protocols)
10. [Governance and Upgrades](#governance-and-upgrades)
11. [Economic Model](#economic-model)
12. [Implementation Details](#implementation-details)

---

## 1. Executive Summary

**InterPlanetary Consensus (IPC)** is a framework for on-demand horizontal scalability of blockchain networks through the deployment of hierarchical subnets. IPC enables applications to achieve planetary scale through recursively scalable subnets, sub-second transactions, robust compute workloads, and highly adaptable WebAssembly runtimes tailored to developer requirements.

### Key Characteristics

- **Hierarchical Architecture**: Subnets are organized in a tree structure, with each subnet capable of spawning child subnets
- **Recursive Scalability**: The protocol is designed as a set of simple parent-child interactions that can be repeated at any level
- **Cross-Chain Messaging**: Native support for secure message passing between parent and child subnets
- **Consensus Flexibility**: Each subnet can choose its own consensus mechanism while maintaining interoperability
- **Economic Integration**: Native token transfers and shared economic security model across the hierarchy

### Protocol Goals

1. **Horizontal Scalability**: Enable unlimited scaling through subnet deployment
2. **Application Flexibility**: Allow applications to choose optimal consensus parameters
3. **Security Inheritance**: Provide configurable security models from full inheritance to independence
4. **Interoperability**: Ensure seamless communication between different layers of the hierarchy
5. **Developer Experience**: Provide familiar Ethereum-compatible execution environments

---

## 2. Architecture Overview

IPC follows a hierarchical subnet architecture where each subnet is an independent blockchain that can spawn child subnets while remaining connected to its parent subnet for security and communication.

### 2.1 Network Topology

```
Rootnet (Filecoin/Ethereum)
├── Subnet A (Layer 2)
│   ├── Subnet A1 (Layer 3)
│   └── Subnet A2 (Layer 3)
└── Subnet B (Layer 2)
    └── Subnet B1 (Layer 3)
        └── Subnet B1a (Layer 4)
```

### 2.2 Architectural Principles

1. **Parent-Child Relationship**: Each subnet has exactly one parent (except the rootnet)
2. **Recursive Design**: Protocol operations are consistent across all hierarchy levels
3. **Autonomous Operation**: Subnets operate independently but can interact with parents/children
4. **Shared Security**: Security can be inherited from parents or independently maintained
5. **Economic Continuity**: Value can flow freely up and down the hierarchy

### 2.3 Key Components

- **Gateway Actor**: Core IPC logic deployed in every subnet
- **Subnet Actor**: Parent-side representation of each child subnet
- **Registry**: Factory for deploying subnet actors
- **Relayers**: Infrastructure for cross-subnet message delivery
- **Fendermint**: Reference consensus implementation using Tendermint + FVM

---

## 3. Core Components

### 3.1 Gateway Actor (IGA)

The **IPC Gateway Actor** is a singleton smart contract deployed in every IPC subnet that implements the core IPC protocol logic.

#### Responsibilities

- **Subnet Registration**: Register child subnets in the hierarchy
- **Cross-Chain Messaging**: Handle incoming and outgoing cross-subnet messages
- **Checkpoint Management**: Create and validate bottom-up checkpoints
- **Collateral Management**: Track and enforce collateral requirements
- **Validator Set Management**: Coordinate validator changes with parent subnet
- **Finality Tracking**: Monitor and apply parent finality

#### Key Methods

```solidity
interface IGateway {
    // Subnet management
    function register(uint256 genesisCircSupply, uint256 collateral) external payable;
    function kill() external;

    // Cross-chain messaging
    function fund(SubnetID calldata subnetId, FvmAddress calldata to) external payable;
    function release(FvmAddress calldata to) external payable;
    function sendContractXnetMessage(IpcEnvelope calldata envelope) external payable;

    // Consensus coordination
    function commitCheckpoint(BottomUpCheckpoint calldata checkpoint) external;
    function commitParentFinality(ParentFinality calldata finality) external;

    // Validator management
    function addStake(uint256 amount) external payable;
    function releaseStake(uint256 amount) external;
}
```

### 3.2 Subnet Actor (ISA)

The **IPC Subnet Actor** is deployed in the parent subnet to represent and manage a specific child subnet.

#### Responsibilities

- **Child Subnet Representation**: Serve as the parent-side endpoint for a child subnet
- **Validator Management**: Track validator set and stake for the child subnet
- **Checkpoint Validation**: Verify and process bottom-up checkpoints from the child
- **Message Routing**: Route top-down messages to the appropriate child subnet
- **Economic Integration**: Manage collateral and token flows

#### Permission Modes

1. **Collateral Mode**: Validator power determined by staked collateral
2. **Federated Mode**: Validator power assigned by subnet owner
3. **Static Mode**: Initial collateral-based power that doesn't change

### 3.3 Registry

The **Subnet Registry** acts as a factory for deploying subnet actors, providing a standardized way to create new subnets.

#### Functions

- **Subnet Deployment**: Create new subnet actor instances
- **Parameter Management**: Set standard subnet parameters
- **Discovery**: Allow enumeration of deployed subnets
- **Governance Integration**: Coordinate with network governance for subnet policies

### 3.4 Fendermint Consensus Layer

**Fendermint** is the reference implementation for IPC subnet nodes, built on:

- **CometBFT**: Byzantine fault-tolerant consensus engine
- **ABCI++**: Enhanced application blockchain interface
- **FVM**: Filecoin Virtual Machine for smart contract execution
- **IPLD**: InterPlanetary Linked Data for content addressing

#### Architecture

```
┌─────────────────┐
│   Application   │ <- IPC Logic, FVM Integration
├─────────────────┤
│     ABCI++      │ <- Consensus Interface
├─────────────────┤
│   CometBFT      │ <- Byzantine Fault Tolerant Consensus
├─────────────────┤
│   Networking    │ <- P2P Communication
└─────────────────┘
```

---

## 4. Addressing and Identity

### 4.1 Subnet Addressing

IPC subnets are uniquely identified by a `SubnetID` consisting of:

```solidity
struct SubnetID {
    uint64 root;        // Chain ID of the root network
    address[] route;    // Array of subnet actor addresses
}
```

#### String Representation

Format: `/r{chainId}/address1/address2/.../addressN`

Examples:
- Root network: `/r314` (Filecoin Mainnet)
- L2 subnet: `/r314/t410fgalav7yo342zbem3kkqhx4l5d43d3iyswlpwkby`
- L3 subnet: `/r314/t410fgalav7yo342zbem3kkqhx4l5d43d3iyswlpwkby/t410fixm5mqenkfm2g6msjt2chs36cxaa7ka745xo2jq`

### 4.2 Actor Addressing

IPC inherits Filecoin's addressing model, supporting:

- **f0 addresses**: Actor ID addresses
- **f2 addresses**: Actor addresses (SECP256K1/BLS)
- **f4/t4 addresses**: Ethereum-compatible addresses (delegated)

#### Address Conversion

The protocol provides utilities to convert between Ethereum addresses and Filecoin f4 addresses:

```bash
ipc-cli util eth-to-f4-addr --addr 0x6BE1Ccf648c74800380d0520D797a170c808b624
# Output: t410fnpq4z5siy5eaaoanauqnpf5bodearnren5fxyoi
```

### 4.3 IPC Address Format

Cross-subnet addressing uses the `IPCAddress` structure:

```solidity
struct IPCAddress {
    SubnetID subnetId;      // Target subnet
    FvmAddress rawAddress;  // Address within subnet
}
```

---

## 5. Subnet Lifecycle

### 5.1 Subnet Creation Process

1. **Actor Deployment**: Deploy subnet actor in parent subnet
2. **Registration**: Call `register()` in parent's gateway with initial parameters
3. **Bootstrap**: Initialize validator set and start consensus
4. **Activation**: Begin processing transactions and creating checkpoints

#### Deployment Parameters

- **Genesis Configuration**: Initial state and parameters
- **Validator Set**: Initial validators and their stakes/powers
- **Economic Parameters**: Collateral requirements, fee structures
- **Consensus Parameters**: Block time, checkpoint periods, batch sizes

### 5.2 Subnet Bootstrap

#### Collateral Mode Bootstrap

1. Minimum number of validators must `join()` the subnet
2. Sufficient total collateral must be staked
3. Subnet actor calls `register()` in parent gateway
4. Child subnet can begin operation

#### Federated Mode Bootstrap

1. Subnet owner calls `setFederatedValidators()` to set initial validator set
2. Validators start their nodes and begin consensus
3. Subnet automatically registers with parent

### 5.3 Subnet Termination

Subnets can be terminated through the `kill()` function, which:

1. Prevents new transactions from being processed
2. Allows withdrawal of remaining funds
3. Cleans up subnet state in the parent
4. Releases validator collateral (after lock period)

---

## 6. Cross-Chain Communication

IPC provides native cross-chain communication through two primary mechanisms:

### 6.1 Top-Down Messages (Parent → Child)

Top-down messages flow from parent to child subnet through the **parent finality** mechanism.

#### Process

1. **Message Queuing**: Messages are queued in the parent subnet's gateway
2. **Finality Monitoring**: Child validators monitor parent chain finality
3. **Finality Commitment**: Child validators reach consensus on parent finality
4. **Message Execution**: Messages are executed when their block is finalized

#### Message Types

- **Fund Transfers**: Move tokens from parent to child
- **Validator Updates**: Propagate validator set changes
- **General Messages**: Arbitrary smart contract calls

#### Implementation

```solidity
struct ParentFinality {
    uint256 height;     // Parent block height
    bytes32 blockHash;  // Parent block hash
}
```

Child validators use a **VoteTally** mechanism to reach consensus on parent finality before committing it to the child chain.

### 6.2 Bottom-Up Messages (Child → Parent)

Bottom-up messages flow from child to parent through the **checkpoint** mechanism.

#### Process

1. **Message Queuing**: Messages are queued in the child subnet's gateway
2. **Checkpoint Creation**: Validators create periodic checkpoints containing message batches
3. **Signature Collection**: Validators sign checkpoints to form quorum certificates
4. **Relayer Submission**: Relayers submit checkpoints to parent subnet
5. **Message Execution**: Parent subnet executes messages after validation

#### Checkpoint Structure

```solidity
struct BottomUpCheckpoint {
    SubnetID subnetID;                    // Child subnet identifier
    uint256 blockHeight;                  // Child block height
    bytes32 blockHash;                    // Child block hash
    uint64 nextConfigurationNumber;       // Next validator configuration
    BottomUpBatch.Commitment msgs;        // Message batch commitment
    CompressedActivityRollup activity;    // Activity summary
}
```

#### Checkpoint Triggers

Checkpoints are created when any of the following conditions are met:

1. **Periodic**: Every `checkpointPeriod` blocks
2. **Message Threshold**: When message queue reaches `MAX_MSGS_PER_BATCH`
3. **Timeout**: After maximum wait time with pending messages

### 6.3 Cross-Net Message Format

All cross-subnet messages use the standardized `IpcEnvelope` format:

```solidity
struct IpcEnvelope {
    IpcMsgKind kind;        // Transfer, Call, or Result
    uint64 localNonce;      // Network-specific nonce
    uint64 originalNonce;   // Source network nonce
    uint256 value;          // Attached value
    IPCAddress to;          // Destination address
    IPCAddress from;        // Source address
    bytes message;          // Encoded message data
}

enum IpcMsgKind {
    Transfer,    // Native token transfers
    Call,        // Smart contract calls
    Result       // Execution results/receipts
}
```

### 6.4 Relayer Infrastructure

Relayers are off-chain processes that facilitate bottom-up message delivery:

#### Functions

- **Event Monitoring**: Watch for `QuorumEvent` emissions in child subnets
- **Message Packaging**: Bundle checkpoint data for parent submission
- **Transaction Submission**: Submit `submitCheckpoint()` transactions to parent
- **Redundancy**: Multiple relayers can operate for reliability

#### Incentive Model

Current implementation uses a fixed reward pool divided among all relayers submitting valid checkpoints within a time window, though this is under active development.

---

## 7. Consensus and Validation

### 7.1 Fendermint Consensus Architecture

Fendermint implements a modular consensus architecture combining:

- **CometBFT**: Proven Byzantine fault-tolerant consensus
- **ABCI++**: Enhanced application interface with proposal control
- **FVM/FEVM**: Ethereum-compatible execution environment
- **IPC Integration**: Native cross-subnet communication

### 7.2 Consensus Flow

1. **Propose**: Leader proposes block with transactions and parent finality
2. **Prevote**: Validators verify proposal and vote
3. **Precommit**: Validators commit to proposal after majority prevote
4. **Finalize**: Block is executed and committed to state

#### ABCI++ Integration

Fendermint uses ABCI++ methods for enhanced control:

- `PrepareProposal`: Include parent finality and cross-subnet messages
- `ProcessProposal`: Validate proposals before voting
- `FinalizeBlock`: Execute transactions and update state

### 7.3 Validator Set Management

#### Validator Changes

Validator set changes follow a two-phase process:

1. **Parent Phase**: Changes are recorded in parent subnet
2. **Child Phase**: Changes are applied in child subnet after confirmation

#### Change Types

```solidity
enum PowerOperation {
    SetPower,      // Update validator power
    SetMetadata    // Update validator metadata
}

struct PowerChange {
    PowerOperation op;
    bytes payload;
    address validator;
}
```

#### Configuration Numbers

Each validator set change is assigned a unique `configurationNumber` to ensure proper ordering and prevent replay attacks.

### 7.4 Security Properties

#### Finality

- **Parent Finality**: Child subnets achieve finality based on parent finality
- **Economic Finality**: Secured by validator collateral and slashing conditions
- **Reorg Resistance**: Checkpointed state prevents long-range attacks

#### Validator Security

- **Stake Requirements**: Minimum collateral for validator participation
- **Slashing Conditions**: Economic penalties for malicious behavior
- **Membership Proofs**: Cryptographic proofs of validator set membership

---

## 8. Security Model

### 8.1 Security Assumptions

1. **Parent Security**: Child subnets inherit security from parent chains
2. **Validator Honesty**: At least 2/3 of validators are honest and online
3. **Economic Incentives**: Validators have economic incentive to maintain security
4. **Cryptographic Security**: Standard cryptographic assumptions (ECDSA, hash functions)

### 8.2 Attack Vectors and Mitigations

#### Long-Range Attacks

**Mitigation**: Checkpoints anchor child state to parent chain, preventing historical rewrites

#### Nothing-at-Stake

**Mitigation**: Validator collateral slashing for provably invalid behavior

#### Eclipse Attacks

**Mitigation**: Multiple parent finality sources and P2P network diversity

#### Cross-Chain Message Attacks

**Mitigation**: Cryptographic commitment schemes and validator set verification

### 8.3 Economic Security

#### Collateral Requirements

- Validators must stake collateral in parent subnet
- Collateral amount determines voting power (in collateral mode)
- Slashing conditions protect against malicious behavior

#### Value Flow Security

- **Fund Operations**: Cryptographically guaranteed value conservation
- **Release Operations**: Burn-and-mint mechanism prevents double-spending
- **Checkpoint Validation**: Parent subnet validates all child value transfers

---

## 9. Network Protocols

### 9.1 P2P Network Layer

Fendermint uses CometBFT's P2P networking, which includes:

- **Gossip Protocol**: Efficient transaction and consensus message propagation
- **Block Sync**: Fast synchronization with network state
- **State Sync**: Snapshot-based rapid bootstrapping

### 9.2 JSON-RPC Interface

Each subnet exposes Ethereum-compatible JSON-RPC endpoints:

```
# Standard Ethereum methods
eth_blockNumber
eth_getBlockByHash
eth_sendTransaction
eth_call
eth_estimateGas

# IPC-specific methods
ipc_subnetInfo
ipc_checkpointInfo
ipc_parentFinality
```

### 9.3 IPLD Integration

IPC uses IPLD for content-addressed data storage:

- **Content Addressing**: All data referenced by cryptographic hashes
- **Efficient Storage**: Deduplication and compression
- **Cross-Chain References**: Consistent data references across subnets

### 9.4 Inter-Subnet Communication Protocol

#### Parent Syncing

Child subnets continuously synchronize with parent state:

1. **Polling**: Regular queries to parent RPC endpoints
2. **Event Filtering**: Monitor relevant parent chain events
3. **Finality Tracking**: Maintain view of parent chain finality
4. **State Caching**: Cache parent state for efficient access

#### Checkpoint Submission

Relayers implement the checkpoint submission protocol:

1. **Event Monitoring**: Subscribe to child subnet `QuorumEvent`s
2. **Data Collection**: Gather checkpoint signatures and metadata
3. **Parent Submission**: Submit complete checkpoint to parent subnet
4. **Verification**: Parent validates checkpoint and executes messages

---

## 10. Governance and Upgrades

### 10.1 Subnet Governance

Each subnet can implement its own governance mechanism:

- **Parameter Updates**: Modify consensus parameters, fees, etc.
- **Validator Management**: Add/remove validators in federated mode
- **Protocol Upgrades**: Upgrade subnet software and smart contracts

### 10.2 Cross-Subnet Coordination

Certain changes require coordination between parent and child:

- **Economic Parameters**: Changes affecting collateral or value flow
- **Protocol Compatibility**: Upgrades that affect cross-subnet messaging
- **Security Parameters**: Changes to validator sets or security models

### 10.3 Upgrade Mechanisms

#### Smart Contract Upgrades

IPC contracts use the Diamond pattern (EIP-2535) for upgrades:

- **Faceted Architecture**: Modular contract functionality
- **Upgrade Safety**: Controlled upgrade process with governance
- **Storage Consistency**: Maintain state across upgrades

#### Consensus Upgrades

Fendermint supports consensus-level upgrades:

- **Coordinated Upgrades**: Network-wide upgrade coordination
- **Backwards Compatibility**: Maintain compatibility during transitions
- **Rollback Capability**: Safe rollback in case of issues

---

## 11. Economic Model

### 11.1 Token Economics

#### Native Tokens

Each subnet can use different token models:

- **Native Asset**: Use parent subnet's native token
- **ERC20 Asset**: Use specific ERC20 token for subnet economy
- **Hybrid Models**: Combine multiple assets

#### Value Transfer

- **Fund**: Move value from parent to child subnet
- **Release**: Move value from child to parent subnet
- **Cross-Subnet**: Transfer value between arbitrary subnets in hierarchy

### 11.2 Fee Structure

#### Transaction Fees

- **Gas Model**: Ethereum-compatible gas pricing
- **Fee Distribution**: Fees paid to subnet validators
- **Cross-Subnet Fees**: Additional fees for cross-subnet messages

#### Infrastructure Costs

- **Relayer Rewards**: Compensation for checkpoint relay services
- **Validator Rewards**: Block rewards and transaction fees
- **Parent Fees**: Costs for parent subnet operations

### 11.3 Collateral Management

#### Staking Requirements

- **Minimum Stake**: Required collateral for validator participation
- **Proportional Power**: Voting power based on stake amount
- **Lock Periods**: Time delays for stake withdrawal

#### Slashing Conditions

- **Provable Faults**: Automatic slashing for cryptographically provable faults
- **Availability**: Penalties for validator downtime
- **Double Signing**: Severe penalties for consensus violations

---

## 12. Implementation Details

### 12.1 Smart Contract Architecture

#### Diamond Pattern Implementation

IPC contracts use EIP-2535 Diamond pattern:

```solidity
// Main diamond contracts
contract GatewayDiamond { /* Diamond proxy */ }
contract SubnetActorDiamond { /* Diamond proxy */ }

// Facet contracts
contract GatewayManagerFacet { /* Core gateway logic */ }
contract CheckpointingFacet { /* Checkpoint handling */ }
contract TopDownFinalityFacet { /* Parent finality */ }
contract XnetMessagingFacet { /* Cross-net messaging */ }
```

#### Storage Patterns

- **AppStorage**: Shared storage across facets
- **Diamond Storage**: Isolated storage per library
- **Upgrade Safety**: Storage layout compatibility

### 12.2 Rust Implementation

#### Core Libraries

```rust
// IPC types and utilities
ipc_types = { path = "ipc/types" }

// Provider library for IPC interactions
ipc_provider = { path = "ipc/provider" }

// Command-line interface
ipc_cli = { path = "ipc/cli" }

// Wallet and identity management
ipc_wallet = { path = "ipc/wallet" }
```

#### Fendermint Architecture

```rust
// Core VM components
fendermint_vm_core       // Basic types and utilities
fendermint_vm_genesis    // Genesis block handling
fendermint_vm_interpreter // Transaction execution
fendermint_vm_topdown    // Parent finality sync
fendermint_vm_snapshot   // State snapshots

// Application layer
fendermint_app          // ABCI++ application
fendermint_abci         // ABCI interface
fendermint_rpc          // JSON-RPC server
```

### 12.3 Build and Deployment

#### Development Environment

```bash
# Install dependencies
sudo apt install build-essential clang cmake pkg-config libssl-dev protobuf-compiler git curl
rustup target add wasm32-unknown-unknown
cargo install --force cargo-make

# Build project
make build-with-ui

# Run tests
make test
```

#### Docker Deployment

```dockerfile
# Fendermint node
FROM ghcr.io/consensus-shipyard/fendermint:latest

# Configuration
COPY config/ /fendermint/config/
COPY genesis.json /fendermint/genesis.json

# Start node
CMD ["fendermint", "run"]
```

### 12.4 Configuration

#### Subnet Parameters

```toml
[subnet]
id = "/r314159/t410fexample..."
parent_endpoint = "https://api.calibration.node.glif.io/rpc/v1"
checkpoint_period = 100
batch_period = 50
max_msgs_per_batch = 1000

[consensus]
block_time = "1s"
timeout_commit = "1s"
max_validators = 100

[network]
p2p_listen = "0.0.0.0:26656"
rpc_listen = "0.0.0.0:26657"
eth_api_listen = "0.0.0.0:8545"
```

#### Gateway Configuration

```json
{
  "gateway": {
    "address": "0x1AEe8A878a22280fc2753b3C63571C8F895D2FE3",
    "subnet_id": "/r314159",
    "checkpoint_period": 100,
    "majority_percentage": 67,
    "active_validators_limit": 100
  }
}
```

---

## Conclusion

The InterPlanetary Consensus (IPC) protocol provides a comprehensive framework for hierarchical blockchain scaling. Through its recursive subnet architecture, native cross-chain communication, and flexible consensus mechanisms, IPC enables applications to achieve planetary scale while maintaining security and interoperability.

The protocol's modular design allows for gradual adoption and customization, while its Ethereum compatibility ensures familiar developer experience and tooling support. With implementations in both Solidity smart contracts and Rust systems software, IPC provides a production-ready foundation for the next generation of scalable blockchain applications.

### Future Development

Key areas for continued development include:

- **Enhanced Economics**: More sophisticated incentive mechanisms for relayers and validators
- **Improved Performance**: Optimizations for high-throughput applications
- **Additional Integrations**: Support for more rootnet types and consensus mechanisms
- **Tooling Enhancement**: Better developer experience and operational tools
- **Security Hardening**: Continued security research and formal verification

The IPC protocol represents a significant advancement in blockchain scalability, providing the foundation for a truly scalable, interoperable Web3 ecosystem.
