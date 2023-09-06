// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
//! Ipc agent sdk, contains the json rpc client to interact with the IPC agent rpc server.

#![feature(let_chains)]

use anyhow::anyhow;
use checkpoint::NativeBottomUpCheckpoint;
use config::ReloadableConfig;
use fvm_shared::{address::Address, clock::ChainEpoch, econ::TokenAmount};
use ipc_identity::{KeyStore, KeyStoreConfig, PersistentKeyStore, Wallet};
use ipc_sdk::{
    cross::CrossMsg,
    subnet::{ConsensusType, ConstructParams},
    subnet_id::SubnetID,
};
use lotus::message::ipc::QueryValidatorSetResponse;
use manager::{fevm::FevmSubnetManager, LotusSubnetManager, SubnetInfo, SubnetManager};
use std::{
    borrow::Borrow,
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub mod checkpoint;
pub mod config;
pub mod jsonrpc;
pub mod lotus;
pub mod manager;

const DEFAULT_REPO_PATH: &str = ".ipc-agent";
const DEFAULT_CONFIG_NAME: &str = "config.toml";

/// The subnet manager connection that holds the subnet config and the manager instance.
pub struct Connection {
    subnet: config::Subnet,
    manager: Box<dyn SubnetManager + 'static>,
}

impl Connection {
    /// Get the subnet config.
    pub fn subnet(&self) -> &config::Subnet {
        &self.subnet
    }

    /// Get the subnet manager instance.
    pub fn manager(&self) -> &dyn SubnetManager {
        self.manager.borrow()
    }
}

pub struct IpcProvider {
    sender: Option<Address>,
    config: Arc<ReloadableConfig>,
    fvm_wallet: Arc<RwLock<Wallet>>,
    evm_keystore: Arc<RwLock<PersistentKeyStore<ethers::types::Address>>>,
}

impl IpcProvider {
    pub fn new(
        config: Arc<ReloadableConfig>,
        fvm_wallet: Arc<RwLock<Wallet>>,
        evm_keystore: Arc<RwLock<PersistentKeyStore<ethers::types::Address>>>,
    ) -> Self {
        Self {
            sender: None,
            config,
            fvm_wallet,
            evm_keystore,
        }
    }

    /// Initializes an `IpcProvider` from the config specified in the
    /// argument's config path.
    pub fn new_from_config(config_path: String) -> anyhow::Result<Self> {
        let config = Arc::new(ReloadableConfig::new(config_path)?);
        let fvm_wallet = Arc::new(RwLock::new(Wallet::new(new_fvm_wallet_from_config(
            config.clone(),
        )?)));
        let evm_keystore = Arc::new(RwLock::new(new_evm_keystore_from_config(config.clone())?));
        Ok(Self::new(config, fvm_wallet, evm_keystore))
    }

    /// Initialized an `IpcProvider` using the default config path.
    pub fn new_default() -> anyhow::Result<Self> {
        Self::new_from_config(default_config_path())
    }

    /// Get the connection instance for the subnet.
    pub fn connection(&self, subnet: &SubnetID) -> Option<Connection> {
        let config = self.config.get_config();
        let subnets = &config.subnets;
        match subnets.get(subnet) {
            Some(subnet) => match &subnet.config {
                config::subnet::SubnetConfig::Fvm(_) => {
                    let manager = Box::new(LotusSubnetManager::from_subnet_with_wallet_store(
                        subnet,
                        self.fvm_wallet.clone(),
                    ));
                    Some(Connection {
                        manager,
                        subnet: subnet.clone(),
                    })
                }
                config::subnet::SubnetConfig::Fevm(_) => {
                    let manager = Box::new(
                        FevmSubnetManager::from_subnet_with_wallet_store(
                            subnet,
                            self.evm_keystore.clone(),
                            self.fvm_wallet.clone(),
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

    /// Set the default account for the provider
    pub fn with_sender(&mut self, from: Address) {
        self.sender = Some(from);
    }

    fn call_sender(&self, from: Option<Address>) -> anyhow::Result<Address> {
        if from.is_none() {
            // get default account
            if self.sender.is_none() {
                return Err(anyhow!(
                    "default account not found for provider. Use `with_sender` to set it up."
                ));
            } else {
                return Ok(self.sender.unwrap());
            }
        }
        Ok(from.unwrap())
    }

    fn check_subnet(&self, subnet: &config::Subnet) -> anyhow::Result<()> {
        match &subnet.config {
            config::subnet::SubnetConfig::Fvm(config) => {
                if config.auth_token.is_none() {
                    log::error!("subnet {:?} does not have auth token", subnet.id);
                    return Err(anyhow!("Internal server error"));
                }
            }
            config::subnet::SubnetConfig::Fevm(_) => {
                // TODO: add more checks later
            }
        }
        Ok(())
    }
}

/// IpcProvider spawns a daemon-less client to interact with IPC subnets.
///
/// At this point the provider assumes that the user providers a `config.toml`
/// with the subnet configuration. This has been inherited by the daemon
/// configuration and will be slowly deprecated.
impl IpcProvider {
    // FIXME: Once the arguments for subnet creation are stabilized,
    // use a SubnetOpts struct to provide the creation arguments and
    // remove this allow
    #[allow(clippy::too_many_arguments)]
    pub async fn create_subnet(
        &self,
        from: Option<Address>,
        parent: &SubnetID,
        subnet_name: String,
        min_validators: u64,
        min_validator_stake: TokenAmount,
        bottomup_check_period: ChainEpoch,
        topdown_check_period: ChainEpoch,
    ) -> anyhow::Result<Address> {
        let conn = match self.connection(parent) {
            None => return Err(anyhow!("target parent subnet not found")),
            Some(conn) => conn,
        };

        let subnet_config = conn.subnet();
        self.check_subnet(subnet_config)?;

        let constructor_params = ConstructParams {
            parent: parent.clone(),
            name: subnet_name,
            ipc_gateway_addr: subnet_config.gateway_addr(),
            consensus: ConsensusType::Mir,
            min_validators,
            min_validator_stake,
            bottomup_check_period,
            topdown_check_period,
            genesis: vec![],
        };

        conn.manager()
            .create_subnet(self.call_sender(from)?, constructor_params)
            .await
    }

    /// Performs the call to join a subnet from a wallet address and staking an amount
    /// of collateral. This function, as well as all of the ones on this trait, can infer
    /// the specific subnet and actors on which to perform the relevant calls from the
    /// SubnetID given as an argument.
    pub async fn join_subnet(
        &self,
        _subnet: SubnetID,
        _from: Address,
        _collateral: TokenAmount,
        _validator_net_addr: String,
        _worker_addr: Address,
    ) -> anyhow::Result<()> {
        todo!()
    }

    /// Sends a request to leave a subnet from a wallet address.
    pub async fn leave_subnet(&self, _subnet: SubnetID, _from: Address) -> anyhow::Result<()> {
        todo!()
    }

    /// Sends a signal to kill a subnet
    pub async fn kill_subnet(&self, _subnet: SubnetID, _from: Address) -> anyhow::Result<()> {
        todo!()
    }

    /// Lists all the registered children in a gateway.
    pub async fn list_child_subnets(
        &self,
        _gateway_addr: Address,
    ) -> anyhow::Result<HashMap<SubnetID, SubnetInfo>> {
        todo!()
    }

    /// Fund injects new funds from an account of the parent chain to a subnet.
    /// Returns the epoch that the fund is executed in the parent.
    pub async fn fund(
        &self,
        _subnet: SubnetID,
        _gateway_addr: Address,
        _from: Address,
        _to: Address,
        _amount: TokenAmount,
    ) -> anyhow::Result<ChainEpoch> {
        todo!()
    }

    /// Release creates a new check message to release funds in parent chain
    /// Returns the epoch that the released is executed in the child.
    pub async fn release(
        &self,
        _subnet: SubnetID,
        _gateway_addr: Address,
        _from: Address,
        _to: Address,
        _amount: TokenAmount,
    ) -> anyhow::Result<ChainEpoch> {
        todo!()
    }

    /// Propagate a cross-net message forward. For `postbox_msg_key`, we are using bytes because different
    /// runtime have different representations. For FVM, it should be `CID` as bytes. For EVM, it is
    /// `bytes32`.
    pub async fn propagate(
        &self,
        _subnet: SubnetID,
        _gateway_addr: Address,
        _from: Address,
        _postbox_msg_key: Vec<u8>,
    ) -> anyhow::Result<()> {
        todo!()
    }

    pub async fn send_cross_message(
        &self,
        _gateway_addr: Address,
        _from: Address,
        _cross_msg: CrossMsg,
    ) -> anyhow::Result<()> {
        todo!()
    }

    /// Sets a new net address to an existing validator
    pub async fn set_validator_net_addr(
        &self,
        _subnet: SubnetID,
        _from: Address,
        _validator_net_addr: String,
    ) -> anyhow::Result<()> {
        todo!()
    }

    /// Sets a new worker address to an existing validator
    pub async fn set_validator_worker_addr(
        &self,
        _subnet: SubnetID,
        _from: Address,
        _validator_worker_addr: Address,
    ) -> anyhow::Result<()> {
        todo!()
    }

    /// Send value between two addresses in a subnet
    pub async fn send_value(
        &self,
        _from: Address,
        _to: Address,
        _amount: TokenAmount,
    ) -> anyhow::Result<()> {
        todo!()
    }

    /// Get the balance of an address
    pub async fn wallet_balance(&self, _address: &Address) -> anyhow::Result<TokenAmount> {
        todo!()
    }

    /// Returns the epoch of the latest top-down checkpoint executed
    pub async fn last_topdown_executed(
        &self,
        _gateway_addr: &Address,
    ) -> anyhow::Result<ChainEpoch> {
        todo!()
    }

    /// Returns the list of checkpoints from a subnet actor for the given epoch range.
    pub async fn list_checkpoints(
        &self,
        _subnet_id: SubnetID,
        _from_epoch: ChainEpoch,
        _to_epoch: ChainEpoch,
    ) -> anyhow::Result<Vec<NativeBottomUpCheckpoint>> {
        todo!()
    }

    /// Returns the validator set
    pub async fn get_validator_set(
        &self,
        _subnet_id: &SubnetID,
        _gateway: Option<Address>,
        _epoch: Option<ChainEpoch>,
    ) -> anyhow::Result<QueryValidatorSetResponse> {
        todo!()
    }

    pub async fn chain_head_height(&self) -> anyhow::Result<ChainEpoch> {
        todo!()
    }

    pub async fn get_top_down_msgs(
        &self,
        _subnet_id: &SubnetID,
        _start_epoch: ChainEpoch,
        _end_epoch: ChainEpoch,
    ) -> anyhow::Result<Vec<CrossMsg>> {
        todo!()
    }

    pub async fn get_block_hash(&self, _height: ChainEpoch) -> anyhow::Result<Vec<u8>> {
        todo!()
    }
}

fn new_fvm_wallet_from_config(config: Arc<ReloadableConfig>) -> anyhow::Result<KeyStore> {
    let repo_str = config.get_config_repo();
    if let Some(repo_str) = repo_str {
        new_keystore_from_path(&repo_str)
    } else {
        Err(anyhow!("No keystore repo found in config"))
    }
}

fn new_evm_keystore_from_config(
    config: Arc<ReloadableConfig>,
) -> anyhow::Result<PersistentKeyStore<ethers::types::Address>> {
    let repo_str = config.get_config_repo();
    if let Some(repo_str) = repo_str {
        new_evm_keystore_from_path(&repo_str)
    } else {
        Err(anyhow!("No keystore repo found in config"))
    }
}

fn new_evm_keystore_from_path(
    repo_str: &str,
) -> anyhow::Result<PersistentKeyStore<ethers::types::Address>> {
    let repo = std::path::Path::new(&repo_str).join(ipc_identity::DEFAULT_KEYSTORE_NAME);
    PersistentKeyStore::new(repo).map_err(|e| anyhow!("Failed to create evm keystore: {}", e))
}

fn new_keystore_from_path(repo_str: &str) -> anyhow::Result<KeyStore> {
    let repo = std::path::Path::new(&repo_str);
    let keystore_config = KeyStoreConfig::Persistent(repo.join(ipc_identity::KEYSTORE_NAME));
    // TODO: we currently only support persistent keystore in the default repo directory.
    KeyStore::new(keystore_config).map_err(|e| anyhow!("Failed to create keystore: {}", e))
}

pub fn default_repo_path() -> String {
    let home = match std::env::var("HOME") {
        Ok(home) => home,
        Err(_) => panic!("cannot get home"),
    };
    format!("{home:}/{:}", DEFAULT_REPO_PATH)
}

pub fn default_config_path() -> String {
    format!("{}/{:}", default_repo_path(), DEFAULT_CONFIG_NAME)
}
