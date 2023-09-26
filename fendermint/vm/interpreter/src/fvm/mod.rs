// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use std::path::PathBuf;

mod check;
mod checkpoint;
mod exec;
mod externs;
mod genesis;
mod query;
pub mod state;
pub mod store;

#[cfg(any(test, feature = "bundle"))]
pub mod bundle;

pub use check::FvmCheckRet;
pub use checkpoint::PowerUpdates;
pub use exec::FvmApplyRet;
use fendermint_crypto::{PublicKey, SecretKey};
use fendermint_eth_hardhat::Hardhat;
pub use fendermint_vm_message::query::FvmQuery;
pub use genesis::FvmGenesisOutput;
pub use query::FvmQueryRet;

use self::state::ipc::GatewayCaller;

pub type FvmMessage = fvm_shared::message::Message;

/// Interpreter working on already verified unsigned messages.
#[derive(Clone)]
pub struct FvmMessageInterpreter<DB, C> {
    contracts: Hardhat,
    /// Tendermint client for broadcasting transactions and run API queries.
    client: C,
    /// If this is a validator node, this should be the key we can use to sign transactions.
    _validator_key: Option<(SecretKey, PublicKey)>,
    /// Overestimation rate applied to gas to ensure that the
    /// message goes through in the gas estimation.
    gas_overestimation_rate: f64,
    /// Gas search step increase used to find the optimal gas limit.
    /// It determines how fine-grained we want the gas estimation to be.
    gas_search_step: f64,
    /// Indicate whether transactions should be fully executed during the checks performed
    /// when they are added to the mempool, or just the most basic ones are performed.
    exec_in_check: bool,
    gateway: GatewayCaller<DB>,
}

impl<DB, C> FvmMessageInterpreter<DB, C> {
    pub fn new(
        client: C,
        validator_key: Option<SecretKey>,
        contracts_dir: PathBuf,
        gas_overestimation_rate: f64,
        gas_search_step: f64,
        exec_in_check: bool,
    ) -> Self {
        // Derive the public keys so it's available to check whether this node is a validator at any point in time.
        let validator_key = validator_key.map(|sk| {
            let pk = sk.public_key();
            (sk, pk)
        });
        Self {
            client,
            _validator_key: validator_key,
            contracts: Hardhat::new(contracts_dir),
            gas_overestimation_rate,
            gas_search_step,
            exec_in_check,
            gateway: GatewayCaller::default(),
        }
    }
}
