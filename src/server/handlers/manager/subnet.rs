// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! The shared subnet manager module for all subnet management related RPC method calls.

use crate::config::subnet::SubnetConfig;
use crate::config::{ReloadableConfig, Subnet};
use crate::manager::{EthSubnetManager, LotusSubnetManager, SubnetManager};
use ipc_identity::PersistentKeyStore;
use ipc_identity::Wallet;
use ipc_sdk::subnet_id::SubnetID;
use std::borrow::Borrow;
use std::sync::{Arc, RwLock};

/// The subnet manager connection that holds the subnet config and the manager instance.
pub struct Connection {
    subnet: Subnet,
    manager: Box<dyn SubnetManager + 'static>,
}

impl Connection {
    /// Get the subnet config.
    pub fn subnet(&self) -> &Subnet {
        &self.subnet
    }

    /// Get the subnet manager instance.
    pub fn manager(&self) -> &dyn SubnetManager {
        self.manager.borrow()
    }
}

/// The json rpc subnet manager connection pool. This struct can be shared by all the subnet methods.
/// As such, there is no need to re-init the same SubnetManager for different methods to reuse connections.
pub struct SubnetManagerPool {
    config: Arc<ReloadableConfig>,
    fvm_wallet: Arc<RwLock<Wallet>>,
    evm_keystore: Arc<RwLock<PersistentKeyStore<ethers::types::Address>>>,
}

impl SubnetManagerPool {
    pub fn new(
        reload_config: Arc<ReloadableConfig>,
        fvm_wallet: Arc<RwLock<Wallet>>,
        evm_keystore: Arc<RwLock<PersistentKeyStore<ethers::types::Address>>>,
    ) -> Self {
        Self {
            config: reload_config,
            fvm_wallet,
            evm_keystore,
        }
    }

    /// Get the connection instance for the subnet.
    pub fn get(&self, subnet: &SubnetID) -> Option<Connection> {
        let config = self.config.get_config();
        let subnets = &config.subnets;
        match subnets.get(subnet) {
            Some(subnet) => match &subnet.config {
                SubnetConfig::Fvm(_) => {
                    let manager = Box::new(LotusSubnetManager::from_subnet_with_wallet_store(
                        subnet,
                        self.fvm_wallet.clone(),
                    ));
                    Some(Connection {
                        manager,
                        subnet: subnet.clone(),
                    })
                }
                SubnetConfig::Fevm(_) => {
                    let manager = Box::new(
                        EthSubnetManager::from_subnet_with_wallet_store(
                            subnet,
                            self.evm_keystore.clone(),
                        )
                        .ok()?,
                    );
                    Some(Connection {
                        manager,
                        subnet: subnet.clone(),
                    })
                }
            },
            None => None,
        }
    }
}
