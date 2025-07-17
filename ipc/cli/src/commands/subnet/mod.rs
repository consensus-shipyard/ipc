// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use self::add_bootstrap_node::{
    AddBootstrapArgs, AddNodeBootstrap, ListBootstrapNodes, ListBootstrapsArgs,
};
use self::join::{StakeSubnet, StakeSubnetArgs, UnstakeSubnet, UnstakeSubnetArgs};
use self::leave::{Claim, ClaimArgs};
use self::rpc::{ChainIdSubnet, ChainIdSubnetArgs};
use crate::commands::subnet::approve::{ApproveSubnet, ApproveSubnetArgs};
pub use crate::commands::subnet::create::{CreateSubnet, CreateSubnetArgs};
use crate::commands::subnet::create_genesis::{CreateGenesis, CreateGenesisArgs};
use crate::commands::subnet::genesis_epoch::{GenesisEpoch, GenesisEpochArgs};
use crate::commands::subnet::init::{InitSubnet, InitSubnetArgs};
pub use crate::commands::subnet::join::{JoinSubnet, JoinSubnetArgs};
pub use crate::commands::subnet::kill::{KillSubnet, KillSubnetArgs};
pub use crate::commands::subnet::leave::{LeaveSubnet, LeaveSubnetArgs};
use crate::commands::subnet::list_subnets::{ListSubnets, ListSubnetsArgs};
use crate::commands::subnet::list_validators::{ListValidators, ListValidatorsArgs};
use crate::commands::subnet::reject_approved::{RejectApprovedSubnet, RejectApprovedSubnetArgs};
use crate::commands::subnet::rpc::{RPCSubnet, RPCSubnetArgs};
use crate::commands::subnet::send_value::{SendValue, SendValueArgs};
use crate::commands::subnet::set_federated_power::{SetFederatedPower, SetFederatedPowerArgs};
use crate::commands::subnet::show_gateway_contract_commit_sha::{
    ShowGatewayContractCommitSha, ShowGatewayContractCommitShaArgs,
};
use crate::commands::subnet::validator::{ValidatorInfo, ValidatorInfoArgs};
use crate::{CommandLineHandler, GlobalArguments};
use clap::{Args, Subcommand};

pub mod add_bootstrap_node;
pub mod approve;
pub mod create;
pub mod create_genesis;
mod genesis_epoch;
pub mod init;
pub mod join;
pub mod kill;
pub mod leave;
pub mod list_subnets;
pub mod list_validators;
pub mod reject_approved;
pub mod rpc;
pub mod send_value;
mod set_federated_power;
pub mod show_gateway_contract_commit_sha;
mod validator;

pub(crate) const ZERO_ADDRESS: &str = "0000000000000000000000000000000000000000";

#[derive(Debug, Args)]
#[command(
    name = "subnet",
    about = "subnet related commands such as create, join and etc"
)]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct SubnetCommandsArgs {
    #[command(subcommand)]
    command: Commands,
}

impl SubnetCommandsArgs {
    pub async fn handle(&self, global: &GlobalArguments) -> anyhow::Result<()> {
        match &self.command {
            Commands::Init(args) => InitSubnet::handle(global, args).await,
            Commands::Create(args) => CreateSubnet::handle(global, args).await,
            Commands::Approve(args) => ApproveSubnet::handle(global, args).await,
            Commands::RejectApproved(args) => RejectApprovedSubnet::handle(global, args).await,
            Commands::List(args) => ListSubnets::handle(global, args).await,
            Commands::ListValidators(args) => ListValidators::handle(global, args).await,
            Commands::Join(args) => JoinSubnet::handle(global, args).await,
            Commands::Rpc(args) => RPCSubnet::handle(global, args).await,
            Commands::ChainId(args) => ChainIdSubnet::handle(global, args).await,
            Commands::Leave(args) => LeaveSubnet::handle(global, args).await,
            Commands::Kill(args) => KillSubnet::handle(global, args).await,
            Commands::SendValue(args) => SendValue::handle(global, args).await,
            Commands::Stake(args) => StakeSubnet::handle(global, args).await,
            Commands::Unstake(args) => UnstakeSubnet::handle(global, args).await,
            Commands::Claim(args) => Claim::handle(global, args).await,
            Commands::AddBootstrap(args) => AddNodeBootstrap::handle(global, args).await,
            Commands::ListBootstraps(args) => ListBootstrapNodes::handle(global, args).await,
            Commands::GenesisEpoch(args) => GenesisEpoch::handle(global, args).await,
            Commands::GetValidator(args) => ValidatorInfo::handle(global, args).await,
            Commands::ShowGatewayContractCommitSha(args) => {
                ShowGatewayContractCommitSha::handle(global, args).await
            }
            Commands::SetFederatedPower(args) => SetFederatedPower::handle(global, args).await,
            Commands::CreateGenesis(args) => CreateGenesis::handle(global, args).await,
        }
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Init(InitSubnetArgs),
    Create(CreateSubnetArgs),
    Approve(ApproveSubnetArgs),
    RejectApproved(RejectApprovedSubnetArgs),
    List(ListSubnetsArgs),
    ListValidators(ListValidatorsArgs),
    Join(JoinSubnetArgs),
    Rpc(RPCSubnetArgs),
    ChainId(ChainIdSubnetArgs),
    Leave(LeaveSubnetArgs),
    Kill(KillSubnetArgs),
    SendValue(SendValueArgs),
    Stake(StakeSubnetArgs),
    Unstake(UnstakeSubnetArgs),
    Claim(ClaimArgs),
    AddBootstrap(AddBootstrapArgs),
    ListBootstraps(ListBootstrapsArgs),
    GenesisEpoch(GenesisEpochArgs),
    GetValidator(ValidatorInfoArgs),
    ShowGatewayContractCommitSha(ShowGatewayContractCommitShaArgs),
    SetFederatedPower(SetFederatedPowerArgs),
    CreateGenesis(CreateGenesisArgs),
}
