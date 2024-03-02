use clap::{Args, Subcommand};
use tendermint_rpc::Url;

#[derive(Args, Debug)]
pub struct ProxyArgs {
    #[command(subcommand)]
    pub command: ProxyCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ProxyCommands {
    Run {
        /// The URL of the Tendermint node's RPC endpoint.
        #[arg(
            long,
            short,
            default_value = "http://127.0.0.1:26657",
            env = "TENDERMINT_RPC_URL"
        )]
        tendermint_url: Url,

        #[arg(
            long,
            short,
            default_value = "/ip4/127.0.0.1/tcp/5001",
            env = "IPFS_RPC_ADDR"
        )]
        ipfs_addr: String,

        #[command(flatten)]
        args: super::rpc::TransArgs,
    },
}
