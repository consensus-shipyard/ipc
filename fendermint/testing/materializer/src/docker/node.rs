// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::{
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use bollard::{
    container::{Config, RemoveContainerOptions},
    service::HostConfig,
    Docker,
};
use ethers::types::H160;

use super::{container::DockerContainer, DockerMaterials, DockerPortRange, DockerWithDropHandle};
use crate::{
    materializer::{NodeConfig, TargetConfig},
    NodeName, ResourceHash,
};

const COMETBFT_IMAGE: &str = "cometbft/cometbft:v0.37.x";
const FENDERMINT_IMAGE: &str = "fendermint:latest";
const RESOLVER_PORT: u32 = 26655;

type EnvVars = Vec<(&'static str, String)>;
type Volumes = Vec<(PathBuf, &'static str)>;

macro_rules! env_vars {
    ( $($key:literal => $value:expr),* $(,)? ) => {
        vec![ $( ($key, $value.to_string()) ),* ]
    };
}

/// A Node consists of multiple docker containers.
pub struct DockerNode {
    /// Logical name of the node in the subnet hierarchy.
    node_name: NodeName,
    fendermint: DockerContainer,
    cometbft: DockerContainer,
    ethapi: Option<DockerContainer>,
}

impl DockerNode {
    pub async fn get_or_create<'a>(
        root: impl AsRef<Path>,
        dh: DockerWithDropHandle,
        node_name: &NodeName,
        node_config: NodeConfig<'a, DockerMaterials>,
        port_range: DockerPortRange,
    ) -> anyhow::Result<Self> {
        let fendermint_name = container_name(node_name, "fendermint");
        let cometbft_name = container_name(node_name, "cometbft");
        let ethapi_name = container_name(node_name, "ethapi");

        let fendermint = DockerContainer::get(&dh, fendermint_name.clone()).await?;
        let cometbft = DockerContainer::get(&dh, cometbft_name.clone()).await?;
        let ethapi = DockerContainer::get(&dh, ethapi_name.clone()).await?;

        // Directory for the node's data volumes
        let node_dir = root.as_ref().join(node_name);
        std::fs::create_dir_all(&node_dir).context("failed to create node dir")?;

        // Get the current user ID to use with docker containers.
        let user = node_dir.metadata()?.uid();

        // Create a directory for keys
        let keys_dir = node_dir.join("keys");
        if !keys_dir.exists() {
            std::fs::create_dir(&keys_dir)?;
        }

        // Create a directory for cometbft
        let cometbft_dir = node_dir.join("cometbft");
        if !cometbft_dir.exists() {
            std::fs::create_dir(&cometbft_dir)?;

            let cometbft_runner = DockerRunner::new(
                &dh.docker,
                COMETBFT_IMAGE,
                user,
                vec![(cometbft_dir.clone(), "/cometbft")],
            );

            // Init cometbft to establish the network key.
            cometbft_runner
                .run_cmd("init")
                .await
                .context("cannot init cometbft")?;

            let fendermint_runner = DockerRunner::new(
                &dh.docker,
                FENDERMINT_IMAGE,
                user,
                vec![
                    (keys_dir.clone(), "/data/keys"),
                    (cometbft_dir.clone(), "/data/cometbft"),
                    (node_config.genesis.path.clone(), "/data/genesis.json"),
                ],
            );

            // Convert fendermint genesis to cometbft.
            fendermint_runner
                .run_cmd(
                    "genesis --genesis-file /data/genesis.json \
                    into-tendermint --out /data/cometbft/config/genesis.json",
                )
                .await
                .context("failed to convert genesis")?;

            // Convert validator private key to cometbft.
            if let Some(v) = node_config.validator {
                let validator_key_path = v.secret_key_path();
                std::fs::copy(validator_key_path, keys_dir.join("validator_key.sk"))
                    .context("failed to copy validator key")?;

                fendermint_runner
                    .run_cmd(
                        "key into-tendermint --secret-key /data/keys/validator_key.sk \
                        --out /data/cometbft/config/priv_validator_key.json",
                    )
                    .await
                    .context("failed to convert validator key")?;
            }

            // Create a network key for the resolver.
            fendermint_runner
                .run_cmd("key gen --out-dir /data/keys --name network_key")
                .await
                .context("failed to create network key")?;
        }

        // Create a directory for fendermint
        let fendermint_dir = node_dir.join("fendermint");
        if !fendermint_dir.exists() {
            std::fs::create_dir(&fendermint_dir)?;
            std::fs::create_dir(&fendermint_dir.join("data"))?;
            std::fs::create_dir(&fendermint_dir.join("logs"))?;
            std::fs::create_dir(&fendermint_dir.join("snapshots"))?;
        }

        // If there is no static env var file, create one with all the common variables.
        let static_env = node_dir.join("static.env");
        if !static_env.exists() {
            let genesis = &node_config.genesis.genesis;
            let ipc = genesis
                .ipc
                .as_ref()
                .ok_or_else(|| anyhow!("ipc config missing"))?;

            let resolver_host_port: u32 = port_range.from;

            let basic: EnvVars = env_vars![
                "LOG_LEVEL"        => "info",
                "RUST_BACKTRACE"   => 1,
                "FM_DATA_DIR"      => "/fendermint/data",
                "FM_LOG_DIR"       => "/fendermint/logs",
                "FM_SNAPSHOTS_DIR" => "/fendermint/snapshots",
                "FM_CHAIN_NAME"    => genesis.chain_name.clone(),
                "FM_IPC_SUBNET_ID" => ipc.gateway.subnet_id,
                "FM_RESOLVER__NETWORK__LOCAL_KEY"          => "/data/keys/network_key.sk",
                "FM_RESOLVER__CONNECTION__LISTEN_ADDR"     => format!("/ip4/0.0.0.0/tcp/${RESOLVER_PORT}"),
                "FM_TENDERMINT_RPC_URL" => format!("http://${cometbft_name}:26657"),
                "TENDERMINT_RPC_URL"    => format!("http://${cometbft_name}:26657"),
                "TENDERMINT_WS_URL"     => format!("ws://${cometbft_name}:26657/websocket"),
            ];

            let topdown: EnvVars = match node_config.parent_node {
                Some(pc) => {
                    let gateway: H160 = pc.deployment.gateway.into();
                    let registry: H160 = pc.deployment.registry.into();
                    match pc.node {
                        // Assume Lotus
                        TargetConfig::External(url) => env_vars![
                            "FM_IPC__TOPDOWN__CHAIN_HEAD_DELAY"        => 20,
                            "FM_IPC__TOPDOWN__PARENT_HTTP_ENDPOINT"    => url,
                            "FM_IPC__TOPDOWN__PARENT_REGISTRY"         => registry,
                            "FM_IPC__TOPDOWN__PARENT_GATEWAY"          => gateway,
                            "FM_IPC__TOPDOWN__EXPONENTIAL_BACK_OFF"    => 5,
                            "FM_IPC__TOPDOWN__EXPONENTIAL_RETRY_LIMIT" => 5                ,
                            "FM_IPC__TOPDOWN__POLLING_INTERVAL"        => 10,
                            "FM_IPC__TOPDOWN__PROPOSAL_DELAY"          => 2,
                            "FM_IPC__TOPDOWN__MAX_PROPOSAL_RANGE"      => 100,
                        ],
                        // Assume Fendermint
                        TargetConfig::Internal(node) => {
                            let parent_ethapi = node.ethapi.as_ref().ok_or_else(|| {
                                anyhow!(
                                    "{node_name} cannot follow {}; ethapi is not running",
                                    node.node_name
                                )
                            })?;
                            env_vars![
                                "FM_IPC__TOPDOWN__CHAIN_HEAD_DELAY"        => 1,
                                "FM_IPC__TOPDOWN__PARENT_HTTP_ENDPOINT"    => format!("http://{}:8445", parent_ethapi.container.name),
                                "FM_IPC__TOPDOWN__PARENT_REGISTRY"         => registry,
                                "FM_IPC__TOPDOWN__PARENT_GATEWAY"          => gateway,
                                "FM_IPC__TOPDOWN__EXPONENTIAL_BACK_OFF"    => 5,
                                "FM_IPC__TOPDOWN__EXPONENTIAL_RETRY_LIMIT" => 5                ,
                                "FM_IPC__TOPDOWN__POLLING_INTERVAL"        => 1,
                                "FM_IPC__TOPDOWN__PROPOSAL_DELAY"          => 0,
                                "FM_IPC__TOPDOWN__MAX_PROPOSAL_RANGE"      => 10,
                            ]
                        }
                    }
                }
                None => env_vars!(),
            };

            let cmt = env_vars![
                "CMT_PROXY_APP" => format!("tcp://{fendermint_name}:26658"),
                "CMT_P2P_PEX"   => true,
                "CMT_RPC_MAX_SUBSCRIPTION_CLIENTS"     => 10,
                "CMT_RPC_MAX_SUBSCRIPTIONS_PER_CLIENT" => 1000,
            ];

            let env = vec![basic, topdown, cmt].concat();

            // Export the env to a file.
            todo!()
        }

        // If there is no dynamic env var file, create an empty one.
        // --env FM_RESOLVER__DISCOVERY__STATIC_ADDRESSES=${RESOLVER_BOOTSTRAPS} \
        // --env CMT_P2P_SEEDS
        todo!();

        if fendermint.is_none() {
            // Create a fendermint container mounting:
            // - the fendermint directory
            // - the docker-entry
            // - the env var files

            //         let fendermint = match fendermint {
            //             Some(container) => container,
            //             None => dh
            //                 .docker
            //                 .create_container(Some(CreateContainerOptions {
            //                     name: fendermint_name.clone(),
            //                     ..Default::default()
            //                 }), Config {
            //                     hostname: Some(fendermint_name.clone()),
            // user,
            // host_config: Some(HostConfig {
            //     init: Some(true ),
            //     binds: ,
            //     port_bindings: ,
            // })
            //                 })
            //                 .await
            //                 .context("failed to create fendermint container")?,
            //         };
            todo!();
        }

        if cometbft.is_none() {
            // Create a CometBFT container mounting:
            // - the cometbft directory
            // - the docker-entry
            // - the env var files
        }

        if node_config.ethapi && ethapi.is_none() {
            // Create a ethapi container mounting:
            // - the docker-entry
            // - the env var files
        }

        // Construct the DockerNode
        todo!()
    }
}

/// Create a container name from a node name and a logical container name, e.g. "cometbft"
/// in a way that we can use it as a hostname without being too long.
///
/// It consists of `{node-id}-{container}-{hash(node-name)}`,
/// e.g. "node-12-cometbft-a1b2c3"
fn container_name(node_name: &NodeName, container: &str) -> String {
    let node_id = node_name
        .path()
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let hash = ResourceHash::digest(node_name.path().to_string_lossy().to_string());
    let hash = hash.to_string();
    let hash = &hash.as_str()[..6];
    format!("{node_id}-{container}-{}", hash)
}

/// Helper for short-lived `docker run` commands.
struct DockerRunner<'a> {
    docker: &'a Docker,
    image: String,
    user: u32,
    volumes: Volumes,
}

impl<'a> DockerRunner<'a> {
    pub fn new(docker: &'a Docker, image: &str, user: u32, volumes: Volumes) -> Self {
        Self {
            docker,
            image: image.to_string(),
            user,
            volumes,
        }
    }

    /// Run a short lived container.
    pub async fn run_cmd(&self, cmd: &str) -> anyhow::Result<()> {
        let config = Config {
            image: Some(self.image.clone()),
            user: Some(self.user.to_string()),
            cmd: Some(vec![cmd.to_string()]),
            attach_stderr: Some(true),
            attach_stdout: Some(true),
            host_config: Some(HostConfig {
                auto_remove: Some(true),
                init: Some(true),
                binds: Some(
                    self.volumes
                        .iter()
                        .map(|(h, c)| format!("{}:{c}", h.to_string_lossy()))
                        .collect(),
                ),
                ..Default::default()
            }),
            ..Default::default()
        };

        let id = self
            .docker
            .create_container::<&str, _>(None, config)
            .await
            .context("failed to create container")?
            .id;

        self.docker
            .start_container::<&str>(&id, None)
            .await
            .context("failed to start container")?;

        // TODO: Output?

        self.docker
            .remove_container(
                &id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await?;

        Ok(())
    }
}
