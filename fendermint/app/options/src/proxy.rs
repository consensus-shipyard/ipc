use clap::{Args, Subcommand};
use tendermint_rpc::Url;

#[derive(Args, Debug)]
pub struct ProxyArgs {
    /// The URL to bind to.
    #[arg(long, short, default_value = "127.0.0.1:8080")]
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
        args: super::rpc::TransArgs,
    },
}
