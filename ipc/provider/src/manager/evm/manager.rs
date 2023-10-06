// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use ipc_actors_abis::{
    gateway_getter_facet, gateway_manager_facet, gateway_messenger_facet,
    subnet_actor_getter_facet, subnet_actor_manager_facet, subnet_registry,
};
use ipc_sdk::evm::{fil_to_eth_amount, payload_to_evm_address, subnet_id_to_evm_addresses};
use ipc_sdk::{eth_to_fil_amount, ethers_address_to_fil_address};

use crate::config::subnet::SubnetConfig;
use crate::config::Subnet;
use crate::lotus::message::ipc::{QueryValidatorSetResponse, SubnetInfo, Validator, ValidatorSet};
use crate::manager::subnet::TopDownCheckpointQuery;
use crate::manager::{EthManager, SubnetManager};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use ethers::abi::Tokenizable;
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::{Signer, SignerMiddleware};
use ethers::providers::{Authorization, Http, Middleware, Provider};
use ethers::signers::{LocalWallet, Wallet};
use ethers::types::{Eip1559TransactionRequest, I256, U256};
use futures_util::StreamExt;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::{address::Address, econ::TokenAmount};
use ipc_identity::{EthKeyAddress, EvmKeyStore, PersistentKeyStore};
use ipc_sdk::cross::CrossMsg;
use ipc_sdk::gateway::Status;
use ipc_sdk::staking::{NewStakingRequest, StakingChangeRequest};
use ipc_sdk::subnet::ConstructParams;
use ipc_sdk::subnet_id::SubnetID;
use num_traits::ToPrimitive;

pub type DefaultSignerMiddleware = SignerMiddleware<Provider<Http>, Wallet<SigningKey>>;

/// Default polling time used by the Ethers provider to check for pending
/// transactions and events. Default is 7, and for our child subnets we
/// can reduce it to the block time (or potentially less)
const ETH_PROVIDER_POLLING_TIME: Duration = Duration::from_secs(1);
/// Maximum number of retries to fetch a transaction receipt.
/// The number of retries should ensure that for the block time
/// of the network the number of retires considering the polling
/// time above waits enough tie to get the transaction receipt.
/// We currently support a low polling time and high number of
/// retries so these numbers accommodate fast subnets with slow
/// roots (like Calibration and mainnet).
const TRANSACTION_RECEIPT_RETRIES: usize = 200;

/// The majority vote percentage for checkpoint submission when creating a subnet.
const SUBNET_MAJORITY_PERCENTAGE: u8 = 60;
const SUBNET_NAME_MAX_LEN: usize = 32;

pub struct EthSubnetManager {
    keystore: Arc<RwLock<PersistentKeyStore<EthKeyAddress>>>,
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
impl TopDownCheckpointQuery for EthSubnetManager {
    async fn genesis_epoch(&self, subnet_id: &SubnetID) -> Result<ChainEpoch> {
        let address = contract_address_from_subnet(subnet_id)?;
        log::info!("querying genesis epoch in evm subnet contract: {address:}");

        let evm_subnet_id = gateway_getter_facet::SubnetID::try_from(subnet_id)?;

        let contract = gateway_getter_facet::GatewayGetterFacet::new(
            self.ipc_contract_info.gateway_addr,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );
        let (exists, subnet) = contract.get_subnet(evm_subnet_id).call().await?;
        if !exists {
            return Err(anyhow!("subnet: {} does not exists", subnet_id));
        }
        Ok(subnet.genesis_epoch.as_u64() as ChainEpoch)
    }

    async fn chain_head_height(&self) -> Result<ChainEpoch> {
        let block = self
            .ipc_contract_info
            .provider
            .get_block_number()
            .await
            .context("cannot get evm block number")?;
        Ok(block.as_u64() as ChainEpoch)
    }

    async fn get_top_down_msgs(
        &self,
        subnet_id: &SubnetID,
        start_epoch: ChainEpoch,
        end_epoch: ChainEpoch,
    ) -> Result<Vec<ipc_sdk::cross::CrossMsg>> {
        self.top_down_msgs(subnet_id, start_epoch, end_epoch).await
    }

    async fn get_block_hash(&self, height: ChainEpoch) -> Result<Vec<u8>> {
        let block = self
            .ipc_contract_info
            .provider
            .get_block(height as u64)
            .await?
            .ok_or_else(|| anyhow!("height does not exist"))?;
        Ok(block
            .hash
            .ok_or_else(|| anyhow!("block hash is empty"))?
            .to_fixed_bytes()
            .to_vec())
    }

    async fn get_validator_changeset(
        &self,
        subnet_id: &SubnetID,
        start: ChainEpoch,
        end: ChainEpoch,
    ) -> Result<Vec<StakingChangeRequest>> {
        let address = contract_address_from_subnet(subnet_id)?;
        log::info!("querying validator changes in evm subnet contract: {address:}");

        let contract = subnet_actor_manager_facet::SubnetActorManagerFacet::new(
            address,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );

        let ev = contract
            .event::<NewStakingRequest>()
            .from_block(start as u64)
            .to_block(end as u64);
        let mut event_stream = ev.stream().await?;

        let mut changes = vec![];
        while let Some(Ok(event)) = event_stream.next().await {
            changes.push(StakingChangeRequest::try_from(event)?);
        }

        Ok(changes)
    }
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

        let route = subnet_id_to_evm_addresses(&params.parent)?;
        log::debug!("root SubnetID as Ethereum type: {route:?}");

        let params = subnet_registry::ConstructorParams {
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
        };

        log::info!("creating subnet on evm with params: {params:?}");

        let signer = self.get_signer(&from)?;
        let signer = Arc::new(signer);
        let registry_contract = subnet_registry::SubnetRegistry::new(
            self.ipc_contract_info.registry_addr,
            signer.clone(),
        );

        let call = call_with_premium_estimation(signer, registry_contract.new_subnet_actor(params))
            .await?;
        // TODO: Edit call to get estimate premium
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

        let signer = Arc::new(self.get_signer(&from)?);
        let contract =
            subnet_actor_manager_facet::SubnetActorManagerFacet::new(address, signer.clone());

        let mut txn = contract.join(
            validator_net_addr,
            subnet_actor_manager_facet::FvmAddress::try_from(worker_addr)?,
        );
        txn.tx.set_value(collateral);
        let txn = call_with_premium_estimation(signer, txn).await?;

        txn.send().await?.await?;

        Ok(())
    }

    async fn leave_subnet(&self, subnet: SubnetID, from: Address) -> Result<()> {
        let address = contract_address_from_subnet(&subnet)?;
        log::info!("leaving evm subnet: {subnet:} at contract: {address:}");

        let signer = Arc::new(self.get_signer(&from)?);
        let contract =
            subnet_actor_manager_facet::SubnetActorManagerFacet::new(address, signer.clone());

        call_with_premium_estimation(signer, contract.leave())
            .await?
            .send()
            .await?
            .await?;

        Ok(())
    }

    async fn kill_subnet(&self, subnet: SubnetID, from: Address) -> Result<()> {
        let address = contract_address_from_subnet(&subnet)?;
        log::info!("kill evm subnet: {subnet:} at contract: {address:}");

        let signer = Arc::new(self.get_signer(&from)?);
        let contract =
            subnet_actor_manager_facet::SubnetActorManagerFacet::new(address, signer.clone());

        call_with_premium_estimation(signer, contract.kill())
            .await?
            .send()
            .await?
            .await?;

        Ok(())
    }

    async fn list_child_subnets(
        &self,
        gateway_addr: Address,
    ) -> Result<HashMap<SubnetID, SubnetInfo>> {
        self.ensure_same_gateway(&gateway_addr)?;

        let gateway_contract = gateway_getter_facet::GatewayGetterFacet::new(
            self.ipc_contract_info.gateway_addr,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );

        let mut s = HashMap::new();

        let evm_subnets = gateway_contract.list_subnets().call().await?;
        log::debug!("raw subnet: {evm_subnets:?}");

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

        let evm_subnet_id = gateway_manager_facet::SubnetID::try_from(&subnet)?;
        log::debug!("evm subnet id to fund: {evm_subnet_id:?}");

        let signer = Arc::new(self.get_signer(&from)?);
        let gateway_contract = gateway_manager_facet::GatewayManagerFacet::new(
            self.ipc_contract_info.gateway_addr,
            signer.clone(),
        );

        let mut txn = gateway_contract.fund(
            evm_subnet_id,
            gateway_manager_facet::FvmAddress::try_from(to)?,
        );
        txn.tx.set_value(value);
        let txn = call_with_premium_estimation(signer, txn).await?;

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

        let signer = Arc::new(self.get_signer(&from)?);
        let gateway_contract = gateway_manager_facet::GatewayManagerFacet::new(
            self.ipc_contract_info.gateway_addr,
            signer.clone(),
        );
        let mut txn = gateway_contract.release(gateway_manager_facet::FvmAddress::try_from(to)?);
        txn.tx.set_value(value);
        let txn = call_with_premium_estimation(signer, txn).await?;

        let pending_tx = txn.send().await?;
        let receipt = pending_tx.retries(TRANSACTION_RECEIPT_RETRIES).await?;
        block_number_from_receipt(receipt)
    }

    /// Propagate the postbox message key. The key should be `bytes32`.
    async fn propagate(
        &self,
        _subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        postbox_msg_key: Vec<u8>,
    ) -> Result<()> {
        if postbox_msg_key.len() != 32 {
            return Err(anyhow!(
                "invalid message cid length, expect 32 but found {}",
                postbox_msg_key.len()
            ));
        }

        self.ensure_same_gateway(&gateway_addr)?;

        log::info!("propagate postbox evm gateway contract: {gateway_addr:} with message key: {postbox_msg_key:?}");

        let signer = Arc::new(self.get_signer(&from)?);
        let gateway_contract = gateway_messenger_facet::GatewayMessengerFacet::new(
            self.ipc_contract_info.gateway_addr,
            signer.clone(),
        );

        let mut key = [0u8; 32];
        key.copy_from_slice(&postbox_msg_key);

        call_with_premium_estimation(signer, gateway_contract.propagate(key))
            .await?
            .send()
            .await?;

        Ok(())
    }

    async fn send_cross_message(
        &self,
        gateway_addr: Address,
        from: Address,
        cross_msg: CrossMsg,
    ) -> Result<()> {
        self.ensure_same_gateway(&gateway_addr)?;

        log::info!("send evm cross messages to gateway contract: {gateway_addr:} with message: {cross_msg:?}");

        let signer = Arc::new(self.get_signer(&from)?);
        let gateway_contract = gateway_messenger_facet::GatewayMessengerFacet::new(
            self.ipc_contract_info.gateway_addr,
            signer.clone(),
        );

        let evm_cross_msg = gateway_messenger_facet::CrossMsg::try_from(cross_msg)?;
        call_with_premium_estimation(signer, gateway_contract.send_cross_message(evm_cross_msg))
            .await?
            .send()
            .await?;

        Ok(())
    }

    async fn set_validator_net_addr(
        &self,
        subnet: SubnetID,
        from: Address,
        net_addr: String,
    ) -> Result<()> {
        let address = contract_address_from_subnet(&subnet)?;
        log::info!(
            "set validator net addr: {net_addr:} on evm subnet: {subnet:} at contract: {address:}"
        );

        let signer = Arc::new(self.get_signer(&from)?);
        let contract =
            subnet_actor_manager_facet::SubnetActorManagerFacet::new(address, signer.clone());

        let txn = contract.set_validator_net_addr(net_addr);

        let txn = call_with_premium_estimation(signer, txn).await?;

        txn.send().await?.await?;

        Ok(())
    }

    async fn set_validator_worker_addr(
        &self,
        subnet: SubnetID,
        from: Address,
        worker_addr: Address,
    ) -> Result<()> {
        let address = contract_address_from_subnet(&subnet)?;
        log::info!("set validator worker addr: {worker_addr:} on evm subnet: {subnet:} at contract: {address:}");

        let signer = Arc::new(self.get_signer(&from)?);
        let contract =
            subnet_actor_manager_facet::SubnetActorManagerFacet::new(address, signer.clone());

        let txn = contract.set_validator_worker_addr(
            subnet_actor_manager_facet::FvmAddress::try_from(worker_addr)?,
        );

        let txn = call_with_premium_estimation(signer, txn).await?;

        txn.send().await?.await?;

        Ok(())
    }

    /// Send value between two addresses in a subnet
    async fn send_value(&self, from: Address, to: Address, amount: TokenAmount) -> Result<()> {
        let signer = Arc::new(self.get_signer(&from)?);
        let (fee, fee_cap) = premium_estimation(signer.clone()).await?;
        let tx = Eip1559TransactionRequest::new()
            .to(payload_to_evm_address(to.payload())?)
            .value(fil_to_eth_amount(&amount)?)
            .max_priority_fee_per_gas(fee)
            .max_fee_per_gas(fee_cap);

        let tx_pending = signer.send_transaction(tx, None).await?;

        log::info!(
            "sending FIL from {from:} to {to:} in tx {:?}",
            tx_pending.tx_hash()
        );
        tx_pending.await?;
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

    async fn get_validator_set(
        &self,
        subnet_id: &SubnetID,
        gateway: Option<Address>,
        epoch: Option<ChainEpoch>,
    ) -> Result<QueryValidatorSetResponse> {
        // we do optionally check as gateway addr is already part of the struct
        if let Some(addr) = gateway {
            self.ensure_same_gateway(&addr)?;
        }

        // get genesis epoch from gateway
        let evm_subnet_id = gateway_getter_facet::SubnetID::try_from(subnet_id)?;
        log::debug!("evm subnet id: {evm_subnet_id:?}");

        let gateway_contract = gateway_getter_facet::GatewayGetterFacet::new(
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

        let contract = subnet_actor_getter_facet::SubnetActorGetterFacet::new(
            address,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );
        let evm_validator_set = if let Some(epoch) = epoch {
            contract
                .get_validator_set()
                .block(epoch as u64)
                .call()
                .await?
        } else {
            contract.get_validator_set().call().await?
        };

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

    async fn get_chain_id(&self) -> Result<String> {
        Ok(self
            .ipc_contract_info
            .provider
            .get_chainid()
            .await?
            .to_string())
    }
}

#[async_trait]
impl EthManager for EthSubnetManager {
    async fn current_epoch(&self) -> Result<ChainEpoch> {
        let block_number = self
            .ipc_contract_info
            .provider
            .get_block_number()
            .await?
            .as_u64();
        Ok(block_number as ChainEpoch)
    }

    async fn bottom_up_checkpoint(
        &self,
        epoch: ChainEpoch,
    ) -> Result<subnet_actor_manager_facet::BottomUpCheckpoint> {
        let gateway_contract = gateway_getter_facet::GatewayGetterFacet::new(
            self.ipc_contract_info.gateway_addr,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );
        let checkpoint = gateway_contract
            .bottom_up_checkpoint(epoch as u64)
            .call()
            .await?;
        log::debug!("raw bottom up checkpoint from gateway: {checkpoint:?}");
        let token = checkpoint.into_token();
        let c = subnet_actor_manager_facet::BottomUpCheckpoint::from_token(token)?;
        Ok(c)
    }

    async fn get_applied_top_down_nonce(&self, subnet_id: &SubnetID) -> Result<u64> {
        let route = subnet_id_to_evm_addresses(subnet_id)?;
        log::debug!("getting applied top down nonce for route: {route:?}");

        let evm_subnet_id = gateway_getter_facet::SubnetID {
            root: subnet_id.root_id(),
            route,
        };

        let gateway_contract = gateway_getter_facet::GatewayGetterFacet::new(
            self.ipc_contract_info.gateway_addr,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );

        let (exists, nonce) = gateway_contract
            .get_applied_top_down_nonce(evm_subnet_id)
            .call()
            .await
            .map_err(|e| anyhow!("cannot get applied top down nonce due to: {e:}"))?;

        if !exists {
            Err(anyhow!("subnet {:?} does not exists", subnet_id))
        } else {
            Ok(nonce)
        }
    }

    async fn top_down_msgs(
        &self,
        subnet_id: &SubnetID,
        start_epoch: ChainEpoch,
        end_epoch: ChainEpoch,
    ) -> Result<Vec<ipc_sdk::cross::CrossMsg>> {
        let route = subnet_id_to_evm_addresses(subnet_id)?;
        log::debug!("getting top down messages for route: {route:?}");

        let subnet_id = gateway_getter_facet::SubnetID {
            root: subnet_id.root_id(),
            route,
        };
        let gateway_contract = gateway_getter_facet::GatewayGetterFacet::new(
            self.ipc_contract_info.gateway_addr,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );
        let raw_msgs = gateway_contract
            .method::<_, Vec<gateway_getter_facet::CrossMsg>>(
                "getTopDownMsgs",
                gateway_getter_facet::GetTopDownMsgsCall {
                    subnet_id,
                    from_block: U256::from(start_epoch),
                    to_block: U256::from(end_epoch),
                },
            )
            .map_err(|e| anyhow!("cannot create the top down msg call: {e:}"))?
            .call()
            .await
            .map_err(|e| anyhow!("cannot get evm top down messages: {e:}"))?;

        let mut msgs = vec![];
        for c in raw_msgs {
            msgs.push(ipc_sdk::cross::CrossMsg::try_from(c)?);
        }
        Ok(msgs)
    }

    async fn validators(&self, subnet_id: &SubnetID) -> Result<Vec<Address>> {
        let address = contract_address_from_subnet(subnet_id)?;
        let contract = subnet_actor_getter_facet::SubnetActorGetterFacet::new(
            address,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );

        let validators = contract.get_validators().call().await?;
        validators
            .iter()
            .map(ethers_address_to_fil_address)
            .collect::<Result<Vec<_>>>()
    }

    async fn gateway_initialized(&self) -> Result<bool> {
        let gateway_contract = gateway_getter_facet::GatewayGetterFacet::new(
            self.ipc_contract_info.gateway_addr,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );
        let initialized = gateway_contract.initialized().call().await?;
        Ok(initialized)
    }

    async fn subnet_bottom_up_checkpoint_period(&self, subnet_id: &SubnetID) -> Result<ChainEpoch> {
        let address = contract_address_from_subnet(subnet_id)?;
        let contract = subnet_actor_getter_facet::SubnetActorGetterFacet::new(
            address,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );
        Ok(contract.bottom_up_check_period().call().await? as ChainEpoch)
    }

    async fn gateway_top_down_check_period(&self) -> Result<ChainEpoch> {
        let gateway_contract = gateway_getter_facet::GatewayGetterFacet::new(
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
        let contract = subnet_actor_getter_facet::SubnetActorGetterFacet::new(
            address,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );
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
        let contract = subnet_actor_getter_facet::SubnetActorGetterFacet::new(
            address,
            Arc::new(self.ipc_contract_info.provider.clone()),
        );
        Ok(contract.min_validators().call().await?)
    }
}

impl EthSubnetManager {
    pub fn new(
        gateway_addr: ethers::types::Address,
        registry_addr: ethers::types::Address,
        chain_id: u64,
        provider: Provider<Http>,
        keystore: Arc<RwLock<PersistentKeyStore<EthKeyAddress>>>,
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
            .get(&addr.into())?
            .ok_or_else(|| anyhow!("address {addr:} does not have private key in key store"))?;
        let wallet = LocalWallet::from_bytes(private_key.private_key())?
            .with_chain_id(self.ipc_contract_info.chain_id);

        Ok(SignerMiddleware::new(
            self.ipc_contract_info.provider.clone(),
            wallet,
        ))
    }

    pub fn from_subnet_with_wallet_store(
        subnet: &Subnet,
        keystore: Arc<RwLock<PersistentKeyStore<EthKeyAddress>>>,
    ) -> Result<Self> {
        let url = subnet.rpc_http().clone();
        let auth_token = subnet.auth_token();

        let SubnetConfig::Fevm(config) = &subnet.config;

        let provider = if auth_token.is_some() {
            Http::new_with_auth(url, Authorization::Bearer(auth_token.unwrap()))?
        } else {
            Http::new(url)
        };

        let mut provider = Provider::new(provider);
        // set polling interval for provider to fit fast child subnets block times.
        // TODO: We may want to make it dynamic so it adjusts depending on the type of network
        // so we don't have a too slow or too fast polling for the underlying block times.
        provider.set_interval(ETH_PROVIDER_POLLING_TIME);
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

/// Receives an input `FunctionCall` and returns a new instance
/// after estimating an optimal `gas_premium` for the transaction
async fn call_with_premium_estimation<B, D, M>(
    signer: Arc<DefaultSignerMiddleware>,
    call: ethers_contract::FunctionCall<B, D, M>,
) -> Result<ethers_contract::FunctionCall<B, D, M>>
where
    B: std::borrow::Borrow<D>,
    M: ethers::abi::Detokenize,
{
    let (max_priority_fee_per_gas, _) = premium_estimation(signer).await?;
    Ok(call.gas_price(max_priority_fee_per_gas))
}

/// Returns an estimation of an optimal `gas_premium` and `gas_fee_cap`
/// for a transaction considering the average premium, base_fee and reward percentile from
/// past blocks
/// This is adaptation of ethers' `eip1559_default_estimator`:
/// https://github.com/gakonst/ethers-rs/blob/5dcd3b7e754174448f9a8cbfc0523896609629f9/ethers-core/src/utils/mod.rs#L476
async fn premium_estimation(
    signer: Arc<DefaultSignerMiddleware>,
) -> Result<(ethers::types::U256, ethers::types::U256)> {
    let base_fee_per_gas = signer
        .get_block(ethers::types::BlockNumber::Latest)
        .await?
        .ok_or_else(|| anyhow!("Latest block not found"))?
        .base_fee_per_gas
        .ok_or_else(|| anyhow!("EIP-1559 not activated"))?;

    let fee_history = signer
        .fee_history(
            ethers::utils::EIP1559_FEE_ESTIMATION_PAST_BLOCKS,
            ethers::types::BlockNumber::Latest,
            &[ethers::utils::EIP1559_FEE_ESTIMATION_REWARD_PERCENTILE],
        )
        .await?;

    let max_priority_fee_per_gas = estimate_priority_fee(fee_history.reward); //overestimate?
    let potential_max_fee = base_fee_surged(base_fee_per_gas);
    let max_fee_per_gas = if max_priority_fee_per_gas > potential_max_fee {
        max_priority_fee_per_gas + potential_max_fee
    } else {
        potential_max_fee
    };

    Ok((max_priority_fee_per_gas, max_fee_per_gas))
}

/// Implementation borrowed from
/// https://github.com/gakonst/ethers-rs/blob/ethers-v2.0.8/ethers-core/src/utils/mod.rs#L582
/// Refer to the implementation for unit tests
fn base_fee_surged(base_fee_per_gas: U256) -> U256 {
    if base_fee_per_gas <= U256::from(40_000_000_000u64) {
        base_fee_per_gas * 2
    } else if base_fee_per_gas <= U256::from(100_000_000_000u64) {
        base_fee_per_gas * 16 / 10
    } else if base_fee_per_gas <= U256::from(200_000_000_000u64) {
        base_fee_per_gas * 14 / 10
    } else {
        base_fee_per_gas * 12 / 10
    }
}

/// Implementation borrowed from
/// https://github.com/gakonst/ethers-rs/blob/ethers-v2.0.8/ethers-core/src/utils/mod.rs#L536
/// Refer to the implementation for unit tests
fn estimate_priority_fee(rewards: Vec<Vec<U256>>) -> U256 {
    let mut rewards: Vec<U256> = rewards
        .iter()
        .map(|r| r[0])
        .filter(|r| *r > U256::zero())
        .collect();
    if rewards.is_empty() {
        return U256::zero();
    }
    if rewards.len() == 1 {
        return rewards[0];
    }
    // Sort the rewards as we will eventually take the median.
    rewards.sort();

    // A copy of the same vector is created for convenience to calculate percentage change
    // between subsequent fee values.
    let mut rewards_copy = rewards.clone();
    rewards_copy.rotate_left(1);

    let mut percentage_change: Vec<I256> = rewards
        .iter()
        .zip(rewards_copy.iter())
        .map(|(a, b)| {
            let a = I256::try_from(*a).expect("priority fee overflow");
            let b = I256::try_from(*b).expect("priority fee overflow");
            ((b - a) * 100) / a
        })
        .collect();
    percentage_change.pop();

    // Fetch the max of the percentage change, and that element's index.
    let max_change = percentage_change.iter().max().unwrap();
    let max_change_index = percentage_change
        .iter()
        .position(|&c| c == *max_change)
        .unwrap();

    // If we encountered a big change in fees at a certain position, then consider only
    // the values >= it.
    let values = if *max_change >= ethers::utils::EIP1559_FEE_ESTIMATION_THRESHOLD_MAX_CHANGE.into()
        && (max_change_index >= (rewards.len() / 2))
    {
        rewards[max_change_index..].to_vec()
    } else {
        rewards
    };

    // Return the median.
    values[values.len() / 2]
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

impl TryFrom<gateway_getter_facet::Subnet> for SubnetInfo {
    type Error = anyhow::Error;

    fn try_from(value: gateway_getter_facet::Subnet) -> Result<Self, Self::Error> {
        Ok(SubnetInfo {
            id: SubnetID::try_from(value.id)?,
            stake: eth_to_fil_amount(&value.stake)?,
            circ_supply: eth_to_fil_amount(&value.circ_supply)?,
            status: match value.status {
                1 => Status::Active,
                2 => Status::Inactive,
                3 => Status::Killed,
                _ => return Err(anyhow!("invalid status: {:}", value.status)),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::manager::evm::manager::contract_address_from_subnet;
    use fvm_shared::address::Address;
    use ipc_sdk::subnet_id::SubnetID;
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
}
