// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! The one place where the parameters send to ipc contracts are preprocessed, i.e.
//! setting default/missing parameter, checking if the parameters are valid.

use crate::config::{Config, Subnet};
use anyhow::anyhow;
use fvm_shared::address::Address;
use ipc_api::subnet::{ConsensusType, ConstructParams};
use ipc_api::subnet_id::SubnetID;
use ipc_wallet::{EthKeyAddress, EvmKeyStore, PersistentKeyStore};
use std::sync::{Arc, RwLock};

const DEFAULT_ACTIVE_VALIDATORS: u16 = 100;
const DEFAULT_POWER_SCALE: i8 = 3;
const DEFAULT_SUBNET_CONSENSUS_TYPE: ConsensusType = ConsensusType::Fendermint;
/// The majority vote percentage for checkpoint submission when creating a subnet.
const SUBNET_MAJORITY_PERCENTAGE: u8 = 67;

/// The one place where the parameters send to ipc contracts are preprocessed, i.e.
/// setting default/missing parameter, checking if the parameters are valid.
#[derive(Clone)]
pub struct Preflight {
    config: Arc<Config>,
    evm_keystore: Option<Arc<RwLock<PersistentKeyStore<EthKeyAddress>>>>,
}

impl Preflight {
    pub fn new(
        config: Arc<Config>,
        evm_keystore: Option<Arc<RwLock<PersistentKeyStore<EthKeyAddress>>>>,
    ) -> Self {
        Self {
            config,
            evm_keystore,
        }
    }

    pub fn get_default_signer(&self) -> anyhow::Result<Option<Address>> {
        let wallet = if let Some(wallet) = &self.evm_keystore {
            wallet
        } else {
            return Ok(None);
        };

        Ok(if let Some(addr) = wallet.write().unwrap().get_default()? {
            Some(Address::try_from(addr)?)
        } else {
            None
        })
    }

    pub fn create_subnet(&self, mut params: ConstructParams) -> anyhow::Result<ConstructParams> {
        let config = self.config(&params.parent)?;

        if params.ipc_gateway_addr.is_none() {
            params.ipc_gateway_addr = Some(config.gateway_addr());
        }

        if params.active_validators_limit.is_none() {
            params.active_validators_limit = Some(DEFAULT_ACTIVE_VALIDATORS);
        }

        if params.power_scale.is_none() {
            params.power_scale = Some(DEFAULT_POWER_SCALE);
        }

        if params.consensus.is_none() {
            params.consensus = Some(DEFAULT_SUBNET_CONSENSUS_TYPE);
        }

        if params.majority_percentage.is_none() {
            params.majority_percentage = Some(SUBNET_MAJORITY_PERCENTAGE);
        }

        Ok(params)
    }

    /// Get the connection instance for the subnet.
    fn config(&self, subnet: &SubnetID) -> anyhow::Result<&Subnet> {
        self.config
            .subnets
            .get(subnet)
            .ok_or_else(|| anyhow!("subnet config does not exist {}", subnet))
    }
}
