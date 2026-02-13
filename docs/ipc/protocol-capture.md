# IPC Protocol Capture

> Protocol capture document describing how IPC works, components, mechanisms, status, open questions, and dependencies.

## Table of Contents

1. [Intro / Overview](#intro--overview)
2. [Concepts (Mechanisms)](#concepts-mechanisms)
3. [Components](#components)
4. [Operation](#operation)
5. [Security Guarantees](#security-guarantees)
6. [Configuration](#configuration)
7. [Terms (Glossary)](#terms-glossary)
8. [Dependencies](#dependencies)
9. [Appendices](#appendices)

---

## Intro / Overview

### Introduction

**InterPlanetary Consensus (IPC)** is a flexible blockchain extensibility solution—a **sidechain system** in which users can delegate operations to dedicated autonomous blockchains (subnets) whose state is anchored back into higher-security, less-capable chains. Subnets can be further extended with lower-level subnets in a **recursive** manner.

IPC subnets are full, sufficiently **decentralized** blockchains **owning their data**, unlike rollups that rely on centralized sequencers and post significant data to the main chain. Subnets are **highly configurable**, **specialized**, and **fine-tuned**. IPC imposes **minimal requirements** on the target chain’s capabilities; subnets can even be anchored into chains with limited programmability (e.g., Bitcoin). Anchoring subnet state into higher-security chains provides **checkpoint-based objectivity**, facilitating dynamic joining and light clients secured against long-range attacks without social consensus. Transitively checkpointing into a common chain establishes **eventual consistency** across autonomous subnets, improving **composability** and coordination of cross-chain operations.

### Features

| Feature | Description |
|---------|-------------|
| **Recursive scalability** | Subnets can spawn child subnets indefinitely, forming a tree hierarchy |
| **Cross-net communication** | Secure message passing between parent and child (top-down and bottom-up) |
| **Flexible consensus** | Each subnet can customize block time, validator requirements, and consensus parameters |
| **Ethereum compatibility** | Full FEVM support for Solidity smart contracts |
| **Native FVM support** | Access to Filecoin's actor model and built-in capabilities |

**Expanded capability areas** (from conceptual framing):

- **Dynamic scalability**: Subnets optimized for performance; checkpoint-based objectivity enables trustless bootstrapping; elastic scaling through programmatic launch/termination
- **Customizability**: Configurable consensus, execution (WASM & EVM), tokenomics, data availability, governance; permissioned or permissionless; regional or application-specific
- **Seamless interoperability**: Native cross-chain communication; protocol-native bridging secured by checkpointing; eventual consistency across subnets
- **Autonomy & sovereignty**: Subnets govern lifecycle, configuration, validator sets; partition tolerance (local finality when disconnected)
- **Objective security**: Checkpoint-based objectivity; trust anchors verifiable without social consensus; long-range attack mitigation
- **Risk containment**: Security boundaries between subnets; failures confined to affected subnet

### Core Concepts

| Concept | Definition |
|---------|------------|
| **Subnet** | Autonomous blockchain whose state is anchored into a parent chain |
| **Parent chain** | Blockchain that a subnet anchors into |
| **Rootnet** | Blockchain with no parent (e.g., Filecoin) |
| **Subnet ID** | Address identifying the subnet (root chain ID + route of subnet actor addresses) |
| **Checkpoint** | Periodic commitment of subnet state to parent; cryptographic reference to finalized chain head |
| **Cross-net messages** | Messages between chains; top-down (parent→child, deposits) and bottom-up (child→parent, withdrawals) |

> **Note**: Currently, cross-net messages are supported for directly linked chains only (single-hop parent↔subnet).

### How IPC Works (Conceptual)

IPC combines several mechanisms:

- **Checkpointing**: Anchors subnet state into parent; ensures checkpoints correspond to actual state; enables bridging
- **Bridging**: Secures transfer of information and assets; relies on checkpointing; uses cross-net messages
- **Subnet ID management**: Governs subnet ID namespaces and assignment
- **Subnet lifecycle management**: Registration, activation, termination
- **Subnet configuration management**: Validator set, voting power, config updates

**Flow**: Given a rootnet (root of trust), new subnets obtain a subnet ID and register. Lifecycle management governs activation (e.g., collateral for PoS). Configuration management handles validator set and updates. Subnets operate autonomously; checkpoints periodically bind state to parent. Bridging uses the same cross-net message carrier for deposits (lock+mint) and withdrawals (burn+release).

### Key Principles

*Proposed from protocol design goals—to be refined:*

- **Objectivity**: Checkpoint-based trust anchors; no social consensus required
- **Autonomy**: Self-governance, self-sovereignty; subnets own their data
- **Impact containment**: Firewalling; failures isolated to affected subnet
- **Flexibility**: Modularity, customizability, extensibility
- **Dynamicity**: On-demand scaling, elasticity
- **Partition tolerance**: Local finality when disconnected; consistency restored on reconnection
- **Seamless interoperability**: Native cross-net communication; eventual consistency

### Requirements & Properties

*Proposed placement: overlaps with [Security Guarantees](#security-guarantees) (trust assumptions) and [Configuration](#configuration) (constraints). Could be a short subsection here or consolidated there.*

- **Core assumptions**: Rootnet exists and is trusted; >2/3 validator honesty; reliable parent RPC
- **Constraints**: Single-hop messaging (current); supply source immutability
- **Guarantees**: Message integrity, ordering, value safety, risk containment

---

## Concepts (Mechanisms)

*Abstract/conceptual level—spans multiple components.*

### Subnet ID Management

**SubnetID** = `root` (chain ID) + `route` (array of subnet actor addresses top-to-bottom). String format: `/r314159/t410f.../t410f...`

- Assigned when subnet actor is deployed (via Registry or custom)
- Route grows as child subnets are created; each child gets a new subnet actor in its parent
- Binary: `keccak256(abi.encode(SubnetID))` for equality checks and storage

### Subnet Lifecycle Management

1. **Creation**: Deploy Subnet Actor in parent (via Registry or custom); configure supply source, permission mode, genesis
2. **Registration**: Once min validators/collateral met, Subnet Actor calls `register` on parent Gateway
3. **Active**: Subnet produces blocks, creates checkpoints, processes messages
4. **Termination**: Subnet can be killed (governance-dependent); collateral released per checkpoint confirmation

### Subnet Configuration Management

Validator set and voting power managed on parent, propagated via top-down:

- **StakeChangeRequest** (join, leave, stake, unstake, setFederatedPower) identified by configuration number
- Propagated via parent finality commitment in child blocks
- Child executes changes; confirms via `next_configuration_number` in checkpoint
- Parent confirms collateral release when checkpoint is committed

### Checkpointing

Periodically anchors subnet state into parent.

**Flow**:
1. Every `bottomup_check_period` blocks (or when `MAX_MSGS_PER_BATCH` hit), validators create checkpoint
2. Validators call `addCheckpointSignature`; quorum (>2/3) triggers `QuorumReached` event
3. Relayer submits checkpoint + signatures to parent
4. Parent verifies against last known validator set, executes bottom-up messages

**Contents**: subnet_id, block_height, block_hash, prev_checkpoint_height, cross_messages, next_configuration_number. Block hash anchors against long-range attacks.

### Messaging

Transport layer for cross-subnet communication.

**Flows**:
- **Top-down**: Messages indexed by parent block height; propagated via parent finality; executed when finality committed
- **Bottom-up**: Batched in checkpoints; validators sign; relayer submits; executed on parent

**Types**: Transfer (fund, release), Call (general contract-to-contract), Result (response). Mechanisms: IpcEnvelope, postbox, LCA routing.

### Bridging

Moving value/assets between subnets—built on messaging.

**Native**: Supply source (Native or ERC20). fund/release for native; lock+mint / burn+release for ERC20.

**Custom**: Linked Token pattern for arbitrary ERC20s via general cross-net Call messages. Lock on origin, mint on target; burn on target, release on origin.

### Consensus

- **Subnet consensus**: CometBFT (BFT, >2/3 honest)
- **Parent finality**: Validators poll parent, vote on parent block via GossipSub; VoteTally detects quorum; proposer includes `ParentFinality` in block

### On-Chain vs Off-Chain

| Aspect | On-Chain | Off-Chain |
|--------|----------|-----------|
| Gateway, Subnet Actor, Registry | ✓ | |
| Message execution, checkpoint validation | ✓ | |
| Consensus (CometBFT) | Runs on nodes | |
| Parent Syncer, IPLD Resolver | | Polls/resolves |
| Relayer, IPC Provider, ipc-cli | | Submits, queries, signs |

---

## Components

### Actors (WASM)

IPC subnets run the **FVM**, which executes **WASM-based actors** (Filecoin execution model):

- Built-in actors (Account, Init, EVM, etc.)
- Custom actors deployable by subnets
- Shared across root (Lotus) and child subnets (Fendermint)

### Contracts (Solidity) — Onchain Actors

**Status**: 🔴 *Needs consolidation with implementation details*

| Contract | Location | Responsibility |
|----------|----------|----------------|
| **Gateway** | Singleton in every subnet | Collateral, firewall, cross-net routing, checkpoint validation |
| **Subnet Actor** | One per child, deployed in parent | Supply source, genesis, permission mode, validator set |
| **Registry** | Per subnet | Factory for Subnet Actor implementations |

**Architecture**: Diamond pattern (EIP-2535), AppStorage for shared state.

### Node (Subnet Validator Node)

**Fendermint** — subnet peer implementation:

| Layer | Component | Responsibility |
|-------|-----------|----------------|
| Consensus | CometBFT | BFT consensus, block proposal, voting, P2P |
| Application | ABCI++ | PrepareProposal, ProcessProposal, FinalizeBlock |
| Execution | FVM/FEVM | Smart contract execution |
| Parent Sync | Parent Syncer | Polls parent RPC, VoteTally for finality |
| Data | IPLD Resolver | CID resolution, data availability |

**Node lifecycle**: Genesis → Bootstrap (register) → Validation → Shutdown (leave/unstake).

### IPC Provider (Parent Chain Provider)

**Status**: 🟢 [needs review]  
**Code**: [ipc/provider/src/lib.rs](https://github.com/consensus-shipyard/ipc/blob/main/ipc/provider/src/lib.rs)

Wraps ethers library and contract ABIs for parent chain interaction. Used by ipc-cli and relayer.

**Responsibilities**:
- Calling parent contracts (Gateway, Subnet Actor, Registry)
- Subnet lifecycle interaction (create, join, leave, list)
- Fetching subnet genesis and top-down queries (messages, validator changes)
- Enabling relayer to interact with parent and child

### Relayer (Off-Chain)

**Status**: 🟡

**Overview**: Monitors child subnet for `QuorumReached` events; assembles checkpoint + validator signatures; submits to parent Gateway via `submitCheckpoint`.

**Flow**: L2 contract event → retrieve signed checkpoint → construct proofs (CometBFT light client) → submit to L1 for execution.

**Dependencies**: Requires funds on L1 to submit transactions.

**Known issues / improvement areas**:
- Needs better events/observability
- No rewards system for incentivization (message fees exist but mechanism needs refinement)
- First-come-first-serve: issues if relayer goes down; no built-in redundancy protocol

---

## Operation

### Creating & Destroying Subnets

**Create**: Deploy Subnet Actor via Registry or custom; configure min_validators, min_collateral, supply_source; wait for bootstrap (join + collateral). Subnet Actor calls `register` on Gateway when ready.

**Destroy**: Governance-dependent; subnet can be killed; validators leave; collateral released as checkpoints confirm.

### Joining & Leaving Subnets

**Join**: Call `join` on Subnet Actor with collateral; creates StakeChangeRequest; propagated via top-down finality; child executes; confirmed in checkpoint.

**Leave**: Call `leave` or `unstake`; same propagation; child removes/reduces; parent releases collateral when checkpoint committed. Claim via `ipc-cli subnet claim`.

### Depositing & Withdrawing Assets

**Deposit (fund)**: Call `fund` (or `fundWithToken` for ERC20) on parent Gateway; top-down message; executed in child when parent finality committed.

**Withdraw (release)**: Call `release` on child Gateway; bottom-up message; batched in checkpoint; relayer submits; executed on parent.

### General Message Passing

Contract-to-contract via `sendContractXnetMessage` (Call kind). IpcEnvelope carries payload; postbox routes at intermediate subnets; destination contract implements `handleIpcMessage`. Result messages propagate back. *Currently single-hop for direct parent-child; multi-hop routing supported in protocol.*

---

## Security Guarantees

### Trust Assumptions

1. **L1 Security**: Root network (e.g., Filecoin) is trusted and secure
2. **Validator honesty**: >2/3 of validator power is honest in each subnet
3. **Economic security**: Collateral discourages malicious behavior
4. **Finality delay**: Parent blocks delayed sufficiently to avoid reorgs
5. **RPC reliability**: Validators have access to reliable parent RPC

### Attack Mitigations

| Threat | Mitigation |
|--------|------------|
| Long-range attacks | Block hash anchored in checkpoints |
| Double-spend | BFT consensus + checkpoint finality |
| Validator collusion | Requires >2/3 stake; economically irrational |
| Checkpoint censorship | Multiple relayers provide redundancy |
| Parent reorg | Finality delay ensures stability |

### Guarantees Provided

- **Message integrity**: Envelopes signed/verified
- **Ordering**: Deterministic execution via parent finality and checkpoint ordering
- **Value safety**: Lock-before-mint; burn-before-release
- **Validator set sync**: Configuration number confirms membership; parent releases collateral only after child confirms
- **Risk containment**: Failures isolated to affected subnet

### Caveats

- Security decreases with hierarchy depth
- Federated subnets rely on admin honesty
- Gateway/Registry upgrades may introduce trust

### Economic Incentives

**Top-down**: Gas paid in parent; no additional fee in child.

**Bottom-up**: Cross-message fee paid by sender (rewards relayer); execution gas paid by relayer (compensated by fee); fee set by subnet policy.

**Collateral**: Required for collateral mode; slashing/penalties depend on subnet governance.

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
| `supply_source_address` | ERC20 address (if ERC20) |

### Operational Parameters

| Parameter | Typical Value | Description |
|-----------|---------------|-------------|
| `finality_delay` | 10–20 blocks | Parent blocks before final |
| `max_proposal_range` | ~50 blocks | Max parent height advance per child block |
| `MAX_MSGS_PER_BATCH` | 100 | Triggers intermediate checkpoint |

### Configuration Files

- **ipc-cli**: `~/.ipc/config.toml` — subnets, keystore_path
- **Fendermint**: `~/.fendermint/config/` — ABCI, DB, resolver, eth RPC

---

## Terms (Glossary)

| Term | Definition |
|------|------------|
| **ABCI++** | Application Blockchain Interface; CometBFT ↔ application |
| **Bottom-up** | Child → parent (checkpoints, messages) |
| **Checkpoint** | Batch of child state + messages with validator signatures |
| **Configuration ID** | Version for validator set |
| **Deposit** | Top-down message carrying assets (fund) |
| **Gateway** | Singleton contract managing IPC in each subnet |
| **LCA** | Lowest common ancestor (routing) |
| **Quorum** | >2/3 validator power |
| **Relayer** | Off-chain process submitting checkpoints |
| **Rootnet** | Chain with no parent |
| **Subnet Actor** | Contract governing a child subnet (in parent) |
| **Supply Source** | Source of subnet native coin (Native or ERC20) |
| **Top-down** | Parent → child (finality, messages) |
| **VoteTally** | Tracks validator votes on parent finality |
| **Withdrawal** | Bottom-up message carrying assets (release) |

---

## Dependencies

### CometBFT

Consensus engine for subnet blocks. ABCI++ interface. Critical for block production, P2P, proposal/vote flow.

### Other Key Dependencies

| Dependency | Usage |
|------------|-------|
| FVM | Execution (WASM, FEVM) |
| IPLD / Bitswap | Data availability, CID resolution |
| Ethers / Foundry | Contracts, testing |
| Filecoin / Lotus | Root network |

---

## Appendices

*Supplementary information: detailed specs, diagrams, implementation notes.*

- [IPC Actors Spec](../../specs/ipc-actors.md)
- [Supply Sources](../../specs/supply-sources.md)
- [Addressing](../../specs/addressing.md)
- [Architecture Overview](./architecture-overview.md)
- [Architecture Quick Reference](./architecture-quick-reference.md)

---

## Merge Notes (v2)

*Mapping from source document to this version:*

| Source Section | Placement |
|----------------|-----------|
| **Introduction, Features, Core Concepts, How IPC Works** | Merged into [Intro / Overview](#intro--overview) |
| **Key Principles** (was TODO) | Added with proposed principles; marked for refinement |
| **Requirements & Properties** (was TODO) | Added as short subsection with note on overlap with Security/Configuration |
| **Abstract/Conceptual Mechanisms** | Same as [Concepts (Mechanisms)](#concepts-mechanisms) — merged |
| **Subnet ID / Lifecycle / Config Management** (were TODO) | Filled from codebase; now under Concepts |
| **Components: Onchain Actors** | → [Contracts (Solidity)](#contracts-solidity--onchain-actors); added status 🔴 |
| **IPC Provider** | Added as new [Component](#ipc-provider-parent-chain-provider); status 🟢 |
| **Relayer** | Enriched with status 🟡, dependencies, known issues |
| **Operation** (was new) | Added new section: Creating/Destroying, Join/Leave, Deposit/Withdraw, General Message Passing |
| **Security & Economics** | [Security Guarantees](#security-guarantees) + new [Economic Incentives](#economic-incentives) subsection |
| **Glossary** | Merged into [Terms (Glossary)](#terms-glossary) |
| **Appendices** | Added with links to specs and architecture docs |

---

**Document Version**: 2.0  
**Last Updated**: February 2026  
**Status**: Living document — merge of protocol capture v1 with enriched source. Expand status, issues, open questions per section as needed.
