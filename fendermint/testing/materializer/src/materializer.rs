// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use async_trait::async_trait;
use ethers::types::H160;
use fvm_shared::econ::TokenAmount;
use std::collections::BTreeMap;
use tendermint_rpc::Url;

use fendermint_vm_genesis::Collateral;

use crate::{
    manifest::Balance, AccountName, NodeName, RelayerName, ResourceHash, SubnetName, TestnetName,
};

/// The materializer is a component to provision resources of a testnet, and
/// to carry out subsequent commands on them, e.g. to restart nodes.
///
/// By contrast, the role of the [Testnet] is to keep related items organised
/// and accessible for the integration tests, carrying out the operations with
/// the help of the materializer, which should keep the [Testnet] itself testable.
///
/// The materializer might not actually instantiate the resources. By returning
/// abstract types instead of concrete values, it is possible to just collect the
/// operations and use them to validate the behaviour of whatever is driving
/// the materializer. We can use this for dry-runs as well.
///
/// A live materializer should persist its logs, so that it can be resumed.
/// For example we can create and run a testnet externally, then parse the manifest
/// and the materializer logs inside a test to talk to one of the nodes, and the
/// materializer should be able to return to the test correct JSON-RPC endpoints.
///
/// Some of the operations of the materializer should be idempotent, e.g. the
/// creation of a wallet or a node should only happen once.
///
/// The types returned might have their own logic to execute when dropped, to free
/// resources. This might happen only if the resource is not an externally managed
/// one, e.g. a testnet set up before tests are run, which the materializer should
/// know.
#[async_trait]
pub trait Materializer {
    /// Represents the entire hierarchy of a testnet, e.g. a common docker network
    /// and directory on the file system. It has its own type so the materializer
    /// doesn't have to remember what it created for a testnet, and different
    /// testnets can be kept isolated from each other.
    type Network: Send + Sync;
    /// Capture where the IPC stack (the gateway and the registry) has been deployed on a subnet.
    /// These are the details which normally go into the `ipc-cli` configuration files.
    type Deployment: Sync + Send;
    /// Represents an account identity, typically a key-value pair.
    type Account: Ord + Sync + Send;
    /// Represents the genesis.json file (can be a file location, or a model).
    type Genesis: Sync + Send;
    /// The address of a dynamically created subnet.
    type Subnet: Sync + Send;
    /// The handle to a node; could be a (set of) docker container(s) or remote addresses.
    type Node: Sync + Send;
    /// The handle to a relayer process.
    type Relayer: Sync + Send;

    /// Create the physical network group.
    ///
    /// The return value should be able to able to represent settings that allow nodes
    /// to connect to each other, as well as perhaps to be labelled as a group
    /// (although for that we can use the common name prefixes as well).
    async fn create_network(&mut self, testnet_name: &TestnetName)
        -> anyhow::Result<Self::Network>;

    /// Create a Secp256k1 keypair for signing transactions or creating blocks.
    fn create_account(&mut self, account_name: &AccountName) -> anyhow::Result<Self::Account>;

    /// Fund an account on the rootnet from the faucet.
    async fn fund_from_faucet(
        &mut self,
        account: &Self::Account,
        reference: Option<ResourceHash>,
    ) -> anyhow::Result<()>;

    /// Deploy the IPC contracts onto the rootnet.
    ///
    /// This is assumed to be used with external subnets, with the API address
    /// being known to the materializer, but not being part of the manifest,
    /// as there can be multiple endpoints to choose from, some better than others.
    ///
    /// The return value should contain at the addresses of the contracts.
    async fn new_deployment(
        &mut self,
        subnet_name: &SubnetName,
        deployer: &Self::Account,
        urls: Vec<Url>,
    ) -> anyhow::Result<Self::Deployment>;

    /// Set the IPC contracts onto the rootnet.
    ///
    /// This is assumed to be used with external subnets, with the API address
    /// being known to the materializer, but not being part of the manifest,
    /// as there can be multiple endpoints to choose from, some better than others.
    fn existing_deployment(
        &mut self,
        subnet_name: &SubnetName,
        gateway: H160,
        registry: H160,
    ) -> anyhow::Result<Self::Deployment>;

    /// Return the well-known IPC contract deployments.
    fn default_deployment(&mut self, subnet_name: &SubnetName) -> anyhow::Result<Self::Deployment>;

    /// Construct the genesis for the rootnet.
    ///
    /// The genesis time and the chain name (which should determine the chain ID and
    /// thus the subnet ID as well) can be chosen by the materializer, or we could make
    /// it part of the manifest.
    fn create_root_genesis(
        &mut self,
        subnet_name: &SubnetName,
        validators: BTreeMap<&Self::Account, Collateral>,
        balances: BTreeMap<&Self::Account, Balance>,
    ) -> anyhow::Result<Self::Genesis>;

    /// Construct the configuration for a node.
    ///
    /// This should create keys, configurations, but hold on from starting so that we can
    /// first learn about the dynamic properties of other nodes in the cluster we depend on,
    /// such as their network identities which are a function of their keys.
    ///
    /// The method is async in case we have to provision some resources remotely.
    async fn create_node<'s, 'a>(
        &'s mut self,
        node_name: &NodeName,
        node_config: NodeConfig<'a, Self>,
    ) -> anyhow::Result<Self::Node>
    where
        's: 'a;

    /// Start a node.
    ///
    /// At this point the identities of any dependency nodes should be known.
    async fn start_node(
        &mut self,
        node: &Self::Node,
        seed_nodes: &[&Self::Node],
    ) -> anyhow::Result<()>;

    /// Create a subnet on the parent subnet ledger.
    ///
    /// The parent nodes are the ones where subnet-creating transactions
    /// can be sent, or it can be empty if it's an external rootnet.
    ///
    /// The result should contain the address of the subnet.
    async fn create_subnet<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, Self>,
        subnet_name: &SubnetName,
        subnet_config: SubnetConfig<'a, Self>,
    ) -> anyhow::Result<Self::Subnet>
    where
        's: 'a;

    /// Fund an account on a target subnet by transferring tokens from the source subnet.
    ///
    /// Only works if the target subnet has been bootstrapped.
    ///
    /// The `reference` can be used to deduplicate repeated transfer attempts.
    async fn fund_subnet<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, Self>,
        account: &Self::Account,
        subnet: &Self::Subnet,
        amount: TokenAmount,
        reference: Option<ResourceHash>,
    ) -> anyhow::Result<()>
    where
        's: 'a;

    /// Join a target subnet as a validator.
    ///
    /// The `reference` can be used to deduplicate repeated transfer attempts.
    async fn join_subnet<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, Self>,
        account: &Self::Account,
        subnet: &Self::Subnet,
        collateral: Collateral,
        balance: Balance,
        reference: Option<ResourceHash>,
    ) -> anyhow::Result<()>
    where
        's: 'a;

    /// Construct the genesis for a subnet, which involves fetching details from the parent.
    async fn create_subnet_genesis<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, Self>,
        subnet: &Self::Subnet,
    ) -> anyhow::Result<Self::Genesis>
    where
        's: 'a;

    /// Create and start a relayer.
    ///
    /// It should follow the given node. If the submit node is empty, it should submit to an external rootnet.
    async fn create_relayer<'s, 'a>(
        &'s mut self,
        parent_submit_config: &SubmitConfig<'a, Self>,
        relayer_name: &RelayerName,
        subnet: &Self::Subnet,
        submitter: &Self::Account,
        follow_node: &Self::Node,
    ) -> anyhow::Result<Self::Relayer>
    where
        's: 'a;
}

/// Options regarding node configuration, e.g. which services to start.
pub struct NodeConfig<'a, M>
where
    M: Materializer + ?Sized,
{
    /// The physical network to join.
    pub network: &'a M::Network,
    /// The genesis of this subnet; it should indicate whether this is a rootnet or a deeper level.
    pub genesis: &'a M::Genesis,
    /// The validator keys if this is a validator node; none if just a full node.
    pub validator: Option<&'a M::Account>,
    /// The node for the top-down syncer to follow; none if this is a root node.
    ///
    /// This can potentially also be used to configure the IPLD Resolver seeds, to connect across subnets.
    pub parent_node: Option<TargetConfig<'a, M>>,
    /// Run the Ethereum API facade or not.
    pub ethapi: bool,
}

/// Options regarding subnet configuration, e.g. how many validators are required.
pub struct SubnetConfig<'a, M>
where
    M: Materializer + ?Sized,
{
    /// Which account to use on the parent to create the subnet.
    ///
    /// This account has to have the necessary balance on the parent.
    pub creator: &'a M::Account,
    /// Number of validators required for bootstrapping a subnet.
    pub min_validators: usize,
}

/// Options for how to submit transactions to a subnet.
pub struct SubmitConfig<'a, M>
where
    M: Materializer + ?Sized,
{
    /// The nodes to which we can send transactions or queries.
    pub nodes: Vec<TargetConfig<'a, M>>,
    /// The location of the IPC contracts on the (generally parent) subnet.
    pub deployment: &'a M::Deployment,
}

pub enum TargetConfig<'a, M>
where
    M: Materializer + ?Sized,
{
    External(Url),
    Internal(&'a M::Node),
}
