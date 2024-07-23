// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

mod cache;
mod error;
mod finality;
pub mod sync;

pub mod convert;
pub mod proxy;
mod toggle;
pub mod voting;

use async_stm::Stm;
use async_trait::async_trait;
use ethers::utils::hex;
use fendermint_vm_message::ipc::{ParentFinalityPayload, TopdownProposalWithCert};
use fvm_shared::clock::ChainEpoch;
use ipc_api::cross::IpcEnvelope;
use ipc_api::staking::StakingChangeRequest;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::time::Duration;

pub use crate::cache::{SequentialAppendError, SequentialKeyCache, ValueIter};
pub use crate::error::Error;
pub use crate::finality::CachedFinalityProvider;
pub use crate::toggle::Toggle;
use crate::voting::payload::TopdownVote;
use crate::voting::quorum::MultiSigCert;

pub type BlockHeight = u64;
pub type Bytes = Vec<u8>;
pub type BlockHash = Bytes;

/// The null round error message
pub(crate) const NULL_ROUND_ERR_MSG: &str = "requested epoch was a null round";
/// Default topdown proposal height range
pub(crate) const DEFAULT_MAX_PROPOSAL_RANGE: BlockHeight = 100;
pub(crate) const DEFAULT_MAX_CACHE_BLOCK: BlockHeight = 500;
pub(crate) const DEFAULT_PROPOSAL_DELAY: BlockHeight = 2;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// The number of blocks to delay before reporting a height as final on the parent chain.
    /// To propose a certain number of epochs delayed from the latest height, we see to be
    /// conservative and avoid other from rejecting the proposal because they don't see the
    /// height as final yet.
    pub chain_head_delay: BlockHeight,
    /// Parent syncing cron period, in seconds
    pub polling_interval: Duration,
    /// Top down exponential back off retry base
    pub exponential_back_off: Duration,
    /// The max number of retries for exponential backoff before giving up
    pub exponential_retry_limit: usize,
    /// The max number of blocks one should make the topdown proposal
    pub max_proposal_range: Option<BlockHeight>,
    /// Max number of blocks that should be stored in cache
    pub max_cache_blocks: Option<BlockHeight>,
    pub proposal_delay: Option<BlockHeight>,
}

impl Config {
    pub fn new(
        chain_head_delay: BlockHeight,
        polling_interval: Duration,
        exponential_back_off: Duration,
        exponential_retry_limit: usize,
    ) -> Self {
        Self {
            chain_head_delay,
            polling_interval,
            exponential_back_off,
            exponential_retry_limit,
            max_proposal_range: None,
            max_cache_blocks: None,
            proposal_delay: None,
        }
    }

    pub fn with_max_proposal_range(mut self, max_proposal_range: BlockHeight) -> Self {
        self.max_proposal_range = Some(max_proposal_range);
        self
    }

    pub fn with_proposal_delay(mut self, proposal_delay: BlockHeight) -> Self {
        self.proposal_delay = Some(proposal_delay);
        self
    }

    pub fn with_max_cache_blocks(mut self, max_cache_blocks: BlockHeight) -> Self {
        self.max_cache_blocks = Some(max_cache_blocks);
        self
    }

    pub fn max_proposal_range(&self) -> BlockHeight {
        self.max_proposal_range
            .unwrap_or(DEFAULT_MAX_PROPOSAL_RANGE)
    }

    pub fn proposal_delay(&self) -> BlockHeight {
        self.proposal_delay.unwrap_or(DEFAULT_PROPOSAL_DELAY)
    }

    pub fn max_cache_blocks(&self) -> BlockHeight {
        self.max_cache_blocks.unwrap_or(DEFAULT_MAX_CACHE_BLOCK)
    }
}

/// The finality view for IPC parent at certain height.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IPCParentFinality {
    /// The latest chain height
    pub height: BlockHeight,
    /// The block hash. For FVM, it is a Cid. For Evm, it is bytes32 as one can now potentially
    /// deploy a subnet on EVM.
    pub block_hash: BlockHash,
}

impl IPCParentFinality {
    pub fn new(height: ChainEpoch, hash: BlockHash) -> Self {
        Self {
            height: height as BlockHeight,
            block_hash: hash,
        }
    }
}

impl Display for IPCParentFinality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IPCParentFinality(height: {}, block_hash: {})",
            self.height,
            hex::encode(&self.block_hash)
        )
    }
}

#[async_trait]
pub trait ParentViewProvider {
    /// Obtain the genesis epoch of the current subnet in the parent
    fn genesis_epoch(&self) -> anyhow::Result<BlockHeight>;
}

pub trait ParentFinalityProvider: ParentViewProvider {
    /// Latest proposal for parent finality
    fn next_proposal(&self) -> Stm<Option<TopdownProposal>>;
    /// The proposal for parent finality at the target height
    fn proposal_at_height(&self, height: BlockHeight) -> Stm<Option<TopdownProposal>>;
    /// Called when finality is committed
    fn set_new_finality(
        &self,
        finality: IPCParentFinality,
        previous_finality: Option<IPCParentFinality>,
    ) -> Stm<()>;
}

pub(crate) fn is_null_round_str(s: &str) -> bool {
    s.contains(NULL_ROUND_ERR_MSG)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct TopdownProposalWithQuorum {
    pub proposal: TopdownProposal,
    pub cert: MultiSigCert,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TopdownProposal {
    V1(ParentFinalityPayload),
}

impl TopdownProposal {
    pub fn v1(finality: ParentFinalityPayload) -> Self {
        Self::V1(finality)
    }

    pub fn block_height(&self) -> BlockHeight {
        match self {
            TopdownProposal::V1(v) => v.height as BlockHeight,
        }
    }

    pub fn block_hash(&self) -> BlockHash {
        match self {
            TopdownProposal::V1(v) => v.block_hash.clone(),
        }
    }

    pub fn finality(&self) -> IPCParentFinality {
        match self {
            TopdownProposal::V1(ref v) => IPCParentFinality {
                height: v.height as BlockHeight,
                block_hash: v.block_hash.clone(),
            },
        }
    }

    pub fn cross_msgs(&self) -> &[IpcEnvelope] {
        match self {
            TopdownProposal::V1(ref v) => v.cross_messages.as_slice(),
        }
    }

    pub fn validator_changes(&self) -> &[StakingChangeRequest] {
        match self {
            TopdownProposal::V1(ref v) => v.validator_changes.as_slice(),
        }
    }

    pub fn vote(&self) -> TopdownVote {
        match self {
            TopdownProposal::V1(ref v) => TopdownVote::v1(
                v.height as BlockHeight,
                v.block_hash.clone(),
                v.side_effect_cid().to_bytes(),
            ),
        }
    }
}

// Because chain interpreter is importing from fendermint_vm_message, no choice but to define the
// type twice.
impl From<TopdownProposalWithCert> for TopdownProposalWithQuorum {
    fn from(value: TopdownProposalWithCert) -> Self {
        Self {
            proposal: TopdownProposal::from(value.proposal),
            cert: MultiSigCert::from(value.cert),
        }
    }
}

impl From<fendermint_vm_message::ipc::TopdownProposal> for TopdownProposal {
    fn from(value: fendermint_vm_message::ipc::TopdownProposal) -> Self {
        match value {
            fendermint_vm_message::ipc::TopdownProposal::V1(f) => Self::V1(f),
        }
    }
}

impl From<TopdownProposal> for fendermint_vm_message::ipc::TopdownProposal {
    fn from(value: TopdownProposal) -> fendermint_vm_message::ipc::TopdownProposal {
        match value {
            TopdownProposal::V1(v) => fendermint_vm_message::ipc::TopdownProposal::V1(v),
        }
    }
}

impl From<TopdownProposalWithQuorum> for TopdownProposalWithCert {
    fn from(value: TopdownProposalWithQuorum) -> Self {
        Self {
            proposal: fendermint_vm_message::ipc::TopdownProposal::from(value.proposal),
            cert: fendermint_vm_message::ipc::MultiSigCert::from(value.cert),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_voting() {}
}
