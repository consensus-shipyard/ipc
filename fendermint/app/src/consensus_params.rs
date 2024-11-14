// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

//! Tracks the consensus parameters of the chain from genesis to the current block.

use tendermint::consensus::Params as TendermintConsensusParams;

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub(crate) struct ConsensusParamsTracker {
    // Store the genesis consensus
    genesis_consensus_params: Arc<Mutex<Option<TendermintConsensusParams>>>,
    // Current block gas limit
    current_block_gas_limit: Arc<AtomicI64>,
}

impl ConsensusParamsTracker {
    pub fn new() -> Self {
        Self {
            genesis_consensus_params: Arc::new(Mutex::new(None)),
            current_block_gas_limit: Arc::new(AtomicI64::new(0)),
        }
    }

    pub fn set_consensus_params(&self, params: TendermintConsensusParams) {
        self.current_block_gas_limit
            .store(params.block.max_gas, Ordering::Relaxed);
        let mut consensus_params = self.genesis_consensus_params.lock().unwrap();
        *consensus_params = Some(params);
    }

    pub fn set_current_block_gas_limit(&self, gas_limit: i64) {
        self.current_block_gas_limit
            .store(gas_limit, Ordering::Relaxed);
    }

    pub fn get_current_block_gas_limit(&self) -> i64 {
        self.current_block_gas_limit.load(Ordering::Relaxed)
    }

    pub fn get_genesis_consensus_params(&self) -> Option<TendermintConsensusParams> {
        let consensus_params = self.genesis_consensus_params.lock().unwrap();
        consensus_params.clone()
    }
}
