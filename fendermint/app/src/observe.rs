// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;

use fendermint_vm_interpreter::fvm::FvmMessage;
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
    MPOOL_RECEIVED: CounterVec = register_counter_vec!("mpool_received", "Mpool received", &["accept"]);
}

impl_traceables!(
    TraceLevel::Info,
    "Proposals",
    BlockProposalReceived<'a>,
    BlockProposalSent<'a>,
    BlockProposalEvaluated<'a>,
    BlockCommitted
);

impl_traceables!(TraceLevel::Info, "Mpool", MpoolReceived);

pub type BlockHeight = u64;

#[derive(Debug)]
pub struct BlockProposalReceived<'a> {
    pub height: BlockHeight,
    pub hash: HexEncodableBlockHash,
    pub size: usize,
    pub tx_count: usize,
    pub validator: &'a Id,
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
    pub reason: Option<&'a str>,
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
pub struct Message {
    pub from: Address,
    pub to: Address,
    pub value: TokenAmount,
    pub gas_limit: u64,
    pub fee_cap: TokenAmount,
    pub premium: TokenAmount,
}

impl From<&FvmMessage> for Message {
    fn from(fvm_message: &FvmMessage) -> Self {
        Message {
            from: fvm_message.from,
            to: fvm_message.to,
            value: fvm_message.value.clone(),
            gas_limit: fvm_message.gas_limit,
            fee_cap: fvm_message.gas_fee_cap.clone(),
            premium: fvm_message.gas_premium.clone(),
        }
    }
}

#[derive(Debug, Default)]
pub struct MpoolReceived {
    // TODO - add cid later on
    // pub message_cid: &'a str,
    pub message: Option<Message>,
    pub accept: bool,
    pub reason: Option<String>,
}

impl Recordable for MpoolReceived {
    fn record_metrics(&self) {
        MPOOL_RECEIVED
            .with_label_values(&[&self.accept.to_string()])
            .inc();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ipc_observability::emit;

    #[test]
    fn test_emit() {
        let id = Id::new([0x01; 20]);

        emit(BlockProposalReceived {
            height: 1,
            hash: HexEncodableBlockHash(vec![0x01, 0x02, 0x03]),
            size: 100,
            tx_count: 10,
            validator: &id,
        });

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
            reason: None,
        });

        emit(BlockCommitted {
            height: 1,
            app_hash: HexEncodableBlockHash(vec![0x01, 0x02, 0x03]),
        });
    }
}
