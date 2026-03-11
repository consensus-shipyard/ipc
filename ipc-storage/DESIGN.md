# IPC Storage: Replication and Storage Proof Design

This document describes the design of data replication and storage proof mechanisms in IPC Storage. The system uses **client-side encryption** for data privacy, **Reed-Solomon erasure encoding** for fault-tolerant replication, and **Provable Data Possession (PDP)** based on Merkle proofs for storage verification.

## Table of Contents

- [Overview](#overview)
- [System Architecture](#system-architecture)
- [Client-Side Encryption](#client-side-encryption)
- [Data Hierarchy](#data-hierarchy)
- [Replication: Reed-Solomon Erasure Encoding](#replication-reed-solomon-erasure-encoding)
- [Merkle Tree Construction](#merkle-tree-construction)
- [On-Chain Commitment](#on-chain-commitment)
- [Data Distribution](#data-distribution)
- [Storage Proof (PDP Challenge)](#storage-proof-pdp-challenge)
- [Verification Process](#verification-process)
- [Security Considerations](#security-considerations)
- [Economic Model](#economic-model)
- [References](#references)

---

## Overview

IPC Storage provides decentralized, verifiable storage with the following guarantees:

1. **Data Privacy**: Client-side encryption ensures storage nodes cannot read user data.
2. **Data Availability**: Reed-Solomon erasure encoding ensures data can be recovered even if some storage nodes fail or become unavailable.
3. **Data Integrity**: Merkle tree commitments allow efficient verification that stored data matches the original.
4. **Proof of Storage**: Random challenge-response protocol proves that storage nodes actually hold the data they claim to store.

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CLIENT                                          │
│                                                                              │
│  ┌──────────┐   ┌───────────┐   ┌──────────┐   ┌─────────┐   ┌───────────┐ │
│  │   Data   │──▶│  Encrypt  │──▶│  Chunk   │──▶│ Erasure │──▶│  Merkle   │ │
│  │          │   │  (AES)    │   │  Split   │   │ Encode  │   │   Tree    │ │
│  └──────────┘   └───────────┘   └──────────┘   └─────────┘   └───────────┘ │
│                                                                  │           │
└──────────────────────────────────────────────────────────────────┼───────────┘
                                                                   │
                                    ┌──────────────────────────────┼──────────┐
                                    │        ON-CHAIN              │          │
                                    │                              ▼          │
                                    │  ┌──────────────────────────────────┐   │
                                    │  │    File Merkle Root Commitment   │   │
                                    │  │    + Storage Metadata            │   │
                                    │  └──────────────────────────────────┘   │
                                    │                                         │
                                    │  ┌──────────────────────────────────┐   │
                                    │  │     Challenge Contract           │   │
                                    │  │     (VRF-based Random Selection) │   │
                                    │  └──────────────────────────────────┘   │
                                    └─────────────────────────────────────────┘

     ┌────────────────┐    ┌────────────────┐    ┌────────────────┐
     │  Storage Node  │    │  Storage Node  │    │  Storage Node  │
     │      (1)       │    │      (2)       │    │     (n)        │
     │                │    │                │    │                │
     │ Encrypted      │    │ Encrypted      │    │ Encrypted      │
     │ Chunk + Proofs │    │ Chunk + Proofs │    │ Chunk + Proofs │
     └────────────────┘    └────────────────┘    └────────────────┘
```

---

## Client-Side Encryption

All data is encrypted on the client before chunking and distribution. Storage nodes only ever see ciphertext and cannot read the underlying data.

### Encryption Scheme

| Component | Algorithm | Description |
|-----------|-----------|-------------|
| **Symmetric encryption** | AES-256-GCM | Encrypts the actual data |
| **Key derivation** | HKDF-SHA256 | Derives encryption key from master secret |
| **Key encryption** | ECIES / RSA-OAEP | Encrypts DEK for storage/sharing |

### Encryption Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           CLIENT ENCRYPTION                                  │
│                                                                              │
│  1. Generate random Data Encryption Key (DEK)                               │
│     DEK = random(256 bits)                                                  │
│                                                                              │
│  2. Encrypt data with DEK                                                   │
│     ciphertext = AES-256-GCM(plaintext, DEK, nonce)                         │
│                                                                              │
│  3. Encrypt DEK with client's public key (for later retrieval)              │
│     encrypted_dek = ECIES_Encrypt(DEK, client_pubkey)                       │
│                                                                              │
│  4. Store encrypted_dek securely (client-side or key management service)    │
│                                                                              │
│  5. Proceed with chunking on ciphertext (not plaintext)                     │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Key Management

```
┌─────────────────────────────────────────────────────────────────┐
│                      KEY HIERARCHY                               │
│                                                                  │
│  Master Key (client-controlled)                                 │
│       │                                                          │
│       ├──▶ File Key 1 (derived via HKDF + file_id)              │
│       │         └──▶ DEK for File 1                             │
│       │                                                          │
│       ├──▶ File Key 2 (derived via HKDF + file_id)              │
│       │         └──▶ DEK for File 2                             │
│       │                                                          │
│       └──▶ ...                                                   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Sharing Encrypted Data

To share data with another user:

1. Retrieve the encrypted DEK
2. Decrypt DEK with owner's private key
3. Re-encrypt DEK with recipient's public key
4. Share re-encrypted DEK with recipient

The actual stored data never changes; only key access is granted.

---

## Data Hierarchy

Data is organized in a three-level hierarchy. All operations occur on **encrypted** data.

```
┌─────────────────────────────────────────────────────────────────┐
│                    Original Data (Plaintext)                     │
└─────────────────────────────────────────────────────────────────┘
                                │
                          Encryption
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Encrypted Data (Ciphertext)                   │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌───────────┬───────────┬───────────┬───────────┬─────────────────┐
│  Chunk 0  │  Chunk 1  │  Chunk 2  │    ...    │    Chunk k-1    │
└───────────┴───────────┴───────────┴───────────┴─────────────────┘
                                │
                        Erasure Encoding
                                │
                                ▼
┌───────────┬───────────┬───────────┬───────────┬─────────────────┐
│  Encoded  │  Encoded  │  Encoded  │    ...    │    Encoded      │
│  Chunk 0  │  Chunk 1  │  Chunk 2  │    ...    │    Chunk n-1    │
│  (data)   │  (data)   │  (parity) │           │    (parity)     │
└───────────┴───────────┴───────────┴───────────┴─────────────────┘
      │
      ▼
┌────────────────────────────────────────────────────┐
│                  Encoded Chunk                      │
├────────────┬────────────┬────────────┬─────────────┤
│   Piece 0  │   Piece 1  │    ...     │  Piece P-1  │
├────────────┼────────────┼────────────┼─────────────┤
│ Leaf 0..L  │ Leaf 0..L  │    ...     │ Leaf 0..L   │
└────────────┴────────────┴────────────┴─────────────┘
```

### Terminology

| Term | Description | Typical Size |
|------|-------------|--------------|
| **Chunk** | A segment of encrypted data before/after erasure encoding | 1-64 MB |
| **Piece** | A subdivision of an encoded chunk | 256 KB - 1 MB |
| **Leaf** | The smallest unit for Merkle tree construction | 256 bytes - 1 KB |

---

## Replication: Reed-Solomon Erasure Encoding

IPC Storage uses Reed-Solomon erasure coding, similar to [Storj](https://storj.io/)'s approach, to achieve fault-tolerant data storage. The implementation lives in the `erasure-encoding` crate.

### How Reed-Solomon Works

Reed-Solomon encoding transforms `k` data shards into `k + m` total shards (where `m` = parity), such that:
- Any `k` of the `k + m` shards are sufficient to reconstruct the original data
- Up to `m` shards can be lost without data loss

### Encoding Parameters

| Parameter | Symbol | Description | Example |
|-----------|--------|-------------|---------|
| Data shards | k | Number of original data shards per chunk | 15 |
| Parity shards | m | Number of redundancy shards per chunk | 8 |
| Total shards | n | k + m = total shards per chunk | 23 |
| Max chunk size | — | Maximum bytes per chunk before RS encoding | 16 MiB |
| Expansion factor | n/k | Storage overhead ratio | 1.53x |

### Architecture

The crate is built around three core traits:

| Trait | Purpose |
|-------|---------|
| `Encoder` | Splits raw bytes into `k` padded shards and computes `m` parity shards |
| `Decoder` | Recovers original data from any `k` of `k + m` shards |
| `NodeAssigner` | Maps each shard to a storage node |

All trait methods are stateless (no `&self`) with an associated `Shard` type. Repair is simply decode followed by encode — no separate `Repairer` trait is needed.

The concrete implementation uses `reed-solomon-simd` for SIMD-accelerated Galois Field arithmetic (`ReedSolomonEncoder`).

### Encoding Process

Large files are split into chunks of up to 16 MiB (`DEFAULT_MAX_CHUNK_SIZE`). Each chunk is independently Reed-Solomon encoded, producing `k + m` shards. This means only one chunk's shards need to be in memory at a time.

```
┌─────────────────────────────────────────────────────────────────┐
│              Encrypted Data (Ciphertext, any size)               │
└─────────────────────────────────────────────────────────────────┘
                                │
                    Split into 16 MiB chunks
                                │
                                ▼
┌───────────┬───────────┬───────────┬───────────┬─────────────────┐
│  Chunk 0  │  Chunk 1  │  Chunk 2  │    ...    │    Chunk C-1    │
│  (16 MiB) │  (16 MiB) │  (16 MiB) │           │   (≤ 16 MiB)   │
└───────────┴───────────┴───────────┴───────────┴─────────────────┘
                                │
              Per-chunk Reed-Solomon Encoding
                                │
                                ▼
          For each chunk, produce k + m shards:
┌───────────┬───────────┬───────────┬───────────┬─────────────────┐
│  Shard 0  │  Shard 1  │    ...    │ Shard k-1 │  Shard k..k+m-1 │
│  (data)   │  (data)   │           │  (data)   │    (parity)     │
└───────────┴───────────┴───────────┴───────────┴─────────────────┘
                                │
                    NodeAssigner distributes
                                │
                                ▼
          Each shard assigned to a storage node
```

**Shard padding**: Each chunk is split into `k` equal-sized shards with zero-padding (rounded up to even size, as required by `reed-solomon-simd`). The `original_data_len` is preserved per chunk so padding can be stripped during decoding.

### Deterministic Shard Assignment

Shards are mapped to storage nodes deterministically using the `blob_id` and the active node list at the encoding epoch. No mapping table is stored — any party can recompute the assignment:

```
rotation_offset = blob_id % num_nodes

For each chunk's k + m shards:
  shard_global = chunk_index * (k + m) + shard_index
  node = nodes[(shard_global + rotation_offset) % num_nodes]
```

The node list is retrieved from on-chain state at the `encoding_epoch`. This ensures pseudo-random distribution across nodes without storing per-shard assignments.

### Storage in Iroh

Each shard is stored in Iroh under a deterministic key:

```
key = blob_id / chunk_index / shard_index
value = shard data (encrypted bytes)
```

Nodes only store the shards assigned to them. During retrieval, the decoder computes which nodes hold which shards and fetches directly.

### Decoding Process

All chunk structure is derivable from `original_len` and the fixed `MAX_CHUNK_SIZE` (16 MiB):

```
num_chunks = original_len.div_ceil(MAX_CHUNK_SIZE)
chunk_data_len(i) = min(MAX_CHUNK_SIZE, original_len - i * MAX_CHUNK_SIZE)
```

Decoding steps:

1. Read `blob_id`, `original_len`, `k`, `m`, `encoding_epoch` from chain
2. Derive chunk structure and shard→node mapping from epoch's node list
3. For each chunk, fetch at least `k` shards from their assigned nodes
4. RS-decode each chunk, truncate to `chunk_data_len` (strips RS padding)
5. Concatenate all chunks and truncate to `original_len`

Per-chunk truncation is essential for multi-chunk files — without it, padding bytes from chunk N would appear before chunk N+1's data, corrupting the output.

### Advantages (Storj Reference)

Following Storj's proven model:

1. **Storage Efficiency**: Configurable expansion (e.g., 1.53x with k=15, m=8) provides better durability than 3x replication
2. **Repair Bandwidth**: Only download `k` shards to repair, not the entire file
3. **Distributed Trust**: No single node holds enough data to reconstruct the file
4. **Flexible Recovery**: Any `k` of `k + m` shards suffice; no specific shards required

---

## Merkle Tree Construction

The Merkle tree is constructed in **three levels** to enable efficient proofs:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            MERKLE TREE STRUCTURE                             │
│                                                                              │
│  Level 0:  File Merkle Root (FMR) ─────────────── submitted on-chain        │
│                    │                                                         │
│  Level 1:  Chunk Merkle Roots (CMR₀, CMR₁, ..., CMRₙ₋₁)                     │
│                    │                                                         │
│  Level 2:  Piece Merkle Roots (PMR₀, PMR₁, ...) ── per chunk                │
│                    │                                                         │
│  Level 3:  Leaf hashes ─────────────────────────── per piece                │
│                    │                                                         │
│  Level 4:  Raw leaf data (encrypted bytes)                                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Level 1: Piece Merkle Trees (Leaves → PMR)

For each piece within an encoded chunk, compute the Piece Merkle Root from its leaves:

```
                    Piece Merkle Root (PMR)
                           │
              ┌────────────┴────────────┐
              │                         │
         Hash(H₀,H₁)               Hash(H₂,H₃)
              │                         │
        ┌─────┴─────┐             ┌─────┴─────┐
        │           │             │           │
       H₀=         H₁=           H₂=         H₃=
    Hash(L₀)    Hash(L₁)      Hash(L₂)    Hash(L₃)
        │           │             │           │
      Leaf₀      Leaf₁         Leaf₂      Leaf₃
   (encrypted) (encrypted)  (encrypted) (encrypted)
```

### Level 2: Chunk Merkle Trees (PMRs → CMR)

All Piece Merkle Roots within a chunk form the leaves of the Chunk Merkle Tree:

```
                         Chunk Merkle Root (CMR)
                                  │
                 ┌────────────────┴────────────────┐
                 │                                 │
          Hash(PMR₀,PMR₁)                   Hash(PMR₂,PMR₃)
                 │                                 │
           ┌─────┴─────┐                     ┌─────┴─────┐
           │           │                     │           │
         PMR₀        PMR₁                  PMR₂        PMR₃
       (Piece 0)   (Piece 1)             (Piece 2)   (Piece 3)
```

### Level 3: File Merkle Tree (CMRs → FMR)

All Chunk Merkle Roots form the leaves of the File Merkle Tree:

```
                         File Merkle Root (FMR)
                        ━━━━━━━━━━━━━━━━━━━━━━━
                          (Submitted On-Chain)
                                  │
                 ┌────────────────┴────────────────┐
                 │                                 │
          Hash(CMR₀,CMR₁)                   Hash(CMR₂,CMR₃)
                 │                                 │
           ┌─────┴─────┐                     ┌─────┴─────┐
           │           │                     │           │
         CMR₀        CMR₁                  CMR₂        CMR₃
       (Chunk 0)   (Chunk 1)             (Chunk 2)   (Chunk 3)
```

### Summary

| Level | Input | Output | Count |
|-------|-------|--------|-------|
| 1 | Leaf data (encrypted bytes) | Piece Merkle Root (PMR) | leaves_per_piece leaves → 1 PMR |
| 2 | PMRs for one chunk | Chunk Merkle Root (CMR) | pieces_per_chunk PMRs → 1 CMR |
| 3 | CMRs for all chunks | File Merkle Root (FMR) | n CMRs → 1 FMR |

---

## On-Chain Commitment

When a client uploads data, the following is submitted on-chain:

### Storage Commitment Structure

```solidity
struct StorageCommitment {
    bytes32 blobId;            // Content-addressed blob identifier
    bytes32 fileMerkleRoot;    // File Merkle Root (FMR)
    uint64 originalLen;        // Original encrypted data size in bytes
    uint16 dataShards;         // Number of data shards per chunk (k)
    uint16 parityShards;       // Number of parity shards per chunk (m)
    uint64 encodingEpoch;      // Epoch for node list lookup
    uint64 expiryBlock;        // Storage expiration block
    address owner;             // Data owner
}
```

Everything else is derivable from these fields:
- `num_chunks = originalLen.div_ceil(MAX_CHUNK_SIZE)`
- `chunk_data_len(i) = min(MAX_CHUNK_SIZE, originalLen - i * MAX_CHUNK_SIZE)`
- Shard→node mapping: deterministic rotation over the epoch's node list

No per-shard or per-chunk metadata is stored on-chain.

### Shard Assignment Overrides

During normal operation, shard→node mapping is computed deterministically. When a repair replaces a failed node, the override is recorded:

```solidity
struct ShardOverride {
    bytes32 blobId;            // Which blob
    uint32 chunkIndex;         // Which chunk
    uint16 shardIndex;         // Which shard within the chunk
    bytes32 newNodeId;         // Replacement node
}
```

Overrides are only created by the repair process. The decoder checks overrides before falling back to the deterministic mapping.

---

## Data Distribution

After encoding, the client distributes shards to storage nodes via Iroh P2P:

### Distribution Flow

```
┌────────────────────────────────────────────────────────────────────────┐
│                              CLIENT                                     │
│                                                                         │
│  1. Encode file → chunks → shards (see Erasure Encoding above)        │
│  2. Compute Merkle trees (FMR, CMRs, PMRs)                            │
│  3. Submit StorageCommitment on-chain                                  │
│  4. For each shard:                                                    │
│     a. Compute target node from deterministic assignment               │
│     b. Store in Iroh: key = blob_id/chunk_index/shard_index           │
│     c. Push shard to assigned node via Iroh P2P                       │
│     d. Include Merkle proof (shard → CMR → FMR) for verification      │
│                                                                         │
└────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                           STORAGE NODE                                  │
│                                                                         │
│  On receiving shard + proof:                                           │
│    1. Verify Merkle proof against on-chain FMR                         │
│    2. Verify shard index matches deterministic assignment for this node│
│    3. Store shard data in Iroh under blob_id/chunk_index/shard_index  │
│    4. Sign acknowledgment (BLS signature over shard hash)              │
│                                                                         │
└────────────────────────────────────────────────────────────────────────┘
```

---

## Storage Proof (PDP Challenge)

The Provable Data Possession (PDP) protocol, inspired by [Filecoin](https://spec.filecoin.io/), uses random challenges to verify storage.

### Challenge Protocol

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        CHALLENGE CONTRACT                                │
│                                                                          │
│  1. Generate random challenge using VRF:                                │
│     - seed = VRF(validator_sk, block_hash || commitment_id)             │
│     - chunk_idx = seed % num_chunks                                     │
│     - piece_idx = (seed >> 8) % pieces_per_chunk                        │
│     - leaf_idx  = (seed >> 16) % leaves_per_piece                       │
│                                                                          │
│  2. Emit Challenge event:                                               │
│     Challenge(commitmentId, chunk_idx, piece_idx, leaf_idx, deadline)   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                          STORAGE NODE                                    │
│                                                                          │
│  On receiving challenge:                                                │
│    1. Retrieve leaf data at (piece_idx, leaf_idx)                       │
│    2. Construct Merkle proof (3 levels):                                │
│       - Level 1: Leaf → Piece Merkle Root (PMR)                         │
│       - Level 2: PMR → Chunk Merkle Root (CMR)                          │
│       - Level 3: CMR → File Merkle Root (FMR)                           │
│    3. Submit proof to contract before deadline                          │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        CHALLENGE CONTRACT                                │
│                                                                          │
│  Verify proof:                                                          │
│    1. Hash the raw leaf data (encrypted bytes)                          │
│    2. Verify path: leaf_hash → PMR (using level-1 proof)                │
│    3. Verify path: PMR → CMR (using level-2 proof)                      │
│    4. Verify path: CMR → FMR (using level-3 proof)                      │
│    5. Compare FMR with on-chain commitment                              │
│    6. If valid, mark challenge as passed; else slash/penalize           │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Challenge Data Structure

```solidity
struct Challenge {
    bytes32 commitmentId;      // Which storage commitment
    uint32 chunkIndex;         // Challenged chunk (determines node)
    uint32 pieceIndex;         // Challenged piece within chunk
    uint32 leafIndex;          // Challenged leaf within piece
    uint64 deadline;           // Block number deadline for response
    bytes32 vrfProof;          // VRF proof for randomness verification
}

struct ChallengeProof {
    bytes leafData;            // Raw leaf bytes (encrypted)
    bytes32[] leafToPmrPath;   // Merkle path: leaf → PMR
    bytes32[] pmrToCmrPath;    // Merkle path: PMR → CMR
    bytes32[] cmrToFmrPath;    // Merkle path: CMR → FMR
}
```

### Challenge Timing

| Parameter | Description | Typical Value |
|-----------|-------------|---------------|
| Challenge interval | Time between challenges per commitment | 1 hour |
| Response deadline | Time allowed for proof submission | 10 minutes |
| Consecutive failures | Failures before slashing | 3 |

**Important**: Response deadline must be shorter than the time required to reconstruct a chunk from other nodes (to prevent lazy node attacks).

---

## Verification Process

### On-Chain Verification (Solidity)

```solidity
function verifyChallenge(
    bytes32 commitmentId,
    Challenge calldata challenge,
    ChallengeProof calldata proof
) external returns (bool) {
    StorageCommitment storage commitment = commitments[commitmentId];

    // 1. Verify VRF proof for challenge randomness
    require(verifyVRF(challenge.vrfProof, commitmentId), "Invalid VRF");

    // 2. Verify leaf data size
    require(proof.leafData.length == commitment.leafSize, "Invalid leaf size");

    // 3. Compute leaf hash
    bytes32 leafHash = keccak256(proof.leafData);

    // 4. Verify leaf → PMR path
    bytes32 computedPMR = computeMerkleRoot(
        leafHash,
        challenge.leafIndex,
        proof.leafToPmrPath
    );

    // 5. Verify PMR → CMR path
    bytes32 computedCMR = computeMerkleRoot(
        computedPMR,
        challenge.pieceIndex,
        proof.pmrToCmrPath
    );

    // 6. Verify CMR → FMR path
    bytes32 computedFMR = computeMerkleRoot(
        computedCMR,
        challenge.chunkIndex,
        proof.cmrToFmrPath
    );

    // 7. Compare with on-chain commitment
    require(computedFMR == commitment.fileMerkleRoot, "Invalid proof");

    return true;
}

function computeMerkleRoot(
    bytes32 leaf,
    uint256 index,
    bytes32[] calldata proof
) internal pure returns (bytes32) {
    bytes32 current = leaf;
    for (uint256 i = 0; i < proof.length; i++) {
        if (index % 2 == 0) {
            current = keccak256(abi.encodePacked(current, proof[i]));
        } else {
            current = keccak256(abi.encodePacked(proof[i], current));
        }
        index = index / 2;
    }
    return current;
}
```

### Proof Size Analysis

| Level | Proof Elements | Size per Element | Typical Total |
|-------|---------------|------------------|---------------|
| Leaf → PMR | log₂(leaves_per_piece) | 32 bytes | ~320 bytes (10 levels) |
| PMR → CMR | log₂(pieces_per_chunk) | 32 bytes | ~256 bytes (8 levels) |
| CMR → FMR | log₂(num_chunks) | 32 bytes | ~224 bytes (7 levels) |
| Leaf data | 1 | leaf_size | ~256 bytes |
| **Total** | | | **~1 KB** |

---

## Security Considerations

### Attack Vectors and Mitigations

| Attack | Description | Mitigation |
|--------|-------------|------------|
| **Data withholding** | Node claims to store data but doesn't | Random challenges require actual data |
| **Lazy node** | Node reconstructs data on-demand from peers instead of storing | Response deadline < reconstruction time |
| **Proof precomputation** | Precompute all possible proofs | Large leaf count makes this infeasible |
| **Collusion** | Nodes share data only for challenges | Unpredictable VRF-based challenge timing |
| **Sybil attack** | Single entity runs multiple nodes | Stake requirements, reputation system |
| **Grinding** | Manipulate random challenge selection | Verifiable Random Functions (VRF) |
| **Data exposure** | Storage nodes read user data | Client-side encryption (AES-256-GCM) |

### Lazy Node Attack - Detailed Mitigation

A malicious node could attempt to:
1. Not store its assigned chunk
2. When challenged, download `k` chunks from other nodes
3. Reconstruct its chunk using Reed-Solomon decoding
4. Respond to challenge with reconstructed data

**Mitigation**: Set response deadline such that:
```
deadline < time_to_download_k_chunks + time_to_decode
```

For a 16 MB chunk with k=64:
- Download 64 × 16 MB = 1 GB from network
- At 100 Mbps: ~80 seconds download time
- Decoding: ~5-10 seconds
- **Response deadline should be < 60 seconds**

### On-Chain Randomness

Challenge randomness must be unpredictable and unbiasable:

```
┌─────────────────────────────────────────────────────────────────┐
│                    VRF-BASED CHALLENGE SELECTION                 │
│                                                                  │
│  Input:                                                         │
│    - validator_secret_key (sk)                                  │
│    - block_hash (public, from recent finalized block)           │
│    - commitment_id (identifies the storage deal)                │
│                                                                  │
│  Process:                                                       │
│    1. vrf_output, vrf_proof = VRF_prove(sk, block_hash || id)   │
│    2. challenge_seed = hash(vrf_output)                         │
│    3. chunk_idx = challenge_seed % num_chunks                   │
│                                                                  │
│  Verification:                                                  │
│    - Anyone can verify VRF_verify(pk, block_hash || id, proof)  │
│    - Validator cannot predict/manipulate output                  │
│    - Block producer cannot bias (uses past finalized block)     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Cryptographic Assumptions

| Component | Algorithm | Security Level |
|-----------|-----------|----------------|
| Data encryption | AES-256-GCM | 256-bit |
| Hash function | Keccak-256 | 128-bit collision resistance |
| Randomness | VRF (e.g., ECVRF) | Unpredictable, verifiable |
| Erasure coding | Reed-Solomon GF(2^8) | Information-theoretic |

### Slashing Conditions

1. **Failed challenge**: Node fails to provide valid proof within deadline
2. **Invalid proof**: Merkle proof verification fails
3. **Repeated failures**: 3 consecutive failed challenges trigger slashing

### Grace Period for Transient Failures

To distinguish between "data lost" and "node temporarily unavailable":

```
Challenge States:
  - PENDING: Challenge issued, awaiting response
  - PASSED: Valid proof submitted
  - FAILED: Invalid proof or deadline missed
  - GRACE: First failure, node gets grace period

Slashing Logic:
  - 1st failure: Enter GRACE state, no slash
  - 2nd consecutive failure: Warning, reduced rewards
  - 3rd consecutive failure: Slash stake
```

---

## Economic Model

IPC Storage uses a dual payment model: **write payments** for storing data and **read payments** for retrieving data. Together these incentivize node operators to both persist data reliably and serve it on demand.

### Write Payment

Users pay upfront to store data on the network. The cost is determined by the size of the data, the duration of storage, and a per-MB price set by the network.

#### Pricing Formula

```
write_cost = price_per_mb × file_size_in_mb × duration
```

| Parameter | Description |
|-----------|-------------|
| `price_per_mb` | Network-determined price per megabyte per unit time |
| `file_size_in_mb` | Total size of the stored file in megabytes |
| `duration` | Storage duration (e.g., in epochs or blocks) |

The user locks this payment into the storage contract when submitting their on-chain storage commitment.

#### Node Reward Claims

Node operators earn rewards by proving they continue to store their assigned data. A node can claim accumulated rewards at any time by submitting a claim transaction, provided they have been passing storage challenges.

```
node_reward = data_size_stored_in_mb × price_per_mb × duration_since_last_claim
```

| Parameter | Description |
|-----------|-------------|
| `data_size_stored_in_mb` | Size of data this node is responsible for (its encoded chunk) |
| `price_per_mb` | Same per-MB rate from the storage commitment |
| `duration_since_last_claim` | Time elapsed since the node's last successful claim |

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         WRITE PAYMENT FLOW                                    │
│                                                                               │
│  1. User submits StorageCommitment on-chain                                  │
│     - Locks payment: price_per_mb × file_size_in_mb × duration               │
│                                                                               │
│  2. Storage nodes store their assigned chunks and respond to challenges      │
│                                                                               │
│  3. Node claims reward:                                                      │
│     - Contract verifies node has passed challenges since last claim          │
│     - Pays out: data_size_stored_in_mb × price_per_mb × elapsed_time        │
│     - Resets the node's last_claim timestamp                                 │
│                                                                               │
│  4. Repeat until storage duration expires                                    │
│                                                                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

Nodes that fail challenges forfeit rewards for the period in which they failed. Repeated failures lead to slashing (see [Security Considerations](#security-considerations)).

### Read Payment

Reading data uses an off-chain **payment ticket** model. Anyone who wants to retrieve a file issues a signed payment ticket to the node operator serving the requested chunk. The node operator can later redeem these tickets on-chain.

#### Pricing Formula

```
read_cost = read_price_per_mb × file_size_in_mb
```

| Parameter | Description |
|-----------|-------------|
| `read_price_per_mb` | Network or market-determined price per megabyte for reads |
| `file_size_in_mb` | Size of the data being read in megabytes |

#### Payment Ticket Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          READ PAYMENT FLOW                                    │
│                                                                               │
│  1. Reader requests chunk from a storage node                                │
│                                                                               │
│  2. Reader issues a signed payment ticket:                                   │
│     - Ticket contains: reader address, node address, chunk id,               │
│       amount (read_price_per_mb × chunk_size_in_mb),                         │
│       nonce, signature                                                       │
│                                                                               │
│  3. Node validates the ticket signature and serves the chunk                 │
│                                                                               │
│  4. Node accumulates tickets and redeems them on-chain in batches            │
│     - Contract verifies signatures and transfers payment                     │
│     - Tickets are marked as spent to prevent double-redemption               │
│                                                                               │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Payment Ticket Structure

```solidity
struct ReadPaymentTicket {
    address reader;            // Who is paying for the read
    address nodeOperator;      // Who is being paid
    bytes32 chunkId;           // Identifier of the chunk being read
    uint256 amount;            // Payment amount (read_price_per_mb × size)
    uint256 nonce;             // Unique nonce to prevent replay
    bytes signature;           // Reader's signature over the ticket
}
```

Batch redemption lets node operators amortize on-chain transaction costs by submitting multiple tickets in a single transaction.

### Summary

| Payment Type | Payer | Recipient | Pricing | Settlement |
|-------------|-------|-----------|---------|------------|
| **Write** | Data owner | Storage nodes | `price_per_mb × size × duration` | On-chain claims (challenge-gated) |
| **Read** | Data reader | Serving node | `read_price_per_mb × size` | Off-chain tickets, redeemed on-chain |

---

## Repair Process

When a storage node fails, the system must reconstruct and redistribute the lost chunk to maintain data availability. IPC Storage uses a **guardian-based repair model** where trusted parties monitor health and perform repairs on behalf of data owners.

### Repair Model Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           REPAIR RESPONSIBILITY                              │
│                                                                              │
│  Data Owner delegates monitoring to a trusted Guardian                      │
│                                                                              │
│    ┌──────────┐         delegates          ┌──────────────┐                 │
│    │  Client  │ ─────────────────────────▶ │   Guardian   │                 │
│    │  (Owner) │                            │  (Trusted)   │                 │
│    └──────────┘                            └──────────────┘                 │
│                                                   │                          │
│                                                   │ monitors challenges      │
│                                                   │ initiates repair         │
│                                                   │ selects new node         │
│                                                   ▼                          │
│                                            ┌──────────────┐                 │
│                                            │   Storage    │                 │
│                                            │    Nodes     │                 │
│                                            └──────────────┘                 │
│                                                                              │
│  No Guardian Delegated = Data at risk if client is offline                  │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Data Structures

```solidity
// Trusted guardian registration (protocol-level)
struct Guardian {
    address guardianAddress;
    bytes32 publicKey;
    uint256 stakedCollateral;      // Skin in the game
    bool isActive;
}

// Per-commitment delegation (client delegates to guardian)
struct RepairDelegation {
    bytes32 commitmentId;
    address guardian;              // Must be in trusted guardian set
    uint64 expiresAt;
}

// Repair event record
struct RepairRecord {
    bytes32 commitmentId;
    uint32 failedChunkIndex;
    address failedNode;
    address newNode;
    address guardian;
    uint256 slashedAmount;
    uint256 guardianReward;        // Portion of slash to guardian
    uint256 clientReimbursement;   // Portion of slash to client
    uint64 timestamp;
}
```

### Repair Trigger Conditions

Repair is triggered when ALL conditions are met:

1. Node fails **N consecutive challenges** (e.g., N = 3)
2. A valid **RepairDelegation** exists for the commitment
3. Guardian is **active** in the trusted guardian set

```
Challenge History:
  Challenge 1: PASSED  ─┐
  Challenge 2: PASSED   │ Counter resets on pass
  Challenge 3: FAILED  ─┘ consecutive_failures = 1
  Challenge 4: FAILED     consecutive_failures = 2
  Challenge 5: FAILED     consecutive_failures = 3 → TRIGGER REPAIR
```

### Repair Execution Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        GUARDIAN REPAIR EXECUTION                             │
│                                                                              │
│  1. DETECT                                                                  │
│     Guardian monitors ChallengeContract events on-chain                     │
│     Detects: Node X failed 3rd consecutive challenge for Commitment C       │
│                                                                              │
│  2. INITIATE ON-CHAIN                                                       │
│     Guardian calls: RepairContract.initiateRepair(commitmentId, chunkIndex) │
│                                                                              │
│     Contract verifies:                                                      │
│       - Guardian is delegated for this commitment                           │
│       - Node has failed required consecutive challenges                     │
│                                                                              │
│     Contract actions:                                                       │
│       - Marks chunk as REPAIRING (prevents duplicate repairs)               │
│       - Slashes failed node's stake                                         │
│       - Emits RepairInitiated event                                         │
│                                                                              │
│  3. RECONSTRUCT OFF-CHAIN                                                   │
│     Guardian performs reconstruction for each lost shard:                   │
│       a. Derive which shards the failed node held (deterministic mapping)  │
│       b. For each affected chunk, fetch k shards from other nodes          │
│       c. RS-decode to reconstruct the chunk                                │
│       d. RS-encode to regenerate the missing shard                         │
│       e. Verify reconstructed shard matches on-chain CMR                   │
│                                                                              │
│  4. SELECT NEW NODE                                                         │
│     Guardian selects replacement node (full discretion):                    │
│       - From available node pool with sufficient stake                      │
│       - Excluding nodes already storing shards for this commitment         │
│                                                                              │
│  5. DISTRIBUTE TO NEW NODE                                                  │
│     Guardian sends to new node:                                             │
│       - Reconstructed shard data (encrypted)                                │
│       - Merkle proof (shard → CMR → FMR)                                   │
│     New node validates and stores under blob_id/chunk_index/shard_index    │
│                                                                              │
│  6. COMPLETE ON-CHAIN                                                       │
│     Guardian calls: RepairContract.completeRepair(                          │
│         commitmentId, chunkIndex, shardIndex, newNodeId, nodeSignature     │
│     )                                                                       │
│                                                                              │
│     Contract verifies:                                                      │
│       - Repair was initiated by this guardian                               │
│       - New node signed acknowledgment of storage                           │
│       - Within repair deadline                                              │
│                                                                              │
│     Contract actions:                                                       │
│       - Records ShardOverride: (blobId, chunkIndex, shardIndex) → newNode  │
│       - Distributes slashed stake to guardian + client                      │
│       - Marks shard as HEALTHY                                              │
│       - Emits RepairCompleted event                                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Economic Flow

The repair economic model is simple: **slashed stake compensates guardian and client**.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         REPAIR ECONOMICS                                     │
│                                                                              │
│  INPUT:                                                                     │
│    Slashed stake from failed node: S                                        │
│                                                                              │
│  DISTRIBUTION:                                                              │
│    Guardian reward:      S × guardian_pct     (e.g., 50%)                   │
│    Client reimbursement: S × (1 - guardian_pct)                             │
│                                                                              │
│  NEW NODE PAYMENT:                                                          │
│    New node takes over the storage deal from failed node                    │
│    Receives ongoing storage payments from original commitment               │
│                                                                              │
│  EXAMPLE (S = 100 tokens, guardian_pct = 50%):                              │
│    Guardian receives: 50 tokens (covers their bandwidth + profit)           │
│    Client receives:   50 tokens (partial compensation for risk)             │
│    New node:          Inherits storage deal, earns future payments          │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Batch Repairs

When multiple chunks fail (e.g., correlated node failures), repairs can be batched:

```solidity
function initiateBatchRepair(
    bytes32 commitmentId,
    uint32[] calldata chunkIndices
) external onlyDelegatedGuardian(commitmentId) {
    for (uint i = 0; i < chunkIndices.length; i++) {
        _initiateRepair(commitmentId, chunkIndices[i]);
    }
}

function completeBatchRepair(
    bytes32 commitmentId,
    uint32[] calldata chunkIndices,
    bytes32[] calldata newNodeIds,
    bytes[] calldata nodeSignatures
) external {
    require(chunkIndices.length == newNodeIds.length, "Length mismatch");
    for (uint i = 0; i < chunkIndices.length; i++) {
        _completeRepair(commitmentId, chunkIndices[i], newNodeIds[i], nodeSignatures[i]);
    }
}
```

### Repair Deadline

Guardian must complete repair within a deadline to prevent indefinite REPAIRING states:

```
repair_deadline = initiation_block + MAX_REPAIR_BLOCKS (e.g., 1 hour)

If deadline exceeded:
  - Any guardian can call: RepairContract.expireRepair(commitmentId, chunkIndex)
  - Chunk status: REPAIRING → FAILED
  - Original guardian forfeits the repair opportunity
  - Another delegated guardian (or client) can initiate fresh repair
```

### Chunk States

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           CHUNK STATE MACHINE                                │
│                                                                              │
│                    ┌───────────────────────────────────┐                    │
│                    │                                   │                    │
│                    ▼                                   │                    │
│  ┌─────────┐   challenge   ┌─────────┐   N failures   ┌┴────────┐          │
│  │ HEALTHY │ ────────────▶ │ AT_RISK │ ─────────────▶ │ SLASHED │          │
│  └─────────┘    failed     └─────────┘                └─────────┘          │
│       ▲                         │                          │               │
│       │                         │ challenge                │               │
│       │                         │ passed                   ▼               │
│       │                         │                    ┌───────────┐         │
│       │                         └──────────────────▶ │ REPAIRING │         │
│       │                                              └───────────┘         │
│       │                                                    │               │
│       │              repair completed                      │               │
│       └────────────────────────────────────────────────────┘               │
│                                                                              │
│       repair deadline exceeded                                              │
│       ┌───────────┐                                                         │
│       │  FAILED   │  (requires fresh repair initiation)                    │
│       └───────────┘                                                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Guardian Trust Model

Guardians are a **trusted, permissioned set** registered at the protocol level:

| Requirement | Description |
|-------------|-------------|
| **Staked Collateral** | Guardians must stake tokens to participate |
| **Registration** | Protocol governance approves guardian additions |
| **Slashing** | Guardians can be slashed for malicious behavior |
| **Reputation** | Track record of successful repairs visible on-chain |

This trusted model avoids DDoS and sybil attack vectors on the repair mechanism.

### No Guardian Fallback

If a client has **no guardian delegated**:

- Client must monitor challenges themselves
- Client must perform repairs themselves (same flow, but client is the actor)
- If client is offline and node fails → **data becomes at-risk**
- Data can still be recovered as long as k chunks remain available
- Once fewer than k chunks available → **data is lost**

This is the client's responsibility. The protocol does not provide automatic fallback.

---

## References

1. **Storj Whitepaper**: [https://storj.io/storj.pdf](https://storj.io/storj.pdf)
   - Reed-Solomon erasure coding parameters
   - Distributed storage architecture

2. **Filecoin Spec - Proof of Data Possession**: [https://spec.filecoin.io/](https://spec.filecoin.io/)
   - PDP challenge-response protocol
   - Merkle tree construction for storage proofs

3. **Reed-Solomon Error Correction**: [https://en.wikipedia.org/wiki/Reed–Solomon_error_correction](https://en.wikipedia.org/wiki/Reed–Solomon_error_correction)
   - Mathematical foundations of erasure coding

4. **Merkle Trees**: [https://en.wikipedia.org/wiki/Merkle_tree](https://en.wikipedia.org/wiki/Merkle_tree)
   - Hash tree structure and verification

5. **ECVRF (Verifiable Random Functions)**: [https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-vrf](https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-vrf)
   - VRF specification for unpredictable randomness

---

## Appendix: Example Calculation

### Parameters for 1 GB File

```
Original data size:     1 GB = 1,073,741,824 bytes
After encryption:       ~1 GB (AES-GCM adds minimal overhead)

Max chunk size:         16 MiB = 16,777,216 bytes
Num chunks:             ceil(1 GB / 16 MiB) = 64

Data shards (k):        15
Parity shards (m):      8
Total shards per chunk: 23

Total shards:           64 chunks × 23 shards = 1,472 shards
Storage nodes:          30
Shards per node:        ~49 (1,472 / 30)

On-chain commitment:
  - blob_id:            32 bytes
  - original_len:       8 bytes
  - data_shards:        2 bytes
  - parity_shards:      2 bytes
  - encoding_epoch:     8 bytes
  Total:                52 bytes (+ Merkle root, expiry, owner)

Storage expansion:      1.53× (23/15)
Recovery threshold:     Any 15 of 23 shards per chunk
Shard→node mapping:     Deterministic (no storage cost)

Challenge response deadline: 30 seconds
  (Reconstruction time at 100 Mbps: ~90 seconds)
```
