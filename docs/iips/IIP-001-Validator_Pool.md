# IPC Shared Validator Pool Architecture Specification

**Version:** 2.0  
**Date:** October 3, 2025  
**Status:** Draft

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Architecture Overview](#2-architecture-overview)
3. [Network Hierarchy](#3-network-hierarchy)
4. [L1 Integration & Gateway Contracts](#4-l1-integration--gateway-contracts)
5. [Validator System](#5-validator-system)
6. [Subnet Deployment & Management](#6-subnet-deployment--management)
7. [Data Availability Layer](#7-data-availability-layer)
8. [Cross-Subnet Communication](#8-cross-subnet-communication)
9. [Token Economics & Incentives](#9-token-economics--incentives)
10. [Scaling Mechanisms](#10-scaling-mechanisms)
11. [Security & Fault Tolerance](#11-security--fault-tolerance)
12. [Bootstrap & Migration Path](#12-bootstrap--migration-path)
13. [Performance Targets](#13-performance-targets)
14. [Future Considerations](#14-future-considerations)

---

## 1. Executive Summary

This specification describes a multi-chain, hierarchical architecture for IPC (Interplanetary Consensus) that enables a single global validator pool to secure thousands of execution subnets across multiple Layer 1 blockchains (Filecoin, Ethereum, etc.) while achieving sub-second block times. The design features four layers: external Layer 1 chains (Filecoin, Ethereum), a Root IPC Chain for global coordination, auto-scaling sharding subnets for regional validator management, and user-deployed execution subnets for fast transaction processing.

**Key Features:**
- Single global validator pool across multiple L1 ecosystems
- Support for Filecoin, Ethereum, and other EVM-compatible L1s
- Recursive subnet hierarchy (subnets can be children of L1s or other subnets)
- Sub-second block times on execution subnets
- Support for 10,000-30,000 validators
- Support for 5,000-15,000 execution subnets across all L1s
- Automatic scaling via dynamic sharding subnet creation
- Direct cross-subnet communication via IBC protocol (even across different L1s)
- Parent-agnostic sharding subnets
- Support for both long-lived and ephemeral subnets
- Reputation-based validator incentives
- Integration with Filecoin for archival storage
- Canonical IPCT token with cross-chain bridges

---

## 2. Architecture Overview

### 2.1 Design Principles

1. **Multi-Chain Native:** Support multiple L1 ecosystems (Filecoin, Ethereum) with a single validator pool
2. **Separation of Concerns:** Execution (fast, local) is separated from data availability (global, asynchronous) and from L1 finality
3. **Hierarchical Scaling:** Four-tier architecture enables horizontal scaling across multiple L1s
4. **Validators-as-a-Service:** Shared global validator pool maximizes efficiency
5. **Recursive Subnets:** Unlimited subnet nesting depth (subnet → subnet → ... → L1)
6. **Configurable Security:** Subnets choose their own security parameters and parent relationships
7. **Seamless Abstraction:** L1 and sharding complexity hidden from end users
8. **Independent but Anchored:** Subnet security is independent but can utilize parent L1 for finality

### 2.2 Network Topology

```
┌─────────────────────────────────────────────────────────────────┐
│                        Root IPC Chain (L0)                          │
│                                                                     │
│  • Global Validator Registry & Stake Management (IPCT)             │
│  • Sharding Subnet Coordination                                    │
│  • Cross-L1 & Cross-Shard Message Routing                          │
│  • Canonical IPCT Token                                            │
│                                                                     │
│       Bridges ↕↕↕                              Bridges ↕↕↕         │
└─────────────┬─────────────────────────────────────────┬────────────┘
              │                                       │
              │                                       │
     ┌────────▼──────────┐                  ┌────────▼─────────┐
     │  Filecoin L1      │                  │  Ethereum L1     │
     │                   │                  │                  │
     │  [Gateway         │                  │  [Gateway        │
     │   Contract]       │                  │   Contract]      │
     │                   │                  │                  │
     │   • Validator     │                  │   • Validator    │
     │     verification  │                  │     verification │
     │   • Subnet        │                  │   • Subnet       │
     │     deployment    │                  │     deployment   │
     │   • Checkpoint    │                  │   • Checkpoint   │
     │     reception     │                  │     reception    │
     │   • wIPCT-FIL     │                  │   • wIPCT-ETH    │
     │     bridge        │                  │     bridge       │
     └────────┬──────────┘                  └────────┬─────────┘
              │                                      │
              │                                      │
       ┌──────┴──────────┐                   ┌──────┴──────────┐
       │                 │                   │                 │
 ┌─────▼─────┐     ┌────▼──────┐      ┌────▼──────┐    ┌────▼──────┐
 │  Shard 1  │     │  Shard 2  │      │  Shard 3  │    │  Shard 4  │
 │ (Global)  │     │ (Global)  │      │ (Global)  │    │ (Global)  │
 │           │     │           │      │           │    │           │
 │ Parent-   │     │ Parent-   │      │ Parent-   │    │ Parent-   │
 │ agnostic  │     │ agnostic  │      │ agnostic  │    │ agnostic  │
 │           │     │           │      │           │    │           │
 │ Manages   │     │ Manages   │      │ Manages   │    │ Manages   │
 │ mixed L1  │     │ mixed L1  │      │ mixed L1  │    │ mixed L1  │
 │ subnets   │     │ subnets   │      │ subnets   │    │ subnets   │
 └─────┬─────┘     └─────┬─────┘      └─────┬─────┘    └─────┬─────┘
       │                 │                  │                │
       │                 │                  │                │
 ┌─────▼──────┐    ┌────▼──────┐     ┌────▼──────┐   ┌─────▼──────┐
 │  Subnet A  │    │ Subnet B  │     │ Subnet X  │   │ Subnet Y   │
 │            │    │           │     │           │   │            │
 │ Parent:    │    │ Parent:   │     │ Parent:   │   │ Parent:    │
 │ Filecoin   │    │ Filecoin  │     │ Ethereum  │   │ Ethereum   │
 │            │    │           │     │           │   │            │
 │ Validators │    │Validators │     │Validators │   │ Validators │
 │ from       │    │from       │     │from       │   │ from       │
 │ Shard 1    │    │Shard 2    │     │Shard 3    │   │ Shard 4    │
 │            │    │           │     │           │   │            │
 │ Checkpoints│    │Checkpoints│     │Checkpoints│   │ Checkpoints│
 │ to Filecoin│    │to Filecoin│     │to Ethereum│   │ to Ethereum│
 │ Gateway    │    │Gateway    │     │Gateway    │   │ Gateway    │
 └─────┬──────┘    └───────────┘     └───────────┘   └─────┬──────┘
       │                                                     │
       │                                                     │
 ┌─────▼──────┐                                       ┌─────▼──────┐
 │ Subnet A1  │                                       │ Subnet Y1  │
 │            │                                       │            │
 │ Parent:    │                                       │ Parent:    │
 │ Subnet A   │                                       │ Subnet Y   │
 │            │                                       │            │
 │ Validators │                                       │ Validators │
 │ from       │                                       │ from       │
 │ Shard 1    │                                       │ Shard 4    │
 │            │                                       │            │
 │ Checkpoints│                                       │ Checkpoints│
 │ to         │                                       │ to         │
 │ Subnet A   │                                       │ Subnet Y   │
 │            │                                       │            │
 │ Grandparent│                                       │ Grandparent│
 │ Filecoin   │                                       │ Ethereum   │
 └────┬───────┘                                       └────┬───────┘
      │                                                    │
      │                                                    │
 ┌────▼────────┐                                     ┌────▼────────┐
 │ Subnet A1a  │                                     │ Subnet Y1a  │
 │             │                                     │             │
 │ Parent:     │                                     │ Parent:     │
 │ Subnet A1   │                                     │ Subnet Y1   │
 │             │                                     │             │
 │ Great-      │                                     │ Great-      │
 │ grandparent │                                     │ grandparent │
 │ Filecoin    │                                     │ Ethereum    │
 └─────────────┘                                     └─────────────┘

Cross-L1 Subnet Communication (via IBC):
Subnet A ↔──────IBC channel──────↔ Subnet X
(Different L1 parents, direct P2P communication, no L1 routing needed)
```

### 2.3 Layer Responsibilities Summary

| Layer | Purpose | Examples | Consensus | Block Time |
|-------|---------|----------|-----------|------------|
| External L1 | Source of finality, asset bridging, ecosystem legitimacy | Filecoin, Ethereum | L1-specific | 12-30s |
| Root IPC (L0) | Global validator coordination, cross-L1 routing, IPCT token | Root IPC Chain | BFT | 6s |
| Sharding (L1) | Validator pool management, parent-agnostic coordination | Shard 1, Shard 2, ... | BFT | 2-6s |
| Execution (L2+) | Fast transaction processing, application logic | User subnets | Configurable | <1s |

---

## 3. Network Hierarchy

### 3.1 External Layer: Layer 1 Blockchains

**Supported L1s:**
- **Filecoin** (primary)
- **Ethereum** (EVM-compatible)
- **Future:** Other EVM-compatible chains (Polygon, Avalanche, etc.)

**L1 Role in IPC:**

**NOT responsible for:** Validator selection, subnet consensus, transaction processing

**Responsible for:**
- Finality anchor for subnets (optional but recommended)
- Gateway contract hosting
- Asset bridging (native L1 tokens ↔ subnets)
- Ecosystem integration and legitimacy
- Dispute resolution (if configured)

**Note on Filecoin:** Filecoin uses F3 (Fast Finality via Aggregated Certificates) as its finality mechanism. When Filecoin-parent subnets checkpoint to the Filecoin Gateway Contract, F3 provides fast finality verification between the L1 and subnets.

**Characteristics:**
- Independent blockchain networks
- Existing security and validator sets
- Slower block times (12-30 seconds)
- Higher transaction costs
- Established ecosystems and user bases

### 3.2 Layer 0: Root IPC Chain

**Purpose:** Global coordination layer connecting multiple L1 ecosystems with a unified validator pool.

**Responsibilities:**

1. **Global Validator Registry:** Single source of truth for all IPC validators
2. **Stake Management:** IPCT staking, slashing, and reward distribution
3. **Sharding Coordination:** Create, manage, and deprecate sharding subnets
4. **Cross-L1 Routing:** Route messages and checkpoints between L1 ecosystems
5. **Bridge Coordination:** Manage bidirectional bridges to L1 gateway contracts
6. **Token Management:** Canonical IPCT token and wrapped token coordination

**Consensus Mechanism:**
- BFT consensus (e.g., CometBFT or similar)
- Block time: 6 seconds
- Finality: 1-2 blocks (~6-12 seconds)

**Validator Set:**
- 50-100 high-reputation validators
- Selected from global pool
- Rotated monthly to prevent centralization
- Highest security tier

**State:**
- Global validator registry with stakes and reputation
- Sharding subnet registry
- L1 gateway contract addresses and bridge states
- Aggregate checkpoint commitments from all shards
- Cross-L1 message routing table
- Canonical IPCT token state

**Bridge Infrastructure:**

Root IPC Chain bridges to each L1:

**Outbound (Root IPC → L1):**
- Post validator set merkle roots to Gateway Contracts
- Forward subnet deployment requests
- Route cross-L1 messages
- Authorize wrapped IPCT minting

**Inbound (L1 → Root IPC):**
- Receive subnet deployment requests from Gateway
- Process L1 finality proofs
- Handle wrapped IPCT burns
- Receive slashing evidence from L1 subnets

### 3.3 Layer 1: Sharding Subnets

**Purpose:** Parent-agnostic validator pool management and coordination layer.

**Key Innovation: Parent Agnostic**
- Sharding subnets don't care which L1 their execution subnets use as parents
- Single shard can manage subnets with Filecoin parents AND Ethereum parents
- Validators in a shard can validate for any execution subnet regardless of L1
- Routing of checkpoints is handled dynamically based on subnet's parent configuration

**Responsibilities:**
1. Manage validator assignments for execution subnets within the shard
2. Process checkpoints from execution subnets (regardless of parent L1)
3. Coordinate data availability within the shard
4. Route checkpoints to appropriate L1 Gateway Contracts
5. Route cross-subnet messages (intra-shard and inter-shard)
6. Report aggregate state to Root IPC Chain
7. Handle execution subnet deployment requests

**Consensus Mechanism:**
- BFT consensus (e.g., CometBFT or similar)
- Block time: 2-6 seconds (target: 4 seconds)
- Finality: 1-2 blocks (~2-12 seconds)

**Validator Set:**
- **Shard validators:** 20-50 validators (10-20% of shard's total pool)
  - Secure the sharding subnet itself
  - Process execution subnet checkpoints
  - Route checkpoints to correct L1 Gateway Contracts
- **Execution validators:** 150-250 validators (80-90% of shard's pool)
  - Assigned to execution subnets
  - Can validate subnets with any L1 parent
- **Total per shard:** 170-300 validators

**Dynamic Checkpoint Routing:**

```python
# Shard processes checkpoint from execution subnet
def process_execution_subnet_checkpoint(checkpoint):
    subnet = get_subnet(checkpoint.subnet_id)
    
    # Route based on subnet's parent type
    if subnet.parent_type == "filecoin":
        route_to_filecoin_gateway(checkpoint)
    elif subnet.parent_type == "ethereum":
        route_to_ethereum_gateway(checkpoint)
    elif subnet.parent_type == "subnet":
        route_to_parent_subnet(checkpoint, subnet.parent_id)
    
    # Always aggregate to Root IPC regardless of parent
    include_in_shard_checkpoint(checkpoint)
```

**Lifecycle:**
- Automatically created by Root IPC Chain when demand increases
- Can be deprecated if demand decreases significantly
- Execution subnets can be migrated between shards during rebalancing
- Shards remain L1-agnostic throughout lifecycle

**State:**
- Local validator registry (subset of global, assigned by Root IPC)
- Execution subnet registry with parent L1 mappings
- Checkpoint commitments from execution subnets
- L1 routing table (which Gateway Contract to use per subnet)
- Intra-shard message queue
- Data availability attestations

### 3.4 Layer 2+: Execution Subnets (Recursive)

**Purpose:** Fast transaction execution for specific applications or use cases.

**Parent Types:** Execution subnets can be children of:
- L1 chains (Filecoin, Ethereum, etc.)
- Other execution subnets (recursive nesting, unlimited depth)

**Responsibilities:**
1. Process user transactions at high speed
2. Maintain subnet-specific state
3. Generate checkpoints for parent (L1 or subnet)
4. Participate in data availability protocol
5. Handle incoming cross-subnet messages
6. Specify parent relationship and checkpoint configuration

**Consensus Mechanism:**
- Configurable (CometBFT, F3, or custom)
- Block time: <1 second (target: 500ms-1s)
- Finality: Depends on consensus choice

**Validator Set:**
- Configurable: 4-21 validators depending on security requirements
- Assigned by parent sharding subnet
- Validators are L1-agnostic (same validator pool regardless of L1)
- Rotated weekly to prevent collusion

**Types:**
- **Long-lived subnets:** Permanent infrastructure, continuously running
- **Ephemeral subnets:** Temporary (minutes to hours), auto-destruct after use

**Configuration Options:**

```python
# Subnet deployment configuration:
{
    "parent_type": "filecoin" | "ethereum" | subnet_id,
    "parent_checkpoint_interval": "configurable per subnet",
    "min_validators": 4-21,
    "security_tier": "economy" | "standard" | "high",
    "geographic_preference": "optional",
    "consensus": "cometbft" | "custom",
    "runtime": "wasm" | "evm" | "custom",
    "data_persistence": "standard" | "archival"
}
```

**Checkpoint Flow Examples:**

**Example 1: Filecoin L1 Parent**
```
Subnet A (child of Filecoin)
  ↓ checkpoint every 100 seconds
Filecoin Gateway Contract
  ↓ stored on Filecoin L1
Filecoin L1 finality (~30 seconds)
```

**Example 2: Recursive Subnets**
```
Subnet A1a (child of Subnet A1)
  ↓ checkpoint every 50 seconds
Subnet A1 (child of Subnet A)
  ↓ checkpoint every 100 seconds
Subnet A (child of Filecoin)
  ↓ checkpoint every 100 seconds
Filecoin Gateway Contract
  ↓ stored on Filecoin L1
Filecoin L1 finality
```

**Example 3: Mixed L1 Siblings**
```
Subnet A (child of Filecoin)  ↔──IBC──↔  Subnet X (child of Ethereum)
     ↓                                         ↓
Filecoin Gateway                          Ethereum Gateway
     ↓                                         ↓
Filecoin L1                                Ethereum L1

Both subnets can communicate directly via IBC
No need to route through L1s
```

---

## 4. L1 Integration & Gateway Contracts

### 4.1 Gateway Contract Architecture

Each supported L1 (Filecoin, Ethereum, etc.) has a Gateway Contract deployed that serves as the interface between IPC and the L1 ecosystem.

**Gateway Contract Responsibilities:**

1. **Validator Verification**
   - Store merkle root of global validator set (from Root IPC)
   - Verify validator signatures using merkle proofs
   - Enable L1 to trust IPC validators without knowing full set

2. **Subnet Deployment Interface**
   - Accept subnet deployment requests from L1 users
   - Forward requests to Root IPC Chain
   - Receive validator assignments back from Root IPC
   - Initialize subnet state on L1

3. **Checkpoint Reception**
   - Receive and verify checkpoints from execution subnets
   - Store checkpoint commitments on L1
   - Provide finality anchor for subnets
   - Enable L1 to query subnet state

4. **Asset Bridging**
   - Lock/unlock native L1 tokens for subnet use
   - Mint/burn wrapped IPCT (wIPCT-FIL, wIPCT-ETH)
   - Handle cross-chain asset transfers
   - Maintain bridge liquidity

5. **Dispute Resolution**
   - Accept fraud proofs or challenges
   - Trigger slashing on Root IPC if necessary
   - Provide mechanism for L1 community to intervene

### 4.2 Gateway Contract Design

#### 4.2.1 Filecoin Gateway Contract

```solidity
contract FilecoinIPCGateway {
    // Validator set tracking
    bytes32 public validatorSetMerkleRoot;
    uint256 public validatorSetEpoch;
    
    // Subnet registry
    mapping(bytes32 => SubnetInfo) public subnets;
    
    // Checkpoint storage
    mapping(bytes32 => mapping(uint256 => Checkpoint)) public checkpoints;
    
    // Bridge state
    mapping(address => uint256) public lockedFIL;
    mapping(address => uint256) public wIPCT_FIL_balance;
    
    struct SubnetInfo {
        bytes32 subnetId;
        address[] validators;
        uint256 checkpointInterval;
        uint256 lastCheckpointBlock;
        bool active;
    }
    
    struct Checkpoint {
        bytes32 stateRoot;
        bytes32 transactionRoot;
        uint256 blockNumber;
        bytes validatorSignatures;
        uint256 timestamp;
    }
    
    // Called by Root IPC Chain bridge
    function updateValidatorSet(
        bytes32 newMerkleRoot,
        uint256 epoch,
        bytes proof
    ) external onlyRootIPCBridge {
        validatorSetMerkleRoot = newMerkleRoot;
        validatorSetEpoch = epoch;
        emit ValidatorSetUpdated(epoch, newMerkleRoot);
    }
    
    // Called by L1 users
    function deploySubnet(
        SubnetParams params,
        uint256 prepaidFees
    ) external payable returns (bytes32 subnetId) {
        require(msg.value >= prepaidFees, "Insufficient fees");
        
        // Forward request to Root IPC Chain
        sendToRootIPC(
            abi.encodeCall(this.deploySubnet, (params, prepaidFees))
        );
        
        subnetId = keccak256(abi.encode(params, block.number));
        emit SubnetDeploymentRequested(subnetId, params);
    }
    
    // Called by subnet validators
    function submitCheckpoint(
        bytes32 subnetId,
        Checkpoint checkpoint,
        bytes[] validatorSignatures,
        bytes[] merkleProofs
    ) external {
        // Verify validators are in current set
        require(
            verifyValidatorSignatures(
                subnetId,
                checkpoint,
                validatorSignatures,
                merkleProofs
            ),
            "Invalid validator signatures"
        );
        
        // Store checkpoint
        checkpoints[subnetId][checkpoint.blockNumber] = checkpoint;
        subnets[subnetId].lastCheckpointBlock = checkpoint.blockNumber;
        
        emit CheckpointSubmitted(subnetId, checkpoint.blockNumber);
    }
    
    // Verify validator signatures using merkle proof
    function verifyValidatorSignatures(
        bytes32 subnetId,
        Checkpoint checkpoint,
        bytes[] signatures,
        bytes[] merkleProofs
    ) internal view returns (bool) {
        bytes32 checkpointHash = keccak256(abi.encode(checkpoint));
        
        // Need 2/3+ signatures
        uint256 requiredSignatures = (subnets[subnetId].validators.length * 2) / 3 + 1;
        uint256 validSignatures = 0;
        
        for (uint i = 0; i < signatures.length; i++) {
            address validator = ecrecover(checkpointHash, signatures[i]);
            
            // Verify validator is in merkle tree using proof
            if (verifyMerkleProof(
                validatorSetMerkleRoot,
                validator,
                merkleProofs[i]
            )) {
                validSignatures++;
            }
        }
        
        return validSignatures >= requiredSignatures;
    }
    
    // Bridge FIL to subnet
    function bridgeToSubnet(
        bytes32 subnetId,
        uint256 amount
    ) external payable {
        require(msg.value == amount, "Incorrect FIL amount");
        lockedFIL[msg.sender] += amount;
        
        // Emit event for subnet to mint wrapped FIL
        emit FILBridgedToSubnet(msg.sender, subnetId, amount);
    }
    
    // Bridge wIPCT-FIL from L1 to Root IPC
    function bridgeIPCTToRootChain(uint256 amount) external {
        require(wIPCT_FIL_balance[msg.sender] >= amount, "Insufficient wIPCT");
        
        // Burn wIPCT-FIL
        wIPCT_FIL_balance[msg.sender] -= amount;
        
        // Send message to Root IPC to unlock canonical IPCT
        sendToRootIPC(
            abi.encodeCall(
                IRootIPCBridge.unlockIPCT,
                (msg.sender, amount)
            )
        );
        
        emit IPCTBridgedToRoot(msg.sender, amount);
    }
    
    // Called by Root IPC bridge when IPCT is locked there
    function mintWrappedIPCT(
        address recipient,
        uint256 amount,
        bytes proof
    ) external onlyRootIPCBridge {
        // Verify proof from Root IPC
        require(verifyRootIPCProof(proof), "Invalid proof");
        
        wIPCT_FIL_balance[recipient] += amount;
        emit WrappedIPCTMinted(recipient, amount);
    }
}
```

#### 4.2.2 Ethereum Gateway Contract

```solidity
// Similar structure to Filecoin Gateway
// Differences:
// - Uses ETH instead of FIL for gas and bridging
// - wIPCT-ETH instead of wIPCT-FIL
// - EVM-native optimizations
// - Compatible with existing Ethereum bridge standards

contract EthereumIPCGateway {
    // Similar structure to FilecoinIPCGateway
    // Adapted for Ethereum's 12-second blocks and gas model
    // ...
}
```

### 4.3 Bridge Protocol: Root IPC ↔ L1 Gateway

**Bidirectional Bridge Components:**

**Root IPC → L1 (Outbound):**

Root IPC Chain monitors:
- New validator set updates (every epoch)
- Subnet deployment responses
- Cross-L1 message routing
- IPCT unlock authorizations

Bridge relayers:
- Submit transactions to L1 Gateway Contracts
- Include cryptographic proofs from Root IPC
- Pay L1 gas fees (reimbursed by Root IPC)

**L1 → Root IPC (Inbound):**

Gateway Contracts emit events:
- Subnet deployment requests
- wIPCT bridge operations
- Checkpoint submissions (for Root IPC awareness)
- Slashing evidence

Bridge relayers:
- Monitor L1 events
- Submit to Root IPC with L1 finality proofs
- Root IPC verifies L1 state before processing

**Security Model:**

Bridge trust assumptions:
1. Root IPC Chain finality (BFT with 50+ validators)
2. L1 finality (Filecoin: 900 epochs, Ethereum: ~15 minutes)
3. Bridge relayer liveness (decentralized relayer set)
4. Cryptographic proofs (merkle proofs, signatures)

No single point of failure:
- Multiple independent relayers
- Root IPC can slash dishonest relayers
- L1 Gateway Contracts verify all proofs
- Users can submit proofs directly if relayers fail

### 4.4 Validator Set Synchronization

**Challenge:** How do L1 Gateway Contracts trust IPC validators without storing the entire validator set?

**Solution: Merkle Root + Proofs**

Every epoch (e.g., daily):

1. Root IPC Chain computes merkle tree of validator set:
```python
validators = [
  {address: 0x123..., stake: 10000, reputation: 1.2},
  {address: 0x456..., stake: 15000, reputation: 1.5},
  ...
]
merkleRoot = computeMerkleRoot(validators)
```

2. Root IPC posts merkleRoot to each L1 Gateway Contract:
```solidity
FilecoinGateway.updateValidatorSet(merkleRoot, epoch, proof)
EthereumGateway.updateValidatorSet(merkleRoot, epoch, proof)
```

3. When subnet checkpoint arrives at Gateway:
```python
checkpoint = {..., signatures: [sig1, sig2, ...]}

Gateway verifies each signature:
- Recover validator address from signature
- Check merkle proof: isValidatorInSet(address, merkleRoot, merkleProof)
- Count valid signatures
- Require 2/3+ threshold
```

4. Gateway accepts checkpoint if threshold met

**Benefits:**
- ✅ Gateway only stores single 32-byte merkle root
- ✅ Can verify any validator using proof
- ✅ No need to sync entire validator set to L1
- ✅ Updates once per epoch (low overhead)

**Gas Efficiency:**
- Merkle root update: ~50,000 gas on Ethereum
- Merkle proof verification: ~20,000 gas per validator
- Acceptable for checkpoint submissions

---

## 5. Validator System

### 5.1 Single Global Validator Pool

**Key Design:** One validator pool serves all L1 ecosystems and all sharding subnets.

**Benefits:**
- ✅ Validators stake once, can validate anywhere
- ✅ Maximum capital efficiency
- ✅ Simplified validator experience
- ✅ Better decentralization (larger pool)
- ✅ Lower barrier to entry

**Validator Perspective:**

Validator joins IPC:
1. Stake 10,000 IPCT on Root IPC Chain (once)
2. Get assigned to a sharding subnet by Root IPC
3. Can validate subnets with ANY parent L1:
   - Filecoin parent subnet
   - Ethereum parent subnet  
   - Another subnet parent
4. Earn rewards in IPCT regardless of parent L1
5. No need for FIL or ETH stake

### 5.2 Validator Roles

#### 5.2.1 Root Chain Validators
- Secure the Root IPC Chain (L0)
- Manage global validator registry
- Coordinate sharding subnet creation/deprecation
- Process aggregate checkpoints from sharding subnets
- Manage bridges to L1 Gateway Contracts
- Highest reputation requirement
- ~50-100 validators
- Rotated monthly

#### 5.2.2 Shard Validators
- Secure Layer 1 sharding subnets
- Process execution subnet checkpoints (regardless of parent L1)
- Route checkpoints to appropriate L1 Gateway Contracts
- Coordinate intra-shard operations
- Route cross-shard and cross-L1 messages
- 10-20% of shard's validator pool (~20-50 validators)
- High reputation preferred
- Rotated monthly

#### 5.2.3 Execution Validators
- Validate transactions on Layer 2+ execution subnets
- Parent-agnostic: can validate for Filecoin, Ethereum, or subnet parents
- Create execution subnet checkpoints
- Participate in data availability within shard
- 80-90% of shard's validator pool (~150-250 validators)
- Assigned to 1-2 execution subnets simultaneously
- Rotated weekly between execution subnets

#### 5.2.4 Storage-Only Validators (Future)
- Do not participate in consensus
- Hold erasure coded chunks for data availability
- Lighter hardware requirements
- Lower rewards, lower barriers to entry

### 5.3 Validator Requirements

#### 5.3.1 Hardware Tiers

**Tier 1: Full Validators**
- Can fulfill any validator role (root, shard, or execution)
- Can validate subnets with any L1 parent
- CPU: 16+ cores
- RAM: 32+ GB
- Storage: 2+ TB NVMe SSD
- Network: 1+ Gbps, <50ms latency to peers
- Supported runtimes: WASM, EVM, and future runtimes

**Tier 2: Specialized Validators**
- Limited to specific runtime environments or L1s
- CPU: 8+ cores
- RAM: 16+ GB
- Storage: 1+ TB SSD
- Network: 500+ Mbps
- Supported runtimes: Subset (e.g., WASM-only or EVM-only)

**Tier 3: Storage Validators (Future)**
- Data availability only
- CPU: 4+ cores
- RAM: 8+ GB
- Storage: 4+ TB HDD acceptable
- Network: 100+ Mbps

#### 5.3.2 Staking Requirements
- **Minimum stake:** 10,000 IPCT (subject to governance adjustment)
- **Staking location:** Root IPC Chain (single global stake)
- **Unbonding period:** 21 days
- **Slashing collateral:** Maintained throughout service
- **No L1 stake required:** Validators don't need FIL or ETH

### 5.4 Validator Lifecycle

#### 5.4.1 Onboarding

1. Validator acquires 10,000+ IPCT
2. Stakes IPCT on Root IPC Chain
3. Registers metadata:
   - Hardware specifications
   - Geographic location
   - Supported runtime environments (WASM, EVM, etc.)
   - Supported L1s (Filecoin, Ethereum, or agnostic)
   - Network connectivity metrics
4. Root IPC Chain assigns to optimal sharding subnet based on:
   - Current shard load
   - Geographic distribution
   - Specialization requirements
5. Sharding subnet assigns role:
   - Shard validator (if high reputation and need exists)
   - Execution validator (default for new validators)
6. Validator begins duties and earning rewards

#### 5.4.2 Rotation Schedule

**Weekly (Execution Validators):**
- Rotate between different execution subnets within same shard
- Can be reassigned to subnet with different L1 parent
- Prevents long-term collusion with specific subnet operators
- ~1/3 of execution subnet's validator committee rotates each week

**Monthly (Shard Validators):**
- Some shard validators rotate back to execution validator role
- Some high-performing execution validators promoted to shard validator role
- Ensures fresh perspectives and prevents stagnation

**Monthly (Root Chain Validators):**
- Partial rotation of root chain validator set
- Highest reputation validators eligible
- ~1/4 of root validators rotate each month

#### 5.4.3 Cross-L1 Assignment Flexibility

Example validator journey:

```
Week 1: Assigned to Subnet A (parent: Filecoin)
Week 2: Assigned to Subnet B (parent: Filecoin)  
Week 3: Assigned to Subnet X (parent: Ethereum)
Week 4: Assigned to Subnet Y (parent: Subnet A)

Same validator, different L1 contexts, same stake
```

### 5.5 Reputation System

#### 5.5.1 Reputation Score Calculation

```
Reputation Score (0.0 to 2.0) = 
  (Uptime Factor × 0.30) +
  (Tenure Factor × 0.20) +
  (Slash History Factor × 0.25) +
  (Network Contribution Factor × 0.25)
```

Where:
- **Uptime Factor:** Based on blocks proposed vs expected, attestations provided (regardless of parent L1)
- **Tenure Factor:** Exponential bonus for long-term participation (caps at 1 year)
- **Slash History Factor:** Penalized for each slash, recovers over time
- **Network Contribution Factor:** 
  - Governance participation
  - Running archival nodes
  - Geographic diversity contribution
  - Multi-L1 support (bonus for validating subnets with different parents)

#### 5.5.2 Reputation Tiers

**Tier 0 (0.0-0.5): New or recently slashed validators**
- Eligible for execution validator role only
- Cannot validate high-security subnets
- Reward multiplier: 0.8x

**Tier 1 (0.5-1.0): Established validators**
- Eligible for execution validator role
- Can validate subnets with any L1 parent
- Standard reward multiplier: 1.0x

**Tier 2 (1.0-1.5): High-reputation validators**
- Eligible for shard validator role
- Priority for high-value subnet assignments
- Proven track record across multiple L1 parents
- Reward multiplier: 1.2x

**Tier 3 (1.5-2.0): Elite validators**
- Eligible for root chain validator role
- Lowest slashing penalties for minor infractions
- Multi-L1 experience preferred
- Reward multiplier: 1.5x

### 5.6 Validator Selection Algorithm

#### 5.6.1 Sharding Subnet Assignment (Root Chain)

```python
def assign_validator_to_shard(validator):
    # Filter eligible shards
    eligible_shards = []
    for shard in all_shards:
        if shard.validator_count < MAX_VALIDATORS_PER_SHARD:
            # Check if shard's L1 mix matches validator preferences
            if validator.supported_l1s == "all" or 
               shard.has_subnets_matching(validator.supported_l1s):
                eligible_shards.append(shard)
    
    if not eligible_shards:
        # All shards full, trigger new shard creation
        create_new_shard()
        return assign_validator_to_shard(validator)
    
    # Prefer shard with lowest validator count (load balancing)
    target_shard = min(eligible_shards, key=lambda s: s.validator_count)
    
    # If geographic preference, prioritize matching region
    if validator.geo_preference:
        geo_matches = [s for s in eligible_shards 
                       if s.region == validator.geo_preference]
        if geo_matches:
            target_shard = min(geo_matches, key=lambda s: s.validator_count)
    
    return target_shard
```

#### 5.6.2 Execution Subnet Assignment (Sharding Subnet)

```python
def assign_validators_to_execution_subnet(subnet_requirements):
    required_count = subnet_requirements.min_validators
    parent_l1 = subnet_requirements.parent_l1  # "filecoin", "ethereum", or subnet_id
    
    # Filter eligible validators in shard
    eligible_validators = []
    for validator in shard.execution_validators:
        # Check capacity
        if validator.current_assignments < MAX_ASSIGNMENTS:
            # Check runtime support
            if validator.supports_runtime(subnet_requirements.runtime):
                # Check L1 support (most validators are L1-agnostic)
                if validator.supported_l1s == "all" or 
                   parent_l1 in validator.supported_l1s:
                    # Check other requirements
                    if meets_requirements(validator, subnet_requirements):
                        eligible_validators.append(validator)
    
    if len(eligible_validators) < required_count:
        return None  # Insufficient validators, queue request
    
    # VRF-based random selection
    seed = hash(current_epoch + subnet_id)
    selected = vrf_select(eligible_validators, required_count, seed)
    
    # Bias toward higher reputation validators for high-security subnets
    if subnet_requirements.security_tier == "high":
        selected = weighted_select(
            eligible_validators, 
            required_count,
            weights=reputation_scores
        )
    
    return selected
```

**Key Point:** Parent L1 is just another filtering criterion. Validators don't need special configuration per L1.

---

## 6. Subnet Deployment & Management

### 6.1 Execution Subnet Types

#### 6.1.1 Long-Lived Subnets

**Characteristics:**
- Continuous operation (months to years)
- Persistent state
- Stable validator assignments with periodic rotation
- Full data availability and archival
- Can be child of L1 or another subnet

**Use Cases:**
- DeFi protocols on Filecoin or Ethereum
- Decentralized applications
- Enterprise blockchain infrastructure
- Gaming platforms
- Storage coordination subnets (e.g., on Filecoin)
- EVM-compatible smart contract platforms (e.g., on Ethereum)

**Pricing Model:**
```
Monthly cost = 
  (base_rate_per_validator × validator_count × 720 hours) +
  (storage_fee × state_size_gb) +
  transaction_fees

Independent of parent L1 choice
```

#### 6.1.2 Ephemeral Subnets

**Characteristics:**
- Short-lived (minutes to hours)
- Minimal genesis state
- Auto-destruct after duration
- Optional data archival to Filecoin
- Can be child of any L1 or subnet

**Use Cases:**
- AI/ML inference jobs
- Temporary game servers
- Financial settlement batches
- Testing and development environments
- Temporary trusted computation zones
- Event-specific applications

**Pricing Model:**
```
Total cost = 
  (base_rate_per_validator × validator_count × duration_minutes × ephemeral_multiplier) +
  (optional_archival_fee)

Where ephemeral_multiplier = 1.5-2.0x (instant availability premium)

Paid upfront in IPCT (or wIPCT on L1)
```

### 6.2 Subnet Deployment Flows

#### 6.2.1 Direct L1 Deployment (Simpler, Recommended)

For subnets that want to be direct children of an L1:

**User perspective (on Filecoin):**
1. User calls `FilecoinGateway.deploySubnet(params, prepaidFees)`
2. Receives subnet_id and RPC endpoints
3. Subnet is ready in ~30-60 seconds

**Under the hood:**
1. FilecoinGateway receives deployment request
2. Gateway sends message to Root IPC Chain: "Need validators for Filecoin subnet with params X"
3. Root IPC selects appropriate sharding subnet
4. Shard assigns validators from pool (L1-agnostic validators)
5. Validators initialize subnet
6. Subnet begins checkpointing to FilecoinGateway
7. Gateway returns subnet info to user

**Flow Diagram:**
```
User → FilecoinGateway → Root IPC → Shard → Validators
                                              ↓
                                           Subnet ← User
                                              ↓
                                    (checkpoints)
                                              ↓
                                      FilecoinGateway
                                              ↓
                                          Filecoin L1
```

#### 6.2.2 Root IPC Deployment (Advanced)

For users who want more control or are deploying on behalf of programmatic system:

**User calls Root IPC directly:**
1. User connects to Root IPC Chain
2. Calls `deploySubnet(params, parent_l1="filecoin", ...)`
3. Root IPC assigns to shard and validators
4. Returns subnet info
5. Subnet checkpoints to Filecoin via Gateway

**Advantage:** More options, direct interaction with IPC  
**Use case:** Advanced users, programmatic deployment

#### 6.2.3 Recursive Subnet Deployment

For subnets that want another subnet as parent:

**User wants Subnet B as child of Subnet A:**

1. User calls Subnet A's deployment interface: `SubnetA.deployChildSubnet(params, prepaidFees)`
2. Subnet A forwards request to its parent shard: "Need validators for child subnet"
3. Shard assigns validators (same pool, L1-agnostic)
4. Validators initialize Subnet B
5. Subnet B checkpoints to Subnet A (not to L1 directly)
6. Subnet A periodically checkpoints to its parent (L1 or another subnet)

**Chain of checkpoints:**
```
Subnet B → Subnet A → Filecoin Gateway → Filecoin L1
```

**Example: Three-Level Recursion**
```
Filecoin L1
    → (checkpoints via Gateway)
Subnet A (storage coordination layer)
    → (checkpoints every 100 seconds)
Subnet A1 (deal matching subnet)
    → (checkpoints every 50 seconds)
Subnet A1a (high-frequency trading subnet)
```

### 6.3 Parent Selection & Configuration

#### 6.3.1 Subnet Configuration Parameters

```solidity
struct SubnetDeploymentParams {
    // Parent configuration
    ParentType parent_type;  // L1_FILECOIN, L1_ETHEREUM, SUBNET
    bytes32 parent_id;       // Gateway address (for L1) or subnet_id
    
    // Checkpoint configuration
    uint256 checkpoint_interval;  // seconds (configurable per subnet)
    bool wait_for_parent_finality;  // true = conservative, false = optimistic
    uint256 parent_finality_depth;  // blocks to wait on parent
    
    // Security configuration
    uint8 min_validators;    // 4-21
    SecurityTier security_tier;  // ECONOMY, STANDARD, HIGH
    
    // Technical configuration
    RuntimeEnvironment runtime;  // WASM, EVM, CUSTOM
    ConsensusType consensus;     // COMETBFT, CUSTOM
    
    // Operational configuration
    GeographicPreference geo_preference;  // optional
    uint256 lifetime_seconds;  // 0 = permanent, >0 = ephemeral
    bool archival_required;
    
    // Payment
    uint256 prepaid_fees;  // in IPCT or wIPCT
}
```

#### 6.3.2 Parent Type Examples

**Filecoin L1 Parent:**
```javascript
params = {
    parent_type: ParentType.L1_FILECOIN,
    parent_id: FILECOIN_GATEWAY_ADDRESS,
    checkpoint_interval: 100,  // seconds
    wait_for_parent_finality: true,
    parent_finality_depth: 900,  // Filecoin epochs
    ...
}
```

**Ethereum L1 Parent:**
```javascript
params = {
    parent_type: ParentType.L1_ETHEREUM,
    parent_id: ETHEREUM_GATEWAY_ADDRESS,
    checkpoint_interval: 100,  // seconds
    wait_for_parent_finality: true,
    parent_finality_depth: 64,  // Ethereum blocks (~15 min)
    ...
}
```

**Subnet Parent (Recursive):**
```javascript
params = {
    parent_type: ParentType.SUBNET,
    parent_id: SUBNET_A_ID,
    checkpoint_interval: 50,  // faster than parent
    wait_for_parent_finality: false,  // optimistic
    ...
}
```

### 6.4 Subnet Lifecycle Management

#### 6.4.1 Scaling Validator Count

Subnet operator can request validator count change:

```
updateSubnetValidators(subnet_id, new_validator_count)
```

**Process:**
1. Request sent to parent sharding subnet
2. Shard validates request and fees
3. If increasing: Assign additional validators (L1-agnostic pool)
4. If decreasing: Remove validators at next rotation boundary
5. Gradual transition to avoid disruption

Validator assignment remains parent-agnostic

#### 6.4.2 Changing Parent (Advanced)

**Scenario:** Subnet wants to migrate from Filecoin to Ethereum parent

**Process:**
1. Submit migration request to Root IPC
2. Root IPC validates feasibility
3. Create final checkpoint on current parent (Filecoin)
4. Initialize new parent relationship (Ethereum Gateway)
5. Sync state to new parent
6. Begin checkpointing to Ethereum instead
7. Validators remain the same (L1-agnostic)

**Downtime:** ~5-10 minutes  
**Use case:** L1 performance issues, cost optimization, ecosystem alignment

#### 6.4.3 Subnet Termination

**Long-lived subnets:**
1. Operator calls `terminateSubnet(subnet_id)`
2. Final checkpoint created to parent (L1 or subnet)
3. If archival requested: Full state pushed to Filecoin
4. Validators released back to pool
5. Remaining fees refunded (if any)

**Ephemeral subnets:**
1. Auto-terminate after duration expires
2. Final checkpoint created to parent
3. Optional archival to Filecoin
4. Validators immediately available for new assignments (any L1)

---

## 7. Data Availability Layer

### 7.1 Erasure Coding Architecture

#### 7.1.1 Checkpoint Data Structure

Every execution subnet creates checkpoints periodically (configurable, default: every 100 blocks or ~100 seconds):

```
Checkpoint = {
  subnet_id: string,
  parent_type: "filecoin" | "ethereum" | subnet_id,
  parent_id: address or subnet_id,
  block_range: {start: number, end: number},
  state_root: hash,
  transaction_merkle_root: hash,
  validator_signatures: MultiSig,
  timestamp: number,
  parent_checkpoint_hash: hash
}
```

#### 7.1.2 Erasure Coding Process

1. Execution subnet validators create checkpoint
2. Serialize checkpoint + block data
3. Erasure code into N chunks (N = number of validators in shard)
4. Redundancy factor: 2x (need 50% of chunks to reconstruct)
5. Each chunk distributed to different validator in parent shard
6. Validators sign attestation: "I have chunk X of checkpoint Y"
7. Attestations collected and posted to parent sharding subnet
8. Checkpoint considered "available" when threshold reached (66%)
9. Sharding subnet routes checkpoint to parent (L1 Gateway or subnet)

**Parent-Agnostic DA:**
- Erasure coding happens within shard (independent of parent L1)
- Chunks distributed to shard validators (same process for all parents)
- After DA confirmation, checkpoint routed to appropriate parent:
  - Filecoin parent → FilecoinGateway
  - Ethereum parent → EthereumGateway
  - Subnet parent → Parent subnet validators

### 7.2 Data Availability Levels

#### 7.2.1 Intra-Shard DA (Standard)

**Configuration:**
- Chunks distributed only within parent shard
- Suitable for most execution subnets
- Lower overhead, faster confirmation
- Parent-agnostic: Same process regardless of L1

**Guarantees:**
- Data available if 50%+ of shard's validators are honest
- Reconstruction possible with any 50% of chunks
- Works identically for Filecoin, Ethereum, or subnet parents

#### 7.2.2 Cross-Shard DA (High Security)

**Configuration:**
- Chunks distributed across multiple shards
- Coordinated by Root IPC Chain
- Higher overhead, maximum security
- Still parent-agnostic

**Guarantees:**
- Data available even if entire shard becomes unavailable
- Reconstruction possible from validators in other shards
- Parent L1 doesn't affect DA security

**Pricing:**
- 1.5-2x cost due to increased network overhead
- Optional for high-value subnets
- Available for any parent type

### 7.3 Data Availability Sampling

#### 7.3.1 Random Sampling Protocol

Root IPC Chain periodically (every epoch):

1. Randomly selects N checkpoints across all shards and L1s
2. For each checkpoint, randomly selects M chunks
3. Challenges validators to provide selected chunks
4. Validators must respond within timeout period (30 seconds)
5. Failed challenges result in slashing
6. Successful challenges reward the validator

**Parent-agnostic:** Challenge works same way for all checkpoint types

**Parameters:**
- N = 10 checkpoints per epoch
- M = 5 chunks per checkpoint
- Challenge reward: 1 IPCT
- Failure to respond: -10 IPCT (slashed)

### 7.4 Archival to Filecoin

#### 7.4.1 Hot vs Cold Storage

**Hot Storage (Recent Data):**
- Last N checkpoints (e.g., 1000 checkpoints or ~27 hours)
- Maintained by validators via erasure coding
- Fast retrieval for recent queries
- Required for data availability protocol
- Independent of subnet's parent L1

**Cold Storage (Historical Data):**
- Data older than finality window
- Always pushed to Filecoin (regardless of parent L1)
- Filecoin's specialty: long-term decentralized storage
- Slower retrieval but permanent availability
- Reduces validator storage burden

**Why Filecoin for all archival:**
- Filecoin is purpose-built for decentralized storage
- Even Ethereum subnets benefit from Filecoin archival
- Single archival layer simplifies architecture
- Creates natural integration with Filecoin ecosystem

#### 7.4.2 Filecoin Integration Process

Every epoch, for checkpoints beyond finality window:

1. Sharding subnet identifies old checkpoints for archival (regardless of parent L1 - Filecoin, Ethereum, or subnet)
2. Execution subnet validators reconstruct full data from chunks
3. Compress and batch multiple checkpoints
4. Create Filecoin storage deal via smart contract
5. Upload data to Filecoin storage providers
6. Record Filecoin deal ID on Root IPC Chain
7. Validators prune local erasure coded chunks
8. State root and deal ID remain on-chain for verification

**Cross-L1 Benefit:**
- Ethereum subnets get cheap, permanent storage via Filecoin
- Filecoin subnets naturally integrate with native storage
- Unified archival layer across all L1 ecosystems
- Single retrieval mechanism for historical data

#### 7.4.3 Historical Data Retrieval

To verify historical state (any parent L1):

1. User queries Root IPC for checkpoint hash and Filecoin deal ID
2. Retrieve data from Filecoin network
3. Verify data hash matches on-chain commitment
4. Can reconstruct full historical state from checkpoint + blocks

Works identically for Filecoin-parent and Ethereum-parent subnets

---

## 8. Cross-Subnet Communication

### 8.1 Messaging Architecture

#### 8.1.1 Design Goals

- **Fast:** <30 seconds for intra-shard, <60 seconds for inter-shard
- **Direct:** Peer-to-peer communication without L1 routing when possible
- **L1-Agnostic:** Subnets with different L1 parents can communicate directly
- **Secure:** Cryptographic proofs prevent forgery
- **Transparent:** Users unaware of shard boundaries or L1 differences

#### 8.1.2 Message Types

**Type A: Intra-Shard, Same L1 Parent**
- Both subnets under same sharding subnet
- Both have same L1 parent (e.g., both Filecoin)
- Fastest path

**Type B: Intra-Shard, Different L1 Parents (Key Innovation)**
- Both subnets under same sharding subnet
- Different L1 parents (e.g., Filecoin and Ethereum)
- Direct P2P communication via IBC

**Type C: Inter-Shard, Same L1 Parent**
- Subnets under different sharding subnets
- Same L1 parent
- Via Root IPC coordination

**Type D: Inter-Shard, Different L1 Parents**
- Subnets under different sharding subnets
- Different L1 parents
- Via Root IPC coordination

### 8.2 Direct IBC Protocol (Type A & B)

**Critical Innovation:** Subnets can communicate directly even with different L1 parents, as long as they use compatible consensus (e.g., both CometBFT).

#### 8.2.1 IBC Connection Setup

Subnet A (parent: Filecoin) wants to communicate with Subnet X (parent: Ethereum):

1. Both subnets use CometBFT consensus (or compatible BFT)
2. Establish IBC channel:
   - Subnet A runs light client of Subnet X
   - Subnet X runs light client of Subnet A
   - Light clients track validator sets and state roots
3. Validator sets are verified:
   - Both subnets' validators come from same global IPC pool
   - Validator signatures verifiable via Root IPC merkle roots
   - No need to trust L1 validators
4. Channel established and ready for messages

#### 8.2.2 Direct P2P Message Flow

Subnet A (Filecoin parent) → Subnet X (Ethereum parent):

1. **Subnet A creates message:**
```python
message = {
  from_subnet: subnet_a_id,
  to_subnet: subnet_x_id,
  nonce: number,
  payload: bytes,
  timestamp: number
}
```

2. **Subnet A validators sign message:**
```python
signature = MultiSig(hash(message), validator_keys)
# Requires 2/3+ of Subnet A's validators to sign
```

3. **Direct P2P broadcast to Subnet X validators:**
   - Subnet A validators send signed message directly to Subnet X
   - No routing through Filecoin L1 or Ethereum L1
   - No routing through parent shard needed

4. **Subnet X validators verify:**
   - Check signatures against Subnet A's validator set
   - Validator set known via light client
   - Verify 2/3+ threshold met
   - Verify nonce (prevent replay)

5. **Subnet X includes message in next block:**
   - Message is now part of Subnet X's state
   - Execution can proceed
   - Total time: ~2-10 seconds

6. **Async finality (optional):**
   - Both subnets checkpoint to their respective parents
   - Subnet A → Filecoin Gateway
   - Subnet X → Ethereum Gateway
   - Root IPC has record of both checkpoints
   - Provides finality proof for dispute resolution

**Key Points:**
- ✅ No L1 involvement in message passing
- ✅ Different L1 parents don't matter
- ✅ Direct P2P = very fast (~2-10 seconds)
- ✅ Secure via BFT validator signatures
- ✅ Finality from Root IPC (both subnets checkpoint there eventually)

**Latency Breakdown:**

Direct IBC message (cross-L1):
- Message creation & signing: ~1-2 seconds
- P2P transmission: ~1-3 seconds  
- Signature verification: <1 second
- Block inclusion: <1 second (next block)
- **Total: ~3-7 seconds (optimistic)**

Finality proof (later):
- Subnet A checkpoints to Filecoin: +100 seconds
- Subnet X checkpoints to Ethereum: +100 seconds
- Both checkpoint to Root IPC: +10 minutes
- **Full finality: ~12 minutes after message**

### 8.3 Inter-Shard Communication (Type C & D)

For subnets in different shards, Root IPC Chain coordinates:

#### 8.3.1 Fast Path with Root Chain Relay

Subnet A (Shard 1, Filecoin) → Subnet Y (Shard 4, Ethereum):

1. **Subnet A creates and signs message** (same as IBC)

2. **Subnet A submits commitment to Shard 1:**
```python
commitment = {
  message_hash: hash(message),
  from_subnet: subnet_a_id,
  to_subnet: subnet_y_id,
  signature_proof: MultiSig
}
```

3. **Shard 1 includes commitment and forwards to Root IPC:**
   - Not waiting for 10-minute checkpoint
   - Special "message relay" transaction
   - Root IPC includes in next block (~6 seconds)

4. **Shard 4 monitors Root IPC for messages to its subnets:**
   - Sees commitment for Subnet Y
   - Requests full message from Shard 1 (direct P2P between shards)

5. **Shard 1 provides full message + proof to Shard 4**

6. **Shard 4 verifies and forwards to Subnet Y:**
   - Subnet Y validators verify signatures
   - Include in next block

**Total latency:** ~20-40 seconds  
**L1 Agnostic:** Process is identical whether subnets have same or different L1 parents.

#### 8.3.2 Standard Path (Checkpoint-Based)

For non-urgent messages:

1. Subnet A creates message
2. Included in Shard 1 checkpoint (up to 10 minutes)
3. Shard 1 checkpoints to Root IPC
4. Root IPC includes checkpoint (~6 seconds)
5. Shard 4 reads from Root IPC checkpoint
6. Shard 4 forwards to Subnet Y

**Total latency:** ~10-15 minutes  
**Lower overhead** on Root IPC

### 8.4 Message Ordering & Replay Protection

#### 8.4.1 Nonce-Based Ordering

Each subnet maintains message nonce per destination:

```python
outgoing_nonces = {
  subnet_b: 42,
  subnet_x: 17,  # Different L1 parent, same nonce tracking
  ...
}
```

- Each message includes nonce
- Receiving subnet verifies nonce is next expected value
- Prevents replay attacks and ensures ordering
- L1 parent difference doesn't affect nonce mechanism

### 8.5 Cross-Subnet Message Fees

```
Message fee = 
  (base_fee × message_size_kb) +
  (routing_fee) +
  (priority_multiplier)
```

**Where:**
- **base_fee:** 0.1 IPCT per KB
- **routing_fee:** 
  - Intra-shard (same or different L1): 0.01 IPCT
  - Inter-shard fast path: 0.05 IPCT
  - Inter-shard standard: 0.02 IPCT
- **priority_multiplier:** 
  - Standard: 1.0x
  - High priority: 2.0x
  - Urgent: 5.0x

**Fees paid in:** IPCT or wIPCT (converted)  
**L1 parent doesn't affect fees**

**Fees split:**
- 40% to source subnet validators
- 40% to destination subnet validators  
- 20% to routing validators (shard validators and/or root validators)

**Example:**

Message from Subnet A (Filecoin) to Subnet X (Ethereum), intra-shard:
- Size: 10 KB
- Base fee: 0.1 × 10 = 1.0 IPCT
- Routing: 0.01 IPCT (intra-shard)
- Priority: Standard (1.0x)
- **Total: 1.01 IPCT**

No additional cost for different L1 parents

---

## 9. Token Economics & Incentives

### 9.1 IPCT Token

**Token Name:** IPC Token (IPCT)  
**Deployment:** Canonical token on Root IPC Chain  

**Purpose:**
- Validator staking collateral (single global stake)
- Subnet deployment fees (across all L1s)
- Transaction fees
- Cross-subnet message fees
- Governance participation

**Multi-Chain Presence:**
- **Root IPC Chain:** Canonical IPCT (native)
- **Filecoin L1:** wIPCT-FIL (wrapped, bridged)
- **Ethereum L1:** wIPCT-ETH (wrapped, bridged)
- **Execution Subnets:** Can use IPCT, wIPCT, or native subnet tokens

### 9.2 Bridge Architecture

#### 9.2.1 Canonical IPCT on Root IPC

Root IPC Chain hosts canonical IPCT:
- Total supply tracked on Root IPC
- Validators stake canonical IPCT
- Emissions distributed in canonical IPCT

#### 9.2.2 Wrapped IPCT on L1s

**Bridge to Filecoin:**

User wants wIPCT-FIL:
1. Lock IPCT on Root IPC Chain
2. Root IPC sends message to FilecoinGateway
3. FilecoinGateway mints wIPCT-FIL to user
4. User can use wIPCT-FIL for Filecoin subnet fees

To unlock:
1. Burn wIPCT-FIL on FilecoinGateway
2. Gateway sends message to Root IPC
3. Root IPC unlocks canonical IPCT
4. User receives IPCT on Root IPC

**Bridge to Ethereum:**
- Same process with EthereumGateway and wIPCT-ETH

**Security:**
- Bridges secured by Root IPC validator set
- Cryptographic proofs required for mint/burn
- Regular audits of locked vs minted supply
- Users can verify balances on-chain

#### 9.2.3 Subnet Token Flexibility

Execution subnets can choose fee token:

**Option 1: IPCT/wIPCT**
- Users pay fees in IPCT (or wrapped version)
- Validators receive IPCT
- Simple, unified economics

**Option 2: Native subnet token**
- Subnet issues own token for gas
- Validators receive subnet token
- Can convert to IPCT via DEX

**Option 3: Parent L1 token**
- Filecoin subnet uses FIL for gas
- Ethereum subnet uses ETH for gas
- Validators receive L1 token, convert to IPCT

**Most common:** IPCT/wIPCT for simplicity

### 9.3 Validator Reward Structure

#### 9.3.1 Base Reward Calculation

Per-epoch reward for validator:

```
total_reward = 
  (base_rate × subnets_validated × demand_multiplier × reputation_multiplier × role_multiplier) +
  subnet_fees +
  transaction_tips +
  message_routing_fees
```

**Where:**
- **base_rate:** Fixed IPCT per subnet per epoch (e.g., 10 IPCT)
- **subnets_validated:** Number of execution subnets assigned to
- **demand_multiplier:** Network utilization scaling (1.0x-2.0x)
- **reputation_multiplier:** 0.8x-1.5x based on reputation tier
- **role_multiplier:** Depends on validator role (see below)

All rewards in canonical IPCT (not affected by subnet's L1 parent)

#### 9.3.2 Role-Based Multiplier

- **Root chain validator:** 2.0x (highest responsibility)
- **Shard validator:** 1.5x (coordinating role, L1-agnostic routing)
- **Execution validator (1 subnet):** 1.0x (baseline)
- **Execution validator (2 subnets):** 2.0x (double duty)
- **Execution validator (3+ subnets, oversubscribed):** 1.5x per subnet
- **Storage validator (future):** 0.3x (lower requirements)

No bonus/penalty for which L1 the subnet uses as parent

#### 9.3.3 Demand-Based Multiplier

```
Network utilization = active_validators / total_validators
```

**Demand multiplier:**
- <70% utilization: 1.0x (normal operation)
- 70-85% utilization: 1.2x (getting busy)
- 85-95% utilization: 1.5x (high demand)
- >95% utilization: 2.0x (critical demand)

Attracts new validators when needed  
Independent of L1 distribution

#### 9.3.4 Example Calculation

**Validator profile:**
- Reputation tier: 2 (multiplier 1.2x)
- Role: Execution validator
- Subnets: 2 (one Filecoin parent, one Ethereum parent)
- Network utilization: 80%
- Epoch duration: 1 day

**Calculation:**
```
base_rate = 10 IPCT
subnets_validated = 2
demand_multiplier = 1.2x (80% utilization)
reputation_multiplier = 1.2x (tier 2)
role_multiplier = 2.0x (2 subnets)

base_reward = 10 × 2 × 1.2 × 1.2 × 2.0 = 57.6 IPCT per day

Additional earnings:
subnet_fees = 5 IPCT (from subnet operators, any L1)
transaction_tips = 2 IPCT (from users)
message_fees = 0.5 IPCT (routing cross-L1 messages)

Total daily reward = 57.6 + 5 + 2 + 0.5 = 65.1 IPCT per day
```

Same reward structure regardless of subnet L1 parents

### 9.4 Subnet Operator Fees

#### 9.4.1 Long-Lived Subnet Pricing

```
Monthly cost = 
  (base_rate_per_validator × validator_count × 720 hours) +
  (storage_fee × state_size_gb) +
  estimated_transaction_fees

Independent of L1 parent choice
```

**Example (Filecoin parent):**
- 7 validators
- base_rate = 0.2 IPCT per validator per hour
- storage = 10 GB @ 0.1 IPCT per GB per month
- transaction fees = variable

```
Monthly cost = (0.2 × 7 × 720) + (0.1 × 10) + txn_fees
             = 1,008 + 1 + variable
             = ~1,009 IPCT + transaction fees
```

**Example (Ethereum parent):**
- Same pricing
- Only difference: checkpoint to Ethereum instead of Filecoin
- Cost paid in IPCT (or wIPCT-ETH)

#### 9.4.2 Ephemeral Subnet Pricing

```
Total cost = 
  (base_rate_per_validator × validator_count × duration_minutes × ephemeral_multiplier) +
  (optional_archival_fee)
```

**Example (Ethereum parent):**
- 5 validators
- 60 minutes duration
- base_rate = 0.2 IPCT per validator per hour
- ephemeral_multiplier = 1.5x
- archival = 10 IPCT (to Filecoin, regardless of parent)

```
Total cost = (0.2 × 5 × 1 × 1.5) + 10
           = 1.5 + 10
           = 11.5 IPCT
```

Same pricing for Filecoin parent subnet

### 9.5 Token Emission Schedule

#### 9.5.1 Inflation Rate

- **Year 1-2:** 10% annual inflation (aggressive validator growth phase)
- **Year 3-5:** 7% annual inflation (moderate growth)
- **Year 6-10:** 5% annual inflation (mature network)
- **Year 10+:** 2-3% annual inflation (maintenance level)

**Emissions distributed:**
- 90% to validators (via base rewards, any L1)
- 10% to ecosystem development fund

#### 9.5.2 Deflationary Mechanisms

**IPCT burned from:**
- 50% of subnet deployment fees (any L1)
- 25% of cross-subnet message fees (including cross-L1 messages)
- 100% of slashing penalties
- 30% of transaction fees (if subnet pays in IPCT/wIPCT)

**Expected net effect:**
- Early years: Inflationary (bootstrapping)
- Mature network: Near-zero or slightly deflationary (high activity)

Cross-L1 activity increases burn rate (more messages)

### 9.6 Slashing Conditions & Penalties

#### 9.6.1 Slashing Tiers

**Minor Infractions:**
- Missed block proposal: -1 IPCT per miss
- Missed attestation: -0.5 IPCT per miss
- Delayed checkpoint: -5 IPCT
- Slow response to DA challenge: -2 IPCT

**Major Infractions:**
- Double signing: -100 IPCT (1% of minimum stake)
- Invalid state transition: -500 IPCT (5% of minimum stake)
- Data unavailability (proven): -200 IPCT
- Coordinated attack attempt: -10,000 IPCT (100% of minimum stake) + permanent ban

#### 9.6.2 L1-Agnostic Enforcement

- Same slashing rules regardless of subnet's L1 parent
- Slashing executed on Root IPC Chain (single stake)
- Propagates to all L1 Gateway Contracts via merkle root updates

---

## 10. Scaling Mechanisms

### 10.1 Dynamic Sharding Subnet Creation

#### 10.1.1 Shard Creation Triggers

Root IPC Chain monitors metrics:
1. Validators per shard
2. Execution subnets per shard (any L1 parent)
3. Checkpoint throughput per shard
4. Geographic distribution
5. L1 distribution (balance across Filecoin, Ethereum, etc.)

**Create new shard when ANY of:**
- Any shard has >250 validators
- Any shard has >120 execution subnets (regardless of L1 mix)
- Checkpoint processing time >50% of target
- Geographic clustering opportunity
- L1 imbalance (one shard has too many Filecoin subnets, create Ethereum-friendly shard)

**Note on L1 Distribution:**
- Shards are parent-agnostic by design
- But may create shards optimized for certain L1 ecosystems
- E.g., "Filecoin-optimized shard" with validators in Filecoin ecosystem proximity
- Doesn't restrict validators, just optimizes placement

#### 10.1.2 Parent-Agnostic Shard Operation

**Shard 1 example:**
- Manages 50 execution subnets
  - 30 with Filecoin parents
  - 15 with Ethereum parents
  - 5 with subnet parents (which may have different L1 grandparents)
- Same validator pool for all
- Routes checkpoints to appropriate destinations
- No impact on shard consensus or performance

### 10.2 Load Balancing Algorithm

#### 10.2.1 Shard Selection for New Execution Subnet

```python
def select_shard_for_new_subnet(subnet_requirements):
    eligible_shards = []
    parent_l1 = subnet_requirements.parent_l1
    
    for shard in all_shards:
        # Check capacity
        if shard.subnet_count >= MAX_SUBNETS_PER_SHARD:
            continue
        
        # Check available validators with L1 support
        available_validators = count_available_validators(
            shard, 
            subnet_requirements,
            parent_l1  # Most validators support all L1s
        )
        if available_validators < subnet_requirements.min_validators:
            continue
        
        # Check geographic match
        if subnet_requirements.geo_preference:
            if shard.region != subnet_requirements.geo_preference:
                continue
        
        eligible_shards.append(shard)
    
    if not eligible_shards:
        # No suitable shard, create new one
        if should_create_new_shard():
            new_shard = create_new_shard(
                geo_preference=subnet_requirements.geo_preference,
                l1_hint=parent_l1  # Hint for optimization, not restriction
            )
            return new_shard
        else:
            return None  # Request queued
    
    # Score shards by suitability
    scores = []
    for shard in eligible_shards:
        # Balance L1 distribution
        l1_balance_score = calculate_l1_balance(shard, parent_l1)
        
        score = (
            (1 - shard.validator_count / MAX_VALIDATORS_PER_SHARD) * 0.3 +
            (1 - shard.subnet_count / MAX_SUBNETS_PER_SHARD) * 0.3 +
            (shard.average_validator_reputation) * 0.2 +
            (l1_balance_score) * 0.1 +  # Prefer balanced L1 distribution
            (1 if shard.region == subnet_requirements.geo_preference else 0.5) * 0.1
        )
        scores.append((shard, score))
    
    # Select highest scoring shard
    target_shard = max(scores, key=lambda x: x[1])[0]
    return target_shard

def calculate_l1_balance(shard, target_l1):
    """
    Prefer shards with balanced L1 distribution
    Slight preference for shard with fewer subnets of target_l1
    (encourages distribution, but doesn't enforce it)
    """
    l1_counts = shard.count_subnets_by_l1()
    total_subnets = shard.subnet_count
    
    if total_subnets == 0:
        return 1.0  # Empty shard, perfect balance
    
    target_l1_ratio = l1_counts.get(target_l1, 0) / total_subnets
    
    # Prefer shards where target L1 is underrepresented
    # But not too heavily weighted (0.1 factor in main score)
    return 1.0 - target_l1_ratio
```

### 10.3 Capacity Planning

#### 10.3.1 Per-Shard Capacity (Parent-Agnostic)

**Validator Capacity:**
- Minimum: 100 validators (viable shard)
- Target: 150-200 validators (optimal operation)
- Maximum: 300 validators (triggers new shard creation)

**Execution Subnet Capacity:**
- Conservative: ~50-100 subnets per shard
- Mix of L1 parents doesn't affect capacity
- Example: 50 subnets = 30 Filecoin + 15 Ethereum + 5 recursive

**Checkpoint Throughput:**
- Each execution subnet checkpoints every ~100 seconds (configurable)
- Shard must process 50-100 checkpoints per 10-minute epoch
- Routes to multiple L1 Gateways in parallel (no bottleneck)
- At 4-second shard blocks: ~150 blocks per epoch
- ~1 checkpoint per shard block average (very feasible)

#### 10.3.2 Root Chain Capacity

**Shard Management:**
- Supports 50-100 sharding subnets
- Each shard checkpoints once per 10 minutes
- At 6-second root chain blocks: 100 blocks per 10 minutes
- Each block can include 1-2 shard checkpoints
- Total capacity: ~100-200 shard checkpoints per epoch

**L1 Bridge Coordination:**
- Updates validator merkle roots to each L1 (once per epoch)
- Processes cross-L1 message commitments
- Routes checkpoint summaries to appropriate L1s
- Parallel processing: Filecoin and Ethereum updates don't block each other

**Cross-Shard Message Relay:**
- Fast path message commitments: ~10-50 per root chain block
- Batching for high volumes
- Independent of L1 diversity

### 10.4 Network Scale Projections

#### 10.4.1 Conservative Estimate

**Root IPC Chain:** 50 sharding subnets  
**Per shard:** 200 validators, 50 execution subnets

**Total capacity:**
- **Validators:** 50 × 200 = 10,000 validators
- **Execution subnets:** 50 × 50 = 2,500 execution subnets
  - Distributed across Filecoin, Ethereum, and recursive subnets
  
**Example L1 distribution:**
- Filecoin-parent subnets: 1,500 (60%)
- Ethereum-parent subnets: 750 (30%)
- Subnet-parent subnets: 250 (10%)

#### 10.4.2 Aggressive Estimate

**Root IPC Chain:** 100 sharding subnets (optimized)  
**Per shard:** 300 validators, 100 execution subnets

**Total capacity:**
- **Validators:** 100 × 300 = 30,000 validators
- **Execution subnets:** 100 × 100 = 10,000 execution subnets
  
**Example L1 distribution at scale:**
- Filecoin-parent subnets: 5,000 (50%)
- Ethereum-parent subnets: 3,500 (35%)
- Other L1-parent subnets: 500 (5%)
- Subnet-parent subnets: 1,000 (10%)

Single validator pool serves all L1s

---

## 11. Security & Fault Tolerance

### 11.1 Byzantine Fault Tolerance

#### 11.1.1 Layer-Specific BFT Guarantees

**Root IPC Chain (L0):**
- Validator set: 50-100 validators
- Byzantine threshold: f < n/3
- Can tolerate: 16-33 Byzantine validators
- Consensus finality: 1-2 blocks (~6-12 seconds)
- Security is L1-independent

**Sharding Subnets (L1):**
- Validator set: 20-50 shard validators
- Byzantine threshold: f < n/3
- Can tolerate: 6-16 Byzantine validators
- Consensus finality: 1-2 blocks (~2-12 seconds)
- Parent-agnostic security: Same BFT regardless of managed subnets' L1 parents

**Execution Subnets (L2+):**
- Validator set: 4-21 validators (configurable)
- Byzantine threshold: f < n/3
- Examples:
  - 4 validators: Tolerates 1 Byzantine (Economy tier)
  - 7 validators: Tolerates 2 Byzantine (Standard tier)
  - 13 validators: Tolerates 4 Byzantine (High tier)
- Security from IPC validators, not from parent L1
- Same security model for Filecoin-parent, Ethereum-parent, or subnet-parent

### 11.2 Security Independence from Parent L1

**Key Design Principle:** Subnet security comes from IPC validators, not parent L1 validators.

#### 11.2.1 What Subnet Gets from Parent L1

**Filecoin Parent:**
- ✅ Finality anchor: Prevents long-range attacks
- ✅ Dispute resolution: Filecoin community can adjudicate if needed
- ✅ Asset bridging: Access to FIL tokens
- ✅ Ecosystem legitimacy: "Official" Filecoin subnet
- ✅ Archival: Leverage Filecoin storage (all subnets get this)
- ❌ NOT needed for: Liveness, validator selection, transaction processing

**Ethereum Parent:**
- ✅ Finality anchor: Prevents long-range attacks
- ✅ Dispute resolution: Ethereum community can adjudicate
- ✅ Asset bridging: Access to ETH and ERC-20 tokens
- ✅ Ecosystem legitimacy: "Official" Ethereum L2/subnet
- ✅ Smart contract integration: Call Ethereum contracts
- ❌ NOT needed for: Liveness, validator selection, transaction processing

**Subnet Parent:**
- ✅ Hierarchical finality: Inherits parent's finality chain
- ✅ Faster checkpointing: Can checkpoint more frequently to subnet than to L1
- ✅ Application coupling: Parent and child can be tightly integrated
- ❌ NOT needed for: Liveness (unless parent is down, subnet continues)

#### 11.2.2 Liveness Independence

**Scenario:** Filecoin L1 has issues (network congestion, hard fork, etc.)

**Impact on Filecoin-parent subnet:**
- Subnet CONTINUES operating normally
- Validators assigned by Root IPC, not Filecoin
- Transactions processed at <1s blocks
- Only checkpointing is delayed
- Once Filecoin recovers, resume checkpointing

**Scenario:** Ethereum L1 has issues

**Impact on Ethereum-parent subnet:**
- Subnet CONTINUES operating normally
- Same as above, validators are IPC validators
- Checkpointing delayed until Ethereum recovers

**Key point:** Subnets don't depend on parent L1 for liveness

### 11.3 Cross-L1 Attack Vectors

#### 11.3.1 L1 Gateway Contract Compromise

**Attack:** Malicious actor gains control of Gateway Contract on one L1.

**Impact:**
- Could accept invalid checkpoints for subnets with that L1 parent
- Could disrupt subnet deployment on that L1
- Could interfere with bridge (lock/unlock tokens)

**Mitigation:**
- Gateway contracts are minimal and audited
- Validator signature verification is cryptographic (can't be bypassed)
- Root IPC monitors Gateway contracts for anomalies
- Emergency pause mechanism via Root IPC governance
- Subnets can migrate to different L1 parent if needed

**Scope of damage:**
- Limited to subnets with that specific L1 parent
- Doesn't affect subnets with other L1 parents
- Doesn't affect validator pool or Root IPC

#### 11.3.2 Bridge Exploit

**Attack:** Malicious actor exploits bridge between Root IPC and L1.

**Impact:**
- Could mint unauthorized wIPCT on L1
- Could steal locked IPCT from Root IPC

**Mitigation:**
- Bridge secured by Root IPC validator set (50-100 validators)
- Cryptographic proofs required for all operations
- Regular audits of locked vs minted supply
- Circuit breakers for large transfers
- Multiple independent relayers (decentralized)
- Users can verify balances on-chain

**Scope of damage:**
- Could affect wIPCT on compromised L1
- Canonical IPCT on Root IPC remains secure
- Other L1 bridges unaffected

#### 11.3.3 Validator Collusion Across L1s

**Attack:** Validators collude to attack multiple subnets across different L1s.

**Impact:**
- Could compromise multiple subnets simultaneously

**Mitigation:**
- Random validator assignment via VRF (same for all L1s)
- Weekly rotation prevents long-term collusion
- Reputation system identifies suspicious patterns
- Global slashing affects validator across all L1s
- Staking on Root IPC creates unified collateral

**Detection:**
- Abnormal behavior patterns across L1s
- Cross-L1 correlation analysis
- Community reporting mechanisms

### 11.4 Failure Modes & Recovery

#### 11.4.1 L1 Gateway Contract Failure

**Scenario:** Filecoin Gateway Contract becomes unavailable

**Impact:**
- New Filecoin-parent subnets can't be deployed
- Existing Filecoin-parent subnets can't checkpoint to Filecoin
- Checkpointing to Root IPC continues (via shard → Root IPC)
- Ethereum-parent subnets unaffected

**Subnet behavior:**
- Continue operating normally
- Checkpoints queue locally
- Once Gateway restored, submit queued checkpoints

**Recovery:**
1. Root IPC detects Gateway failure
2. Deploy new Gateway Contract on Filecoin
3. Update Root IPC with new Gateway address
4. Resume checkpointing to new Gateway
5. Existing subnets migrate to new Gateway automatically

**Downtime for checkpointing:** ~1-24 hours (depends on recovery speed)  
**Downtime for subnet operation:** 0 (subnets continue running)

#### 11.4.2 Root IPC Chain Failure

**Scenario:** Root IPC Chain has consensus failure (<67% validators online)

**Impact:**
- No new validator assignments
- No new shard creation
- Cross-shard messages delayed
- Sharding subnets continue operating
- Execution subnets continue operating

**Behavior:**
- All subnets continue producing blocks
- Checkpoints queue in sharding subnets
- Cross-shard messages queue in Root IPC (when it returns)
- Validators continue duties based on last known assignments

**Recovery:**
1. Root IPC validators restored
2. Resume consensus from last finalized block
3. Process queued checkpoints from shards
4. Process queued cross-shard messages
5. Resume normal operations

**Downtime for coordination:** Hours (worst case)  
**Downtime for subnets:** 0 (continue operating)  
**Data loss:** None (checkpoints queued)

This is a catastrophic but low-probability event

#### 11.4.3 Multi-L1 Simultaneous Failure

**Scenario:** Both Filecoin and Ethereum have issues simultaneously

**Impact:**
- Subnets with L1 parents can't checkpoint to L1s
- But checkpointing to Root IPC continues
- Subnets continue operating
- Cross-subnet communication continues (via IBC, independent of L1s)

**Behavior:**
- Business as usual for subnet operations
- Only L1 finality anchoring is delayed
- Once L1s recover, resume checkpointing

This scenario demonstrates why subnet security is independent from L1s

---

## 12. Bootstrap & Migration Path

### 12.1 Network Phases

#### 12.1.1 Phase 1: Single L1 (Filecoin), Single Shard (Months 1-6)

**Configuration:**
- Root IPC Chain operational
- Bridge to Filecoin only
- Single sharding subnet
- Up to 200 validators
- Up to 50-100 execution subnets (all Filecoin parents)

**Goals:**
- Prove out architecture with Filecoin ecosystem
- Establish validator community
- Refine tokenomics
- Build tooling and SDKs

**Why Filecoin First:**
- Natural fit: IPC born from Filecoin ecosystem
- Storage integration: Leverage Filecoin for archival
- Existing relationships with Filecoin community

#### 12.1.2 Phase 2: Multi-L1 Introduction (Months 6-12)

**Trigger:** Filecoin subnets stable, community demand for Ethereum

**Actions:**
- Deploy Ethereum Gateway Contract
- Establish Root IPC ↔ Ethereum bridge
- Enable wIPCT-ETH minting
- Allow Ethereum-parent subnet deployments
- Same validator pool serves both L1s

**Configuration:**
- Root IPC Chain bridges to Filecoin + Ethereum
- 2-5 sharding subnets
- 200-500 validators
- 100-300 execution subnets
- Mix of Filecoin-parent and Ethereum-parent
- Same shards manage both

**Milestones:**
- First Ethereum-parent subnet deployed
- First cross-L1 IBC message (Filecoin subnet → Ethereum subnet)
- Validator serves both Filecoin and Ethereum subnets

#### 12.1.3 Phase 3: Full Multi-Shard, Multi-L1 (Month 12-24)

**Configuration:**
- Root IPC bridges to Filecoin + Ethereum + (potentially others)
- 5-20 sharding subnets
- 1,000-5,000 validators
- 500-1,500 execution subnets
- Distributed across L1s
- High volume of cross-L1 communication

**Characteristics:**
- Fully parent-agnostic sharding
- Automatic shard creation/rebalancing
- Geographic sharding active
- High cross-L1 message volumes
- Mature tooling and ecosystem

#### 12.1.4 Phase 4: Mature Multi-Chain Network (Year 2+)

**Configuration:**
- Root IPC bridges to 3+ L1s
- 20-50 sharding subnets
- 5,000-10,000 validators
- 2,000-5,000 execution subnets

**Characteristics:**
- Production-ready infrastructure
- Multiple L1 ecosystems thriving
- Recursive subnet patterns established
- Advanced features (specialized shards, ZK proofs, etc.)

### 12.2 Gateway Contract Deployment

#### 12.2.1 Filecoin Gateway (Phase 1)

**Deployment steps:**
1. Audit Gateway Contract code
2. Deploy to Filecoin mainnet
3. Initialize with Root IPC bridge address
4. Fund with initial liquidity (for bridge)
5. Register Gateway address on Root IPC
6. Begin accepting subnet deployments

**Timeline:** Month 1-2 of Phase 1

#### 12.2.2 Ethereum Gateway (Phase 2)

**Deployment steps:**
1. Adapt Gateway Contract for Ethereum (gas optimizations)
2. Audit Ethereum-specific code
3. Deploy to Ethereum mainnet
4. Establish Root IPC ↔ Ethereum bridge
5. Fund with initial ETH and wIPCT liquidity
6. Register Gateway address on Root IPC
7. Update validator assignment logic (now L1-agnostic)
8. Enable Ethereum-parent subnet deployments

**Timeline:** Month 6-8

### 12.3 Validator Migration to Multi-L1

**Challenge:** Validators initially onboarded for Filecoin only. How to enable multi-L1 support?

**Solution: Automatic for Most Validators**

**Existing validators (Phase 1):**
- Already staked on Root IPC (not Filecoin)
- No L1-specific stake required
- Mostly running L1-agnostic software

**Phase 2 transition:**
1. Validators receive software update
   - Add Ethereum RPC support
   - Update checkpoint routing logic
2. No re-staking required
3. No downtime
4. Gradually assigned to Ethereum-parent subnets
5. Begin earning rewards from both L1 ecosystems

**Opt-out:** Validators can choose to only support Filecoin
- Register preference on Root IPC
- Won't be assigned to Ethereum-parent subnets
- May earn slightly less (fewer assignment opportunities)

### 12.4 Testing & Rollout

#### 12.4.1 Filecoin Integration Testing

**Stage 1: Internal testnet (2 months)**
- Root IPC + Filecoin Gateway on testnet
- Single shard, 10-20 internal validators
- Deploy test subnets with Filecoin parents
- Checkpoint flow testing

**Stage 2: Public testnet (2 months)**
- Open to Filecoin community
- Incentivized testing program
- Real subnet deployments
- Bug bounties

**Stage 3: Canary mainnet (1 month)**
- Limited mainnet deployment
- Real IPCT stake, limited exposure
- 1-2 shards, 50-100 validators
- Gradual migration of early subnets

**Stage 4: Full Filecoin mainnet (ongoing)**

#### 12.4.2 Ethereum Integration Testing

**Stage 1: Ethereum testnet integration (2 months)**
- Deploy Gateway on Goerli/Sepolia
- Test bridge with Filecoin mainnet
- Cross-L1 message testing
- First Ethereum-parent testnet subnet

**Stage 2: Ethereum mainnet bridge (1 month)**
- Deploy Gateway on Ethereum mainnet
- Limited validator set initially
- First Ethereum-parent mainnet subnet
- Monitor closely for issues

**Stage 3: Full Ethereum integration (ongoing)**
- All validators support Ethereum
- High volume of Ethereum-parent subnets
- Cross-L1 messaging at scale

---

## 13. Performance Targets

### 13.1 Latency Targets

#### 13.1.1 Block Times

| Layer | Target | Acceptable | Notes |
|-------|--------|------------|-------|
| Root IPC (L0) | 6s | 6-12s | Coordinates multiple L1s |
| Sharding Subnet (L1) | 4s | 2-6s | Parent-agnostic |
| Execution Subnet (L2+) | 1s | 0.5-2s | Same for all L1 parents |

#### 13.1.2 Cross-Subnet Messaging

| Message Type | Target Latency | Notes |
|--------------|----------------|-------|
| Intra-shard, same L1 | 5s | Traditional |
| Intra-shard, different L1 (IBC) | 5s | Key innovation |
| Inter-shard, same L1 (fast) | 20s | Via Root IPC relay |
| Inter-shard, different L1 (fast) | 20s | Via Root IPC relay |
| Inter-shard (standard) | 12m | Via checkpoints |

**Key Point:** Cross-L1 communication (Filecoin ↔ Ethereum) is as fast as same-L1 communication (~5 seconds via IBC).

### 13.2 Throughput Targets

#### 13.2.1 Transactions Per Second

**Per Execution Subnet:**
- Target: 1,000-5,000 TPS
- Independent of parent L1 choice
- Same performance for Filecoin, Ethereum, or subnet parents

**Network-Wide TPS:**

**Conservative (2,500 execution subnets):**
- 2,500 subnets × 1,000 TPS = 2,500,000 TPS aggregate
- Distributed across Filecoin and Ethereum ecosystems

**Aggressive (10,000 execution subnets):**
- 10,000 subnets × 2,500 TPS = 25,000,000 TPS aggregate

#### 13.2.2 Cross-L1 Message Throughput

**Target:** 1,000-10,000 cross-L1 messages per second at scale

Example at 10,000 messages/second:
- ~864 million messages per day
- Enables high-frequency cross-L1 interactions
- DeFi, gaming, social apps across L1 boundaries

### 13.3 Scalability Targets

#### 13.3.1 Year 1 Targets (Filecoin Focus)

- **L1s supported:** Filecoin (Ethereum in beta)
- **Validators:** 200-1,000
- **Execution Subnets:** 50-200
  - Filecoin-parent: 50-180
  - Ethereum-parent: 0-20 (late in year)
- **Sharding Subnets:** 1-5
- **Aggregate TPS:** 50,000-200,000
- **Cross-L1 messages/day:** 0-10,000 (late in year)

#### 13.3.2 Year 3 Targets (Multi-L1 Mature)

- **L1s supported:** Filecoin, Ethereum, 1-2 others
- **Validators:** 2,000-5,000
- **Execution Subnets:** 500-1,000
  - Filecoin-parent: 300-600 (60%)
  - Ethereum-parent: 150-300 (30%)
  - Other/recursive: 50-100 (10%)
- **Sharding Subnets:** 10-20
- **Aggregate TPS:** 500,000-1,000,000
- **Cross-L1 messages/day:** 1,000,000-5,000,000

#### 13.3.3 Year 5+ Targets (Full Scale)

- **L1s supported:** 5+ major chains
- **Validators:** 10,000-30,000
- **Execution Subnets:** 2,500-10,000
  - Multi-L1 distribution
  - Deep recursive hierarchies
- **Sharding Subnets:** 50-100
- **Aggregate TPS:** 2,500,000-25,000,000
- **Cross-L1 messages/day:** 10,000,000-100,000,000

Enables true multi-chain ecosystem at scale

### 13.4 Cost Targets

#### 13.4.1 Validator Costs

- **Base Rate:** 100 IPCT per validator per month
- **Demand Multiplier:** 1-5x based on supply/demand
- **Reputation Premium:** +10-50% for high-reputation validators

#### 13.4.2 Subnet Deployer Costs

**5-validator subnet:**
- 500 IPCT/month (~$50-500/month depending on IPCT price)

**20-validator subnet:**
- 2,000 IPCT/month (~$200-2,000/month)

**100-validator subnet:**
- 10,000 IPCT/month (~$1,000-10,000/month)

---

## 14. Future Considerations

### 14.1 Additional L1 Integration

#### 14.1.1 Candidate L1s

**Near-term (Year 2-3):**
- **Polygon:** Large EVM ecosystem, low fees
- **Avalanche:** Fast finality, subnets concept aligns with IPC
- **Cosmos Hub:** IBC native, natural fit

**Long-term (Year 3+):**
- **Solana:** High performance, different architecture (challenge)
- **Polkadot:** Shared security model, interesting overlap
- **Near:** Sharding-native design
- **Other EVM chains:** As demand arises

#### 14.1.2 Integration Requirements

For new L1 to be supported:
1. Smart contract capability (for Gateway Contract) OR trusted bridge mechanism
2. Sufficient decentralization and security
3. Active ecosystem and user base
4. Community demand from IPC validator set
5. Economic viability (gas costs, bridge costs)

**Process:**
1. Community proposal and vote
2. Deploy Gateway Contract on new L1
3. Establish Root IPC ↔ L1 bridge
4. Security audit
5. Gradual rollout (testnet → limited mainnet → full support)

### 14.2 Cross-L1 Liquidity & Assets

#### 14.2.1 Native Asset Bridges

**Challenge:** Users want to use FIL on Ethereum subnets, ETH on Filecoin subnets, etc.

**Solution: Gateway-Mediated Bridges**

Example: Bridge FIL to Ethereum subnet
1. User locks FIL on Filecoin Gateway
2. Filecoin Gateway sends message to Root IPC
3. Root IPC routes to Ethereum Gateway
4. Ethereum Gateway mints wFIL-ETH
5. User can use wFIL-ETH on Ethereum-parent subnets
6. Reverse process to unlock

**Security:** Same as IPCT bridge (Root IPC validator set)

#### 14.2.2 Cross-L1 DEX Integration

**Concept:** DEX deployed as IPC subnet enables cross-L1 swaps

Example: Uniswap-style AMM on IPC
- Pool 1: FIL / wIPCT
- Pool 2: ETH / wIPCT
- Pool 3: FIL / ETH (via wIPCT intermediary)

**Users can swap assets across L1s:**
- Fast: <5 seconds via IBC
- Cheap: Subnet transaction fees, not L1 fees
- Secure: IPC validator set

This creates unified liquidity across L1 ecosystems

### 14.3 L1-Specific Optimizations

#### 14.3.1 Filecoin Storage Integration

**Specialized subnet type: "Storage Coordination Subnet"**

Features:
- Native Filecoin storage deal integration
- High-frequency deal matching (<1s)
- Batch deals into Filecoin checkpoints
- Leverage Filecoin for data availability

**Use case:** Next-gen storage marketplace on Filecoin

#### 14.3.2 Ethereum DeFi Integration

**Specialized subnet type: "DeFi Rollup Subnet"**

Features:
- EVM-compatible (run Solidity contracts)
- Fast finality (<1s)
- Cheap transactions (1000x cheaper than Ethereum)
- Checkpoint to Ethereum for finality
- Bridge to Ethereum tokens (USDC, DAI, etc.)

**Use case:** High-frequency DeFi applications

### 14.4 Governance Evolution

#### 14.4.1 Multi-L1 Governance

**Challenge:** How to govern a network spanning multiple L1 ecosystems?

**Proposal: Tiered Governance**

**Tier 1: Root IPC Governance**
- IPCT token holders vote
- Applies to all L1s
- Examples: Validator staking requirements, slashing rules

**Tier 2: L1-Specific Governance**
- Specific to one L1 ecosystem
- Examples: Gateway contract upgrades, L1-specific parameters

**Tier 3: Subnet Governance**
- Specific to individual subnet
- Examples: Consensus changes, fee structures

Ensures autonomy while maintaining coordination

### 14.5 Research Directions

#### 14.5.1 ZK Proofs for Cross-L1 Communication

**Concept:** Replace IBC light clients with ZK proofs

**Benefits:**
- Smaller proofs (~few KB vs full headers)
- Faster verification
- Better privacy

**Challenge:**
- Proving time overhead
- Complexity

**Timeline:** Year 3+ research

#### 14.5.2 Optimistic Cross-L1 Bridges

**Concept:** Assume messages are valid, challenge if not

**Benefits:**
- Faster bridge transfers (minutes vs hours)
- Lower cost (fewer on-chain verifications)

**Challenge:**
- Challenge period introduces latency
- Requires watchers

Could be applied to IPCT bridges between Root IPC and L1s

---

## Appendix A: Glossary

- **Base Rate:** Fixed IPCT reward per validator per epoch for participation.
- **Byzantine Fault Tolerance (BFT):** Ability of a distributed system to reach consensus despite some validators acting maliciously.
- **Canonical IPCT:** Native IPCT token on Root IPC Chain (single source of truth).
- **Checkpoint:** Periodic snapshot of subnet state posted to parent (L1 Gateway, subnet, or shard) for finality and data availability.
- **CometBFT:** BFT consensus mechanism optimized for fast block times, compatible with IBC.
- **Data Availability (DA):** Guarantee that blockchain data can be retrieved and verified.
- **Ephemeral Subnet:** Short-lived execution subnet that auto-destructs after a specified duration.
- **Erasure Coding:** Data encoding technique that enables reconstruction from partial data.
- **Execution Subnet (Layer 2+):** User-deployed subnet for fast transaction processing. Can be child of L1 or another subnet.
- **F3:** Fast Finality via Aggregated Certificates; Filecoin's finality mechanism used for verification between Filecoin L1 and IPC subnets.
- **Gateway Contract:** Smart contract deployed on L1 (Filecoin, Ethereum) that interfaces between IPC and the L1 ecosystem.
- **IBC (Inter-Blockchain Communication):** Protocol for direct peer-to-peer communication between subnets, even with different L1 parents.
- **IPCT:** IPC Token; native token on Root IPC Chain for staking, fees, and governance.
- **L1 / Layer 1:** External blockchain (Filecoin, Ethereum, etc.) that subnets can use as parent.
- **L0 / Layer 0:** Root IPC Chain; top-level coordination layer connecting multiple L1s.
- **Long-Lived Subnet:** Permanently-running execution subnet with persistent state.
- **Parent-Agnostic:** Design principle where sharding subnets and validators don't depend on which L1 their execution subnets use as parents.
- **Power Table:** On-chain registry tracking validator stake and assignments.
- **Recursive Subnet:** Subnet whose parent is another subnet (not an L1), enabling arbitrary nesting depth.
- **Reputation Score:** Metric reflecting validator reliability, tenure, and performance across all L1s.
- **Root IPC Chain:** Layer 0 of IPC hierarchy; global coordinator bridging multiple L1 ecosystems.
- **Sharding Subnet:** Layer 1 subnet managing a pool of validators and routing checkpoints to appropriate L1 Gateways.
- **Slashing:** Penalty (IPCT deduction) for validator misbehavior.
- **VRF (Verifiable Random Function):** Cryptographic function for provably random validator selection.
- **wIPCT:** Wrapped IPCT token on L1 (wIPCT-FIL on Filecoin, wIPCT-ETH on Ethereum).

---

## Appendix B: Open Questions & Future Work

### B.1 Technical

1. **Bridge security:** What is the optimal security model for Root IPC ↔ L1 bridges? Optimistic rollup style vs ZK proofs?
2. **L1 finality variability:** How to handle different finality times across L1s? (Filecoin 30s, Ethereum 15 min)
3. **Gateway gas optimization:** Can we further optimize Gateway Contract gas costs on Ethereum?
4. **Cross-L1 atomic swaps:** Can we enable atomic swaps between assets on different L1s without trusted intermediaries?
5. **L1 diversity limits:** Is there a practical limit to how many L1s Root IPC can support?
6. **Validator L1 preferences:** Should validators be able to express L1 preferences, or remain fully agnostic?

### B.2 Economic

1. **Cross-L1 fee market:** How should fees be priced for cross-L1 messages vs same-L1 messages?
2. **wIPCT liquidity:** How to bootstrap liquidity for wIPCT on each L1?
3. **L1-specific incentives:** Should there be incentives for validators to serve underrepresented L1s?
4. **Bridge reserve ratios:** What's the optimal ratio of locked IPCT to minted wIPCT?

### B.3 Governance

1. **L1 addition process:** What criteria and process for adding new L1 support?
2. **L1 deprecation:** How to gracefully deprecate support for an L1 if needed?
3. **Gateway upgrades:** How to upgrade Gateway Contracts without disrupting existing subnets?
4. **Cross-L1 disputes:** How to resolve disputes involving subnets with different L1 parents?

### B.4 Operational

1. **Multi-L1 monitoring:** What dashboards and tools are needed for cross-L1 operations?
2. **Validator L1 connectivity:** Do validators need to run full nodes for each L1, or just light clients?
3. **L1 RPC reliability:** How to handle L1 RPC outages gracefully?
4. **Emergency L1 disconnection:** Process for temporarily disconnecting from a problematic L1?

---

## Appendix C: References & Related Work

### C.1 Multi-Chain Architectures
- **Polkadot:** Parachains and relay chain
- **Cosmos:** Hub and zones with IBC
- **LayerZero:** Omnichain messaging protocol
- **Axelar:** Cross-chain communication

### C.2 L1 Integration Patterns
- **Optimistic Rollups:** Arbitrum, Optimism (Ethereum L2s)
- **ZK Rollups:** zkSync, StarkNet (Ethereum L2s)
- **Avalanche Subnets:** Application-specific blockchains
- **Polygon Supernets:** Enterprise blockchain infrastructure

### C.3 Consensus & Communication
- **CometBFT:** https://docs.cometbft.com/
- **IBC Protocol:** https://ibcprotocol.org/
- **F3:** Filecoin's finality mechanism for L1-subnet verification

### C.4 Bridges & Interoperability
- **Cross-chain bridges:** Wormhole, Multichain, Synapse
- **Bridge security:** Security models and trade-offs
- **Optimistic bridges:** Fraud proof mechanisms

---

## Document Control

### Version History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-10-02 | Claude (for IPC Team) | Initial specification |
| 2.0 | 2025-10-03 | Claude (for IPC Team) | Multi-L1 architecture, parent-agnostic shards, IBC communication, recursive subnets, Gateway contracts |

**Review Status:** Draft - Awaiting review by IPC core team

**Approval:** Pending

### Next Steps

1. Core team review and feedback on multi-L1 design
2. Technical feasibility assessment for Gateway contracts and bridges
3. Prototype development for:
   - Filecoin Gateway Contract
   - Ethereum Gateway Contract
   - Root IPC bridges
   - Parent-agnostic shard routing
   - Cross-L1 IBC messaging
4. Testnet implementation roadmap
5. Finalize specification v2.1

---

**END OF SPECIFICATION**