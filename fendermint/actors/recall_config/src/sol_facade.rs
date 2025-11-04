// Copyright 2022-2024 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::credit::TokenCreditRate;
use fvm_shared::{address::Address, clock::ChainEpoch};
use recall_actor_sdk::evm::TryIntoEVMEvent;
use recall_sol_facade::{
    config as sol,
    primitives::U256,
    types::{BigUintWrapper, H160},
};

pub struct ConfigAdminSet {
    pub admin: Address,
}
impl ConfigAdminSet {
    pub fn new(admin: Address) -> Self {
        Self { admin }
    }
}
impl TryIntoEVMEvent for ConfigAdminSet {
    type Target = sol::Events;
    fn try_into_evm_event(self) -> Result<Self::Target, anyhow::Error> {
        let admin: H160 = self.admin.try_into()?;
        Ok(sol::Events::ConfigAdminSet(sol::ConfigAdminSet {
            admin: admin.into(),
        }))
    }
}

pub struct ConfigSet {
    pub blob_capacity: u64,
    pub token_credit_rate: TokenCreditRate,
    pub blob_credit_debit_interval: ChainEpoch,
    pub blob_min_ttl: ChainEpoch,
    pub blob_default_ttl: ChainEpoch,
    pub blob_delete_batch_size: u64,
    pub account_debit_batch_size: u64,
}
impl TryIntoEVMEvent for ConfigSet {
    type Target = sol::Events;
    fn try_into_evm_event(self) -> Result<Self::Target, anyhow::Error> {
        Ok(sol::Events::ConfigSet(sol::ConfigSet {
            blobCapacity: U256::from(self.blob_capacity),
            tokenCreditRate: BigUintWrapper(self.token_credit_rate.rate().clone()).into(),
            blobCreditDebitInterval: U256::from(self.blob_credit_debit_interval),
            blobMinTtl: U256::from(self.blob_min_ttl),
            blobDefaultTtl: U256::from(self.blob_default_ttl),
            blobDeleteBatchSize: U256::from(self.blob_delete_batch_size),
            accountDebitBatchSize: U256::from(self.account_debit_batch_size),
        }))
    }
}
