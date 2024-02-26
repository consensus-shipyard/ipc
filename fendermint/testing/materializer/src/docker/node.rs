// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::{
    collections::{BTreeMap, HashMap},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context};
use bollard::{
    container::{Config, CreateContainerOptions, RemoveContainerOptions},
    service::{HostConfig, PortBinding},
    Docker,
};
use ethers::types::H160;
use lazy_static::lazy_static;

use super::{
    container::DockerContainer, DockerConstruct, DockerMaterials, DockerNetwork, DockerPortRange,
    DockerWithDropHandle,
};
use crate::{
    docker::DOCKER_ENTRY_FILE_NAME,
    materializer::{NodeConfig, TargetConfig},
    materials::export_file,
    NodeName, ResourceHash,
};

// TODO: Add these to the materializer.
const COMETBFT_IMAGE: &str = "cometbft/cometbft:v0.37.x";
const FENDERMINT_IMAGE: &str = "fendermint:latest";

/// The static environment variables are the ones we can assign during node creation,
/// ie. they don't depend on other nodes' values which get determined during their creation.
const STATIC_ENV: &str = "static.env";
/// The dynamic environment variables are ones we can only during the start of the node,
/// by which time all other nodes will have been created. Examples of this are network
/// identities which depend on network keys being created; in order to create a fully
/// connected network, we first need all network keys to be created, then we can look
/// all of them up during the start of each node.
/// These go into a separate file just so it's easy to recreate them.
const DYNAMIC_ENV: &str = "dynamic.env";

const RESOLVER_P2P_PORT: u32 = 26655;
const COMETBFT_P2P_PORT: u32 = 26656;
const COMETBFT_RPC_PORT: u32 = 26657;
const FENDERMINT_ABCI_PORT: u32 = 26658;
const ETHAPI_RPC_PORT: u32 = 8445;

lazy_static! {
    static ref STATIC_ENV_PATH: String = format!("/opt/docker/{STATIC_ENV}");
    static ref DYNAMIC_ENV_PATH: String = format!("/opt/docker/{DYNAMIC_ENV}");
    static ref DOCKER_ENTRY_PATH: String = format!("/opt/docker/{DOCKER_ENTRY_FILE_NAME}");
}

type EnvVars = BTreeMap<&'static str, String>;
type Volumes = Vec<(PathBuf, &'static str)>;

macro_rules! env_vars {
    ( $($key:literal => $value:expr),* $(,)? ) => {
        BTreeMap::from([ $( ($key, $value.to_string()) ),* ])
    };
}

/// A Node consists of multiple docker containers.
pub struct DockerNode {
    /// Logical name of the node in the subnet hierarchy.
    node_name: NodeName,
    fendermint: DockerContainer,
    cometbft: DockerContainer,
    ethapi: Option<DockerContainer>,
    port_range: DockerPortRange,
    /// This is the file system directory were all the artifacts
    /// regarding this node are stored, such as docker volumes and keys.
    path: PathBuf,
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

            // We'll need to run some cometbft and fendermint commands.
            // NOTE: Currently the Fendermint CLI commands live in the
            // `app` crate in a way that they can't be imported. We
            // could move them to the `lib.rs` from `main.rs` and
            // then we wouldn't need docker for some of these steps.
            // However, at least this way they are tested.

            let cometbft_runner = DockerRunner::new(
                &dh,
                node_name,
                COMETBFT_IMAGE,
                user,
                vec![(cometbft_dir.clone(), "/cometbft")],
            );

            let fendermint_runner = DockerRunner::new(
                &dh,
                node_name,
                FENDERMINT_IMAGE,
                user,
                vec![
                    (keys_dir.clone(), "/fendermint/keys"),
                    (cometbft_dir.clone(), "/cometbft"),
                    (node_config.genesis.path.clone(), "/fendermint/genesis.json"),
                ],
            );

            // Init cometbft to establish the network key.
            cometbft_runner
                .run_cmd("init")
                .await
                .context("cannot init cometbft")?;

            // Convert fendermint genesis to cometbft.
            fendermint_runner
                .run_cmd(
                    "genesis --genesis-file /fendermint/genesis.json \
                    into-tendermint --out /cometbft/config/genesis.json",
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
                        "key into-tendermint --secret-key /fendermint/keys/validator_key.sk \
                        --out cometbft/config/priv_validator_key.json",
                    )
                    .await
                    .context("failed to convert validator key")?;
            }

            // Create a network key for the resolver.
            fendermint_runner
                .run_cmd("key gen --out-dir /fendermint/keys --name network_key")
                .await
                .context("failed to create network key")?;
        }

        // Create a directory for fendermint
        let fendermint_dir = node_dir.join("fendermint");
        if !fendermint_dir.exists() {
            std::fs::create_dir(&fendermint_dir)?;
            std::fs::create_dir(fendermint_dir.join("data"))?;
            std::fs::create_dir(fendermint_dir.join("logs"))?;
            std::fs::create_dir(fendermint_dir.join("snapshots"))?;
        }

        // If there is no static env var file, create one with all the common variables.
        let static_env = node_dir.join(STATIC_ENV);
        if !static_env.exists() {
            let genesis = &node_config.genesis.genesis;
            let ipc = genesis
                .ipc
                .as_ref()
                .ok_or_else(|| anyhow!("ipc config missing"))?;

            let resolver_host_port: u32 = port_range.from;

            let mut env: EnvVars = env_vars![
                "LOG_LEVEL"        => "info",
                "RUST_BACKTRACE"   => 1,
                "FM_NETWORK "      => "testnet",
                "FM_DATA_DIR"      => "/fendermint/data",
                "FM_LOG_DIR"       => "/fendermint/logs",
                "FM_SNAPSHOTS_DIR" => "/fendermint/snapshots",
                "FM_CHAIN_NAME"    => genesis.chain_name.clone(),
                "FM_IPC_SUBNET_ID" => ipc.gateway.subnet_id,
                "FM_RESOLVER__NETWORK__LOCAL_KEY"      => "/fendermint/keys/network_key.sk",
                "FM_RESOLVER__CONNECTION__LISTEN_ADDR" => format!("/ip4/0.0.0.0/tcp/${RESOLVER_P2P_PORT}"),
                "FM_TENDERMINT_RPC_URL" => format!("http://${cometbft_name}:{COMETBFT_RPC_PORT}"),
                "TENDERMINT_RPC_URL"    => format!("http://${cometbft_name}:{COMETBFT_RPC_PORT}"),
                "TENDERMINT_WS_URL"     => format!("ws://${cometbft_name}:{COMETBFT_RPC_PORT}/websocket"),
                "FM_ABCI__LISTEN__PORT" => FENDERMINT_ABCI_PORT,
                "FM_ETH__LISTEN__PORT"  => ETHAPI_RPC_PORT,
            ];

            if node_config.validator.is_some() {
                env.extend(env_vars![
                    "FM_VALIDATOR_KEY__KIND" => "ethereum",
                    "FM_VALIDATOR_KEY__PATH" => "/fendermint/keys/validator_key.sk",
                ]);
            }

            if let Some(pc) = node_config.parent_node {
                let gateway: H160 = pc.deployment.gateway.into();
                let registry: H160 = pc.deployment.registry.into();
                let topdown = match pc.node {
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
                            "FM_IPC__TOPDOWN__PARENT_HTTP_ENDPOINT"    => format!("http://{}:{ETHAPI_RPC_PORT}", parent_ethapi.container.name),
                            "FM_IPC__TOPDOWN__PARENT_REGISTRY"         => registry,
                            "FM_IPC__TOPDOWN__PARENT_GATEWAY"          => gateway,
                            "FM_IPC__TOPDOWN__EXPONENTIAL_BACK_OFF"    => 5,
                            "FM_IPC__TOPDOWN__EXPONENTIAL_RETRY_LIMIT" => 5                ,
                            "FM_IPC__TOPDOWN__POLLING_INTERVAL"        => 1,
                            "FM_IPC__TOPDOWN__PROPOSAL_DELAY"          => 0,
                            "FM_IPC__TOPDOWN__MAX_PROPOSAL_RANGE"      => 10,
                        ]
                    }
                };
                env.extend(topdown);
            }

            env.extend(env_vars![
                "CMT_PROXY_APP" => format!("tcp://{fendermint_name}:{FENDERMINT_ABCI_PORT}"),
                "CMT_P2P_PEX"   => true,
                "CMT_RPC_MAX_SUBSCRIPTION_CLIENTS"     => 10,
                "CMT_RPC_MAX_SUBSCRIPTIONS_PER_CLIENT" => 1000,
            ]);

            // Export the env to a file.
            export_env(&static_env, &env).context("failed to export env")?;
        }

        // If there is no dynamic env var file, create an empty one so it can be mounted.
        let dynamic_env = node_dir.join(DYNAMIC_ENV);
        if !dynamic_env.exists() {
            // The values will be assigned when the node is started.
            // --env FM_RESOLVER__DISCOVERY__STATIC_ADDRESSES=${RESOLVER_BOOTSTRAPS}
            // --env CMT_P2P_SEEDS
            export_env(&dynamic_env, &Default::default())?;
        }

        // All containers will be started with the docker entry and all env files.
        let volumes = |vs: Volumes| {
            let common: Volumes = vec![
                (static_env.clone(), STATIC_ENV_PATH.as_str()),
                (dynamic_env.clone(), DYNAMIC_ENV_PATH.as_str()),
                (
                    root.as_ref().join("scripts").join(DOCKER_ENTRY_FILE_NAME),
                    DOCKER_ENTRY_PATH.as_str(),
                ),
            ];
            [common, vs].concat()
        };

        // Wrap an entry point with the docker entry script.
        let entrypoint = |ep: &str| {
            format!(
                "{} '{ep}' {} {}",
                *DOCKER_ENTRY_PATH, *STATIC_ENV_PATH, *DYNAMIC_ENV_PATH
            )
        };

        // Create a fendermint container mounting:
        let fendermint = match fendermint {
            Some(c) => c,
            None => {
                let creator = DockerRunner::new(
                    &dh,
                    node_name,
                    FENDERMINT_IMAGE,
                    user,
                    volumes(vec![
                        (keys_dir.clone(), "/fendermint/keys"),
                        (fendermint_dir.join("data"), "/fendermint/data"),
                        (fendermint_dir.join("logs"), "/fendermint/logs"),
                        (fendermint_dir.join("snapshots"), "/fendermint/snapshots"),
                    ]),
                );

                creator
                    .create(
                        fendermint_name,
                        node_config.network,
                        vec![(port_range.resolver_p2p_host_port(), RESOLVER_P2P_PORT)],
                        entrypoint("fendermint run"),
                    )
                    .await
                    .context("failed to create fendermint")?
            }
        };

        // Create a CometBFT container
        let cometbft = match cometbft {
            Some(c) => c,
            None => {
                let creator = DockerRunner::new(
                    &dh,
                    node_name,
                    COMETBFT_IMAGE,
                    user,
                    volumes(vec![(cometbft_dir.clone(), "/cometbft")]),
                );

                creator
                    .create(
                        cometbft_name,
                        node_config.network,
                        vec![
                            (port_range.cometbft_p2p_host_port(), COMETBFT_P2P_PORT),
                            (port_range.cometbft_rpc_host_port(), COMETBFT_RPC_PORT),
                        ],
                        entrypoint("cometbft start"),
                    )
                    .await
                    .context("failed to create fendermint")?
            }
        };

        // Create a ethapi container
        let ethapi = match ethapi {
            None if node_config.ethapi => {
                let creator =
                    DockerRunner::new(&dh, node_name, FENDERMINT_IMAGE, user, volumes(vec![]));

                let c = creator
                    .create(
                        ethapi_name,
                        node_config.network,
                        vec![(port_range.ethapi_rpc_host_port(), ETHAPI_RPC_PORT)],
                        entrypoint("fendermint eth run"),
                    )
                    .await
                    .context("failed to create ethapi")?;

                Some(c)
            }
            other => other,
        };

        // Construct the DockerNode
        Ok(DockerNode {
            node_name: node_name.clone(),
            fendermint,
            cometbft,
            ethapi,
            port_range,
            path: node_dir,
        })
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

struct DockerRunner<'a> {
    dh: &'a DockerWithDropHandle,
    node_name: NodeName,
    image: String,
    user: u32,
    volumes: Volumes,
}

impl<'a> DockerRunner<'a> {
    pub fn new(
        dh: &'a DockerWithDropHandle,
        node_name: &NodeName,
        image: &str,
        user: u32,
        volumes: Volumes,
    ) -> Self {
        Self {
            dh,
            node_name: node_name.clone(),
            image: image.to_string(),
            user,
            volumes,
        }
    }

    fn docker(&self) -> &Docker {
        &self.dh.docker
    }

    // Tag containers with resource names.
    fn labels(&self) -> HashMap<String, String> {
        [
            ("testnet", self.node_name.testnet().path()),
            ("node", self.node_name.path()),
        ]
        .into_iter()
        .map(|(n, p)| (n.to_string(), p.to_string_lossy().to_string()))
        .collect()
    }

    /// Run a short lived container.
    pub async fn run_cmd(&self, cmd: &str) -> anyhow::Result<()> {
        let config = Config {
            image: Some(self.image.clone()),
            user: Some(self.user.to_string()),
            cmd: Some(vec![cmd.to_string()]),
            attach_stderr: Some(true),
            attach_stdout: Some(true),
            labels: Some(self.labels()),
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
            .docker()
            .create_container::<&str, _>(None, config)
            .await
            .context("failed to create container")?
            .id;

        self.docker()
            .start_container::<&str>(&id, None)
            .await
            .context("failed to start container")?;

        // TODO: Output?

        self.docker()
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

    /// Create a container to be started later.
    pub async fn create(
        &self,
        name: String,
        network: &DockerNetwork,
        // Host <-> Container port mappings
        ports: Vec<(u32, u32)>,
        entrypoint: String,
    ) -> anyhow::Result<DockerContainer> {
        let config = Config {
            hostname: Some(name.clone()),
            image: Some(self.image.clone()),
            user: Some(self.user.to_string()),
            entrypoint: Some(vec![entrypoint]),
            cmd: None,
            host_config: Some(HostConfig {
                init: Some(true),
                binds: Some(
                    self.volumes
                        .iter()
                        .map(|(h, c)| format!("{}:{c}", h.to_string_lossy()))
                        .collect(),
                ),
                port_bindings: Some(
                    ports
                        .into_iter()
                        .flat_map(|(h, c)| {
                            let binding = PortBinding {
                                host_ip: None,
                                host_port: Some(h.to_string()),
                            };
                            // Emitting both TCP and UDP, just in case.
                            vec![
                                (format!("{c}/tcp"), Some(vec![binding.clone()])),
                                (format!("{c}/udp"), Some(vec![binding])),
                            ]
                        })
                        .collect(),
                ),
                network_mode: Some(network.network().name.clone()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let id = self
            .docker()
            .create_container::<String, _>(
                Some(CreateContainerOptions {
                    name: name.clone(),
                    ..Default::default()
                }),
                config,
            )
            .await
            .context("failed to create container")?
            .id;

        Ok(DockerContainer {
            dh: self.dh.clone(),
            container: DockerConstruct {
                id,
                name,
                external: false,
            },
        })
    }
}

fn export_env(file_path: impl AsRef<Path>, env: &EnvVars) -> anyhow::Result<()> {
    let env = env
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>();

    export_file(file_path, env.join("\n"))
}
