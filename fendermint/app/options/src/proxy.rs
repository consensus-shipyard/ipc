use super::parse::parse_token_amount;
use clap::{Args, Subcommand, ValueEnum};
use fvm_shared::econ::TokenAmount;
use std::path::PathBuf;
use tendermint_rpc::Url;

#[derive(Args, Debug)]
pub struct ProxyArgs {
    /// The URL to bind to.
    #[arg(long, short, default_value = "127.0.0.1:26650")]
    pub bind: String,

    /// The URL of the Tendermint node's RPC endpoint.
    #[arg(
        long,
        short,
        default_value = "http://127.0.0.1:26657",
        env = "TENDERMINT_RPC_URL"
    )]
    pub url: Url,

    /// An optional HTTP/S proxy through which to submit requests to the
    /// Tendermint node's RPC endpoint.
    #[arg(long)]
    pub proxy_url: Option<Url>,

    #[command(subcommand)]
    pub command: ProxyCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ProxyCommands {
    Start {
        #[command(flatten)]
        args: TransArgs,
    },
}

/// Arguments common to transactions and transfers.
#[derive(Args, Debug, Clone)]
pub struct TransArgs {
    /// Name of chain the for which the message will be signed.
    #[arg(long, short, env = "FM_CHAIN_NAME")]
    pub chain_name: String,
    /// Path to the secret key of the sender to sign the transaction.
    #[arg(long, short)]
    pub secret_key: PathBuf,
    /// Price of gas.
    ///
    /// Any discrepancy between this and the base fee is paid for
    /// by the validator who puts the transaction into the block.
    #[arg(long, value_parser = parse_token_amount, default_value = "0")]
    pub gas_fee_cap: TokenAmount,
    /// Gas premium.
    #[arg(long, value_parser = parse_token_amount, default_value = "0")]
    pub gas_premium: TokenAmount,
    /// Whether to wait for the results from Tendermint or not.
    #[arg(long, short, default_value = "commit")]
    pub broadcast_mode: BroadcastMode,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum BroadcastMode {
    /// Do no wait for the results.
    Async,
    /// Wait for the result of `check_tx`.
    Sync,
    /// Wait for the result of `deliver_tx`.
    Commit,
}
