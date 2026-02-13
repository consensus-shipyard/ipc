# IPC Architecture Overview

## Introduction

This document provides a comprehensive overview of the InterPlanetary Consensus (IPC) architecture, describing the components, their responsibilities, and how they connect subnets to the L1 root network.

**InterPlanetary Consensus (IPC)** is a framework that enables on-demand horizontal scalability of blockchain networks by deploying "subnets" that can run different consensus algorithms and customize their execution environment. IPC provides:

- **Recursive scalability**: Subnets can spawn child subnets indefinitely
- **Cross-net communication**: Secure message passing between parent and child networks
- **Flexible consensus**: Each subnet can customize block time, validator requirements, and consensus parameters
- **Ethereum compatibility**: Full FEVM support for Solidity smart contracts
- **Native FVM support**: Access to Filecoin's actor model and built-in capabilities

## Architecture Diagrams

This document includes three complementary diagrams:

### 1. Detailed Architecture Diagram
Shows all components, internal structure, and detailed interactions.

![IPC Detailed Architecture](../../.cursor/projects/Users-philip-github-ipc/assets/ipc-architecture-diagram.png)

### 2. Simplified Architecture Diagram
High-level view of the key layers and primary communication flows.

![IPC Simplified Architecture](../../.cursor/projects/Users-philip-github-ipc/assets/ipc-architecture-simple.png)

### 3. Message Flow Sequences
Step-by-step flows for common operations (fund and release).

![IPC Message Flows](../../.cursor/projects/Users-philip-github-ipc/assets/ipc-message-flows.png)

### 4. Validator Lifecycle Diagram
State transitions and operations for validators throughout their lifecycle.

![IPC Validator Lifecycle](../../.cursor/projects/Users-philip-github-ipc/assets/ipc-validator-lifecycle.png)

## System Components

### 1. L1 Root Network (Filecoin/Calibration)

**Responsibility**: The root network serves as the trust anchor for the entire IPC hierarchy.

**Components**:
- **Lotus/FEVM Node**: The Filecoin node implementation that provides the foundational blockchain
- **Gateway Contract**: Singleton contract managing IPC protocol logic, collateral, firewalls, and cross-net interactions
- **Subnet Actor Contract**: User-defined contract implementing subnet-specific logic (one per child subnet)
- **Registry Contract**: Factory contract for deploying reference implementations of subnet actors

**Key Functions**:
- Source of truth for validator sets and collateral
- Execution environment for top-down messages
- Reception point for bottom-up checkpoints from child subnets
- RPC endpoint for subnet validators to query parent state

### 2. Fendermint Nodes (Subnet Validators)

**Responsibility**: Peer implementation of IPC subnets, running the actual subnet blockchain.

**Architecture**: Each Fendermint node is composed of:

#### a. CometBFT Consensus Layer
- Byzantine Fault Tolerant consensus engine (formerly Tendermint)
- Handles block proposal, voting, and finalization
- Provides networking layer for validator P2P communication
- Uses ABCI++ interface to communicate with application layer

#### b. ABCI++ Application Layer
- Implements ABCI++ interface methods (`PrepareProposal`, `ProcessProposal`, `FinalizeBlock`)
- Manages transaction lifecycle from mempool to execution
- Validates proposals including parent finality commitments
- Coordinates between consensus and execution layers

#### c. FVM/FEVM Execution Layer
- Filecoin Virtual Machine for smart contract execution
- Supports both native FVM actors and FEVM (Ethereum compatibility)
- Executes top-down messages from parent
- Manages state transitions and storage

#### d. Parent Syncer Module
- Polls parent network RPC for finalized state
- Caches parent blocks with configurable finality delay
- Publishes votes on observed parent blocks via GossipSub
- Implements quorum detection via VoteTally mechanism
- Two implementations: `LotusParentSyncer` and `TendermintAwareSyncer`

#### e. IPLD Resolver Module
- Resolves Content Identifiers (CIDs) across the network
- Stores and retrieves IPLD data from distributed store
- Facilitates data availability for checkpoint validation
- Maintains connections with nodes in parent/child/peer subnets

#### f. Gateway & Registry Contracts (Deployed per subnet)
- Each subnet deploys its own Gateway and Registry at genesis
- Enables recursive subnet creation
- Manages child subnet interactions

### 3. User Interaction Layer

#### a. ipc-cli (Command Line Interface)
**Responsibility**: Primary user interface for interacting with IPC

**Key Commands**:
- Wallet management: `wallet new`, `wallet import`, `wallet export`
- Subnet operations: `subnet create`, `subnet join`, `subnet leave`, `subnet list`
- Cross-net messages: `cross-msg fund`, `cross-msg release`
- Validator operations: `subnet stake`, `subnet unstake`, `subnet claim`
- Checkpoint operations: `checkpoint relayer`, `checkpoint list-bottomup`

#### b. IpcProvider Library
**Responsibility**: Rust library providing programmatic access to IPC functionality

**Features**:
- Used internally by ipc-cli
- Available for building custom IPC applications
- Handles transaction signing and submission
- Manages interaction with Gateway and Subnet Actor contracts

#### c. Wallet/Key Management
**Responsibility**: Secure storage and management of cryptographic keys

**Features**:
- EVM wallet with keystore in `~/.ipc`
- Support for multiple addresses and default key configuration
- Key export in multiple formats (hex, base64, fendermint-compatible)
- Integration with Metamask and other wallets

### 4. Relayer Processes

**Responsibility**: Bridge bottom-up information from child subnets to parent.

**Operation Flow**:
1. Monitor child subnet for `QuorumReached` events via RPC
2. Retrieve signed checkpoint with validator signatures
3. Submit checkpoint to parent subnet's Gateway contract
4. Execute bottom-up messages as part of checkpoint submission

**Incentivization**:
- Relayers receive cross-net message fees for checkpoint submissions
- Rewards claimed via `ipc-cli subnet claim --reward`
- Anyone can run a relayer (permissionless design)
- Redundancy encouraged for network resilience

**Key Properties**:
- Trustless operation (cryptographic verification via quorum signatures)
- No single point of failure (multiple relayers can operate)
- Economically incentivized but not required to be validators

## Cross-Net Communication Flows

### Top-Down Flow (Parent → Child)

**Purpose**: Propagate parent state, messages, and validator changes to child subnet.

**Components**:
1. **Parent Finality Proposal**:
   - Validators poll parent via RPC
   - Parent Syncer caches blocks with finality delay
   - Validators vote on parent block height + hash via GossipSub
   - Quorum detected by VoteTally
   - Block proposer includes `ParentFinality` in proposal

2. **Proposal Validation**:
   - Each validator checks proposed height exists in cache or RPC
   - Validates block hash matches
   - Rejects if invalid, accepts if valid

3. **Proposal Execution**:
   - Commit new parent finality to ledger
   - Fetch and apply validator changes (join/leave/stake)
   - Fetch and execute top-down messages (fund operations)
   - Indexed by parent block height for deterministic execution

**Message Types**:
- `fund`: Send tokens from parent address to child address
- `pre-fund`: Include balance in child subnet genesis (before bootstrap)
- Validator changes: join, leave, stake, unstake, setFederatedPower

**Key Parameters**:
- Finality delay: Number of parent blocks to wait before considering final
- Max proposal range: Limits how far ahead proposal can be from last committed

### Bottom-Up Flow (Child → Parent)

**Purpose**: Propagate checkpoints and messages from child to parent.

**Checkpoint Creation Flow**:

1. **Trigger Conditions** (any of these):
   - Fixed period reached (`bottomup-check-period`)
   - Message queue exceeds `MAX_MSGS_PER_BATCH`
   - Maximum wait time exceeded with pending messages

2. **Checkpoint Creation**:
   - Deterministic process during block execution
   - All validators call `createCheckpoint` on Gateway contract
   - Checkpoint contains:
     - `subnet_id`: Child subnet identifier
     - `block_height`: Child block height
     - `block_hash`: Child block hash (prevents long-range attacks)
     - `prev_checkpoint_height`: Links checkpoints in chain
     - `next_configuration_number`: Validator set for next checkpoint
     - `cross_messages`: Bottom-up messages (or CID commitment)

3. **Signature Collection**:
   - After checkpoint committed in block, validators broadcast signature transactions
   - Validators call `addCheckpointSignature` with their signature
   - Signatures accumulate in child ledger
   - Includes Merkle proof of validator membership (power table)

4. **Quorum Reached**:
   - When >2/3 of validator power has signed
   - Gateway emits `QuorumReached` event
   - Checkpoint ready for relayer pickup

5. **Relayer Submission**:
   - Relayer queries `QuorumReached` events
   - Calls `submitCheckpoint` in parent Gateway with:
     - Checkpoint data
     - Bundle of validator signatures (quorum certificate)
   - Parent validates signatures against last known validator set
   - Updates `next_configuration_number` to apply validator changes

6. **Parent Execution**:
   - Bottom-up messages executed in parent
   - Validator set synchronized to child's confirmed state
   - Relayer receives rewards from message fees

**Message Types**:
- `release`: Send tokens from child address to parent address
- `pre-release`: Reclaim genesis balance before subnet bootstrap
- General cross-net messages (planned for future)

**Configuration Updates**:
- Child reports `next_configuration_number` in checkpoint
- Parent confirms stake changes when checkpoint committed
- Creates deterministic ordering of validator set evolution

## Subnet Lifecycle

### 1. Subnet Creation

```bash
# Deploy subnet actor (via registry or custom)
ipc-cli subnet create --parent /r314159 --min-validators 4

# For federated subnets
ipc-cli subnet set-federated-validators --subnet <SUBNET_ID> --validators <ADDRS>
```

**Process**:
1. User deploys Subnet Actor contract in parent
2. Subnet Actor registered in parent's Registry
3. Subnet waits for minimum collateral/validators (collateral mode)
4. Once conditions met, subnet calls `register` on parent Gateway
5. Subnet is bootstrapped and can start producing blocks

### 2. Joining as Validator

```bash
# Join subnet with collateral
ipc-cli subnet join --subnet <SUBNET_ID> --collateral 10

# Add more collateral
ipc-cli subnet stake --subnet <SUBNET_ID> --collateral 5
```

**Process**:
1. Validator calls `join` on Subnet Actor with collateral
2. Creates `StakeChangeRequest` with configuration number
3. Change propagated to child via top-down finality
4. Child executes membership change
5. Child confirms in next checkpoint via `next_configuration_number`
6. Parent confirms collateral change when checkpoint committed

### 3. Running Validator Node

```bash
# Start Fendermint validator
fendermint run --home-dir ~/.fendermint \
  --network <NETWORK_CONFIG> \
  --subnet-id <SUBNET_ID>
```

**Components Started**:
- CometBFT consensus node
- ABCI++ application server
- Ethereum RPC API (for user transactions)
- Parent Syncer (background task)
- IPLD Resolver (background task)

### 4. Cross-Net Operations

#### Fund (Parent → Child)
```bash
# Send funds to child subnet
ipc-cli cross-msg fund --subnet <SUBNET_ID> --to <ADDR> 100

# Check parent finality in child
ipc-cli cross-msg parent-finality --subnet <SUBNET_ID>
```

**Flow**:
1. User calls `fund` on parent Gateway
2. Message added to top-down queue
3. Validators include parent finality in child blocks
4. When finality committed, message executed in child
5. Funds appear in child address

#### Release (Child → Parent)
```bash
# Send funds back to parent
ipc-cli cross-msg release --subnet <SUBNET_ID> --to <ADDR> 50

# Monitor checkpoint submission
ipc-cli checkpoint list-bottomup --subnet <SUBNET_ID> --from-epoch 0 --to-epoch 100
```

**Flow**:
1. User calls `release` on child Gateway
2. Message added to bottom-up queue
3. Checkpoint created at period boundary
4. Validators sign checkpoint
5. Relayer submits to parent
6. Message executed in parent, funds released

### 5. Running Relayer

```bash
# Start relayer for subnet
ipc-cli checkpoint relayer --subnet <SUBNET_ID>

# Claim rewards
ipc-cli subnet claim --subnet <SUBNET_ID> --reward
```

### 6. Leaving Subnet

```bash
# Reduce stake
ipc-cli subnet unstake --subnet <SUBNET_ID> --collateral 5

# Leave entirely
ipc-cli subnet leave --subnet <SUBNET_ID>

# Claim returned collateral
ipc-cli subnet claim --subnet <SUBNET_ID>
```

**Process**:
1. Validator calls `leave` or `unstake`
2. Creates `StakeChangeRequest`
3. Propagated to child via top-down finality
4. Child removes validator or reduces power
5. Child confirms in checkpoint
6. Parent releases collateral when checkpoint committed
7. Validator claims collateral

## Recursive Subnet Architecture

**Key Property**: Any subnet can spawn child subnets, creating a tree hierarchy.

**Implications**:
- Each subnet level has its own Gateway and Registry
- Subnets can customize consensus parameters (block time, checkpoint period)
- Cross-net messages can traverse multiple levels
- Validators at each level are independent
- Security decreases with depth (trust assumptions compound)

**Example Hierarchy**:
```
L1 (Filecoin Mainnet)
├─ Subnet A (Gaming subnet, fast blocks)
│  ├─ Subnet A1 (Game instance 1)
│  └─ Subnet A2 (Game instance 2)
└─ Subnet B (DeFi subnet, EVM compatible)
   └─ Subnet B1 (Lending protocol subnet)
```

## Security Model

### Trust Assumptions

1. **L1 Security**: Root network (Filecoin) is trusted and secure
2. **Validator Honesty**: >2/3 of validator power is honest in each subnet
3. **Economic Security**: Collateral requirements discourage malicious behavior
4. **Finality Delay**: Parent blocks delayed sufficiently to avoid reorgs
5. **RPC Reliability**: Validators have access to reliable parent RPC nodes

### Attack Mitigations

1. **Long-Range Attacks**: Prevented by anchoring block hash in checkpoints
2. **Double-Spend**: Prevented by BFT consensus and checkpoint finality
3. **Validator Collusion**: Requires >2/3 stake, economically irrational
4. **Checkpoint Censorship**: Multiple relayers provide redundancy
5. **Parent Reorg**: Finality delay ensures parent state is stable

### Gas Economics

**Top-Down Messages**:
- Gas paid in parent subnet
- Execution cost in child (no additional fee)

**Bottom-Up Messages**:
- Cross-message fee paid by sender (rewards relayer)
- Execution gas paid by relayer (compensated by fee)
- Fee amount set by subnet policy

## Network Types and Modes

### Permission Modes

1. **Collateral Mode**:
   - Permissionless (anyone can join)
   - Requires minimum collateral stake
   - Voting power proportional to stake
   - Typical for public subnets

2. **Federated Mode**:
   - Permissioned (admin sets validators)
   - Admin calls `setFederatedPower` to set validator set
   - Voting power set by admin
   - Typical for consortium/private subnets

### Network Types

1. **FEVM** (Filecoin EVM):
   - Ethereum Virtual Machine compatibility
   - Supports Solidity smart contracts
   - Standard Ethereum RPC API
   - Compatible with Web3 tooling

2. **FVM** (Filecoin VM):
   - Native Filecoin execution environment
   - Actor-based programming model
   - WASM runtime
   - Access to Filecoin built-in actors

## Configuration Parameters

### Subnet Creation Parameters

- `min_validators`: Minimum validators required to bootstrap
- `min_collateral`: Minimum total collateral required
- `bottomup_check_period`: Blocks between checkpoints
- `active_validators_limit`: Maximum active validators
- `majority_percentage`: Quorum threshold (typically 67%)

### Operational Parameters

- **Finality Delay**: Blocks to wait before considering parent block final (e.g., 10-20 blocks)
- **Max Proposal Range**: Maximum parent height advance per child block (prevents stalls)
- **Max Messages Per Batch**: Triggers intermediate checkpoint if exceeded
- **Checkpoint Period**: Fixed interval between checkpoints (e.g., every 100 blocks)

## Monitoring and Observability

### Key Metrics

1. **Checkpoint Health**:
   - Time since last checkpoint submitted
   - Percentage of checkpoints with quorum
   - Average checkpoint submission delay

2. **Validator Performance**:
   - Signature submission rate
   - Uptime and block participation
   - Parent syncer cache hit rate

3. **Cross-Net Message Flow**:
   - Top-down message execution lag
   - Bottom-up message queue depth
   - Message fee statistics

4. **Network Connectivity**:
   - P2P peer count
   - Parent RPC latency and availability
   - IPLD resolver data availability

### Debugging Commands

```bash
# Check subnet status
ipc-cli subnet list --subnet <PARENT_ID>

# View checkpoint history
ipc-cli checkpoint list-bottomup --from-epoch <N> --to-epoch <M> --subnet <SUBNET_ID>

# Check validator changes
ipc-cli checkpoint list-validator-changes --from-epoch <N> --to-epoch <M> --subnet <SUBNET_ID>

# Check quorum events
ipc-cli checkpoint quorum-reached-events --from-epoch <N> --to-epoch <M> --subnet <SUBNET_ID>

# Check if checkpoint submitted
ipc-cli checkpoint has-submitted-bottomup-height --subnet <SUBNET_ID> --submitter <ADDR>

# Get latest checkpoint height
ipc-cli checkpoint last-bottomup-checkpoint-height --subnet <SUBNET_ID>

# Check wallet balances
ipc-cli wallet balances --wallet-type evm --subnet <SUBNET_ID>

# View parent finality
ipc-cli cross-msg parent-finality --subnet <SUBNET_ID>

# List top-down messages
ipc-cli cross-msg list-topdown-msgs --subnet <SUBNET_ID> --epoch <EPOCH>
```

## Technical Implementation Details

### Smart Contract Architecture (Diamond Pattern)

IPC uses the Diamond Pattern (EIP-2535) for upgradeable contracts:

**Components**:
- **Diamond**: Main contract with storage and router
- **Facets**: Separate contracts with function implementations
- **DiamondCut**: Upgrade mechanism for adding/replacing facets

**Contracts**:
- `GatewayDiamond.sol`: Main IPC protocol logic
- `SubnetActorDiamond.sol`: Subnet-specific governance
- `SubnetRegistryDiamond.sol`: Subnet actor factory

**Storage Patterns**:
- AppStorage pattern for shared storage in facets
- Diamond Storage for library-specific storage
- Reentrancy guards adapted for Diamond pattern

### IPLD and Data Availability

**CID Resolution**:
- Checkpoint may contain CID instead of full message list
- IPLD Resolver fetches data from IPLD network
- Validators verify data availability before voting
- Uses Bitswap protocol for data exchange

**NC-Max Approach**:
- CIDs proposed for resolution in advance
- Data dissemination moved out of critical consensus path
- Validators reject proposals with unavailable data

### Signature and Verification

**Checkpoint Signatures**:
- Validators sign using their validator key (BLS or ECDSA)
- Merkle proof of power table membership included
- Parent verifies quorum threshold met (>2/3 total power)
- Signature aggregation possible for efficiency

**Power Table**:
- Merkle tree of validator public keys and power
- Root hash included in checkpoint
- Enables efficient verification of validator membership
- Updated incrementally as validators join/leave

## Deployment Scenarios

### Local Development (Anvil)

```bash
# Start Anvil with IPC accounts funded
./scripts/setup-anvil-with-ipc-keys.sh

# Deploy IPC contracts
cd contracts && forge script script/DeployGateway.s.sol

# Configure ipc-cli for local
ipc-cli config init
```

### Calibration Testnet

**Configuration** (`~/.ipc/config.toml`):
```toml
keystore_path = "~/.ipc"

[[subnets]]
id = "/r314159"

[subnets.config]
network_type = "fevm"
provider_http = "https://api.calibration.node.glif.io/rpc/v1"
gateway_addr = "0x1AEe8A878a22280fc2753b3C63571C8F895D2FE3"
registry_addr = "0x0b4e239FF21b40120cDa817fba77bD1B366c1bcD"
```

**Steps**:
1. Get tFIL from [Calibration Faucet](https://faucet.calibration.fildev.network/funds.html)
2. Create subnet and join as validator
3. Deploy Fendermint infrastructure
4. Start validating and relay checkpoints

### Production (Mainnet)

**Considerations**:
- Use reliable RPC infrastructure (load balanced, redundant)
- Monitor validator performance and checkpoint submission
- Run multiple relayers for redundancy
- Implement alerting for checkpoint delays
- Regular validator key rotation and security audits
- Backup and disaster recovery procedures

## Future Enhancements

### Planned Features

1. **General Cross-Net Messages**: Beyond fund/release, arbitrary contract calls
2. **Message Inbox Pattern**: Decouple message delivery from execution
3. **Improved Relayer Rewards**: More sophisticated incentive mechanisms
4. **Light Client Verification**: Parent state verification without full RPC trust
5. **Cross-Subnet Atomic Swaps**: Multi-hop cross-net transactions
6. **Subnet Sharding**: Horizontal scaling within subnet level
7. **Checkpoint Compression**: Reduce checkpoint size via commitment schemes

### Research Areas

1. **Finality Gadgets**: Faster finality without compromising security
2. **Zero-Knowledge Checkpoints**: Privacy-preserving cross-net messages
3. **Optimistic Checkpoints**: Faster confirmation with fraud proofs
4. **Dynamic Validator Selection**: Reputation-based validator sets
5. **Cross-Chain Bridges**: Integration with non-IPC blockchains

## References

### Documentation
- [IPC Main README](../../README.md)
- [IPC Usage Guide](./usage.md)
- [Contract Documentation](./contracts.md)
- [Fendermint Architecture](../fendermint/architecture.md)
- [Deploying Hierarchy](./deploying-hierarchy.md)

### Specifications
- [IPC Actors Spec](../../specs/ipc-actors.md)
- [Bottom-Up Interactions](../../specs/bottom-up-interaction.md)
- [Top-Down Finality](../../specs/topdown.md)
- [Checkpointing](../fendermint/checkpointing.md)

### External Resources
- [Tendermint/CometBFT Documentation](https://docs.cometbft.com/)
- [ABCI++ Specification](https://github.com/cometbft/cometbft/tree/main/spec/abci)
- [EIP-2535 Diamond Standard](https://eips.ethereum.org/EIPS/eip-2535)
- [IPLD Specification](https://ipld.io/specs/)

## Glossary

- **ABCI++**: Application Blockchain Interface, protocol for CometBFT to communicate with application
- **Bottom-Up**: Information flow from child subnet to parent
- **Checkpoint**: Batch of messages and state commitments from child to parent
- **Configuration ID**: Identifier for validator set version
- **CID**: Content Identifier in IPLD
- **Facet**: Function implementation contract in Diamond pattern
- **Fendermint**: IPC subnet node implementation
- **FEVM**: Filecoin EVM, Ethereum-compatible execution environment
- **FVM**: Filecoin Virtual Machine
- **Gateway**: Singleton contract managing IPC protocol in each subnet
- **IPC**: InterPlanetary Consensus
- **IPLD**: InterPlanetary Linked Data
- **Quorum**: >2/3 of validator power threshold
- **Relayer**: Process submitting checkpoints from child to parent
- **Subnet Actor**: Contract governing a specific child subnet
- **Top-Down**: Information flow from parent to child subnet
- **Vote Tally**: Component tracking validator votes on parent finality

---

**Document Version**: 1.0  
**Last Updated**: January 2026  
**Status**: Living Document
