// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

pub mod bottomup;
pub mod broadcast;
mod executions;
mod externs;
pub mod interpreter;
pub mod observe;
pub mod state;
pub mod store;
pub mod upgrades;
pub use interpreter::FvmMessagesInterpreter;

#[cfg(any(test, feature = "bundle"))]
pub mod bundle;

pub mod activity;
pub(crate) mod gas;
pub(crate) mod gas_estimation;
mod validator;

pub use bottomup::PowerUpdates;
use fendermint_crypto::{PublicKey, SecretKey};
pub use fendermint_vm_message::query::FvmQuery;
use fvm_shared::address::Address;

pub use self::broadcast::Broadcaster;

pub type FvmMessage = fvm_shared::message::Message;
pub type BaseFee = fvm_shared::econ::TokenAmount;
pub type BlockGasLimit = u64;

#[derive(Clone)]
pub struct ValidatorContext<C> {
    /// The secret key the validator uses to produce blocks.
    pub secret_key: SecretKey,
    /// The public key identifying the validator (corresponds to the secret key.)
    pub public_key: PublicKey,
    /// The address associated with the public key.
    pub addr: Address,
    /// Used to broadcast transactions. It might use a different secret key for
    /// signing transactions than the validator's block producing key.
    pub broadcaster: Broadcaster<C>,
}

impl<C> ValidatorContext<C> {
    pub fn new(secret_key: SecretKey, addr: Address, broadcaster: Broadcaster<C>) -> Self {
        // Derive the public keys so it's available to check whether this node is a validator at any point in time.
        let public_key = secret_key.public_key();
        Self {
            secret_key,
            public_key,
            addr,
            broadcaster,
        }
    }

    pub fn broadcaster(&self) -> &Broadcaster<C> {
        &self.broadcaster
    }
}
