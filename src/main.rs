mod cli;
mod config;
mod jsonrpc;
mod lotus;
mod manager;
mod server;

#[tokio::main]
async fn main() {
    cli::cli().await;
}
