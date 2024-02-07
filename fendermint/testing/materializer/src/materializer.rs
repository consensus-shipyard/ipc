// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use async_trait::async_trait;
use fvm_shared::address::Address;
use std::collections::BTreeMap;

use fendermint_vm_genesis::Collateral;

use crate::{manifest::Balance, AccountName, NodeName, SubnetName};

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
/// For example we can create an run a testnet externally, then parse the manifest
/// and the materializer logs inside a test to talk to one of the nodew, and the
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
    type Account;
    type Genesis;
    type Node;

    /// Create a Secp256k1 keypair for signing transactions or creating blocks.
    fn create_account(&mut self, account_name: AccountName) -> Self::Account;

    /// Fund an account on the rootnet from the faucet.
    async fn fund_from_faucet(&mut self, account: &Self::Account) -> anyhow::Result<()>;

    /// Deploy the IPC contracts onto the rootnet.
    ///
    /// This is assumed to be used with external subnets, with the API address
    /// being known to the materializer, but not being part of the manifest,
    /// as there can be multiple endpoints to choose from, some better than others.
    ///
    /// The materializer should remember the addresses of the deployments.
    async fn deploy_root_ipc(&mut self, deployer: &Self::Account) -> anyhow::Result<()>;

    /// Set the IPC contracts onto the rootnet.
    ///
    /// This is assumed to be used with external subnets, with the API address
    /// being known to the materializer, but not being part of the manifest,
    /// as there can be multiple endpoints to choose from, some better than others.
    ///
    /// The materializer should remember the addresses of the deployments.
    async fn set_root_ipc(&mut self, gateway: Address, registry: Address) -> anyhow::Result<()>;

    /// Construct the genesis for a subnet.
    ///
    /// The genesis time and the chain name (which should determine the chain ID and
    /// thus the subnet ID as well) can be chosen by the materializer.
    fn create_root_genesis(
        &mut self,
        subnet_name: SubnetName,
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
    async fn create_node(
        &mut self,
        node_name: NodeName,
        node_config: NodeConfig<Self>,
    ) -> anyhow::Result<Self::Node>;

    /// Start a node.
    ///
    /// At this point the identities of any dependency nodes should be known.
    async fn start_node(
        &mut self,
        node: &Self::Node,
        seed_nodes: &[&Self::Node],
    ) -> anyhow::Result<()>;
}

pub struct NodeConfig<'a, M>
where
    M: Materializer + ?Sized,
{
    /// The genesis of this subnet; it should indicate whether this is a rootnet or a deeper level.
    pub genesis: &'a M::Genesis,
    /// The validator keys if this is a validator node; none if just a full node.
    pub validator: Option<&'a M::Account>,
    /// The node for the top-down syncer to follow; none if this is a root node, or if the parent is an external address.
    pub parent_node: Option<&'a M::Node>,
    /// Run the Ethereum API facade or not.
    pub ethapi: bool,
}
