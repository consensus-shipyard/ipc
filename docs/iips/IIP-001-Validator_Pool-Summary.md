# IPC Shared Validator Pool - Executive Overview

**Version:** 1.0
**Date:** October 7, 2025
**Status:** Summary Document

> **Full Specification:** See [IIP-001-Validator_Pool.md](./IIP-001-Validator_Pool.md) for complete technical details.

---

## The Problem We're Solving

Users want to deploy application-specific subnets on demand—fast transaction processing, custom logic, and dedicated resources—without the complexity and cost of recruiting, managing, and coordinating their own validator sets. Today's options force developers to either:

- **Run their own infrastructure** (expensive, time-consuming, requires validator expertise)
- **Use shared L1s** (congested, expensive, slow)
- **Deploy on a single ecosystem** (locked into Filecoin OR Ethereum, not both)

This creates a significant barrier to entry for developers who want blockchain infrastructure that "just works."

## The Solution: Validators-as-a-Service

IPC introduces a **shared global validator pool** that enables on-demand subnet deployment across multiple Layer 1 ecosystems (Filecoin, Ethereum, and more). The Root IPC Chain automatically handles validator assignment, orchestration, reputation tracking, and rewards—making subnet deployment as simple as calling a smart contract.

### Key Benefits

**For Subnet Deployers:**
- Deploy a subnet in ~30-60 seconds
- No validator recruitment needed
- Pay only for validators you use
- Choose your parent L1 (Filecoin, Ethereum, or another subnet)
- Sub-second block times with configurable security

**For Validators:**
- Stake once (10,000 IPCT), validate anywhere
- Automatic assignments across multiple L1 ecosystems
- Reputation-based assignment and rewards
- No need to hold multiple L1 tokens (FIL, ETH, etc.)

### Scale Targets

- **Validators:** 10,000-30,000 globally
- **Execution Subnets:** 5,000-15,000 across all L1s
- **Block Times:** <1 second on execution subnets
- **Cross-L1 Communication:** ~5 seconds via IBC protocol
- **Aggregate Throughput:** 2.5M-25M TPS network-wide

---

## Architecture: Four-Layer Hierarchy

The IPC architecture separates concerns across four layers, enabling massive horizontal scaling:

```
┌────────────────────────────────────────────────────────┐
│              Root IPC Chain (L0)                       │
│                                                        │
│  • Global validator registry & IPCT staking            │
│  • Validator assignment & reputation                   │
│  • Cross-L1 coordination                               │
│  • Canonical IPCT token                                │
│                                                        │
│       Bridges ↕↕              Bridges ↕↕               │
└────────┬──────────────────────────────┬────────────────┘
         │                              │
    ┌────▼──────┐                 ┌────▼──────┐
    │ Filecoin  │                 │ Ethereum  │
    │ L1        │                 │ L1        │
    │           │                 │           │
    │ Gateway   │                 │ Gateway   │
    │ Contract  │                 │ Contract  │
    └────┬──────┘                 └────┬──────┘
         │                             │
    ┌────▼────────┐              ┌─────▼───────┐
    │ Shard 1     │              │ Shard 3     │
    │ (Global)    │              │ (Global)    │
    │             │              │             │
    │ Manages     │              │ Manages     │
    │ mixed L1    │              │ mixed L1    │
    │ subnets     │              │ subnets     │
    └────┬────────┘              └─────┬───────┘
         │                             │
    ┌────▼────────┐              ┌─────▼───────┐
    │ Subnet A    │◄────IBC─────►│ Subnet X    │
    │ (Filecoin)  │              │ (Ethereum)  │
    └─────────────┘              └─────────────┘
```

### Layer 0: Root IPC Chain

The **Root IPC Chain** is the global coordination layer that bridges multiple L1 ecosystems with a unified validator pool.

**Responsibilities:**
- **Global Validator Registry:** Single source of truth for all 10K-30K validators
- **Stake Management:** IPCT staking, slashing, and reward distribution
- **Validator Assignment:** Automated assignment to sharding subnets and execution subnets
- **Cross-L1 Routing:** Route messages and checkpoints between different L1 ecosystems
- **Bridge Coordination:** Manage bidirectional bridges to L1 Gateway Contracts
- **Reputation System:** Track validator performance across all L1s

**Consensus:** BFT consensus (50-100 elite validators), 6-second blocks

### Layer 1 (External): L1 Blockchains

**Supported L1s:** Filecoin (primary), Ethereum, and future EVM-compatible chains

**L1 Gateway Contracts** serve as the interface between IPC and each L1 ecosystem:

- **Validator Verification:** Store merkle root of global validator set (enables L1 to trust IPC validators without storing full set)
- **Subnet Deployment Interface:** Accept deployment requests from L1 users, forward to Root IPC
- **Checkpoint Reception:** Receive and verify checkpoints from execution subnets
- **Asset Bridging:** Lock/unlock native L1 tokens (FIL, ETH) for subnet use
- **wIPCT Bridge:** Mint/burn wrapped IPCT tokens (wIPCT-FIL, wIPCT-ETH)

**Key Point:** L1s provide finality anchoring and asset bridging, but subnets don't depend on L1s for liveness or security.

### Layer 2: Sharding Subnets

**Sharding subnets** manage validator pools and coordinate execution subnet operations. These are **parent-agnostic**—a single shard can manage subnets with Filecoin parents AND Ethereum parents simultaneously.

**Responsibilities:**
- Assign validators to execution subnets (regardless of L1 parent)
- Process checkpoints from execution subnets
- Route checkpoints to appropriate L1 Gateway Contracts
- Coordinate cross-subnet messaging (intra-shard)
- Report aggregate state to Root IPC

**Validator Pool per Shard:**
- 20-50 shard validators (secure the sharding subnet itself)
- 150-250 execution validators (assigned to execution subnets)
- Total: 170-300 validators per shard

**Scaling:** Root IPC automatically creates new shards when demand exceeds capacity (>250 validators or >120 subnets per shard).

### Layer 3+: Execution Subnets (Recursive)

**Execution subnets** are where user applications run with fast transaction processing.

**Characteristics:**
- Sub-second block times (target: 500ms-1s)
- 4-21 validators (configurable based on security needs)
- Can be children of L1 chains OR other subnets (unlimited recursion depth)
- Configurable consensus, runtime (WASM, EVM), and checkpoint intervals

**Types:**
- **Long-lived:** Permanent infrastructure (DeFi, gaming, enterprise apps)
- **Ephemeral:** Temporary subnets (minutes to hours) for specific tasks, auto-destruct after use

**Parent Selection:** Subnet operators choose their parent:
- Filecoin L1 (leverage Filecoin storage ecosystem)
- Ethereum L1 (leverage Ethereum DeFi ecosystem)
- Another subnet (recursive hierarchies for specialized use cases)

---

## IPCT Token & Multi-Chain Economics

### Why Deploy IPCT on Root IPC Chain?

The **Root IPC Chain** hosts the canonical IPCT token for several critical reasons:

**1. Unified Stake Across L1s**
- Validators stake **once** on Root IPC to serve **all** L1 ecosystems
- No need to stake separately on Filecoin, Ethereum, or other L1s
- Single collateral pool = maximum capital efficiency

**2. L1-Agnostic Economics**
- Validators earn the same rewards whether validating Filecoin-parent or Ethereum-parent subnets
- Prevents fragmentation of validator economics across L1s
- Simplifies validator experience dramatically

**3. Neutral Coordination Layer**
- Root IPC is not controlled by any single L1 ecosystem
- Fair, unbiased validator assignment across all L1s
- Enables true multi-chain validator marketplace

**4. Single Source of Truth**
- Total IPCT supply tracked in one place
- All slashing, rewards, and reputation in one registry
- L1 Gateway Contracts verify validators via merkle proofs from Root IPC

### Bridging IPCT to L1 Ecosystems

Users and subnet deployers need IPCT on L1s (Filecoin, Ethereum) to pay for subnet deployment and fees. IPC uses **wrapped IPCT** tokens on each L1:

#### Bridge Architecture

**Root IPC → Filecoin (wIPCT-FIL):**
1. User locks IPCT on Root IPC Chain
2. Root IPC sends cryptographic proof to Filecoin Gateway Contract
3. Gateway Contract verifies proof and mints wIPCT-FIL to user
4. User can now use wIPCT-FIL to pay for Filecoin-parent subnet fees

**Root IPC → Ethereum (wIPCT-ETH):**
1. Same process with Ethereum Gateway Contract
2. Gateway mints wIPCT-ETH
3. User can pay for Ethereum-parent subnet fees

**Reverse (wIPCT → IPCT):**
1. Burn wIPCT on L1 Gateway Contract
2. Gateway sends message to Root IPC with burn proof
3. Root IPC unlocks canonical IPCT to user

#### Bridge Security

- Secured by Root IPC validator set (50-100 high-reputation validators)
- Cryptographic proofs required for all mint/burn operations
- Multiple independent relayers (decentralized, no single point of failure)
- Regular audits of locked IPCT vs minted wIPCT supply
- Emergency pause mechanism via governance

**Trust Model:** Users trust the Root IPC validator set (same validators securing their subnets), not third-party bridge operators.

### Token Use Cases

**IPCT on Root IPC Chain:**
- Validator staking collateral (10,000 IPCT minimum)
- Governance voting
- Direct subnet deployment (advanced users)

**wIPCT on L1s (Filecoin, Ethereum):**
- Subnet deployment fees (call Gateway Contract)
- Subnet operational fees (pay validators)
- Cross-subnet message fees
- Can be traded on L1 DEXs (liquidity)

**Benefits of this model:**
- Validators never need to hold FIL or ETH to validate
- Subnet deployers can use native L1 tokens OR wIPCT (flexible)
- Single token unifies economics across multiple L1 ecosystems

---

## Validator System

### How It Works: Validator Perspective

**Step 1: Stake Once**
- Acquire 10,000+ IPCT (minimum stake requirement)
- Stake on Root IPC Chain (one-time operation)
- Register hardware specs and preferences

**Step 2: Automatic Assignment**
- Root IPC assigns you to a sharding subnet
- Sharding subnet assigns you to 1-2 execution subnets simultaneously
- You can validate subnets with ANY L1 parent (Filecoin, Ethereum, or subnet)
- **Multi-subnet validation:** Run multiple subnet clients in parallel, earn rewards from each
- **Weekly rotation:** Every week, ~1/3 of validators rotate to different execution subnets
  - Prevents long-term collusion
  - Exposure to different L1 ecosystems
  - Can switch from Filecoin-parent subnet to Ethereum-parent subnet seamlessly

**Step 3: Earn Rewards**
- Base rewards in IPCT (10-100 IPCT per day depending on assignments)
- Subnet fees (paid by subnet operators)
- Transaction tips (from users)
- Cross-subnet message routing fees
- Reputation multiplier (0.8x-1.5x based on performance)

### Validator Roles

| Role | Count per Shard | Responsibility | Reward Multiplier |
|------|----------------|----------------|-------------------|
| **Root Chain Validator** | 50-100 globally | Secure Root IPC, coordinate all L1s | 2.0x |
| **Shard Validator** | 20-50 | Secure sharding subnet, route checkpoints | 1.5x |
| **Execution Validator** | 150-250 | Validate transactions on execution subnets | 1.0-2.0x |

**Key Innovation:** Same validator can serve Filecoin-parent subnet one week, Ethereum-parent subnet the next week—no configuration changes needed.

### Reputation System

Validators earn reputation scores (0.0-2.0) based on:

- **Uptime:** Blocks proposed, attestations provided (30% weight)
- **Tenure:** Long-term participation bonus (20% weight)
- **Slash History:** Penalties for misbehavior (25% weight)
- **Network Contribution:** Governance, geographic diversity, multi-L1 support (25% weight)

**Benefits of Higher Reputation:**
- Better subnet assignments (higher-value subnets)
- Eligibility for Root Chain validator role
- Higher reward multipliers (up to 1.5x)
- Lower penalties for minor infractions

**Slashing Examples:**
- Missed block: -1 IPCT
- Double signing: -100 IPCT (1% of stake)
- Invalid state transition: -500 IPCT (5% of stake)
- Coordinated attack: -10,000 IPCT (100% of stake) + ban

---

## Data Availability & Storage

### How Subnet Data is Stored and Secured

When execution subnets process transactions, that data must be available for verification and potential reconstruction. IPC uses a **distributed erasure coding system** managed at the sharding subnet level—ensuring data remains available even if some validators go offline.

### Erasure Coding Process

Every execution subnet creates periodic **checkpoints** (default: every 100 seconds or ~100 blocks):

**Step 1: Checkpoint Creation**
- Execution subnet validators finalize a batch of blocks
- Create checkpoint containing:
  - State root (merkle root of current state)
  - Transaction merkle root
  - Block range (e.g., blocks 1000-1100)
  - Validator signatures (2/3+ threshold)
  - Parent checkpoint hash

**Step 2: Erasure Encoding**
- Checkpoint + block data is serialized
- Erasure coded into **N chunks** (N = number of validators in parent shard)
- **Redundancy factor: 2x** — Only need 50% of chunks to reconstruct full data
- Example: 200 validators in shard = 200 chunks, need any 100 to rebuild

**Step 3: Distributed Storage**
- Each chunk sent to different validator in parent sharding subnet
- Validators store chunks locally (SSD storage)
- Validators sign attestation: "I have chunk X of checkpoint Y"
- Checkpoint considered "available" when 66%+ of validators confirm

**Step 4: Checkpoint Routing**
- Sharding subnet routes checkpoint commitment to appropriate destination:
  - Filecoin-parent subnet → Filecoin Gateway Contract
  - Ethereum-parent subnet → Ethereum Gateway Contract
  - Subnet-parent → Parent subnet validators
- Aggregate checkpoint also sent to Root IPC Chain

### Storage Tiers: Hot vs Cold

**Hot Storage (Recent Data):**
- Last ~1,000 checkpoints (~27 hours of data)
- Maintained via erasure coding across shard validators
- Fast retrieval for recent queries and state verification
- Required for data availability challenges
- Applies to subnets with any L1 parent

**Cold Storage (Historical Data):**
- Data older than finality window (beyond hot storage)
- **Archived to Filecoin** regardless of subnet's parent L1
- Why Filecoin for all archival:
  - Filecoin is purpose-built for decentralized long-term storage
  - Even Ethereum-parent subnets benefit from cheap Filecoin archival
  - Single archival layer simplifies architecture
  - Creates natural cross-ecosystem integration

**Archival Process:**
1. Sharding subnet identifies checkpoints beyond hot storage window
2. Validators reconstruct full data from erasure coded chunks
3. Compress and batch multiple checkpoints
4. Create Filecoin storage deal via smart contract
5. Upload to Filecoin storage providers
6. Record Filecoin deal ID on Root IPC Chain
7. Validators prune local chunks, keep only commitment hash

### Data Availability Challenges

To ensure validators actually store the data they claim to:

**Random Sampling Protocol:**
- Root IPC Chain randomly selects checkpoints each epoch
- Challenges validators to provide specific erasure coded chunks
- Validators must respond within 30 seconds
- **Success:** Validator earns +1 IPCT bonus
- **Failure:** Validator slashed -10 IPCT

This keeps validators honest and ensures data remains retrievable.

### Benefits of This Model

**For Subnet Deployers:**
- Data automatically distributed and backed up
- No manual storage management
- Can retrieve any historical state from Filecoin
- Same storage model regardless of L1 parent

**For the Network:**
- No single point of failure (distributed across 100+ validators)
- 50% of shard validators can go offline and data remains available
- Efficient storage (erasure coding is space-efficient)
- Long-term archival handled by Filecoin (specialized storage network)

**Cross-L1 Benefit:**
- Ethereum-parent subnets get cheap permanent storage via Filecoin
- Filecoin-parent subnets naturally integrate with native storage
- Unified retrieval mechanism for all historical data

---

## Subnet Deployment: Simple as a Function Call

### How to Deploy a Subnet

**Option 1: Via L1 Gateway Contract (Recommended)**

For a Filecoin-parent subnet:
```javascript
// Call Filecoin Gateway Contract
FilecoinGateway.deploySubnet({
    min_validators: 7,              // Security level
    checkpoint_interval: 100,       // seconds
    security_tier: "standard",      // economy/standard/high
    runtime: "evm",                 // wasm/evm
    prepaid_fees: 1000              // IPCT (or wIPCT-FIL)
})

// Returns: subnet_id and RPC endpoints in ~30-60 seconds
```

For an Ethereum-parent subnet:
```javascript
// Call Ethereum Gateway Contract
EthereumGateway.deploySubnet({...})  // Same interface
```

**Under the hood:**
1. Gateway Contract receives request
2. Forwards to Root IPC Chain
3. Root IPC selects optimal sharding subnet
4. Shard assigns 7 validators from global pool
5. Validators initialize subnet and start producing blocks
6. Subnet begins checkpointing to Gateway Contract
7. You receive subnet_id and can start deploying contracts

**Total time: ~30-60 seconds**

**Option 2: Direct to Root IPC (Advanced)**
- More configuration options
- Programmatic deployments
- Supports recursive subnet parents

### Subnet Types

**Long-Lived Subnets:**
- Continuous operation (months to years)
- Persistent state
- Validator rotation for security
- Use cases: DeFi, gaming, enterprise apps

**Cost Example (7 validators):**
- ~1,009 IPCT per month ($100-1,000 depending on IPCT price)
- Same cost for Filecoin or Ethereum parent

**Ephemeral Subnets:**
- Short-lived (minutes to hours)
- Auto-destruct after duration
- Optional archival to Filecoin
- Use cases: AI/ML inference, temporary game servers, testing

**Cost Example (5 validators, 60 minutes):**
- ~11.5 IPCT ($1-100 depending on IPCT price)
- Paid upfront

### Configurable Parameters

- **Parent Type:** Filecoin, Ethereum, or another subnet
- **Security Tier:** 4-21 validators (economy to high security)
- **Checkpoint Interval:** How often to checkpoint to parent (default: 100s)
- **Runtime:** WASM, EVM, or custom
- **Consensus:** CometBFT or custom
- **Geographic Preference:** Optional regional validator preference
- **Lifetime:** Permanent (0) or ephemeral (duration in seconds)

---

## Cross-L1 Communication: Breaking Down Silos

### The Multi-Chain Challenge

Traditional blockchain ecosystems operate in silos:
- Filecoin apps can't easily talk to Ethereum apps
- Bridging assets is slow (minutes to hours) and expensive
- Users must choose one ecosystem or manage multiple wallets
- Liquidity and users are fragmented

### IPC Solution: Direct Cross-L1 Communication

IPC enables **subnets with different L1 parents to communicate directly** via the IBC protocol—no need to route through L1s.

#### Example: Filecoin Subnet ↔ Ethereum Subnet

**Scenario:** Storage marketplace on Filecoin-parent subnet wants to accept payments from DeFi app on Ethereum-parent subnet.

**Traditional approach:**
1. Ethereum app sends transaction to Ethereum L1 (~15 min finality)
2. Bridge Ethereum → Filecoin (~30 min)
3. Filecoin app processes on Filecoin L1 (~30 min)
4. Total: **~75 minutes, high fees**

**IPC approach with direct IBC:**
1. Ethereum-parent subnet sends IBC message to Filecoin-parent subnet
2. Both subnets run CometBFT consensus (compatible)
3. Validator signatures verified via Root IPC merkle roots
4. Message delivered peer-to-peer
5. Total: **~5 seconds, minimal fees**

#### Benefits of Cross-L1 Communication

**Speed:**
- Intra-shard messaging: ~5 seconds (same or different L1 parents)
- Inter-shard messaging: ~20-40 seconds (fast path via Root IPC relay)
- **No L1 finality needed** for optimistic execution

**Cost:**
- Subnet transaction fees (1000x cheaper than L1)
- Small routing fee (0.01-0.05 IPCT depending on path)
- No expensive L1 gas fees

**Security:**
- Secured by IPC validators (BFT consensus with 2/3+ threshold)
- Validator sets verified via Root IPC merkle proofs
- Cryptographic signatures prevent message forgery
- Optional finality proof via checkpoints to L1s

**User Experience:**
- Seamless cross-L1 interactions
- Users unaware of underlying L1 differences
- Single wallet, single transaction
- Apps can compose across ecosystems

#### Practical Use Cases

**Cross-L1 DeFi:**
- Borrow ETH on Ethereum subnet, use FIL on Filecoin subnet as collateral
- Arbitrage opportunities across L1 ecosystems
- Unified liquidity pools spanning multiple L1s

**Cross-L1 Storage + Compute:**
- Store data on Filecoin subnet (cheap storage)
- Process with compute on Ethereum subnet (DeFi integration)
- Coordinate via fast IBC messages

**Cross-L1 Gaming:**
- Game logic on Filecoin subnet (fast, cheap)
- NFT marketplace on Ethereum subnet (established ecosystem)
- Real-time asset transfers via IBC

**Cross-L1 Enterprise:**
- Private consortium subnet on Filecoin
- Public-facing services on Ethereum
- Secure cross-L1 data sharing

### Future: Multi-L1 Liquidity

As IPC adds more L1s (Polygon, Avalanche, Cosmos), cross-L1 communication becomes even more powerful:

- **Unified DEX:** Trade FIL, ETH, AVAX, MATIC on single subnet with ~5 second swaps
- **Cross-L1 Governance:** Vote on Filecoin subnet, execute on Ethereum subnet
- **Multi-Chain dApps:** Deploy once, access users from all L1 ecosystems

---

## Scaling & Roadmap

### Scale Projections

**Conservative (Year 3):**
- 5,000 validators
- 1,000 execution subnets (60% Filecoin, 30% Ethereum, 10% others)
- 10-20 sharding subnets
- 500K-1M TPS aggregate

**Aggressive (Year 5+):**
- 30,000 validators
- 10,000 execution subnets across all L1s
- 50-100 sharding subnets
- 2.5M-25M TPS aggregate
- 10M-100M cross-L1 messages per day

### Rollout Phases

**Phase 1 (Months 1-6): Single L1 Foundation**
- Root IPC Chain operational
- Filecoin Gateway deployed
- Single sharding subnet
- 200 validators, 50-100 execution subnets
- Prove architecture with Filecoin ecosystem

**Phase 2 (Months 6-12): Multi-L1 Introduction**
- Ethereum Gateway deployed
- Enable wIPCT-ETH minting
- First cross-L1 IBC messages
- 2-5 sharding subnets
- 500 validators, 100-300 execution subnets

**Phase 3 (Months 12-24): Full Multi-Shard Operation**
- 5-20 sharding subnets
- 1,000-5,000 validators
- Automatic shard creation/rebalancing
- High cross-L1 message volumes
- Additional L1s (Polygon, Avalanche)

**Phase 4 (Year 2+): Mature Multi-Chain Network**
- 20-50 sharding subnets
- 5,000-10,000 validators
- 2,000-5,000 execution subnets
- Production-ready for enterprise

---

## Getting Started

### For Subnet Deployers

**Ready to deploy your subnet?**

1. **Acquire wIPCT** on your preferred L1 (Filecoin or Ethereum)
2. **Call Gateway Contract** with your subnet configuration
3. **Receive subnet_id and RPC endpoints** in ~30-60 seconds
4. **Deploy your application** and start processing transactions

**Resources:**
- Deployment guide: [docs/ipc/usage.md](../ipc/usage.md)
- Configuration reference: [docs/ipc/contracts.md](../ipc/contracts.md)
- Testnet access: [Contact IPC team]

### For Validators

**Ready to join the validator pool?**

1. **Acquire 10,000+ IPCT** (minimum stake)
2. **Set up infrastructure** (16+ cores, 32+ GB RAM, 2+ TB SSD)
3. **Stake on Root IPC Chain** and register metadata
4. **Get automatically assigned** and start earning rewards

**Resources:**
- Validator setup guide: [docs/ipc/validators.md](../ipc/validators.md)
- Hardware requirements: [docs/ipc/requirements.md](../ipc/requirements.md)
- Testnet participation: [Contact IPC team]

### For Developers

**Build cross-L1 applications:**
- IPC SDK documentation: [docs/ipc/sdk.md](../ipc/sdk.md)
- Cross-subnet messaging guide: [docs/ipc/messaging.md](../ipc/messaging.md)
- Example applications: [demos/](../../demos/)

---

## Conclusion

The IPC Shared Validator Pool transforms subnet deployment from a complex, multi-week validator recruitment process into a simple function call. By introducing a global validator pool coordinated by the Root IPC Chain, we enable:

- **On-demand subnet deployment** (~30-60 seconds)
- **Validators-as-a-Service** (stake once, serve anywhere)
- **Multi-L1 ecosystem support** (Filecoin, Ethereum, and more)
- **Fast cross-L1 communication** (~5 seconds via IBC)
- **Massive scale** (10K-30K validators, 5K-15K subnets)

This architecture makes blockchain infrastructure as accessible as cloud computing—developers focus on building applications, and IPC handles the complex validator coordination behind the scenes.

**For complete technical details,** see the full specification: [IIP-001-Validator_Pool.md](./IIP-001-Validator_Pool.md)

---

**Questions or feedback?** Contact the IPC team or visit our documentation at [docs/ipc/](../ipc/)
