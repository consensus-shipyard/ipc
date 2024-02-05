use std::collections::BTreeMap;

// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use async_trait::async_trait;
use fendermint_vm_genesis::Collateral;

use crate::manifest::{AccountId, Balance, ResourceName};

/// The identifier of a materialized account, that is, a handle to the keys.
pub type AccountName = ResourceName;

/// Resource name of a generated `genesis.json` file.
pub type GenesisName = ResourceName;

/// The materializer is similar to a State Monad in that it receives self as state,
/// along with some operations parameters and a resource ID, potentially modifies
/// its state to remember what it did, and returns a resource name which can be
/// used in further operations to refer to the results of previous steps.
///
/// For example if we ask the materializer to create `node-1` on `/subnets/subnet-1`,
/// it might return a resource name such as `/subnets/subnet-1/nodes/node-1`, which
/// is a handle we can use in subsequent operations, for example to restart the node,
/// or perhaps to use it as a seed node on `node-2` of the same subnet.
///
/// The materializer might not immediately instantiate the resources. By returning
/// resource names instead of concrete values, it is possible to just collect the
/// operations and use them to validate the behaviour of whatever is driving
/// the materializer. We can use this for dry-runs as well.
///
/// A live materializer should persist its logs, so that it can be resumed.
/// For example we can create an run a testnet externally, then parse the manifest
/// and the materializer logs inside a test to talk to one of the nodew, and the
/// materializer should be able to return to the test correct JSON-RPC endpoints.
///
/// Some of the operations of the materializer should be idempotent, e.g. the
/// creation of a wallet or a node should only happpen once.
///
/// The operations of a materializer might fail if they are executed in the wrong
/// order. For example if we ask the materializer to fund a wallet on a non-existing
/// subnet, it might return an error if that subnet hasn't been created yet.
#[async_trait]
pub trait Materializer {
    /// Create a Secp256k1 keypair for signing transactions or creating blocks.
    fn create_account(&mut self, account_id: AccountId) -> AccountName;

    /// Deploy the IPC contracts onto the rootnet.
    ///
    /// This is assumed to work with external subnets, with the address being known
    /// to the materializer, but not being part of the manifest as there can be
    /// multiple endpoints to choose from, some better than others.
    ///
    /// The materializer should remember the addresses of the deployments.
    async fn deploy_ipc(&mut self, deployer: AccountName) -> anyhow::Result<()>;

    /// Create a genesis file.
    ///
    /// Returns the identifier of the generated file which we can use to refer to
    /// this file when creating nodes on the subnet.
    ///
    /// The genesis can include multiple files, if necessary. Its format can be
    /// specific to the system we are materializing, which is why it's here.
    ///
    /// Returns an error if the accounts don't exist.
    ///
    /// The genesis time and the chain name (which should determine the chain ID and
    /// thus the subnet ID as well) can be chosen by the materializer.
    fn create_genesis(
        &self,
        validators: BTreeMap<AccountName, Collateral>,
        balances: BTreeMap<AccountName, Balance>,
    ) -> anyhow::Result<GenesisName>;
}
