// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use anyhow::anyhow;
use async_trait::async_trait;
use ethers::abi::{Function, FunctionExt, Tokenizable};
use ethers::types::{Selector, U256};
use ethers_contract::encode_function_data;
use ipc_actors_abis::register_subnet_facet;
use ipc_api::evm::payload_to_evm_address;
use std::collections::btree_map::BTreeMap;
use std::collections::HashMap;

use crate::config::serialize::{serialize_address_to_str, serialize_bytes_to_str};
use crate::manager::{BottomUpCheckpointRelayer, EthSubnetManager, GetBlockHashResult, SubnetGenesisInfo, SubnetInfo, SubnetManager, TopDownFinalityQuery, TopDownQueryPayload};
use crate::VMType;
use fvm_ipld_encoding::{BytesSer, RawBytes};
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use fvm_shared::MethodNum;
use ipc_api::checkpoint::{
    BottomUpCheckpoint, BottomUpCheckpointBundle, QuorumReachedEvent, Signature,
};
use ipc_api::cross::IpcEnvelope;
use ipc_api::eth_to_fil_amount;
use ipc_api::ethers_address_to_fil_address;
use ipc_api::staking::{StakingChangeRequest, ValidatorInfo};
use ipc_api::subnet::ConstructParams;
use ipc_api::subnet_id::SubnetID;
use serde::Serialize;

const INVOKE_CONTRACT: MethodNum = 3844450837;

#[derive(Serialize)]
struct MockedFVMTxn {
    #[serde(serialize_with = "serialize_address_to_str")]
    from: Address,
    #[serde(serialize_with = "serialize_address_to_str")]
    to: Address,
    /// The value, display as string to align with evm display
    value: String,
    #[serde(serialize_with = "serialize_bytes_to_str")]
    method_params: Vec<u8>,
    method: MethodNum,
}

#[derive(Serialize)]
struct MockedEVMTxn {
    from: ethers::types::Address,
    to: ethers::types::Address,
    value: U256,
    #[serde(serialize_with = "serialize_bytes_to_str")]
    calldata: Vec<u8>,
    #[serde(serialize_with = "serialize_bytes_to_str")]
    method: Selector,
}

impl TryFrom<MockedEVMTxn> for MockedFVMTxn {
    type Error = anyhow::Error;

    fn try_from(value: MockedEVMTxn) -> Result<Self, Self::Error> {
        Ok(Self {
            from: ethers_address_to_fil_address(&value.from)?,
            to: ethers_address_to_fil_address(&value.to)?,
            value: eth_to_fil_amount(&value.value)?.to_string(),
            method_params: to_fvm_calldata(&value.calldata)?,
            method: INVOKE_CONTRACT,
        })
    }
}

/// EVM subnet dry run handler. For call transactions, it will directly execute the transaction and
/// send to the chain to fetch actual data. For send transactions, it prints the payload to console.
pub struct EvmSubnetDryRun {
    /// The vm type that determines the message format
    vm_type: VMType,
    /// The subnet manager caller
    caller: EthSubnetManager,
}

#[async_trait]
impl SubnetManager for EvmSubnetDryRun {
    async fn create_subnet(
        &self,
        from: Address,
        params: ConstructParams,
    ) -> anyhow::Result<Address> {
        let evm = self.create_subnet_evm(&from, params)?;

        let s = match self.vm_type {
            VMType::Evm => serde_json::to_string(&evm)?,
            VMType::Fvm => serde_json::to_string(&MockedFVMTxn::try_from(evm)?)?,
        };

        println!("txn to send: \n {}", s);
        Ok(Address::new_id(0))
    }

    async fn join_subnet(
        &self,
        _subnet: SubnetID,
        _from: Address,
        _collateral: TokenAmount,
        _metadata: Vec<u8>,
    ) -> anyhow::Result<ChainEpoch> {
        self.todo()
    }

    async fn pre_fund(
        &self,
        _subnet: SubnetID,
        _from: Address,
        _balance: TokenAmount,
    ) -> anyhow::Result<()> {
        self.todo()
    }

    async fn pre_release(
        &self,
        _subnet: SubnetID,
        _from: Address,
        _amount: TokenAmount,
    ) -> anyhow::Result<()> {
        self.todo()
    }

    async fn stake(
        &self,
        _subnet: SubnetID,
        _from: Address,
        _collateral: TokenAmount,
    ) -> anyhow::Result<()> {
        self.todo()
    }

    async fn unstake(
        &self,
        _subnet: SubnetID,
        _from: Address,
        _collateral: TokenAmount,
    ) -> anyhow::Result<()> {
        self.todo()
    }

    async fn leave_subnet(&self, _subnet: SubnetID, _from: Address) -> anyhow::Result<()> {
        self.todo()
    }

    async fn kill_subnet(&self, _subnet: SubnetID, _from: Address) -> anyhow::Result<()> {
        self.todo()
    }

    async fn list_child_subnets(
        &self,
        _gateway_addr: Address,
    ) -> anyhow::Result<HashMap<SubnetID, SubnetInfo>> {
        self.todo()
    }

    async fn claim_collateral(&self, _subnet: SubnetID, _from: Address) -> anyhow::Result<()> {
        self.todo()
    }

    async fn fund(
        &self,
        _subnet: SubnetID,
        _gateway_addr: Address,
        _from: Address,
        _to: Address,
        _amount: TokenAmount,
    ) -> anyhow::Result<ChainEpoch> {
        self.todo()
    }

    async fn fund_with_token(
        &self,
        _subnet: SubnetID,
        _from: Address,
        _to: Address,
        _amount: TokenAmount,
    ) -> anyhow::Result<ChainEpoch> {
        self.todo()
    }

    async fn release(
        &self,
        _gateway_addr: Address,
        _from: Address,
        _to: Address,
        _amount: TokenAmount,
    ) -> anyhow::Result<ChainEpoch> {
        self.todo()
    }

    async fn propagate(
        &self,
        _subnet: SubnetID,
        _gateway_addr: Address,
        _from: Address,
        _postbox_msg_key: Vec<u8>,
    ) -> anyhow::Result<()> {
        self.todo()
    }

    async fn send_value(
        &self,
        _from: Address,
        _to: Address,
        _amount: TokenAmount,
    ) -> anyhow::Result<()> {
        self.todo()
    }

    async fn wallet_balance(&self, address: &Address) -> anyhow::Result<TokenAmount> {
        self.caller.wallet_balance(address).await
    }

    async fn get_chain_id(&self) -> anyhow::Result<String> {
        self.caller.get_chain_id().await
    }

    async fn get_commit_sha(&self) -> anyhow::Result<[u8; 32]> {
        self.caller.get_commit_sha().await
    }

    async fn get_genesis_info(&self, subnet: &SubnetID) -> anyhow::Result<SubnetGenesisInfo> {
        self.caller.get_genesis_info(subnet).await
    }

    async fn add_bootstrap(
        &self,
        _subnet: &SubnetID,
        _from: &Address,
        _endpoint: String,
    ) -> anyhow::Result<()> {
        self.todo()
    }

    async fn list_bootstrap_nodes(&self, subnet: &SubnetID) -> anyhow::Result<Vec<String>> {
        self.caller.list_bootstrap_nodes(subnet).await
    }

    async fn get_validator_info(
        &self,
        subnet: &SubnetID,
        validator: &Address,
    ) -> anyhow::Result<ValidatorInfo> {
        self.caller.get_validator_info(subnet, validator).await
    }

    async fn set_federated_power(
        &self,
        _from: &Address,
        _subnet: &SubnetID,
        _validators: &[Address],
        _public_keys: &[Vec<u8>],
        _federated_power: &[u128],
    ) -> anyhow::Result<ChainEpoch> {
        self.todo()
    }
}

#[async_trait]
impl BottomUpCheckpointRelayer for EvmSubnetDryRun {
    async fn submit_checkpoint(
        &self,
        _submitter: &Address,
        _checkpoint: BottomUpCheckpoint,
        _signatures: Vec<Signature>,
        _signatories: Vec<Address>,
    ) -> anyhow::Result<ChainEpoch> {
        self.todo()
    }

    async fn last_bottom_up_checkpoint_height(
        &self,
        subnet_id: &SubnetID,
    ) -> anyhow::Result<ChainEpoch> {
        self.caller.last_bottom_up_checkpoint_height(subnet_id).await
    }

    async fn checkpoint_period(&self, subnet_id: &SubnetID) -> anyhow::Result<ChainEpoch> {
        self.caller.checkpoint_period(subnet_id).await
    }

    async fn checkpoint_bundle_at(
        &self,
        height: ChainEpoch,
    ) -> anyhow::Result<BottomUpCheckpointBundle> {
        self.caller.checkpoint_bundle_at(height).await
    }

    async fn quorum_reached_events(
        &self,
        height: ChainEpoch,
    ) -> anyhow::Result<Vec<QuorumReachedEvent>> {
        self.caller.quorum_reached_events(height).await
    }

    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch> {
        BottomUpCheckpointRelayer::current_epoch(&self.caller).await
    }
}

#[async_trait]
impl TopDownFinalityQuery for EvmSubnetDryRun {
    async fn genesis_epoch(&self, subnet_id: &SubnetID) -> anyhow::Result<ChainEpoch> {
        self.caller.genesis_epoch(subnet_id).await
    }

    async fn chain_head_height(&self) -> anyhow::Result<ChainEpoch> {
        self.caller.chain_head_height().await
    }

    async fn get_top_down_msgs(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
    ) -> anyhow::Result<TopDownQueryPayload<Vec<IpcEnvelope>>> {
        self.caller.get_top_down_msgs(subnet_id, epoch).await
    }

    async fn get_block_hash(&self, height: ChainEpoch) -> anyhow::Result<GetBlockHashResult> {
        self.caller.get_block_hash(height).await
    }

    async fn get_validator_changeset(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
    ) -> anyhow::Result<TopDownQueryPayload<Vec<StakingChangeRequest>>> {
        self.caller.get_validator_changeset(subnet_id, epoch).await
    }

    async fn latest_parent_finality(&self) -> anyhow::Result<ChainEpoch> {
        self.caller.latest_parent_finality().await
    }
}

impl EvmSubnetDryRun {
    pub fn new(vm_type: VMType, caller: EthSubnetManager) -> Self {
        Self { vm_type, caller }
    }

    /// A todo marker that does not panic
    fn todo<T>(&self) -> anyhow::Result<T> {
        Err(anyhow!("not implemented yet"))
    }

    fn create_subnet_evm(
        &self,
        from: &Address,
        params: ConstructParams,
    ) -> anyhow::Result<MockedEVMTxn> {
        let converted = register_subnet_facet::ConstructorParams::try_from(params)?;
        log::debug!("converted constructor params: {converted:?}");

        let to = converted.ipc_gateway_addr;
        let from = payload_to_evm_address(from.payload())?;
        log::debug!("from: {}, to: {}", from, to);

        let (calldata, method) = to_evm_calldata(
            &register_subnet_facet::REGISTERSUBNETFACET_ABI.functions,
            "newSubnetActor",
            // ethers needs the params to be tuple here
            (converted,),
        )?;

        Ok(MockedEVMTxn {
            from,
            to,
            value: U256::zero(),
            method,
            calldata,
        })
    }
}

fn to_evm_calldata<T: Tokenizable>(
    functions: &BTreeMap<String, Vec<Function>>,
    func_name: &str,
    args: T,
) -> anyhow::Result<(Vec<u8>, Selector)> {
    let func = functions
        .get(func_name)
        .ok_or_else(|| anyhow!("function {} not found in abi", func_name))?
        .first()
        .ok_or_else(|| anyhow!("function is empty, abi does not seem to be valid"))?;

    let selector = func.selector();
    let evm_data = encode_function_data(func, args)?.to_vec();

    Ok((evm_data, selector))
}

fn to_fvm_calldata(evm_data: &[u8]) -> anyhow::Result<Vec<u8>> {
    let params = RawBytes::serialize(BytesSer(evm_data))?;
    Ok(params.to_vec())
}
