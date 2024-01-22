// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, Context};
use ethers::abi::Token;
use ethers::types as et;
use ethers::types::Bytes;

use fendermint_vm_message::conv::{from_eth, from_fvm};
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::ActorID;

use super::{
    fevm::{ContractCaller, MockProvider, NoRevert},
    FvmExecState,
};
use crate::fvm::FvmApplyRet;
use fendermint_crypto::SecretKey;
use fendermint_vm_actor_interface::ipc;
use fendermint_vm_actor_interface::{
    eam::EthAddress,
    init::builtin_actor_eth_addr,
    ipc::{AbiHash, ValidatorMerkleTree, GATEWAY_ACTOR_ID},
};
use fendermint_vm_genesis::{Power, Validator};
use fendermint_vm_message::signed::sign_secp256k1;
use fendermint_vm_topdown::IPCParentFinality;
use fvm_shared::econ::TokenAmount;
use ipc_actors_abis::checkpointing_facet::CheckpointingFacet;
use ipc_actors_abis::gateway_getter_facet::GatewayGetterFacet;
use ipc_actors_abis::gateway_getter_facet::{self as getter, gateway_getter_facet};
use ipc_actors_abis::top_down_finality_facet::TopDownFinalityFacet;
use ipc_actors_abis::xnet_messaging_facet::XnetMessagingFacet;
use ipc_actors_abis::{checkpointing_facet, top_down_finality_facet, xnet_messaging_facet};
use ipc_api::cross::IpcEnvelope;
use ipc_api::staking::StakingChangeRequest;

#[derive(Clone)]
pub struct GatewayCaller<DB> {
    addr: EthAddress,
    getter: ContractCaller<DB, GatewayGetterFacet<MockProvider>, NoRevert>,
    checkpointing: ContractCaller<
        DB,
        CheckpointingFacet<MockProvider>,
        checkpointing_facet::CheckpointingFacetErrors,
    >,
    topdown: ContractCaller<
        DB,
        TopDownFinalityFacet<MockProvider>,
        top_down_finality_facet::TopDownFinalityFacetErrors,
    >,
    xnet: ContractCaller<
        DB,
        XnetMessagingFacet<MockProvider>,
        xnet_messaging_facet::XnetMessagingFacetErrors,
    >,
}

impl<DB> Default for GatewayCaller<DB> {
    fn default() -> Self {
        Self::new(GATEWAY_ACTOR_ID)
    }
}

impl<DB> GatewayCaller<DB> {
    pub fn new(actor_id: ActorID) -> Self {
        // A masked ID works for invoking the contract, but internally the EVM uses a different
        // ID and if we used this address for anything like validating that the sender is the gateway,
        // we'll face bitter disappointment. For that we have to use the delegated address we have in genesis.
        let addr = builtin_actor_eth_addr(actor_id);
        Self {
            addr,
            getter: ContractCaller::new(addr, GatewayGetterFacet::new),
            checkpointing: ContractCaller::new(addr, CheckpointingFacet::new),
            topdown: ContractCaller::new(addr, TopDownFinalityFacet::new),
            xnet: ContractCaller::new(addr, XnetMessagingFacet::new),
        }
    }

    pub fn addr(&self) -> EthAddress {
        self.addr
    }
}

impl<DB: Blockstore> GatewayCaller<DB> {
    /// Check that IPC is configured in this deployment.
    pub fn enabled(&self, state: &mut FvmExecState<DB>) -> anyhow::Result<bool> {
        match state.state_tree_mut().get_actor(GATEWAY_ACTOR_ID)? {
            None => Ok(false),
            Some(a) => Ok(!state.builtin_actors().is_placeholder_actor(&a.code)),
        }
    }

    /// Return true if the current subnet is the root subnet.
    pub fn is_root(&self, state: &mut FvmExecState<DB>) -> anyhow::Result<bool> {
        self.subnet_id(state).map(|id| id.route.is_empty())
    }

    /// Return the current subnet ID.
    pub fn subnet_id(&self, state: &mut FvmExecState<DB>) -> anyhow::Result<getter::SubnetID> {
        self.getter.call(state, |c| c.get_network_name())
    }

    /// Fetch the period with which the current subnet has to submit checkpoints to its parent.
    pub fn bottom_up_check_period(&self, state: &mut FvmExecState<DB>) -> anyhow::Result<u64> {
        Ok(self
            .getter
            .call(state, |c| c.bottom_up_check_period())?
            .as_u64())
    }

    /// Fetch the bottom-up message batch enqueued for a given checkpoint height.
    pub fn bottom_up_msg_batch(
        &self,
        state: &mut FvmExecState<DB>,
        height: u64,
    ) -> anyhow::Result<getter::BottomUpMsgBatch> {
        let batch = self.getter.call(state, |c| {
            c.bottom_up_msg_batch(ethers::types::U256::from(height))
        })?;
        Ok(batch)
    }

    /// Insert a new checkpoint at the period boundary.
    pub fn create_bottom_up_checkpoint(
        &self,
        state: &mut FvmExecState<DB>,
        checkpoint: checkpointing_facet::BottomUpCheckpoint,
        power_table: &[Validator<Power>],
    ) -> anyhow::Result<()> {
        // Construct a Merkle tree from the power table, which we can use to validate validator set membership
        // when the signatures are submitted in transactions for accumulation.
        let tree =
            ValidatorMerkleTree::new(power_table).context("failed to create validator tree")?;

        let total_power = power_table.iter().fold(et::U256::zero(), |p, v| {
            p.saturating_add(et::U256::from(v.power.0))
        });

        self.checkpointing.call(state, |c| {
            c.create_bottom_up_checkpoint(checkpoint, tree.root_hash().0, total_power)
        })
    }

    /// Retrieve checkpoints which have not reached a quorum.
    pub fn incomplete_checkpoints(
        &self,
        state: &mut FvmExecState<DB>,
    ) -> anyhow::Result<Vec<getter::BottomUpCheckpoint>> {
        self.getter.call(state, |c| c.get_incomplete_checkpoints())
    }

    /// Apply all pending validator changes, returning the newly adopted configuration number, or 0 if there were no changes.
    pub fn apply_validator_changes(&self, state: &mut FvmExecState<DB>) -> anyhow::Result<u64> {
        self.topdown.call(state, |c| c.apply_finality_changes())
    }

    /// Get the currently active validator set.
    pub fn current_validator_set(
        &self,
        state: &mut FvmExecState<DB>,
    ) -> anyhow::Result<getter::Membership> {
        self.getter.call(state, |c| c.get_current_membership())
    }

    /// Construct the input parameters for adding a signature to the checkpoint.
    ///
    /// This will need to be broadcasted as a transaction.
    pub fn add_checkpoint_signature_calldata(
        &self,
        checkpoint: checkpointing_facet::BottomUpCheckpoint,
        power_table: &[Validator<Power>],
        validator: &Validator<Power>,
        secret_key: &SecretKey,
    ) -> anyhow::Result<et::Bytes> {
        debug_assert_eq!(validator.public_key.0, secret_key.public_key());

        let height = checkpoint.block_height;
        let weight = et::U256::from(validator.power.0);

        let hash = checkpoint.abi_hash();

        let signature = sign_secp256k1(secret_key, &hash);
        let signature =
            from_fvm::to_eth_signature(&signature, false).context("invalid signature")?;
        let signature = et::Bytes::from(signature.to_vec());

        let tree =
            ValidatorMerkleTree::new(power_table).context("failed to construct Merkle tree")?;

        let membership_proof = tree
            .prove(validator)
            .context("failed to construct Merkle proof")?
            .into_iter()
            .map(|p| p.into())
            .collect();

        let call = self.checkpointing.contract().add_checkpoint_signature(
            height,
            membership_proof,
            weight,
            signature,
        );

        let calldata = call
            .calldata()
            .ok_or_else(|| anyhow!("no calldata for adding signature"))?;

        Ok(calldata)
    }

    /// Commit the parent finality to the gateway and returns the previously committed finality.
    /// None implies there is no previously committed finality.
    pub fn commit_parent_finality(
        &self,
        state: &mut FvmExecState<DB>,
        finality: IPCParentFinality,
    ) -> anyhow::Result<(FvmApplyRet, Option<IPCParentFinality>)> {
        let evm_finality = top_down_finality_facet::ParentFinality::try_from(finality)?;

        let ret = self
            .topdown
            .call_with_return(state, |c| c.commit_parent_finality(evm_finality))?;

        let (has_committed, prev_finality) = ret.decoded_ret()?;

        Ok(if !has_committed {
            (ret.into_return(), None)
        } else {
            (
                ret.into_return(),
                Some(IPCParentFinality::from(prev_finality)),
            )
        })
    }

    pub fn store_validator_changes(
        &self,
        state: &mut FvmExecState<DB>,
        changes: Vec<StakingChangeRequest>,
        final_ret: &mut TopDownApplyRetAggregator,
    ) -> anyhow::Result<()> {
        if changes.is_empty() {
            return Ok(());
        }

        let mut change_requests = vec![];
        for c in changes {
            change_requests.push(top_down_finality_facet::StakingChangeRequest::try_from(c)?);
        }

        let v = self
            .topdown
            .call_with_return(state, |c| c.store_validator_changes(change_requests))?;
        final_ret.agg_return(v.into_return());

        Ok(())
    }

    /// Call this function to mint some FIL to the gateway contract
    pub fn mint_to_gateway(
        &self,
        state: &mut FvmExecState<DB>,
        value: TokenAmount,
    ) -> anyhow::Result<()> {
        let state_tree = state.state_tree_mut();
        state_tree.mutate_actor(ipc::GATEWAY_ACTOR_ID, |actor_state| {
            actor_state.balance += value;
            Ok(())
        })?;
        Ok(())
    }

    pub fn apply_cross_messages(
        &self,
        state: &mut FvmExecState<DB>,
        cross_messages: Vec<IpcEnvelope>,
        final_ret: &mut TopDownApplyRetAggregator,
    ) -> anyhow::Result<()> {
        for msg in cross_messages {
            let msg = xnet_messaging_facet::IpcEnvelope::try_from(msg)
                .context("failed to convert cross messages")?;
            self.apply_cross_msg(state, msg, final_ret)?;
        }

        Ok(())
    }

    fn apply_cross_msg(
        &self,
        state: &mut FvmExecState<DB>,
        msg: xnet_messaging_facet::IpcEnvelope,
        final_ret: &mut TopDownApplyRetAggregator,
    ) -> anyhow::Result<()> {
        let (msg, success, ret) = match self
            .xnet
            .call_with_return(state, |c| c.apply_msg_with_ret(msg.clone()))
        {
            Ok(v) => {
                let (has_return, apply_ret) = v.decoded_ret()?;

                final_ret.agg_return(v.into_return());

                if !has_return {
                    return Ok(());
                }

                (apply_ret.cross_msg, apply_ret.success, apply_ret.ret)
            }
            Err(e) => {
                let ret = ethers::abi::encode(&[Token::String(e.to_string())]);
                (msg, false, Bytes::from(ret))
            }
        };

        let v = self
            .xnet
            .call_with_return(state, |c| c.send_receipt(msg, success, ret))?;
        final_ret.agg_return(v.into_return());

        Ok(())
    }

    pub fn get_latest_parent_finality(
        &self,
        state: &mut FvmExecState<DB>,
    ) -> anyhow::Result<IPCParentFinality> {
        let r = self
            .getter
            .call(state, |c| c.get_latest_parent_finality())?;
        Ok(IPCParentFinality::from(r))
    }

    /// Get the Ethereum adresses of validators who signed a checkpoint.
    pub fn checkpoint_signatories(
        &self,
        state: &mut FvmExecState<DB>,
        height: u64,
    ) -> anyhow::Result<Vec<EthAddress>> {
        let (_, _, addrs, _) = self.getter.call(state, |c| {
            c.get_checkpoint_signature_bundle(ethers::types::U256::from(height))
        })?;

        let addrs = addrs.into_iter().map(|a| a.into()).collect();

        Ok(addrs)
    }
}

/// Total amount of tokens to mint as a result of top-down messages arriving at the subnet.
pub fn tokens_to_mint(msgs: &[ipc_api::cross::IpcEnvelope]) -> TokenAmount {
    msgs.iter()
        .fold(TokenAmount::from_atto(0), |mut total, msg| {
            // Both fees and value are considered to enter the ciruculating supply of the subnet.
            // Fees might be distributed among subnet validators.
            total += &msg.value;
            total
        })
}

/// Total amount of tokens to burn as a result of bottom-up messages leaving the subnet.
pub fn tokens_to_burn(msgs: &[gateway_getter_facet::IpcEnvelope]) -> TokenAmount {
    msgs.iter()
        .fold(TokenAmount::from_atto(0), |mut total, msg| {
            // Both fees and value were taken from the sender, and both are going up to the parent subnet:
            // https://github.com/consensus-shipyard/ipc-solidity-actors/blob/e4ec0046e2e73e2f91d7ab8ae370af2c487ce526/src/gateway/GatewayManagerFacet.sol#L143-L150
            // Fees might be distirbuted among relayers.
            total += from_eth::to_fvm_tokens(&msg.value);
            total
        })
}

/// Aggregates the return data from different contract calls in top down message execution.
/// This struct only aggregates the gas, fee and events
pub struct TopDownApplyRetAggregator {
    final_return: FvmApplyRet,

    is_appending_events: bool,
    is_adding_gas: bool,
}

impl TopDownApplyRetAggregator {
    pub fn default(ret: FvmApplyRet) -> Self {
        Self {
            final_return: ret,
            is_appending_events: false,
            is_adding_gas: true,
        }
    }

    pub fn into_return(self) -> FvmApplyRet {
        self.final_return
    }

    pub fn agg_return(&mut self, ret: FvmApplyRet) {
        debug_assert_eq!(self.final_return.from, ret.from);
        debug_assert_eq!(self.final_return.to, ret.to);
        debug_assert_eq!(self.final_return.method_num, ret.method_num);

        self.final_return.emitters.extend(&ret.emitters);

        if self.is_appending_events {
            self.final_return
                .apply_ret
                .events
                .extend(ret.apply_ret.events);
        }

        if self.is_adding_gas {
            self.final_return.apply_ret.gas_burned += ret.apply_ret.gas_burned;
            self.final_return.apply_ret.gas_refund += ret.apply_ret.gas_refund;
            self.final_return.apply_ret.base_fee_burn += ret.apply_ret.base_fee_burn;
            self.final_return.apply_ret.miner_tip += ret.apply_ret.miner_tip;
            self.final_return.apply_ret.over_estimation_burn += ret.apply_ret.over_estimation_burn;
            self.final_return.apply_ret.penalty += ret.apply_ret.penalty;
            self.final_return.apply_ret.refund += ret.apply_ret.refund;
        }
    }
}
