// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::lotus::message::ipc::SubnetInfo;
use crate::manager::cometbft::SignedHeader;
use anyhow::Result;
use async_trait::async_trait;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::{address::Address, econ::TokenAmount};
use ipc_actors_abis::checkpointing_facet::StateCommitmentBreakDown;
use ipc_actors_abis::subnet_actor_activity_facet::ValidatorClaim;
use ipc_actors_abis::subnet_actor_checkpoint_facet::LastCommitmentHeights;
use ipc_actors_abis::subnet_actor_checkpointing_facet::Inclusion;
use ipc_actors_abis::subnet_actor_getter_facet::ListPendingCommitmentsEntry;
use ipc_api::checkpoint::consensus::ValidatorData;
use ipc_api::cross::IpcEnvelope;
use ipc_api::staking::{PowerChangeRequest, ValidatorInfo};
use ipc_api::subnet::{Asset, ConstructParams, PermissionMode};
use ipc_api::subnet_id::SubnetID;
use ipc_api::validator::Validator;
use std::collections::{BTreeMap, HashMap};

/// Trait to interact with a subnet and handle its lifecycle.
#[async_trait]
pub trait SubnetManager:
    Send + Sync + TopDownFinalityQuery + SignedHeaderRelayer + ValidatorRewarder
{
    /// Deploys a new subnet actor on the `parent` subnet and with the
    /// configuration passed in `ConstructParams`.
    /// The result of the function is the ID address for the subnet actor from which the final
    /// subnet ID can be inferred.
    async fn create_subnet(&self, from: Address, params: ConstructParams) -> Result<Address>;

    /// Performs the call to join a subnet from a wallet address and staking an amount
    /// of collateral. This function, as well as all of the ones on this trait, can infer
    /// the specific subnet and actors on which to perform the relevant calls from the
    /// SubnetID given as an argument.
    async fn join_subnet(
        &self,
        subnet: SubnetID,
        from: Address,
        collateral: TokenAmount,
        metadata: Vec<u8>,
    ) -> Result<ChainEpoch>;

    /// Approves a subnet to be bootstrapped on the gateway. Only the gateway contract owner can perform this operation.
    async fn approve_subnet(&self, subnet: SubnetID, from: Address) -> Result<()>;

    /// Revokes the approval for a subnet to be bootstrapped on the gateway. Only the gateway contract owner can perform this operation.
    async fn reject_approved_subnet(&self, subnet: SubnetID, from: Address) -> Result<()>;

    /// Adds some initial balance to an address before a child subnet bootstraps to make
    /// it available in the subnet at genesis.
    async fn pre_fund(&self, subnet: SubnetID, from: Address, balance: TokenAmount) -> Result<()>;

    /// Releases initial funds from an address for a subnet that has not yet been bootstrapped
    async fn pre_release(&self, subnet: SubnetID, from: Address, amount: TokenAmount)
        -> Result<()>;

    /// Allows validators that have already joined the subnet to stake more collateral
    /// and increase their power in the subnet.
    async fn stake(&self, subnet: SubnetID, from: Address, collateral: TokenAmount) -> Result<()>;

    /// Allows validators that have already joined the subnet to unstake collateral
    /// and reduce their power in the subnet.
    async fn unstake(&self, subnet: SubnetID, from: Address, collateral: TokenAmount)
        -> Result<()>;

    /// Sends a request to leave a subnet from a wallet address.
    async fn leave_subnet(&self, subnet: SubnetID, from: Address) -> Result<()>;

    /// Sends a signal to kill a subnet
    async fn kill_subnet(&self, subnet: SubnetID, from: Address) -> Result<()>;

    /// Lists all the registered children in a gateway.
    async fn list_child_subnets(
        &self,
        gateway_addr: Address,
    ) -> Result<HashMap<SubnetID, SubnetInfo>>;

    /// Claims any collateral that may be available to claim by validators that
    /// have left the subnet.
    async fn claim_collateral(&self, subnet: SubnetID, from: Address) -> Result<()>;

    /// Fund injects new funds from an account of the parent chain to a subnet.
    /// Returns the epoch that the fund is executed in the parent.
    async fn fund(
        &self,
        subnet: SubnetID,
        gateway_addr: Address,
        from: Address,
        to: Address,
        amount: TokenAmount,
    ) -> Result<ChainEpoch>;

    /// Sends funds to a specified subnet receiver using ERC20 tokens.
    /// This function locks the amount of ERC20 tokens into custody and then mints the supply in the specified subnet.
    /// It checks if the subnet's supply strategy is ERC20 and if not, the operation is reverted.
    /// It allows for free injection of funds into a subnet and is protected against reentrancy.
    ///
    /// # Arguments
    ///
    /// * `subnetId` - The ID of the subnet where the funds will be sent to.
    /// * `from`     - The funding address.
    /// * `to`       - The funded address.
    /// * `amount`   - The amount of ERC20 tokens to be sent.
    async fn fund_with_token(
        &self,
        subnet: SubnetID,
        from: Address,
        to: Address,
        amount: TokenAmount,
    ) -> Result<ChainEpoch>;

    /// Grants an allowance to the `from` address to withdraw up to `amount` of tokens from the contract at `token_address`.
    /// This function sets up an approval, allowing the `from` address to later transfer or utilize the tokens from the specified ERC20 token contract.
    /// The primary use case is to enable subsequent contract interactions that require an upfront allowance,
    /// such as depositing tokens into a contract that requires an allowance check.
    ///
    /// The operation ensures that the caller has the necessary authority and token balance before setting the allowance.
    /// It is crucial for enabling controlled access to the token funds without transferring the ownership.
    /// Note that calling this function multiple times can overwrite the existing allowance with the new value.
    ///
    /// # Arguments
    ///
    /// * `from`         - The address granting the approval.
    /// * `token_address`- The contract address of the ERC20 token for which the approval is being granted.
    /// * `amount`       - The maximum amount of tokens `from` is allowing to be used.
    ///
    /// # Returns
    ///
    /// * `Result<()>`   - An empty result indicating success or an error on failure, encapsulating any issues encountered during the approval process.
    async fn approve_token(
        &self,
        subnet: SubnetID,
        from: Address,
        amount: TokenAmount,
    ) -> Result<ChainEpoch>;

    /// Release creates a new check message to release funds in parent chain
    /// Returns the epoch that the released is executed in the child.
    async fn release(
        &self,
        gateway_addr: Address,
        from: Address,
        to: Address,
        amount: TokenAmount,
    ) -> Result<ChainEpoch>;

    /// Send value between two addresses in a subnet
    async fn send_value(&self, from: Address, to: Address, amount: TokenAmount) -> Result<()>;

    /// Get the balance of an address
    async fn wallet_balance(&self, address: &Address) -> Result<TokenAmount>;

    /// Get chainID for the network.
    /// Returning as a `String` because the maximum value for an EVM
    /// networks is a `U256` that wouldn't fit in an integer type.
    async fn get_chain_id(&self) -> Result<String>;

    /// Get commit sha for deployed contracts
    async fn get_commit_sha(&self) -> Result<[u8; 32]>;

    /// Gets the subnet supply source
    async fn get_subnet_supply_source(&self, subnet: &SubnetID) -> Result<Asset>;

    /// Gets the subnet collateral source
    async fn get_subnet_collateral_source(&self, subnet: &SubnetID) -> Result<Asset>;

    /// Gets the genesis information required to bootstrap a child subnet
    async fn get_genesis_info(&self, subnet: &SubnetID) -> Result<SubnetGenesisInfo>;

    /// Advertises the endpoint of a bootstrap node for the subnet.
    async fn add_bootstrap(
        &self,
        subnet: &SubnetID,
        from: &Address,
        endpoint: String,
    ) -> Result<()>;

    /// Lists the bootstrap nodes of a subnet
    async fn list_bootstrap_nodes(&self, subnet: &SubnetID) -> Result<Vec<String>>;

    /// Get the validator information
    async fn get_validator_info(
        &self,
        subnet: &SubnetID,
        validator: &Address,
    ) -> Result<ValidatorInfo>;

    /// Lists all the validators
    async fn list_validators(&self, subnet: &SubnetID) -> Result<Vec<(Address, ValidatorInfo)>>;

    async fn list_subnet_active_validators(
        &self,
        subnet: &SubnetID,
    ) -> Result<Vec<(Address, ValidatorInfo)>>;

    async fn list_waiting_validators(
        &self,
        subnet: &SubnetID,
    ) -> Result<Vec<(Address, ValidatorInfo)>>;

    async fn set_federated_power(
        &self,
        from: &Address,
        subnet: &SubnetID,
        validators: &[Address],
        public_keys: &[Vec<u8>],
        federated_power: &[u128],
    ) -> Result<ChainEpoch>;
}

#[derive(Debug)]
pub struct SubnetGenesisInfo {
    pub chain_id: u64,
    pub bottom_up_checkpoint_period: u64,
    pub majority_percentage: u8,
    pub active_validators_limit: u16,
    pub min_collateral: TokenAmount,
    pub genesis_epoch: ChainEpoch,
    pub validators: Vec<Validator>,
    pub genesis_balances: BTreeMap<Address, TokenAmount>,
    pub permission_mode: PermissionMode,
    pub supply_source: Asset,
    pub genesis_subnet_ipc_contracts_owner: ethers::types::Address,
}

/// The generic payload that returns the block hash of the data returning block with the actual
/// data payload.
#[derive(Debug)]
pub struct TopDownQueryPayload<T> {
    pub value: T,
    pub block_hash: Vec<u8>,
}

#[derive(Default, Debug)]
pub struct GetBlockHashResult {
    pub parent_block_hash: Vec<u8>,
    pub block_hash: Vec<u8>,
}

/// Trait to interact with a subnet to query the necessary information for top down checkpoint.
#[async_trait]
pub trait TopDownFinalityQuery: Send + Sync {
    /// Returns the genesis epoch that the subnet is created in parent network
    async fn genesis_epoch(&self, subnet_id: &SubnetID) -> Result<ChainEpoch>;
    /// Returns the chain head height
    async fn chain_head_height(&self) -> Result<ChainEpoch>;
    /// Returns the list of top down messages
    async fn get_top_down_msgs(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
    ) -> Result<TopDownQueryPayload<Vec<IpcEnvelope>>>;
    /// Get the block hash
    async fn get_block_hash(&self, height: ChainEpoch) -> Result<GetBlockHashResult>;
    /// Get the validator change set from start to end block.
    async fn get_validator_changeset(
        &self,
        subnet_id: &SubnetID,
        epoch: ChainEpoch,
    ) -> Result<TopDownQueryPayload<Vec<PowerChangeRequest>>>;
    /// Returns the latest parent finality committed in a child subnet
    async fn latest_parent_finality(&self) -> Result<ChainEpoch>;
}

#[async_trait]
pub trait SignedHeaderRelayer: Send + Sync {
    async fn submit_signed_header(
        &self,
        submitter: &Address,
        subnet_id: &SubnetID,
        header: SignedHeader,
    ) -> Result<ChainEpoch>;

    async fn query_commitment(
        &self,
        height: ChainEpoch,
    ) -> Result<Option<StateCommitmentBreakDown>>;

    async fn get_last_commitment_heights(
        &self,
        subnet_id: &SubnetID,
    ) -> Result<LastCommitmentHeights>;

    async fn confirm_validator_change(
        &self,
        height: ChainEpoch,
        submitter: &Address,
        subnet_id: &SubnetID,
        commitment: StateCommitmentBreakDown,
    ) -> Result<ChainEpoch>;

    async fn last_submission_height(&self, subnet_id: &SubnetID) -> Result<ChainEpoch>;

    async fn submission_period(&self, subnet_id: &SubnetID) -> Result<ChainEpoch>;

    async fn current_epoch(&self) -> Result<ChainEpoch>;

    async fn list_active_validators(
        &self,
        subnet: &SubnetID,
    ) -> Result<Vec<(Address, ValidatorInfo)>>;

    /// Lists the pending bottom-up batch commitments for the given subnet.
    async fn list_pending_bottom_up_batch_commitments(
        &self,
        subnet_id: &SubnetID,
    ) -> Result<Vec<ListPendingCommitmentsEntry>>;

    /// Prepares the next messages and their inclusion proof
    /// that should be executed based on the current pending commitment.
    async fn make_next_bottom_up_batch_inclusions(
        &self,
        current: &ListPendingCommitmentsEntry,
    ) -> Result<Vec<Inclusion>>;
    /// Executes a batch of committed bottom-up messages.
    async fn execute_bottom_up_batch(
        &self,
        submitter: &Address,
        subnet_id: &SubnetID,
        height: ChainEpoch,
        inclusions: Vec<Inclusion>,
    ) -> Result<ChainEpoch>;
}

/// The validator reward related functions, such as check reward and claim reward for mining blocks
/// in the child subnet
#[async_trait]
pub trait ValidatorRewarder: Send + Sync {
    /// Query validator claims, indexed by checkpoint height, to batch claim rewards.
    async fn query_reward_claims(
        &self,
        validator_addr: &Address,
        from_checkpoint: ChainEpoch,
        to_checkpoint: ChainEpoch,
    ) -> Result<Vec<(u64, ValidatorClaim)>>;

    /// Query validator rewards in the current subnet, without obtaining proofs.
    async fn query_validator_rewards(
        &self,
        validator: &Address,
        from_checkpoint: ChainEpoch,
        to_checkpoint: ChainEpoch,
    ) -> Result<Vec<(u64, ValidatorData)>>;

    /// Claim validator rewards in a batch for the specified subnet.
    async fn batch_subnet_claim(
        &self,
        submitter: &Address,
        reward_claim_subnet: &SubnetID,
        reward_origin_subnet: &SubnetID,
        claims: Vec<(u64, ValidatorClaim)>,
    ) -> Result<()>;
}
