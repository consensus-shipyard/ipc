// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub use crate::manager::evm::{ethers_address_to_fil_address, fil_to_eth_amount};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cid::Cid;
use ethers::abi::Tokenizable;
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::{abigen, Signer, SignerMiddleware};
use ethers::providers::{Authorization, Http, Middleware, Provider};
use ethers::signers::{LocalWallet, Wallet};
use ethers::types::TransactionRequest;
use fvm_shared::address::Payload;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::{address::Address, econ::TokenAmount};
use ipc_gateway::BottomUpCheckpoint;
use ipc_identity::{EvmKeyStore, PersistentKeyStore};
use ipc_sdk::subnet_id::SubnetID;
use ipc_subnet_actor::ConstructParams;
use num_traits::ToPrimitive;

use crate::config::subnet::SubnetConfig;
use crate::config::Subnet;
use crate::lotus::message::ipc::{QueryValidatorSetResponse, SubnetInfo, Validator, ValidatorSet};
use crate::manager::{EthManager, SubnetManager};

pub type DefaultSignerMiddleware = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;

/// The majority vote percentage for checkpoint submission when creating a subnet.
const SUBNET_MAJORITY_PERCENTAGE: u8 = 60;
const TRANSACTION_RECEIPT_RETRIES: usize = 10;
const SUBNET_NAME_MAX_LEN: usize = 32;

// Create type bindings for the IPC Solidity contracts
abigen!(Gateway, "contracts/Gateway.json");
abigen!(SubnetContract, "contracts/SubnetActor.json");
abigen!(SubnetRegistry, "contracts/SubnetRegistry.json");

pub struct EthSubnetManager {
    keystore: Arc<RwLock<PersistentKeyStore<ethers::types::Address>>>,
    ipc_contract_info: IPCContractInfo,
}

/// Keep track of the on chain information for the subnet manager
struct IPCContractInfo {
    gateway_addr: ethers::types::Address,
    registry_addr: ethers::types::Address,
    chain_id: u64,
    provider: Provider<Http>,
}

#[async_trait]
impl SubnetManager for EthSubnetManager {
    async fn create_subnet(&self, from: Address, params: ConstructParams) -> Result<Address> {
        self.ensure_same_gateway(&params.ipc_gateway_addr)?;

        let name_len = params.name.as_bytes().len();
        if name_len > SUBNET_NAME_MAX_LEN {
            return Err(anyhow!("subnet name too long"));
        }
        let mut name = [0u8; SUBNET_NAME_MAX_LEN];
        name[0..name_len].copy_from_slice(params.name.as_bytes());

        let min_validator_stake = params
            .min_validator_stake
            .atto()
            .to_u128()
            .ok_or_else(|| anyhow!("invalid min validator stake"))?;

        log::debug!("calling create subnet for EVM manager");

        let route = agent_subnet_to_evm_addresses(&params.parent)?;
        log::debug!("root SubnetID as Ethereum type: {route:?}");

        let params = subnet_registry::ConstructParams {
            parent_id: subnet_registry::SubnetID {
                root: params.parent.root_id(),
                route,
            },
            name,
            ipc_gateway_addr: self.ipc_contract_info.gateway_addr,
            consensus: params.consensus as u64 as u8,
            min_activation_collateral: ethers::types::U256::from(min_validator_stake),
            min_validators: params.min_validators,
            bottom_up_check_period: params.bottomup_check_period as u64,
            top_down_check_period: params.topdown_check_period as u64,
            majority_percentage: SUBNET_MAJORITY_PERCENTAGE,
            genesis: ethers::types::Bytes::default(),
        };

        log::info!("creating subnet on evm with params: {params:?}");

        let signer = self.get_signer(&from)?;
        let registry_contract =
            SubnetRegistry::new(self.ipc_contract_info.registry_addr, Arc::new(signer));

        let call = registry_contract.new_subnet_actor(params);
        let pending_tx = call.send().await?;
        // We need the retry to parse the deployment event. At the time of this writing, it's a bug
        // in current FEVM that without the retries, events are not picked up.
        // See https://github.com/filecoin-project/community/discussions/638 for more info and updates.
        let receipt = pending_tx.retries(TRANSACTION_RECEIPT_RETRIES).await?;
        match receipt {
            Some(r) => {
                for log in r.logs {
                    log::debug!("log: {log:?}");

                    match ethers_contract::parse_log::<subnet_registry::SubnetDeployedFilter>(log) {
                        Ok(subnet_deploy) => {
                            let subnet_registry::SubnetDeployedFilter { subnet_addr } =
                                subnet_deploy;

                            log::debug!("subnet deployed at {subnet_addr:?}");
                            return ethers_address_to_fil_address(&subnet_addr);
                        }
                        Err(_) => {
                            log::debug!("no event for subnet actor published yet, continue");
                            continue;
                        }
                    }
                }
                Err(anyhow!("no logs receipt"))
            }
            None => Err(anyhow!("no receipt for event, txn not successful")),
        }
    }

    async fn join_subnet(
        &self,
        subnet: SubnetID,
        from: Address,
        collateral: TokenAmount,
        validator_net_addr: String,
        worker_addr: Address,
    ) -> Result<()> {
        let collateral = collateral
            .atto()
            .to_u128()
            .ok_or_else(|| anyhow!("invalid min validator stake"))?;

        let address = contract_address_from_subnet(&subnet)?;
        log::info!(
            "interacting with evm subnet contract: {address:} with collateral: {collateral:}"
        );

        let signer = self.get_signer(&from)?;
        let contract = SubnetContract::new(address, Arc::new(signer));

        let mut txn = contract.join(
            validator_net_addr,
            subnet_contract::FvmAddress::from(worker_addr),
        );
        txn.tx.set_value(collateral);

        txn.send().await?.await?;

        Ok(())
    }

    async fn leave_subnet(&self, subnet: SubnetID, from: Address) -> Result<()> {
        let address = contract_address_from_subnet(&subnet)?;
        log::info!("leaving evm subnet: {subnet:} at contract: {address:}");

        let signer = self.get_signer(&from)?;
        let contract = SubnetContract::new(address, Arc::new(signer));

        contract.leave().send().await?.await?;

        Ok(())
    }

    async fn kill_subnet(&self, subnet: SubnetID, from: Address) -> Result<()> {
        let address = contract_address_from_subnet(&subnet)?;
        log::info!("kill evm subnet: {subnet:} at contract: {address:}");

        let signer = self.get_signer(&from)?;
        let contract = SubnetContract::new(address, Arc::new(signer));

        contract.kill().send().await?.await?;

        Ok(())
    }

    async fn list_child_subnets(
        &self,
        gateway_addr: Address,
    ) -> Result<HashMap<SubnetID, SubnetInfo>> {
        self.ensure_same_gateway(&gateway_addr)?;

        let gateway_contract = Gateway::new(
            self.ipc_contract_info.gateway_addr,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );

        let mut s = HashMap::new();

        let evm_subnets = gateway_contract.list_subnets().call().await?;
        for subnet in evm_subnets {
            let info = SubnetInfo::try_from(subnet)?;
            s.insert(info.id.clone(), info);
        }

        Ok(s)
    }

    async fn fund(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        to: Address,
        amount: TokenAmount,
    ) -> Result<ChainEpoch> {
        self.ensure_same_gateway(&gateway_addr)?;

        let value = amount
            .atto()
            .to_u128()
            .ok_or_else(|| anyhow!("invalid value to fund"))?;

        log::info!("fund with evm gateway contract: {gateway_addr:} with value: {value:}, original: {amount:?}");

        let evm_subnet_id = gateway::SubnetID::try_from(&subnet)?;
        log::debug!("evm subnet id to fund: {evm_subnet_id:?}");

        let signer = self.get_signer(&from)?;
        let gateway_contract = Gateway::new(self.ipc_contract_info.gateway_addr, Arc::new(signer));
        let mut txn = gateway_contract.fund(evm_subnet_id, gateway::FvmAddress::try_from(to)?);
        txn.tx.set_value(value);

        let pending_tx = txn.send().await?;
        let receipt = pending_tx.retries(TRANSACTION_RECEIPT_RETRIES).await?;
        block_number_from_receipt(receipt)
    }

    async fn release(
        &self,
        _subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        to: Address,
        amount: TokenAmount,
    ) -> Result<ChainEpoch> {
        self.ensure_same_gateway(&gateway_addr)?;

        let value = amount
            .atto()
            .to_u128()
            .ok_or_else(|| anyhow!("invalid value to fund"))?;

        log::info!("release with evm gateway contract: {gateway_addr:} with value: {value:}");

        let signer = self.get_signer(&from)?;
        let gateway_contract = Gateway::new(self.ipc_contract_info.gateway_addr, Arc::new(signer));
        let mut txn = gateway_contract.release(gateway::FvmAddress::try_from(to)?);
        txn.tx.set_value(value);

        let pending_tx = txn.send().await?;
        let receipt = pending_tx.retries(TRANSACTION_RECEIPT_RETRIES).await?;
        block_number_from_receipt(receipt)
    }

    async fn propagate(
        &self,
        _subnet: SubnetID,
        _gateway_addr: Address,
        _from: Address,
        _postbox_msg_cid: Cid,
    ) -> Result<()> {
        todo!()
    }

    async fn set_validator_net_addr(
        &self,
        _subnet: SubnetID,
        _from: Address,
        _validator_net_addr: String,
    ) -> Result<()> {
        todo!()
    }

    async fn whitelist_propagator(
        &self,
        _subnet: SubnetID,
        _gateway_addr: Address,
        _postbox_msg_cid: Cid,
        _from: Address,
        _to_add: Vec<Address>,
    ) -> Result<()> {
        todo!()
    }

    /// Send value between two addresses in a subnet
    async fn send_value(&self, from: Address, to: Address, amount: TokenAmount) -> Result<()> {
        let tx = TransactionRequest::new()
            .to(payload_to_evm_address(to.payload())?)
            .value(fil_to_eth_amount(&amount)?);

        let signer = self.get_signer(&from)?;
        let tx_pending = signer.send_transaction(tx, None).await?;

        log::info!(
            "sending FIL from {from:} to {to:} in tx {:?}",
            tx_pending.tx_hash()
        );
        Ok(())
    }

    async fn wallet_balance(&self, address: &Address) -> Result<TokenAmount> {
        let balance = self
            .ipc_contract_info
            .provider
            .clone()
            .get_balance(payload_to_evm_address(address.payload())?, None)
            .await?;
        Ok(TokenAmount::from_atto(balance.as_u128()))
    }

    async fn last_topdown_executed(&self, _gateway_addr: &Address) -> Result<ChainEpoch> {
        todo!()
    }

    async fn list_checkpoints(
        &self,
        _subnet_id: SubnetID,
        _from_epoch: ChainEpoch,
        _to_epoch: ChainEpoch,
    ) -> Result<Vec<BottomUpCheckpoint>> {
        todo!()
    }

    async fn get_validator_set(
        &self,
        subnet_id: &SubnetID,
        gateway: Address,
    ) -> Result<QueryValidatorSetResponse> {
        self.ensure_same_gateway(&gateway)?;

        // get genesis epoch from gateway
        let evm_subnet_id = gateway::SubnetID::try_from(subnet_id)?;
        log::debug!("evm subnet id: {evm_subnet_id:?}");

        let gateway_contract = Gateway::new(
            self.ipc_contract_info.gateway_addr,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );
        let (exists, evm_subnet) = gateway_contract.get_subnet(evm_subnet_id).call().await?;
        if !exists {
            return Err(anyhow!("subnet: {subnet_id:?} does not exists"));
        }
        let genesis_epoch = evm_subnet.genesis_epoch.as_u64() as i64;

        let min_validators = self.min_validators(subnet_id).await?;

        // get validator set
        let address = contract_address_from_subnet(subnet_id)?;
        log::debug!("get validator info for subnet: {subnet_id:} at contract: {address:}");

        let contract =
            SubnetContract::new(address, Arc::new(self.ipc_contract_info.provider.clone()));
        let evm_validator_set = contract.get_validator_set().call().await?;

        let mut validators = vec![];
        for v in evm_validator_set.validators.into_iter() {
            validators.push(Validator {
                // we are using worker address here so that the fvm validator node can pick up
                // the correct
                addr: Address::try_from(v.worker_addr.clone())?.to_string(),
                net_addr: v.net_addresses,
                worker_addr: Some(Address::try_from(v.worker_addr)?.to_string()),
                weight: v.weight.to_string(),
            });
        }
        let mut validator_set = ValidatorSet {
            validators: None,
            configuration_number: evm_validator_set.configuration_number,
        };
        if !validators.is_empty() {
            validator_set.validators = Some(validators);
        }

        Ok(QueryValidatorSetResponse {
            validator_set,
            min_validators,
            genesis_epoch,
        })
    }
}

#[async_trait]
impl EthManager for EthSubnetManager {
    async fn gateway_last_voting_executed_epoch(&self) -> Result<ChainEpoch> {
        let gateway_contract = Gateway::new(
            self.ipc_contract_info.gateway_addr,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );
        let u = gateway_contract.last_voting_executed_epoch().call().await?;
        Ok(u as ChainEpoch)
    }

    async fn subnet_last_voting_executed_epoch(&self, subnet_id: &SubnetID) -> Result<ChainEpoch> {
        let address = contract_address_from_subnet(subnet_id)?;
        let contract =
            SubnetContract::new(address, Arc::new(self.ipc_contract_info.provider.clone()));
        let u = contract.last_voting_executed_epoch().call().await?;
        Ok(u as ChainEpoch)
    }

    async fn current_epoch(&self) -> Result<ChainEpoch> {
        let block_number = self
            .ipc_contract_info
            .provider
            .get_block_number()
            .await?
            .as_u64();
        Ok(block_number as ChainEpoch)
    }

    async fn submit_top_down_checkpoint(
        &self,
        from: &Address,
        checkpoint: TopDownCheckpoint,
    ) -> Result<ChainEpoch> {
        log::debug!("submit top down checkpoint: {:?}", checkpoint);

        let signer = self.get_signer(from)?;
        let gateway_contract = Gateway::new(self.ipc_contract_info.gateway_addr, Arc::new(signer));

        let txn = gateway_contract.submit_top_down_checkpoint(checkpoint);
        let pending_tx = txn.send().await?;
        let receipt = pending_tx.retries(TRANSACTION_RECEIPT_RETRIES).await?;
        block_number_from_receipt(receipt)
    }

    async fn submit_bottom_up_checkpoint(
        &self,
        from: &Address,
        checkpoint: subnet_contract::BottomUpCheckpoint,
    ) -> Result<ChainEpoch> {
        let route = &checkpoint.source.route;

        log::debug!(
            "submit bottom up checkpoint: {:?} to address: {:?}",
            checkpoint,
            route[route.len() - 1]
        );

        let signer = self.get_signer(from)?;
        let contract = SubnetContract::new(route[route.len() - 1], Arc::new(signer));

        let txn = contract.submit_checkpoint(checkpoint);
        let pending_tx = txn.send().await?;
        let receipt = pending_tx.retries(TRANSACTION_RECEIPT_RETRIES).await?;
        block_number_from_receipt(receipt)
    }

    async fn has_voted_in_subnet(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
        validator: &Address,
    ) -> Result<bool> {
        let address = contract_address_from_subnet(subnet_id)?;
        let validator = payload_to_evm_address(validator.payload())?;

        let contract =
            SubnetContract::new(address, Arc::new(self.ipc_contract_info.provider.clone()));

        let has_voted = contract
            .has_validator_voted_for_submission(epoch as u64, validator)
            .call()
            .await?;
        Ok(has_voted)
    }

    async fn has_voted_in_gateway(&self, epoch: ChainEpoch, validator: &Address) -> Result<bool> {
        let address = payload_to_evm_address(validator.payload())?;

        let gateway_contract = Gateway::new(
            self.ipc_contract_info.gateway_addr,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );
        let has_voted = gateway_contract
            .has_validator_voted_for_submission(epoch as u64, address)
            .call()
            .await?;
        Ok(has_voted)
    }

    async fn bottom_up_checkpoint(
        &self,
        epoch: ChainEpoch,
    ) -> Result<subnet_contract::BottomUpCheckpoint> {
        let gateway_contract = Gateway::new(
            self.ipc_contract_info.gateway_addr,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );
        let (exists, checkpoint) = gateway_contract
            .bottom_up_checkpoint_at_epoch(epoch as u64)
            .call()
            .await?;
        if !exists {
            Err(anyhow!(
                "bottom up checkpoint not exists at epoch: {epoch:}"
            ))
        } else {
            let token = checkpoint.into_token();
            let c = subnet_contract::BottomUpCheckpoint::from_token(token)?;
            Ok(c)
        }
    }

    async fn top_down_msgs(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
        nonce: u64,
    ) -> Result<Vec<gateway::CrossMsg>> {
        let route = agent_subnet_to_evm_addresses(subnet_id)?;
        log::debug!("getting top down messages for route: {route:?}");

        let subnet_id = gateway::SubnetID {
            root: subnet_id.root_id(),
            route,
        };
        let gateway_contract = Gateway::new(
            self.ipc_contract_info.gateway_addr,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );
        let r = gateway_contract
            .method::<_, Vec<gateway::CrossMsg>>(
                "getTopDownMsgs",
                gateway::GetTopDownMsgsCall {
                    subnet_id,
                    from_nonce: nonce,
                },
            )
            .map_err(|e| anyhow!("cannot create the top down msg call: {e:}"))?
            .block(epoch as u64)
            .call()
            .await
            .map_err(|e| anyhow!("cannot get evm top down messages: {e:}"))?;
        Ok(r)
    }

    async fn validators(&self, subnet_id: &SubnetID) -> Result<Vec<Address>> {
        let address = contract_address_from_subnet(subnet_id)?;
        let contract =
            SubnetContract::new(address, Arc::new(self.ipc_contract_info.provider.clone()));

        let validators = contract.get_validators().call().await?;
        validators
            .iter()
            .map(ethers_address_to_fil_address)
            .collect::<Result<Vec<_>>>()
    }

    async fn gateway_initialized(&self) -> Result<bool> {
        let gateway_contract = Gateway::new(
            self.ipc_contract_info.gateway_addr,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );
        let initialized = gateway_contract.initialized().call().await?;
        Ok(initialized)
    }

    async fn subnet_bottom_up_checkpoint_period(&self, subnet_id: &SubnetID) -> Result<ChainEpoch> {
        let address = contract_address_from_subnet(subnet_id)?;
        let contract =
            SubnetContract::new(address, Arc::new(self.ipc_contract_info.provider.clone()));
        Ok(contract.bottom_up_check_period().call().await? as ChainEpoch)
    }

    async fn gateway_top_down_check_period(&self) -> Result<ChainEpoch> {
        let gateway_contract = Gateway::new(
            self.ipc_contract_info.gateway_addr,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );
        Ok(gateway_contract.top_down_check_period().call().await? as ChainEpoch)
    }

    async fn prev_bottom_up_checkpoint_hash(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
    ) -> Result<[u8; 32]> {
        let address = contract_address_from_subnet(subnet_id)?;
        let contract =
            SubnetContract::new(address, Arc::new(self.ipc_contract_info.provider.clone()));
        let (exists, hash) = contract
            .bottom_up_checkpoint_hash_at_epoch(epoch as u64)
            .await?;
        if !exists {
            return if epoch == 0 {
                // first ever epoch, return empty bytes
                return Ok([0u8; 32]);
            } else {
                Err(anyhow!("checkpoint does not exists: {epoch:}"))
            };
        }
        Ok(hash)
    }

    async fn min_validators(&self, subnet_id: &SubnetID) -> Result<u64> {
        let address = contract_address_from_subnet(subnet_id)?;
        let contract =
            SubnetContract::new(address, Arc::new(self.ipc_contract_info.provider.clone()));
        Ok(contract.min_validators().call().await?)
    }
}

impl EthSubnetManager {
    pub fn new(
        gateway_addr: ethers::types::Address,
        registry_addr: ethers::types::Address,
        chain_id: u64,
        provider: Provider<Http>,
        keystore: Arc<RwLock<PersistentKeyStore<ethers::types::Address>>>,
    ) -> Self {
        Self {
            keystore,
            ipc_contract_info: IPCContractInfo {
                gateway_addr,
                registry_addr,
                chain_id,
                provider,
            },
        }
    }

    pub fn ensure_same_gateway(&self, gateway: &Address) -> Result<()> {
        let evm_gateway_addr = payload_to_evm_address(gateway.payload())?;
        if evm_gateway_addr != self.ipc_contract_info.gateway_addr {
            Err(anyhow!("Gateway address not matching with config"))
        } else {
            Ok(())
        }
    }

    /// Get the ethers singer instance.
    /// We use filecoin addresses throughout our whole code-base
    /// and translate them to evm addresses when relevant.
    fn get_signer(&self, addr: &Address) -> Result<DefaultSignerMiddleware> {
        // convert to its underlying eth address
        let addr = payload_to_evm_address(addr.payload())?;
        let keystore = self.keystore.read().unwrap();
        let private_key = keystore
            .get(&addr)?
            .ok_or_else(|| anyhow!("address {addr:} does not have private key in key store"))?;
        let wallet = LocalWallet::from_bytes(private_key.private_key())?
            .with_chain_id(self.ipc_contract_info.chain_id);

        Ok(SignerMiddleware::new(
            self.ipc_contract_info.provider.clone(),
            wallet,
        ))
    }
}

impl EthSubnetManager {
    pub fn from_subnet_with_wallet_store(
        subnet: &Subnet,
        keystore: Arc<RwLock<PersistentKeyStore<ethers::types::Address>>>,
    ) -> Result<Self> {
        let url = subnet.rpc_http().clone();
        let auth_token = subnet.auth_token();

        let config = if let SubnetConfig::Fevm(config) = &subnet.config {
            config
        } else {
            return Err(anyhow!("not evm config"));
        };

        let provider = if auth_token.is_some() {
            Http::new_with_auth(url, Authorization::Bearer(auth_token.unwrap()))?
        } else {
            Http::new(url)
        };

        let provider = Provider::new(provider);
        let gateway_address = payload_to_evm_address(config.gateway_addr.payload())?;
        let registry_address = payload_to_evm_address(config.registry_addr.payload())?;

        Ok(Self::new(
            gateway_address,
            registry_address,
            subnet.id.chain_id(),
            provider,
            keystore,
        ))
    }
}

/// Get the block number from the transaction receipt
fn block_number_from_receipt(
    receipt: Option<ethers::types::TransactionReceipt>,
) -> Result<ChainEpoch> {
    match receipt {
        Some(r) => {
            let block_number = r
                .block_number
                .ok_or_else(|| anyhow!("cannot get block number"))?;
            Ok(block_number.as_u64() as ChainEpoch)
        }
        None => Err(anyhow!(
            "txn sent to network, but receipt cannot be obtained, please check scanner"
        )),
    }
}

/// Convert the ipc SubnetID type to an evm address. It extracts the last address from the Subnet id
/// children and turns it into evm address.
fn contract_address_from_subnet(subnet: &SubnetID) -> Result<ethers::types::Address> {
    let children = subnet.children();
    let ipc_addr = children
        .last()
        .ok_or_else(|| anyhow!("{subnet:} has no child"))?;

    payload_to_evm_address(ipc_addr.payload())
}

/// Convert the ipc SubnetID type to a vec of evm addresses. It extracts all the children addresses
/// in the subnet id and turns them as a vec of evm addresses.
pub(crate) fn agent_subnet_to_evm_addresses(
    subnet: &SubnetID,
) -> Result<Vec<ethers::types::Address>> {
    let children = subnet.children();
    children
        .iter()
        .map(|addr| payload_to_evm_address(addr.payload()))
        .collect::<Result<_>>()
}

/// Util function to convert Fil address payload to evm address. Only delegated address is supported.
pub(crate) fn payload_to_evm_address(payload: &Payload) -> Result<ethers::types::Address> {
    match payload {
        Payload::Delegated(delegated) => {
            let slice = delegated.subaddress();
            Ok(ethers::types::Address::from_slice(&slice[0..20]))
        }
        _ => Err(anyhow!("address provided is not delegated")),
    }
}

#[cfg(test)]
mod tests {
    use crate::manager::evm::manager::{
        agent_subnet_to_evm_addresses, contract_address_from_subnet,
    };
    use fvm_shared::address::Address;
    use ipc_sdk::subnet_id::SubnetID;
    use primitives::EthAddress;
    use std::str::FromStr;

    #[test]
    fn test_agent_subnet_to_evm_address() {
        let addr = Address::from_str("f410ffzyuupbyl2uiucmzr3lu3mtf3luyknthaz4xsrq").unwrap();
        let id = SubnetID::new(0, vec![addr]);

        let eth = contract_address_from_subnet(&id).unwrap();
        assert_eq!(
            format!("{eth:?}"),
            "0x2e714a3c385ea88a09998ed74db265dae9853667"
        );
    }

    #[test]
    fn test_agent_subnet_to_evm_addresses() {
        let eth_addr = EthAddress::from_str("0x0000000000000000000000000000000000000000").unwrap();
        let addr = Address::from(eth_addr);
        let addr2 = Address::from_str("f410ffzyuupbyl2uiucmzr3lu3mtf3luyknthaz4xsrq").unwrap();

        let id = SubnetID::new(0, vec![addr, addr2]);

        let addrs = agent_subnet_to_evm_addresses(&id).unwrap();

        let a =
            ethers::types::Address::from_str("0x0000000000000000000000000000000000000000").unwrap();
        let b =
            ethers::types::Address::from_str("0x2e714a3c385ea88a09998ed74db265dae9853667").unwrap();

        assert_eq!(addrs, vec![a, b]);
    }
}
