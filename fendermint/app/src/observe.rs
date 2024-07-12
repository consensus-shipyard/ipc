// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use hex;
use std::fmt;

use ipc_observability::{impl_traceable, impl_traceables, Recordable, TraceLevel, Traceable};

impl_traceables!(
    TraceLevel::Info,
    "Proposals",
    BlockProposalReceived<'a>,
    BlockProposalSent,
    BlockProposalAccepted<'a>,
    BlockProposalRejected<'a>,
    BlockCommitted
);

pub type BlockHeight = u64;
/// Hex encodable block hash.
pub struct HexEncodableBlockHash(pub Vec<u8>);

impl fmt::Debug for HexEncodableBlockHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

#[derive(Debug)]
pub struct BlockProposalReceived<'a> {
    pub height: BlockHeight,
    pub hash: HexEncodableBlockHash,
    pub size: usize,
    pub tx_count: usize,
    pub validator: &'a str,
}

impl Recordable for BlockProposalReceived<'_> {
    fn record_metrics(&self) {}
}

#[derive(Debug)]
pub struct BlockProposalSent {
    pub height: BlockHeight,
    pub size: usize,
    pub tx_count: usize,
}

impl Recordable for BlockProposalSent {
    fn record_metrics(&self) {}
}

#[derive(Debug)]
pub struct BlockProposalAccepted<'a> {
    pub height: BlockHeight,
    pub hash: HexEncodableBlockHash,
    pub size: usize,
    pub tx_count: usize,
    pub validator: &'a str,
}

impl Recordable for BlockProposalAccepted<'_> {
    fn record_metrics(&self) {}
}

#[derive(Debug)]
pub struct BlockProposalRejected<'a> {
    pub height: BlockHeight,
    pub size: usize,
    pub tx_count: usize,
    pub validator: &'a str,
    pub reason: &'a str,
}

impl Recordable for BlockProposalRejected<'_> {
    fn record_metrics(&self) {}
}

#[derive(Debug)]
pub struct BlockCommitted {
    pub height: BlockHeight,
    pub app_hash: HexEncodableBlockHash,
}

impl Recordable for BlockCommitted {
    fn record_metrics(&self) {}
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

        emit(BlockProposalSent {
            height: 1,
            size: 100,
            tx_count: 10,
        });

        emit(BlockProposalAccepted {
            height: 1,
            hash: HexEncodableBlockHash(vec![0x01, 0x02, 0x03]),
            size: 100,
            tx_count: 10,
            validator: "validator",
        });

        emit(BlockProposalRejected {
            height: 1,
            size: 100,
            tx_count: 10,
            validator: "validator",
            reason: "reason",
        });

        emit(BlockCommitted {
            height: 1,
            app_hash: HexEncodableBlockHash(vec![0x01, 0x02, 0x03]),
        });
    }
}
