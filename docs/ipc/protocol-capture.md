# IPC Protocol Capture

> Protocol capture document describing how IPC works, components, mechanisms, status, open questions, and dependencies.

## Table of Contents

1. [Intro / Overview](#intro--overview)
2. [Components](#components)
3. [Concepts (Mechanisms)](#concepts-mechanisms)
4. [Security Guarantees](#security-guarantees)
5. [Configuration](#configuration)
6. [Terms](#terms)
7. [Dependencies](#dependencies)

---

## Intro / Overview

### What IPC Is

**InterPlanetary Consensus (IPC)** is a framework for on-demand horizontal scalability of blockchain networks through subnet deployment. Subnets are child blockchains that can run different consensus algorithms and customize their execution environment while staying anchored to a parent network (e.g., Filecoin).

### Features

| Feature | Description |
|---------|-------------|
| **Recursive scalability** | Subnets can spawn child subnets indefinitely, forming a tree hierarchy |
| **Cross-net communication** | Secure message passing between parent and child (top-down and bottom-up) |
| **Flexible consensus** | Each subnet can customize block time, validator requirements, and consensus parameters |
| **Ethereum compatibility** | Full FEVM support for Solidity smart contracts |
| **Native FVM support** | Access to Filecoin's actor model and built-in capabilities |

### What IPC Aims To Do

- Enable deployment of specialized blockchains (subnets) without deploying a new L1
- Provide native cross-subnet communication (eliminating the need for custom bridges within the hierarchy)
- Allow subnets to inherit security from the parent while maintaining independent consensus
- Support both permissionless (collateral-based) and permissioned (federated) validator sets

---

## Components

### Actors (WASM)

IPC subnets run the **Filecoin Virtual Machine (FVM)**, which executes **WASM-based actors**—the native Filecoin execution model. These include:

- **Built-in actors**: Account, Init, EVM (for FEVM), Payment Channel, and others from the Filecoin built-in actors bundle
- **Custom actors**: Subnets can deploy custom WASM actors in addition to the built-ins
- **Actor model**: Stateful, message-passing execution; actors have addresses, balance, and state

The FVM/WASM layer is shared across the root (Lotus) and child subnets (Fendermint). IPC does not define new WASM actors; it leverages the existing Filecoin actor model for execution within each subnet.

### Contracts (Solidity)

IPC protocol logic is implemented in **Solidity smart contracts** (FEVM):

| Contract | Location | Responsibility |
|----------|----------|----------------|
| **Gateway** | Singleton in every subnet | Manages collateral, firewall, cross-net message routing, checkpoint validation |
| **Subnet Actor** | One per child subnet, deployed in parent | Subnet-specific governance: supply source, genesis, permission mode, validator set |
| **Registry** | Per subnet | Factory for deploying reference Subnet Actor implementations |

**Architecture**: Contracts use the Diamond pattern (EIP-2535) for upgradeability. Facets implement distinct functionality; AppStorage pattern for shared state.

### Node

The **Fendermint** node is the subnet peer implementation. Architecture:

| Layer | Component | Responsibility |
|-------|-----------|----------------|
| Consensus | CometBFT | BFT consensus, block proposal, voting, P2P networking |
| Application | ABCI++ | `PrepareProposal`, `ProcessProposal`, `FinalizeBlock`; transaction lifecycle |
| Execution | FVM/FEVM | Smart contract execution, state transitions |
| Parent Sync | Parent Syncer | Polls parent RPC, caches blocks, VoteTally for parent finality |
| Data | IPLD Resolver | CID resolution, Bitswap, data availability for checkpoints |

**Root network (L1)**: Lotus runs Filecoin and hosts the root Gateway, Registry, and Subnet Actor contracts.

#### Node Lifecycle

1. **Genesis**: Subnet deploys Gateway and Registry at genesis
2. **Bootstrap**: Subnet Actor calls `register` on parent Gateway once min validators/collateral met
3. **Validation**: Node runs CometBFT consensus, applies top-down messages, creates checkpoints
4. **Shutdown**: Validator can `leave` or `unstake`; collateral released when checkpoint confirms

### Relayer (Off-Chain Component)

The **relayer** is a permissionless off-chain process that bridges bottom-up information from child to parent.

**Flow**:
1. Monitors child subnet for `QuorumReached` events (checkpoint signed by >2/3 validators)
2. Retrieves signed checkpoint and validator signature bundle
3. Calls `submitCheckpoint` on parent Gateway
4. Parent validates signatures, executes bottom-up messages, updates validator set

**Properties**: Trustless (cryptographic verification), economically incentivized (message fees), redundant (anyone can run). Not required to be a validator.

---

## Concepts (Mechanisms)

### Messaging

**Definition**: The transport layer for cross-subnet communication—how data moves between subnets.

**Flows**:
- **Top-down (parent → child)**: Messages indexed by parent block height; propagated via parent finality commitment in child blocks; executed when finality is committed
- **Bottom-up (child → parent)**: Messages batched in checkpoints; validators sign; relayer submits; executed on parent

**Message types**:
- **Transfer**: fund, release, pre-fund, pre-release (native asset moves)
- **Call**: Arbitrary contract-to-contract (general cross-net messages)
- **Result**: Response to Call/Transfer, propagated back to source

**Mechanisms**: IpcEnvelope, postbox (intermediate subnet routing), LCA (lowest common ancestor) routing, applyType (TopDown vs BottomUp).

### Bridging

**Definition**: Moving value/assets between subnets—the application layer built on messaging.

**Native bridging** (supply source):
- **Native supply**: Parent's native coin (e.g., FIL) becomes subnet native coin; fund/release move it
- **ERC20 supply**: ERC20 on parent locked; minted in subnet; burn in subnet, release on parent

**Custom bridging**: Linked Token pattern—lock-mint / burn-release for arbitrary ERC20s using general cross-net Call messages.

**Relationship to messaging**: Bridging uses messaging as transport. fund/release are both message types and bridging primitives.

### Subnet

**Definition**: A child blockchain in the IPC hierarchy, with its own validators, consensus, and execution.

**Identity**: SubnetID = `root` (chain ID) + `route` (array of subnet actor addresses top-to-bottom). String format: `/r314159/t410f.../t410f...`

**Properties**: Recursive (can spawn children), configurable (block time, checkpoint period, supply source, permission mode).

### Checkpointing

**Definition**: Batching of child state and bottom-up messages for submission to the parent.

**Flow**:
1. Every `bottomup_check_period` blocks (or when `MAX_MSGS_PER_BATCH` hit), validators create checkpoint
2. Validators call `addCheckpointSignature`; quorum (>2/3) triggers `QuorumReached` event
3. Relayer submits checkpoint + signatures to parent
4. Parent verifies against last known validator set, executes messages

**Checkpoint contents**: subnet_id, block_height, block_hash, prev_checkpoint_height, cross_messages, next_configuration_number.

### Consensus

**Subnet consensus**: CometBFT (BFT, >2/3 honest). Each subnet has independent validator set.

**Parent finality (consensus on parent state)**:
- Validators poll parent, cache blocks with finality delay
- Vote on parent block (height + hash) via GossipSub
- VoteTally detects quorum; proposer includes `ParentFinality` in block
- Validators accept/reject block based on proposed finality

### On-Chain vs Off-Chain

| Aspect | On-Chain | Off-Chain |
|--------|----------|-----------|
| Gateway, Subnet Actor, Registry | ✓ | |
| Message execution, checkpoint validation | ✓ | |
| Consensus (CometBFT) | Runs on nodes | |
| Parent Syncer | | Polls parent RPC |
| Relayer | | Submits checkpoints |
| ipc-cli | | Signs, submits txs via RPC |

**Note**: "Actors" in IPC spec (Gateway, Subnet Actor) are Solidity contracts (on-chain). Filecoin built-in actors (WASM) are also on-chain. The relayer and CLI are off-chain.

---

## Security Guarantees

### Trust Assumptions

1. **L1 Security**: Root network (e.g., Filecoin) is trusted and secure
2. **Validator honesty**: >2/3 of validator power is honest in each subnet
3. **Economic security**: Collateral requirements discourage malicious behavior
4. **Finality delay**: Parent blocks are delayed sufficiently before use to avoid reorgs
5. **RPC reliability**: Validators have access to reliable parent RPC nodes

### Attack Mitigations

| Threat | Mitigation |
|--------|------------|
| Long-range attacks | Block hash anchored in checkpoints |
| Double-spend | BFT consensus + checkpoint finality |
| Validator collusion | Requires >2/3 stake; economically irrational |
| Checkpoint censorship | Multiple relayers provide redundancy |
| Parent reorg | Finality delay ensures parent state stability |

### Guarantees Provided

- **Message integrity**: Envelopes signed/verified; no forgery
- **Ordering**: Deterministic execution via parent finality and checkpoint ordering
- **Value safety**: Lock-before-mint semantics; burn-before-release
- **Validator set sync**: Configuration number in checkpoint confirms membership changes; parent releases collateral only after child confirms

### Caveats

- Security decreases with hierarchy depth (trust assumptions compound)
- Federated subnets rely on admin honesty for validator set
- Gateway/Registry upgrades may introduce trust (depends on upgrade governance)

---

## Configuration

### Subnet Creation Parameters

| Parameter | Description |
|-----------|-------------|
| `min_validators` | Minimum validators to bootstrap |
| `min_collateral` | Minimum total collateral |
| `bottomup_check_period` | Blocks between checkpoints |
| `active_validators_limit` | Max active validators |
| `majority_percentage` | Quorum threshold (typically 67%) |
| `supply_source_kind` | Native or ERC20 |
| `supply_source_address` | ERC20 contract address (if ERC20) |

### Operational Parameters

| Parameter | Typical Value | Description |
|-----------|---------------|-------------|
| `finality_delay` | 10–20 blocks | Parent blocks to wait before final |
| `max_proposal_range` | ~50 blocks | Max parent height advance per child block |
| `MAX_MSGS_PER_BATCH` | 100 | Triggers intermediate checkpoint if exceeded |

### Configuration Files

- **ipc-cli**: `~/.ipc/config.toml` — subnets (id, provider_http, gateway_addr, registry_addr), keystore_path
- **Fendermint**: `~/.fendermint/config/` — ABCI, DB, resolver, eth RPC ports

---

## Terms

| Term | Definition |
|------|------------|
| **ABCI++** | Application Blockchain Interface; CometBFT ↔ application protocol |
| **Bottom-up** | Information flow child → parent (checkpoints, messages) |
| **Checkpoint** | Batch of child state + messages submitted to parent with validator signatures |
| **Configuration ID** | Version number for validator set |
| **Gateway** | Singleton contract managing IPC in each subnet |
| **LCA** | Lowest common ancestor (for routing cross-subnet messages) |
| **Quorum** | >2/3 of validator power |
| **Relayer** | Off-chain process submitting checkpoints to parent |
| **Subnet Actor** | Contract governing a specific child subnet (deployed in parent) |
| **Supply Source** | Source of subnet native coin (Native or ERC20) |
| **Top-down** | Information flow parent → child (finality, messages, validator changes) |
| **VoteTally** | Mechanism tracking validator votes on parent finality |

---

## Dependencies

### CometBFT

- **Role**: Consensus engine for subnet blocks
- **Version**: See Cargo.toml / go.mod in repo
- **Interface**: ABCI++
- **Critical for**: Block production, validator P2P, proposal/vote flow

### Other Key Dependencies

| Dependency | Usage |
|------------|-------|
| FVM | Execution (WASM actors, FEVM) |
| IPLD / Bitswap | Data availability, CID resolution |
| Ethers / Foundry | Contract deployment, testing |
| Filecoin / Lotus | Root network implementation |

---

**Document Version**: 1.0  
**Last Updated**: February 2026  
**Status**: Living document — expand with status, issues, open questions per section as needed.
