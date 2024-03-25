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
use crate::manager::{
    BottomUpCheckpointRelayer, GetBlockHashResult, SubnetGenesisInfo, SubnetInfo, SubnetManager,
    TopDownFinalityQuery, TopDownQueryPayload,
};
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

pub struct EvmSubnetDryRun {
    vm_type: VMType,
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
        subnet: SubnetID,
        from: Address,
        collateral: TokenAmount,
        metadata: Vec<u8>,
    ) -> anyhow::Result<ChainEpoch> {
        todo!()
    }

    async fn pre_fund(
        &self,
        subnet: SubnetID,
        from: Address,
        balance: TokenAmount,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn pre_release(
        &self,
        subnet: SubnetID,
        from: Address,
        amount: TokenAmount,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn stake(
        &self,
        subnet: SubnetID,
        from: Address,
        collateral: TokenAmount,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn unstake(
        &self,
        subnet: SubnetID,
        from: Address,
        collateral: TokenAmount,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn leave_subnet(&self, subnet: SubnetID, from: Address) -> anyhow::Result<()> {
        todo!()
    }

    async fn kill_subnet(&self, subnet: SubnetID, from: Address) -> anyhow::Result<()> {
        todo!()
    }

    async fn list_child_subnets(
        &self,
        gateway_addr: Address,
    ) -> anyhow::Result<HashMap<SubnetID, SubnetInfo>> {
        todo!()
    }

    async fn claim_collateral(&self, subnet: SubnetID, from: Address) -> anyhow::Result<()> {
        todo!()
    }

    async fn fund(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        to: Address,
        amount: TokenAmount,
    ) -> anyhow::Result<ChainEpoch> {
        todo!()
    }

    async fn fund_with_token(
        &self,
        subnet: SubnetID,
        from: Address,
        to: Address,
        amount: TokenAmount,
    ) -> anyhow::Result<ChainEpoch> {
        todo!()
    }

    async fn release(
        &self,
        gateway_addr: Address,
        from: Address,
        to: Address,
        amount: TokenAmount,
    ) -> anyhow::Result<ChainEpoch> {
        todo!()
    }

    async fn propagate(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        postbox_msg_key: Vec<u8>,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn send_value(
        &self,
        from: Address,
        to: Address,
        amount: TokenAmount,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn wallet_balance(&self, address: &Address) -> anyhow::Result<TokenAmount> {
        todo!()
    }

    async fn get_chain_id(&self) -> anyhow::Result<String> {
        todo!()
    }

    async fn get_commit_sha(&self) -> anyhow::Result<[u8; 32]> {
        todo!()
    }

    async fn get_genesis_info(&self, subnet: &SubnetID) -> anyhow::Result<SubnetGenesisInfo> {
        todo!()
    }

    async fn add_bootstrap(
        &self,
        subnet: &SubnetID,
        from: &Address,
        endpoint: String,
    ) -> anyhow::Result<()> {
        todo!()
    }

    async fn list_bootstrap_nodes(&self, subnet: &SubnetID) -> anyhow::Result<Vec<String>> {
        todo!()
    }

    async fn get_validator_info(
        &self,
        subnet: &SubnetID,
        validator: &Address,
    ) -> anyhow::Result<ValidatorInfo> {
        todo!()
    }

    async fn set_federated_power(
        &self,
        from: &Address,
        subnet: &SubnetID,
        validators: &[Address],
        public_keys: &[Vec<u8>],
        federated_power: &[u128],
    ) -> anyhow::Result<ChainEpoch> {
        todo!()
    }
}

#[async_trait]
impl BottomUpCheckpointRelayer for EvmSubnetDryRun {
    async fn submit_checkpoint(
        &self,
        submitter: &Address,
        checkpoint: BottomUpCheckpoint,
        signatures: Vec<Signature>,
        signatories: Vec<Address>,
    ) -> anyhow::Result<ChainEpoch> {
        todo!()
    }

    async fn last_bottom_up_checkpoint_height(
        &self,
        subnet_id: &SubnetID,
    ) -> anyhow::Result<ChainEpoch> {
        todo!()
    }

    async fn checkpoint_period(&self, subnet_id: &SubnetID) -> anyhow::Result<ChainEpoch> {
        todo!()
    }

    async fn checkpoint_bundle_at(
        &self,
        height: ChainEpoch,
    ) -> anyhow::Result<BottomUpCheckpointBundle> {
        todo!()
    }

    async fn quorum_reached_events(
        &self,
        height: ChainEpoch,
    ) -> anyhow::Result<Vec<QuorumReachedEvent>> {
        todo!()
    }

    async fn current_epoch(&self) -> anyhow::Result<ChainEpoch> {
        todo!()
    }
}

#[async_trait]
impl TopDownFinalityQuery for EvmSubnetDryRun {
    async fn genesis_epoch(&self, subnet_id: &SubnetID) -> anyhow::Result<ChainEpoch> {
        todo!()
    }

    async fn chain_head_height(&self) -> anyhow::Result<ChainEpoch> {
        todo!()
    }

    async fn get_top_down_msgs(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
    ) -> anyhow::Result<TopDownQueryPayload<Vec<IpcEnvelope>>> {
        todo!()
    }

    async fn get_block_hash(&self, height: ChainEpoch) -> anyhow::Result<GetBlockHashResult> {
        todo!()
    }

    async fn get_validator_changeset(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
    ) -> anyhow::Result<TopDownQueryPayload<Vec<StakingChangeRequest>>> {
        todo!()
    }

    async fn latest_parent_finality(&self) -> anyhow::Result<ChainEpoch> {
        todo!()
    }
}

impl EvmSubnetDryRun {
    pub fn new(vm_type: VMType) -> Self {
        Self { vm_type }
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
