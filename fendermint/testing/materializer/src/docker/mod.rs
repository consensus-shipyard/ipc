// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{bail, Context};
use async_trait::async_trait;
use bollard::{
    container::{ListContainersOptions, RemoveContainerOptions},
    network::ListNetworksOptions,
    secret::{ContainerSummary, Network},
    Docker,
};
use ethers::{
    core::rand::{rngs::StdRng, SeedableRng},
    types::H160,
};
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_core::{chainid, Timestamp};
use fendermint_vm_genesis::{
    ipc::{GatewayParams, IpcParams},
    Account, Actor, ActorMeta, Collateral, Genesis, SignerAddr, Validator, ValidatorKey,
};
use fvm_shared::{bigint::Zero, econ::TokenAmount, version::NetworkVersion};
use ipc_api::subnet_id::SubnetID;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};
use tendermint_rpc::Url;

use crate::{
    manifest::Balance,
    materializer::{Materializer, NodeConfig, SubmitConfig, SubnetConfig},
    materials::{
        export_json, export_script, import_json, DefaultAccount, DefaultDeployment, DefaultGenesis,
        DefaultSubnet, Materials,
    },
    NodeName, RelayerName, ResourceHash, ResourceName, SubnetName, TestnetName,
};

mod container;
mod dropper;
mod network;
mod node;
mod relayer;
mod runner;

pub use dropper::DropPolicy;
pub use network::DockerNetwork;
pub use node::DockerNode;
pub use relayer::DockerRelayer;

use self::dropper::DropHandle;

const STATE_JSON_FILE_NAME: &str = "materializer-state.json";

const DOCKER_ENTRY_SCRIPT: &str = include_str!("../../scripts/docker-entry.sh");
const DOCKER_ENTRY_FILE_NAME: &str = "docker-entry.sh";

const PORT_RANGE_START: u32 = 30000;
const PORT_RANGE_SIZE: u32 = 100;

type Volumes = Vec<(PathBuf, &'static str)>;
type EnvVars = BTreeMap<&'static str, String>;

#[macro_export]
macro_rules! env_vars {
    ( $($key:literal => $value:expr),* $(,)? ) => {
        BTreeMap::from([ $( ($key, $value.to_string()) ),* ])
    };
}

pub struct DockerMaterials;

impl Materials for DockerMaterials {
    type Deployment = DefaultDeployment;
    type Account = DefaultAccount;
    type Genesis = DefaultGenesis;
    type Subnet = DefaultSubnet;

    type Network = DockerNetwork;
    type Node = DockerNode;
    type Relayer = DockerRelayer;
}

/// A thing constructed by docker.
#[derive(Debug, Clone)]
pub struct DockerConstruct {
    /// Unique ID of the thing.
    pub id: String,
    /// The name of the thing that we can use in docker commands.
    pub name: String,
    /// Indicate whether the thing was created outside the test,
    /// or it can be destroyed when it goes out of scope.
    pub keep: bool,
}

/// Allocated (inclusive) range we can use to expose containers' ports on the host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerPortRange {
    pub from: u32,
    pub to: u32,
}

/// Mapping ports assuming a 100 size ranges.
///
/// The ports on the host are assigned so that they end with the same number as the internal one,
/// which is hopefully a little bit intuitive for anyone who is familiar with the default values.
impl DockerPortRange {
    /// Mapping the internal 26655 port to the host.
    pub fn resolver_p2p_host_port(&self) -> u32 {
        self.from + 55
    }

    /// Mapping the internal 26656 port to the host.
    pub fn cometbft_p2p_host_port(&self) -> u32 {
        self.from + 56
    }

    /// Mapping the internal 26657 port to the host.
    pub fn cometbft_rpc_host_port(&self) -> u32 {
        self.from + 57
    }

    /// Mapping the internal 8445 port to the host.
    pub fn ethapi_rpc_host_port(&self) -> u32 {
        self.from + 45
    }
}

/// State of the materializer that it persists, so that it can resume operations.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DockerMaterializerState {
    /// Port ranges ever allocated by this materializer.
    port_ranges: BTreeMap<NodeName, DockerPortRange>,
}

pub struct DockerMaterializer {
    dir: PathBuf,
    rng: StdRng,
    docker: bollard::Docker,
    drop_handle: dropper::DropHandle,
    drop_chute: dropper::DropChute,
    drop_policy: dropper::DropPolicy,
    state: DockerMaterializerState,
}

impl DockerMaterializer {
    /// Create a materializer with a directory where all the
    /// testnets can live next to each other.
    pub fn new(dir: &Path, seed: u64) -> anyhow::Result<Self> {
        let docker =
            Docker::connect_with_local_defaults().context("failed to connect to Docker")?;

        // Create a runtime for the execution of drop tasks.
        let (drop_handle, drop_chute) = dropper::start(docker.clone());

        // Read in the state if it exists, otherwise create a default one.
        let state = import_json(dir.join(STATE_JSON_FILE_NAME))
            .context("failed to read state")?
            .unwrap_or_default();

        let m = Self {
            dir: dir.into(),
            rng: StdRng::seed_from_u64(seed),
            docker,
            drop_handle,
            drop_chute,
            state,
            drop_policy: DropPolicy::default(),
        };

        m.save_state().context("failed to save state")?;
        m.export_scripts().context("failed to export scripts")?;

        Ok(m)
    }

    pub fn with_policy(mut self, policy: DropPolicy) -> Self {
        self.drop_policy = policy;
        self
    }

    /// Remove all traces of a testnet.
    pub async fn remove(&mut self, testnet_name: &TestnetName) -> anyhow::Result<()> {
        let testnet = testnet_name.path_string();

        let mut filters = HashMap::new();
        filters.insert("label".to_string(), vec![format!("testnet={}", testnet)]);

        let containers: Vec<ContainerSummary> = self
            .docker
            .list_containers(Some(ListContainersOptions {
                all: true,
                filters,
                ..Default::default()
            }))
            .await
            .context("failed to list docker containers")?;

        let ids = containers.into_iter().filter_map(|c| c.id);

        for id in ids {
            eprintln!("removing docker container {id}");
            self.docker
                .remove_container(
                    &id,
                    Some(RemoveContainerOptions {
                        force: true,
                        v: true,
                        ..Default::default()
                    }),
                )
                .await
                .with_context(|| format!("failed to remove container {id}"))?;
        }

        let mut filters = HashMap::new();
        filters.insert("name".to_string(), vec![testnet]);

        let networks: Vec<Network> = self
            .docker
            .list_networks(Some(ListNetworksOptions { filters }))
            .await
            .context("failed to list networks")?;

        let ids = networks.into_iter().filter_map(|n| n.id);

        for id in ids {
            eprintln!("removing docker network {id}");
            self.docker
                .remove_network(&id)
                .await
                .context("failed to remove network")?;
        }

        let dir = self.dir.join(testnet_name.path());
        if let Err(e) = std::fs::remove_dir_all(&dir) {
            if !e.to_string().contains("No such file") {
                bail!(
                    "failed to remove testnet directory {}: {e:?}",
                    dir.to_string_lossy()
                );
            }
        };

        Ok(())
    }

    /// Replace the dropper with a new one and return the existing one so that we can await all the drop tasks being completed.
    pub fn take_dropper(&mut self) -> DropHandle {
        let (mut drop_handle, mut drop_chute) = dropper::start(self.docker.clone());
        std::mem::swap(&mut drop_handle, &mut self.drop_handle);
        std::mem::swap(&mut drop_chute, &mut self.drop_chute);
        // By dropping the `drop_chute` the only the existing docker constructs will keep a reference to it.
        // The caller can decide when it's time to wait on the handle, when the testnet have been dropped.
        drop_handle
    }

    /// Path to a directory based on a resource name.
    fn path<T: AsRef<ResourceName>>(&self, name: T) -> PathBuf {
        let name: &ResourceName = name.as_ref();
        self.dir.join(&name.0)
    }

    /// Path where the state of the materializer is saved.
    fn state_path(&self) -> PathBuf {
        self.dir.join(STATE_JSON_FILE_NAME)
    }

    /// Directory where scripts are exported, to be mounted into containers.
    fn scripts_dir(&self) -> PathBuf {
        self.dir.join("scripts")
    }

    /// Export scripts that need to be mounted.
    fn export_scripts(&self) -> anyhow::Result<()> {
        let scripts_dir = self.scripts_dir();
        export_script(scripts_dir.join("docker-entry.sh"), DOCKER_ENTRY_SCRIPT)?;
        Ok(())
    }

    /// Update the state, save it to JSON, then return whatever value the update returns.
    fn update_state<F, T>(&mut self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(&mut DockerMaterializerState) -> T,
    {
        let value = f(&mut self.state);
        self.save_state()?;
        Ok(value)
    }

    /// Write the state to a JSON file.
    fn save_state(&self) -> anyhow::Result<()> {
        export_json(self.state_path(), &self.state).context("failed to export state")
    }

    /// Return an existing genesis by parsing it from the `genesis.json` of the subnet,
    /// or create a new one and export it.
    fn get_or_create_genesis<F>(
        &self,
        subnet_name: &SubnetName,
        make_genesis: F,
    ) -> anyhow::Result<DefaultGenesis>
    where
        F: FnOnce() -> anyhow::Result<Genesis>,
    {
        let subnet_path = self.path(subnet_name);
        let genesis_path = subnet_path.join("genesis.json");

        let genesis = match import_json(&genesis_path).context("failed to read genesis")? {
            Some(genesis) => genesis,
            None => {
                let genesis = make_genesis().context("failed to make genesis")?;
                export_json(&genesis_path, &genesis).context("failed to export genesis")?;
                genesis
            }
        };

        Ok(DefaultGenesis {
            name: subnet_name.clone(),
            genesis,
            path: genesis_path,
        })
    }

    /// Pick a range for a container. Remember the choice so that we can recreate
    /// this materializer in a test and allocate more if needed without clashes.
    fn port_range(&mut self, node_name: &NodeName) -> anyhow::Result<DockerPortRange> {
        if let Some(range) = self.state.port_ranges.get(node_name) {
            return Ok(range.clone());
        }
        // Currently the range allocations are not dropped from the materializer,
        // so the length can be used to derive the next available port. Otherwise
        // we could loop through to find an unused slot.
        let node_count = self.state.port_ranges.len() as u32;
        let from = PORT_RANGE_START + PORT_RANGE_SIZE * node_count;
        let to = from + PORT_RANGE_SIZE;
        let range = DockerPortRange { from, to };
        self.update_state(|s| s.port_ranges.insert(node_name.clone(), range.clone()))?;
        Ok(range)
    }
}

#[async_trait]
impl Materializer<DockerMaterials> for DockerMaterializer {
    async fn create_network(
        &mut self,
        testnet_name: &TestnetName,
    ) -> anyhow::Result<<DockerMaterials as Materials>::Network> {
        DockerNetwork::get_or_create(
            self.docker.clone(),
            self.drop_chute.clone(),
            testnet_name.clone(),
            &self.drop_policy,
        )
        .await
    }

    /// Create a new key-value pair, or return an existing one.
    fn create_account(
        &mut self,
        account_name: &crate::AccountName,
    ) -> anyhow::Result<DefaultAccount> {
        DefaultAccount::get_or_create(&mut self.rng, &self.dir, account_name)
    }

    async fn fund_from_faucet<'s, 'a>(
        &'s mut self,
        account: &'a DefaultAccount,
        reference: Option<ResourceHash>,
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        todo!("use curl or something to trigger the faucet")
    }

    async fn new_deployment<'s, 'a>(
        &'s mut self,
        subnet_name: &SubnetName,
        deployer: &'a DefaultAccount,
        urls: Vec<Url>,
    ) -> anyhow::Result<DefaultDeployment>
    where
        's: 'a,
    {
        todo!("use the deploy scripts to create a new IPC stack on L1")
    }

    fn existing_deployment(
        &mut self,
        subnet_name: &SubnetName,
        gateway: H160,
        registry: H160,
    ) -> anyhow::Result<DefaultDeployment> {
        Ok(DefaultDeployment {
            name: subnet_name.clone(),
            gateway: EthAddress::from(gateway),
            registry: EthAddress::from(registry),
        })
    }

    fn default_deployment(
        &mut self,
        subnet_name: &SubnetName,
    ) -> anyhow::Result<DefaultDeployment> {
        Ok(DefaultDeployment::builtin(subnet_name.clone()))
    }

    /// Check if a genesis file already exists. If so, parse it, otherwise
    /// create an in-memory representation of a genesis file and export it.
    fn create_root_genesis<'a>(
        &mut self,
        subnet_name: &SubnetName,
        validators: BTreeMap<&'a DefaultAccount, Collateral>,
        balances: BTreeMap<&'a DefaultAccount, Balance>,
    ) -> anyhow::Result<DefaultGenesis> {
        self.get_or_create_genesis(subnet_name, || {
            let chain_name = subnet_name.path_string();
            let chain_id = chainid::from_str_hashed(&chain_name)?;
            // TODO: Some of these hardcoded values can go into the manifest.
            let genesis = Genesis {
                chain_name,
                timestamp: Timestamp::current(),
                network_version: NetworkVersion::V21,
                base_fee: TokenAmount::zero(),
                power_scale: 3,
                validators: validators
                    .into_iter()
                    .map(|(v, c)| Validator {
                        public_key: ValidatorKey(*v.public_key()),
                        power: c,
                    })
                    .collect(),
                accounts: balances
                    .into_iter()
                    .map(|(a, b)| Actor {
                        meta: ActorMeta::Account(Account {
                            owner: SignerAddr(a.fvm_addr()),
                        }),
                        balance: b.0,
                    })
                    .collect(),
                eam_permission_mode: fendermint_vm_genesis::PermissionMode::Unrestricted,
                ipc: Some(IpcParams {
                    gateway: GatewayParams {
                        subnet_id: SubnetID::new_root(chain_id.into()),
                        // TODO: The gateway constructor doesn't allow 0 bottom-up-checkpoint-period even on the rootnet!
                        bottom_up_check_period: 1,
                        majority_percentage: 67,
                        active_validators_limit: 100,
                    },
                }),
            };
            Ok(genesis)
        })
    }

    /// Get or create all docker containers that constitute to a Node.
    async fn create_node<'s, 'a>(
        &'s mut self,
        node_name: &NodeName,
        node_config: NodeConfig<'a, DockerMaterials>,
    ) -> anyhow::Result<DockerNode>
    where
        's: 'a,
    {
        // Pick a port range.
        let port_range = self
            .port_range(node_name)
            .context("failed to pick port range")?;

        // We could write a (shared) docker-compose.yaml file and .env file per node,
        // however the `bollard` library doesn't support docker-compose, so different
        // techniques would need to be used. Alternatively we can just use `Docker`
        // and run three different containers.
        DockerNode::get_or_create(
            &self.dir,
            self.docker.clone(),
            self.drop_chute.clone(),
            &self.drop_policy,
            node_name,
            node_config,
            port_range,
        )
        .await
        .context("failed to create node")
    }

    async fn start_node<'s, 'a>(
        &'s mut self,
        node: &'a DockerNode,
        seed_nodes: &'a [&'a DockerNode],
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        // Overwrite the env file which has seed addresses, then start the node (unless it's already running).
        node.start(seed_nodes).await
    }

    async fn create_subnet<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, DockerMaterials>,
        subnet_name: &SubnetName,
        subnet_config: SubnetConfig<'a, DockerMaterials>,
    ) -> anyhow::Result<DefaultSubnet>
    where
        's: 'a,
    {
        todo!("use the ipc-cli to create a new subnet on the parent")
    }

    async fn fund_subnet<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, DockerMaterials>,
        account: &'a DefaultAccount,
        subnet: &'a DefaultSubnet,
        amount: fvm_shared::econ::TokenAmount,
        reference: Option<ResourceHash>,
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        todo!("use the ipc-cli to fund an existing subnet on the parent")
    }

    async fn join_subnet<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, DockerMaterials>,
        account: &'a DefaultAccount,
        subnet: &'a DefaultSubnet,
        collateral: fendermint_vm_genesis::Collateral,
        balance: Balance,
        reference: Option<ResourceHash>,
    ) -> anyhow::Result<()>
    where
        's: 'a,
    {
        todo!("use the ipc-cli to join an existing subnet on the parent")
    }

    async fn create_subnet_genesis<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, DockerMaterials>,
        subnet: &'a DefaultSubnet,
    ) -> anyhow::Result<DefaultGenesis>
    where
        's: 'a,
    {
        todo!("use the fendermint CLI to fetch the genesis of a subnet from the parent")
    }

    async fn create_relayer<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, DockerMaterials>,
        relayer_name: &RelayerName,
        subnet: &'a DefaultSubnet,
        submitter: &'a DefaultAccount,
        follow_node: &'a DockerNode,
    ) -> anyhow::Result<DockerRelayer>
    where
        's: 'a,
    {
        todo!("docker run relayer unless it is already running")
    }
}
