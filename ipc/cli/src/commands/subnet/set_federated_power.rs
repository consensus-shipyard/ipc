use std::str::FromStr;
use async_trait::async_trait;
use clap::Args;
use fvm_shared::address::Address;
use ipc_api::subnet_id::SubnetID;
use crate::{CommandLineHandler, GlobalArguments};
use crate::commands::{f64_to_token_amount, get_ipc_provider, require_fil_addr_from_str};

/// The command to set federated power.
pub struct SetFederatedPower;

#[async_trait]
impl CommandLineHandler for crate::commands::subnet::SetFederatedPower {
    type Arguments = crate::commands::subnet::SetFederatedPowerArgs;

    async fn handle(global: &GlobalArguments, arguments: &Self::Arguments) -> anyhow::Result<()> {
        log::debug!("set federated power with args: {:?}", arguments);

        let provider = get_ipc_provider(global)?;
        let subnet = SubnetID::from_str(&arguments.subnet)?;

        let addresses: Vec<Address> = arguments.validator_addresses.iter().map(
            |address| require_fil_addr_from_str(address).unwrap()
        ).collect();

        let public_keys: Vec<Vec<u8>> = arguments.validator_pubkeys.iter().map(
            |key| hex::decode(key).unwrap()
        ).collect();

        provider.set_federated_power(&subnet, &addresses, &public_keys, &arguments.validator_power).await
    }
}

#[derive(Debug, Args)]
#[command(name = "stake", about = "Set federated power for validators")]
pub struct SetFederatedPowerArgs {
    #[arg(long, help = "The subnet to release collateral from")]
    pub subnet: String,
    #[arg(long, num_args = 1.., help = "Addresses of validators, separated by space")]
    pub validator_addresses: Vec<String>,
    #[arg(long, num_args = 1.., help = "Public keys of validators, separated by space")]
    pub validator_pubkeys: Vec<String>,
    #[arg(long, num_args = 1.., help = "Federated of validators, separated by space")]
    pub validator_power: Vec<u128>,
}
