// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;

use fendermint_vm_interpreter::errors::ProcessError;
use tendermint::account::Id;

use ipc_observability::{
    impl_traceable, impl_traceables, lazy_static, register_metrics, serde::HexEncodableBlockHash,
    Recordable, TraceLevel, Traceable,
};

use prometheus::{register_counter_vec, CounterVec, Registry};

register_metrics! {
    PROPOSALS_BLOCK_PROPOSAL_RECEIVED: CounterVec
        = register_counter_vec!("proposals_block_proposal_received", "Block proposal received", &["height"]);
    PROPOSALS_BLOCK_PROPOSAL_SENT: CounterVec
        = register_counter_vec!("proposals_block_proposal_sent", "Block proposal sent", &["height"]);
    PROPOSALS_BLOCK_PROPOSAL_ACCEPTED: CounterVec
        = register_counter_vec!("proposals_block_proposal_accepted", "Block proposal accepted", &["height"]);
    PROPOSALS_BLOCK_PROPOSAL_REJECTED: CounterVec
        = register_counter_vec!("proposals_block_proposal_rejected", "Block proposal rejected", &["height"]);
    PROPOSALS_BLOCK_COMMITTED: CounterVec
        = register_counter_vec!("proposals_block_committed", "Block committed", &["height"]);
    MPOOL_RECEIVED: CounterVec = register_counter_vec!("mpool_received", "Mpool received", &["accept", "from", "to"]);
    MPOOL_RECEIVED_INVALID_MESSAGE: CounterVec
        = register_counter_vec!("mpool_received_invalid_message", "Mpool received invalid message", &["reason"]);
}

impl_traceables!(
    TraceLevel::Info,
    "Proposals",
    BlockProposalReceived<'a>,
    BlockProposalSent<'a>,
    BlockProposalEvaluated<'a>,
    BlockCommitted
);

impl_traceables!(
    TraceLevel::Info,
    "Mpool",
    MpoolReceived<'a>,
    MpoolReceivedInvalidMessage<'a>
);

pub type BlockHeight = u64;

#[derive(Debug)]
pub struct BlockProposalReceived<'a> {
    pub height: BlockHeight,
    pub hash: HexEncodableBlockHash,
    pub size: usize,
    pub tx_count: usize,
    pub validator: &'a str,
}

impl Recordable for BlockProposalReceived<'_> {
    fn record_metrics(&self) {
        PROPOSALS_BLOCK_PROPOSAL_RECEIVED
            .with_label_values(&[&self.height.to_string()])
            .inc();
    }
}

#[derive(Debug)]
pub struct BlockProposalSent<'a> {
    pub validator: &'a Id,
    pub height: BlockHeight,
    pub size: usize,
    pub tx_count: usize,
}

impl Recordable for BlockProposalSent<'_> {
    fn record_metrics(&self) {
        PROPOSALS_BLOCK_PROPOSAL_SENT
            .with_label_values(&[&self.height.to_string()])
            .inc();
    }
}

#[derive(Debug)]
pub struct BlockProposalEvaluated<'a> {
    pub height: BlockHeight,
    pub hash: HexEncodableBlockHash,
    pub size: usize,
    pub tx_count: usize,
    pub validator: &'a Id,
    pub accept: bool,
    pub reason: Option<ProcessError>,
}

impl Recordable for BlockProposalEvaluated<'_> {
    fn record_metrics(&self) {
        if self.accept {
            PROPOSALS_BLOCK_PROPOSAL_ACCEPTED
                .with_label_values(&[&self.height.to_string()])
                .inc();
        } else {
            PROPOSALS_BLOCK_PROPOSAL_REJECTED
                .with_label_values(&[&self.height.to_string()])
                .inc();
        }
    }
}

#[derive(Debug)]
pub struct BlockCommitted {
    pub height: BlockHeight,
    pub app_hash: HexEncodableBlockHash,
}

impl Recordable for BlockCommitted {
    fn record_metrics(&self) {
        PROPOSALS_BLOCK_COMMITTED
            .with_label_values(&[&self.height.to_string()])
            .inc();
    }
}

#[derive(Debug)]
pub struct MpoolReceived<'a> {
    // TODO - add cid later on
    // pub message_cid: &'a str,
    pub from: &'a Address,
    pub to: &'a Address,
    pub value: &'a TokenAmount,
    pub param_len: usize,
    pub gas_limit: u64,
    pub fee_cap: &'a TokenAmount,
    pub premium: &'a TokenAmount,
    pub accept: bool,
    pub reason: Option<&'a str>,
}

impl Recordable for MpoolReceived<'_> {
    fn record_metrics(&self) {
        MPOOL_RECEIVED
            .with_label_values(&[
                &self.accept.to_string(),
                self.from.to_string().as_str(),
                self.to.to_string().as_str(),
            ])
            .inc();
    }
}

#[derive(Debug)]
pub struct MpoolReceivedInvalidMessage<'a> {
    pub reason: &'a str,
    pub description: &'a str,
}

impl Recordable for MpoolReceivedInvalidMessage<'_> {
    fn record_metrics(&self) {
        MPOOL_RECEIVED_INVALID_MESSAGE
            .with_label_values(&[self.reason])
            .inc();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ipc_observability::emit;

    #[test]
    fn test_emit() {
        emit(BlockProposalReceived {
            height: 1,
            hash: HexEncodableBlockHash(vec![0x01, 0x02, 0x03]),
            size: 100,
            tx_count: 10,
            validator: "validator",
        });

        let id = Id::new([0x01; 20]);

        emit(BlockProposalSent {
            height: 1,
            size: 100,
            tx_count: 10,
            validator: &id,
        });

        emit(BlockProposalEvaluated {
            height: 1,
            hash: HexEncodableBlockHash(vec![0x01, 0x02, 0x03]),
            size: 100,
            tx_count: 10,
            validator: &id,
            accept: true,
            reason: None,
        });

        emit(BlockProposalEvaluated {
            height: 1,
            hash: HexEncodableBlockHash(vec![0x01, 0x02, 0x03]),
            size: 100,
            tx_count: 10,
            validator: &id,
            accept: false,
            reason: Some(ProcessError::CheckpointNotResolved),
        });

        emit(BlockCommitted {
            height: 1,
            app_hash: HexEncodableBlockHash(vec![0x01, 0x02, 0x03]),
        });
    }
}
