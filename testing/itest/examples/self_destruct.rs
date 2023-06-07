// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use ipc_sdk::subnet_id::SubnetID;
use itest::infra::SubnetInfra;
use itest::{infra, set_network_from_env, DEFAULT_ROOT};
use std::str::FromStr;
use std::sync::atomic::AtomicU16;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

#[tokio::main]
async fn main() {
    set_network_from_env();
    run().await.unwrap();
}

/// This spawns the infra only and will tear down the whole setup once done. This is useful for
/// initial testing of infra scripts.
async fn run() -> anyhow::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let eudico_binary_path =
        std::env::var("EUDICO_BIN").unwrap_or_else(|_| "/home/admin/lotus/eudico".to_string());
    let ipc_root_folder =
        std::env::var("IPC_ROOT_FOLDER").unwrap_or_else(|_| "/home/admin/.ipc-agent".to_string());
    let parent_lotus_path = std::env::var("PARENT_LOTUS_PATH")
        .unwrap_or_else(|_| "/home/admin/.lotus-local-net0".to_string());
    let parent_subnet_id_str =
        std::env::var("PARENT_SUBNET_ID").unwrap_or_else(|_| DEFAULT_ROOT.to_string());
    let subnet_name = std::env::var("SUBNET_NAME").unwrap_or_else(|_| "test-subnet".to_string());

    let api_port_sequence = Arc::new(AtomicU16::new(10));
    let config = infra::SubnetConfig::new(
        subnet_name,
        "t1cp4q4lqsdhob23ysywffg2tvbmar5cshia4rweq".to_string(),
        parent_lotus_path,
        ipc_root_folder,
        2,
        eudico_binary_path,
        SubnetID::from_str(&parent_subnet_id_str).unwrap(),
        api_port_sequence,
    );

    let mut infra = SubnetInfra::new(config);
    infra.create_subnet().await?;
    infra.start_nodes()?;
    infra.fund_node_wallets()?;
    infra.start_validators().await?;

    log::info!("nodes and validators are all up");

    infra.update_ipc_agent_config().await?;
    log::info!("ipc agent config updated");

    // wait for the validators to be mining
    sleep(Duration::from_secs(100));
    log::info!("wait for validators to be ready");

    infra.trigger_ipc_config_reload().await?;
    log::info!("triggered ipc agent config reload");

    // do some manual observation if needed
    sleep(Duration::from_secs(100));
    log::info!("everything is ready, the cluster will self destroy in 100 seconds.");

    infra.tear_down()?;
    log::info!("infra tear down");

    infra.remove_from_ipc_agent_config().await?;
    log::info!("removed subnet from ipc agent config");

    infra.trigger_ipc_config_reload().await?;
    log::info!("triggered ipc agent config reload");

    Ok(())
}
