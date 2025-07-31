// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use anyhow::anyhow;
use ethers::prelude::*;

use tendermint::block::signed_header::SignedHeader as TendermintSignedHeader;
use tendermint::block::commit_sig::CommitSig as TendermintCommitSig;
use tendermint::time::Time as TendermintTime;
use tendermint::account::Id;
use tendermint::PublicKey;

#[derive(Debug, Clone, EthAbiType, EthAbiCodec)]
pub struct SignedHeader {
    pub header: LightHeader,
    pub commit: Commit,
}

#[derive(Debug, Clone, EthAbiType, EthAbiCodec)]
pub struct LightHeader {
    pub version: Consensus,
    pub chain_id: String,
    pub height: i64,
    pub time: Timestamp,
    pub last_block_id: BlockId,
    pub last_commit_hash: Bytes,
    pub data_hash: Bytes,
    pub validators_hash: Bytes,
    pub next_validators_hash: Bytes,
    pub consensus_hash: Bytes,
    pub app_hash: Bytes,
    pub last_results_hash: Bytes,
    pub evidence_hash: Bytes,
    pub proposer_address: Bytes,
}

#[derive(Debug, Clone, EthAbiType, EthAbiCodec)]
pub struct Commit {
    pub height: i64,
    pub round: i32,
    pub block_id: BlockId,
    pub signatures: Vec<CommitSig>,
}

#[derive(Debug, Clone, EthAbiType, EthAbiCodec)]
pub struct Consensus {
    pub block: u64,
    pub app: u64,
}

#[derive(Debug, Clone, EthAbiType, EthAbiCodec)]
pub struct Timestamp {
    pub seconds: u64,
    pub nanos: u32,
}

#[derive(Debug, Clone, EthAbiType, EthAbiCodec)]
pub struct BlockId {
    pub hash: [u8; 32],  // assuming bytes32
    pub part_set_header: PartSetHeader,
}

#[derive(Debug, Clone, EthAbiType, EthAbiCodec)]
pub struct PartSetHeader {
    pub total: u32,
    pub hash: [u8; 32],
}

#[derive(Debug, Clone, EthAbiType, EthAbiCodec)]
pub struct CommitSig {
    pub block_id_flag: u8,  // enum
    pub validator_address: Bytes,
    pub timestamp: Timestamp,
    pub signature: Bytes,
}

impl From<TendermintSignedHeader> for SignedHeader {
    fn from(tm_header: TendermintSignedHeader) -> Self {
        SignedHeader {
            header: tm_header.header.into(),
            commit: tm_header.commit.into(),
        }
    }
}

impl From<tendermint::block::Header> for LightHeader {
    fn from(tm_header: tendermint::block::Header) -> Self {
        LightHeader {
            version: Consensus {
                block: tm_header.version.block,
                app: tm_header.version.app,
            },
            chain_id: tm_header.chain_id.to_string(),
            height: tm_header.height.value() as i64,
            time: Timestamp::from(tm_header.time),
            last_block_id: tm_header.last_block_id.map(|id| id.into()).unwrap_or_default(),
            last_commit_hash: Bytes::from(tm_header.last_commit_hash.unwrap_or_default().as_bytes().to_vec()),
            data_hash: Bytes::from(tm_header.data_hash.unwrap_or_default().as_bytes().to_vec()),
            validators_hash: Bytes::from(tm_header.validators_hash.as_bytes().to_vec()),
            next_validators_hash: Bytes::from(tm_header.next_validators_hash.as_bytes().to_vec()),
            consensus_hash: Bytes::from(tm_header.consensus_hash.as_bytes().to_vec()),
            app_hash: Bytes::from(tm_header.app_hash.as_bytes().to_vec()),
            last_results_hash: Bytes::from(tm_header.last_results_hash.unwrap_or_default().as_bytes().to_vec()),
            evidence_hash: Bytes::from(tm_header.evidence_hash.unwrap_or_default().as_bytes().to_vec()),
            proposer_address: Bytes::from(tm_header.proposer_address.as_bytes().to_vec()),
        }
    }
}

impl From<tendermint::block::Commit> for Commit {
    fn from(tm_commit: tendermint::block::Commit) -> Self {
        Commit {
            height: tm_commit.height.value() as i64,
            round: tm_commit.round.value() as i32,
            block_id: tm_commit.block_id.into(),
            signatures: tm_commit.signatures.into_iter().map(|sig| sig.into()).collect(),
        }
    }
}

impl From<tendermint::block::Id> for BlockId {
    fn from(tm_block_id: tendermint::block::Id) -> Self {
        BlockId {
            hash: tm_block_id.hash.as_bytes().try_into().unwrap_or([0u8; 32]),
            part_set_header: tm_block_id.part_set_header.into(),
        }
    }
}

impl From<tendermint::block::parts::Header> for PartSetHeader {
    fn from(tm_part_header: tendermint::block::parts::Header) -> Self {
        PartSetHeader {
            total: tm_part_header.total,
            hash: tm_part_header.hash.as_bytes().try_into().unwrap_or([0u8; 32]),
        }
    }
}

impl From<TendermintCommitSig> for CommitSig {
    fn from(tm_sig: TendermintCommitSig) -> Self {
        match tm_sig {
            TendermintCommitSig::BlockIdFlagAbsent {} => CommitSig {
                block_id_flag: 1, // BLOCK_ID_FLAG_ABSENT
                validator_address: Bytes::new(),
                timestamp: Timestamp { seconds: 0, nanos: 0 },
                signature: Bytes::default(),
            },
            TendermintCommitSig::BlockIdFlagCommit {
                validator_address,
                timestamp,
                signature,
            } => CommitSig {
                block_id_flag: 2, // BLOCK_ID_FLAG_COMMIT
                validator_address: Bytes::from(validator_address.as_bytes().to_vec()),
                timestamp: Timestamp::from(timestamp),
                signature: signature.map(|s| Bytes::from(s.as_bytes().to_vec())).unwrap_or_default(),
            },
            TendermintCommitSig::BlockIdFlagNil { validator_address, timestamp, signature } => CommitSig {
                block_id_flag: 3, // BLOCK_ID_FLAG_NIL
                validator_address: Bytes::from(validator_address.as_bytes().to_vec()),
                timestamp: Timestamp::from(timestamp),
                signature: signature.map(|s| Bytes::from(s.as_bytes().to_vec())).unwrap_or_default(),
            },
        }
    }
}

// Default implementations for missing data
impl Default for BlockId {
    fn default() -> Self {
        Self {
            hash: [0u8; 32],
            part_set_header: PartSetHeader::default(),
        }
    }
}

impl Default for PartSetHeader {
    fn default() -> Self {
        Self {
            total: 0,
            hash: [0u8; 32],
        }
    }
}

impl From<TendermintTime> for Timestamp {
    fn from(time: TendermintTime) -> Self {
        let nanos_total = time.unix_timestamp_nanos();
        Timestamp {
            seconds: (nanos_total / 1_000_000_000) as u64,
            nanos: (nanos_total % 1_000_000_000) as u32,
        }
    }
}


impl SignedHeader {
    /// Order the commitment payload against the public keys, i.e. using public key to cometbft account
    /// id to order the validators in pre commit cert.
    pub fn order_commit_against<'a, I: Iterator<Item = &'a [u8]>>(&mut self, pubkeys: I) -> anyhow::Result<()> {
        let account_to_index = pubkeys
            .into_iter()
            .map(pubkey_to_account_id)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .enumerate()
            .map(|(index, id)| (id.into(), index))
            .collect::<HashMap<Vec<u8>, usize>>();

        let f = |a: &[u8]| {
            account_to_index.get(a).cloned().unwrap_or(usize::MAX)
        };

        self.commit
            .signatures
            .sort_by(|a, b| {
                let a_index = f(a.validator_address.as_ref());
                let b_index = f(b.validator_address.as_ref());
                a_index.cmp(&b_index)
            });

        Ok(())
    }
}

fn pubkey_to_account_id(uncompressed: &[u8]) -> anyhow::Result<Id> {
    let compressed = uncompressed_to_compressed(uncompressed)?;
    let pubkey = PublicKey::from_raw_secp256k1(&compressed).ok_or_else(|| anyhow!("could not create secp256k1 pubkey"))?;
    Ok(Id::from(pubkey))
}

fn uncompressed_to_compressed(uncompressed: &[u8]) -> anyhow::Result<Vec<u8>> {
    if uncompressed.len() != 65 || uncompressed[0] != 0x04 {
        return Err(anyhow!("Invalid uncompressed pubkey"));
    }

    let mut compressed = Vec::with_capacity(33);
    let y_last_byte = uncompressed[64];
    compressed.push(if y_last_byte % 2 == 0 { 0x02 } else { 0x03 });
    compressed.extend_from_slice(&uncompressed[1..33]); // x coordinate

    Ok(compressed)
}