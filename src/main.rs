use ipc_agent::cli;

#[tokio::main]
async fn main() {
    cli::cli().await;
}
