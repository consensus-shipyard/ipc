// Copyright 2022-2025 Protocol Labs
// SPDX-License-Identifier: MIT

use anyhow::anyhow;
use ethers::prelude::*;
use std::collections::HashMap;

use tendermint::account::Id;
use tendermint::block::commit_sig::CommitSig as TendermintCommitSig;
use tendermint::block::signed_header::SignedHeader as TendermintSignedHeader;
use tendermint::time::Time as TendermintTime;
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
    pub height: i64, // int64 in Solidity
    pub time: Timestamp,
    pub last_block_id: BlockId,
    pub last_commit_hash: Bytes,     // bytes in Solidity (dynamic)
    pub data_hash: Bytes,            // bytes in Solidity (dynamic)
    pub validators_hash: Bytes,      // bytes in Solidity (dynamic)
    pub next_validators_hash: Bytes, // bytes in Solidity (dynamic)
    pub consensus_hash: Bytes,       // bytes in Solidity (dynamic)
    pub app_hash: Bytes,             // bytes in Solidity (dynamic)
    pub last_results_hash: Bytes,    // bytes in Solidity (dynamic)
    pub evidence_hash: Bytes,        // bytes in Solidity (dynamic)
    pub proposer_address: Bytes,     // bytes in Solidity (dynamic)
}

#[derive(Debug, Clone, EthAbiType, EthAbiCodec)]
pub struct Consensus {
    pub block: u64, // uint64 in Solidity
    pub app: u64,   // uint64 in Solidity
}

#[derive(Debug, Clone, EthAbiType, EthAbiCodec)]
pub struct Timestamp {
    pub seconds: i64, // int64 in Solidity (not uint64!)
    pub nanos: i32,   // int32 in Solidity (not uint32!)
}

#[derive(Debug, Clone, EthAbiType, EthAbiCodec)]
pub struct BlockId {
    pub hash: Bytes, // bytes in Solidity (dynamic, not bytes32!)
    pub part_set_header: PartSetHeader,
}

#[derive(Debug, Clone, EthAbiType, EthAbiCodec)]
pub struct PartSetHeader {
    pub total: u32,  // uint32 in Solidity
    pub hash: Bytes, // bytes in Solidity (dynamic, not bytes32!)
}

#[derive(Debug, Clone, EthAbiType, EthAbiCodec)]
pub struct Commit {
    pub height: i64, // int64 in Solidity
    pub round: i32,  // int32 in Solidity
    pub block_id: BlockId,
    pub signatures: Vec<CommitSig>,
}

#[derive(Debug, Clone, EthAbiType, EthAbiCodec)]
pub struct CommitSig {
    pub block_id_flag: u8,        // BlockIDFlag enum in Solidity (uint32)
    pub validator_address: Bytes, // bytes in Solidity
    pub timestamp: Timestamp,
    pub signature: Bytes, // bytes in Solidity
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
            last_block_id: tm_header
                .last_block_id
                .map(|id| id.into())
                .unwrap_or_default(),
            last_commit_hash: Bytes::from(
                tm_header
                    .last_commit_hash
                    .unwrap_or_default()
                    .as_bytes()
                    .to_vec(),
            ),
            data_hash: Bytes::from(tm_header.data_hash.unwrap_or_default().as_bytes().to_vec()),
            validators_hash: Bytes::from(tm_header.validators_hash.as_bytes().to_vec()),
            next_validators_hash: Bytes::from(tm_header.next_validators_hash.as_bytes().to_vec()),
            consensus_hash: Bytes::from(tm_header.consensus_hash.as_bytes().to_vec()),
            app_hash: Bytes::from(tm_header.app_hash.as_bytes().to_vec()),
            last_results_hash: Bytes::from(
                tm_header
                    .last_results_hash
                    .unwrap_or_default()
                    .as_bytes()
                    .to_vec(),
            ),
            evidence_hash: Bytes::from(
                tm_header
                    .evidence_hash
                    .unwrap_or_default()
                    .as_bytes()
                    .to_vec(),
            ),
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
            signatures: tm_commit
                .signatures
                .into_iter()
                .map(|sig| sig.into())
                .collect(),
        }
    }
}

impl From<tendermint::block::Id> for BlockId {
    fn from(tm_block_id: tendermint::block::Id) -> Self {
        BlockId {
            hash: Bytes::from(tm_block_id.hash.as_bytes().to_vec()),
            part_set_header: tm_block_id.part_set_header.into(),
        }
    }
}

impl From<tendermint::block::parts::Header> for PartSetHeader {
    fn from(tm_part_header: tendermint::block::parts::Header) -> Self {
        PartSetHeader {
            total: tm_part_header.total,
            hash: Bytes::from(tm_part_header.hash.as_bytes().to_vec()),
        }
    }
}

impl From<TendermintCommitSig> for CommitSig {
    fn from(tm_sig: TendermintCommitSig) -> Self {
        match tm_sig {
            TendermintCommitSig::BlockIdFlagAbsent {} => CommitSig {
                block_id_flag: 1, // BLOCK_ID_FLAG_ABSENT (matches enum in Solidity)
                validator_address: Bytes::new(),
                timestamp: Timestamp {
                    seconds: 0,
                    nanos: 0,
                },
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
                signature: signature
                    .map(|s| Bytes::from(s.as_bytes().to_vec()))
                    .unwrap_or_default(),
            },
            TendermintCommitSig::BlockIdFlagNil {
                validator_address,
                timestamp,
                signature,
            } => CommitSig {
                block_id_flag: 3, // BLOCK_ID_FLAG_NIL
                validator_address: Bytes::from(validator_address.as_bytes().to_vec()),
                timestamp: Timestamp::from(timestamp),
                signature: signature
                    .map(|s| Bytes::from(s.as_bytes().to_vec()))
                    .unwrap_or_default(),
            },
        }
    }
}

// Default implementations for missing data
impl Default for BlockId {
    fn default() -> Self {
        Self {
            hash: Bytes::new(),
            part_set_header: PartSetHeader::default(),
        }
    }
}

impl Default for PartSetHeader {
    fn default() -> Self {
        Self {
            total: 0,
            hash: Bytes::new(),
        }
    }
}

impl From<TendermintTime> for Timestamp {
    fn from(time: TendermintTime) -> Self {
        // IMPORTANT: Solidity expects int64 seconds and int32 nanos
        let nanos_total = time.unix_timestamp_nanos();
        Timestamp {
            seconds: (nanos_total / 1_000_000_000) as i64, // int64
            nanos: (nanos_total % 1_000_000_000) as i32,   // int32
        }
    }
}

impl SignedHeader {
    /// Order the commitment payload against the public keys, i.e. using public key to cometbft account
    /// id to order the validators in pre commit cert.
    pub fn order_commit_against<'a, I: Iterator<Item = &'a [u8]>>(
        &mut self,
        pubkeys: I,
    ) -> anyhow::Result<()> {
        let account_to_index = pubkeys
            .into_iter()
            .map(pubkey_to_account_id)
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .enumerate()
            .map(|(index, id)| (id.into(), index))
            .collect::<HashMap<Vec<u8>, usize>>();

        // check the validators are included in the public keys
        if let Some(v) = self
            .commit
            .signatures
            .iter()
            .find(|s| !account_to_index.contains_key(s.validator_address.as_ref()))
        {
            return Err(anyhow!(
                "validator address not found in the public keys: {:}",
                hex::encode(v.validator_address.as_ref())
            ));
        }

        // safe to unwrap as validator must be found
        let f = |a: &[u8]| account_to_index.get(a).cloned().unwrap();

        self.commit.signatures.sort_by(|a, b| {
            let a_index = f(a.validator_address.as_ref());
            let b_index = f(b.validator_address.as_ref());
            a_index.cmp(&b_index)
        });

        Ok(())
    }
}

fn pubkey_to_account_id(uncompressed: &[u8]) -> anyhow::Result<Id> {
    let compressed = uncompressed_to_compressed(uncompressed)?;
    let pubkey = PublicKey::from_raw_secp256k1(&compressed)
        .ok_or_else(|| anyhow!("could not create secp256k1 pubkey"))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use ethers::abi::Tokenizable;

    fn create_example_signed_header() -> SignedHeader {
        SignedHeader {
            header: LightHeader {
                version: Consensus {
                    block: 11,
                    app: 0,
                },
                chain_id: "1671263715227509".to_string(),
                height: 10,
                time: Timestamp {
                    seconds: 1754924475,
                    nanos: 753738680,
                },
                last_block_id: BlockId {
                    hash: Bytes::from(ethers::utils::hex::decode("cdf54989b2af7335f147497cce3462143805ace148e54e87a2478070da92c4ed").unwrap()),
                    part_set_header: PartSetHeader {
                        total: 1,
                        hash: Bytes::from(ethers::utils::hex::decode("76434337d10b011ab9d18dd8f0c9ccc58a7ccf069e0ace8c07772569489a489e").unwrap()),
                    },
                },
                last_commit_hash: Bytes::from(ethers::utils::hex::decode("8da7c3c5ccacf3277b63b0ecbc1897fd57e3f17372c3aeaa7eb366b69855d9a7").unwrap()),
                data_hash: Bytes::from(ethers::utils::hex::decode("b8b9fe4ec01144702ae02277199c9f5de07e26614bb378fa434cb3410d847551").unwrap()),
                validators_hash: Bytes::from(ethers::utils::hex::decode("6aa2b4fb8892eb46abe6d5b9b5e7e86a749d1fbd8e355e3a6b5f5426ef3e6790").unwrap()),
                next_validators_hash: Bytes::from(ethers::utils::hex::decode("6aa2b4fb8892eb46abe6d5b9b5e7e86a749d1fbd8e355e3a6b5f5426ef3e6790").unwrap()),
                consensus_hash: Bytes::from(ethers::utils::hex::decode("895734b58a6cb41a56bfe448f135d54fa01dc948164ee7e409960f0d8958d42c").unwrap()),
                app_hash: Bytes::from(ethers::utils::hex::decode("fcbeb04f3c0175e06b8ef9d731476e88f2d37b98bca65b7e983356c92c9c53e9").unwrap()),
                last_results_hash: Bytes::from(ethers::utils::hex::decode("7e23c5dbd335ecce8cad567dc6bf69373995bd718d63b562b46126a3d6574b95").unwrap()),
                evidence_hash: Bytes::from(ethers::utils::hex::decode("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855").unwrap()),
                proposer_address: Bytes::from(ethers::utils::hex::decode("905b1c0098887ea9033946de1eab5427c97a82ad").unwrap()),
            },
            commit: Commit {
                height: 10,
                round: 0,
                block_id: BlockId {
                    hash: Bytes::from(ethers::utils::hex::decode("910035b9ff5ddd3f2434d198d718cbe1c97b3a484b6799b4106f072070a046ce").unwrap()),
                    part_set_header: PartSetHeader {
                        total: 1,
                        hash: Bytes::from(ethers::utils::hex::decode("648d2cb39485249faf92acb13db138eac1212579c433a8a2aa6e85e05d69f2bd").unwrap()),
                    },
                },
                signatures: vec![CommitSig {
                    block_id_flag: 2, // BLOCK_ID_FLAG_COMMIT
                    validator_address: Bytes::from(ethers::utils::hex::decode("905b1c0098887ea9033946de1eab5427c97a82ad").unwrap()),
                    timestamp: Timestamp {
                        seconds: 1754924476,
                        nanos: 811159237,
                    },
                    signature: Bytes::from(ethers::utils::hex::decode("284f7f673bf73a515a8829dd29edc8671094e62d94db5cfa869bb62b4e8b6eff51c44f2662fb6fef1e37239d9a7d14707971feeddd1e9ba87c2ca5bafc1b6d9e").unwrap()),
                }],
            },
        }
    }

    #[test]
    fn test_abi_encoding() {
        let header = create_example_signed_header();

        // Test that it encodes without panic
        let tokens = vec![header.into_token()];
        let encoded = ethers::abi::encode(&tokens);
        let expected = "0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000005c0000000000000000000000000000000000000000000000000000000000000000b00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000689a05bb000000000000000000000000000000000000000000000000000000002ced23b800000000000000000000000000000000000000000000000000000000000002400000000000000000000000000000000000000000000000000000000000000340000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000000000000000000000000000000003c000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000440000000000000000000000000000000000000000000000000000000000000048000000000000000000000000000000000000000000000000000000000000004c00000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000054000000000000000000000000000000000000000000000000000000000000000103136373132363337313532323735303900000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000020cdf54989b2af7335f147497cce3462143805ace148e54e87a2478070da92c4ed00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000002076434337d10b011ab9d18dd8f0c9ccc58a7ccf069e0ace8c07772569489a489e00000000000000000000000000000000000000000000000000000000000000208da7c3c5ccacf3277b63b0ecbc1897fd57e3f17372c3aeaa7eb366b69855d9a70000000000000000000000000000000000000000000000000000000000000020b8b9fe4ec01144702ae02277199c9f5de07e26614bb378fa434cb3410d84755100000000000000000000000000000000000000000000000000000000000000206aa2b4fb8892eb46abe6d5b9b5e7e86a749d1fbd8e355e3a6b5f5426ef3e679000000000000000000000000000000000000000000000000000000000000000206aa2b4fb8892eb46abe6d5b9b5e7e86a749d1fbd8e355e3a6b5f5426ef3e67900000000000000000000000000000000000000000000000000000000000000020895734b58a6cb41a56bfe448f135d54fa01dc948164ee7e409960f0d8958d42c0000000000000000000000000000000000000000000000000000000000000020fcbeb04f3c0175e06b8ef9d731476e88f2d37b98bca65b7e983356c92c9c53e900000000000000000000000000000000000000000000000000000000000000207e23c5dbd335ecce8cad567dc6bf69373995bd718d63b562b46126a3d6574b950000000000000000000000000000000000000000000000000000000000000020e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b8550000000000000000000000000000000000000000000000000000000000000014905b1c0098887ea9033946de1eab5427c97a82ad000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000180000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000020910035b9ff5ddd3f2434d198d718cbe1c97b3a484b6799b4106f072070a046ce000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000020648d2cb39485249faf92acb13db138eac1212579c433a8a2aa6e85e05d69f2bd00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000689a05bc0000000000000000000000000000000000000000000000000000000030594ec500000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000014905b1c0098887ea9033946de1eab5427c97a82ad0000000000000000000000000000000000000000000000000000000000000000000000000000000000000040284f7f673bf73a515a8829dd29edc8671094e62d94db5cfa869bb62b4e8b6eff51c44f2662fb6fef1e37239d9a7d14707971feeddd1e9ba87c2ca5bafc1b6d9e";
        assert_eq!(expected, ethers::utils::hex::encode(&encoded));
    }
}
