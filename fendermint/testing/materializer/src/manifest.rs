// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

// See https://github.com/cometbft/cometbft/blob/v0.38.5/test/e2e/pkg/manifest.go for inspiration.

use fvm_shared::{address::Address, econ::TokenAmount};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{collections::BTreeMap, path::PathBuf};

use fendermint_vm_encoding::IsHumanReadable;
use fendermint_vm_genesis::Collateral;

/// An ID identifying a resource within its parent.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceId(pub String);

/// The name of a resource consists of its ID and all the IDs of its ancestors
/// concatenated into a URL-like path.
///
/// See <https://cloud.google.com/apis/design/resource_names>
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResourceName(pub PathBuf);

/// A human readable name for an account.
pub type AccountId = ResourceId;

/// A human readable name for a subnet.
pub type SubnetId = ResourceId;

/// A human readable name for a node.
pub type NodeId = ResourceId;

/// A human readable name for a relayer.
pub type RelayerId = ResourceId;

pub type SubnetMap = BTreeMap<SubnetId, Subnet>;
pub type BalanceMap = BTreeMap<AccountId, Balance>;
pub type CollateralMap = BTreeMap<AccountId, Collateral>;
pub type NodeMap = BTreeMap<NodeId, Node>;
pub type RelayerMap = BTreeMap<RelayerId, Relayer>;

/// The manifest is a static description of a testnet.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    /// All the accounts we want to act with across the entire subnet hierarchy.
    ///
    /// Each account will have its pair of private and public keys.
    ///
    /// In the rootnet, if we are dealing with Calibration, they will get their
    /// initial balance from the Faucet, which should give 100 tFIL ("testnet" FIL),
    /// which is why there is no definition for the account balances for the root.
    ///
    /// This would be different if we deployed a root in the test, e.g. using
    /// Fendermint itself, in which case we could set whatever balance we wanted.
    pub accounts: BTreeMap<AccountId, Account>,

    /// Which account to use to deploy the IPC contracts, if we have to.
    pub deployment: RootDeployment,

    /// Subnets created on the rootnet.
    pub subnets: SubnetMap,
}

/// Any potential attributes of an account.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Account {}

/// Account balance.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Balance(#[serde_as(as = "IsHumanReadable")] pub TokenAmount);

/// Ways we can hook up with IPC contracts on the rootnet.
///
/// The rootnet is generally expected to be Calibration net,
/// where IPC contracts are deployed from Hardhat, and multiple
/// instances of the gateway exist, each with a different version
/// and an address we learn after deployment.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RootDeployment {
    /// Deploy a new IPC contract stack using one of the accounts.
    /// This can take a long time, but ensures we are testing with
    /// contracts that have the same version as the client.
    New { deployer: AccountId },
    /// Use one of the existing deployments, given by the delegated address of
    /// the Gateway and Registry contracts.
    Existing {
        #[serde_as(as = "IsHumanReadable")]
        gateway: Address,
        #[serde_as(as = "IsHumanReadable")]
        registry: Address,
    },
}

/// An IPC subnet.
///
/// The balance of the account on the parent subnet, as declared in this manifest,
/// _does not_ have to account for the collateral/balance we have to take from it to join/fund the subnet.
/// When we create the testnet configuration from the manifest we will account for this with a rollup,
/// so we don't have to do that much mental arithmetic and run into frustrating errors during setup.
/// If we want to test trying to join with more than what we have, we can do so in the integration test.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Subnet {
    /// The account we use to create the subnet.
    pub creator: AccountId,
    /// Collateral of the initial validator set.
    ///
    /// These validators will join the subnet with these collaterals after the subnet is created.
    pub validators: CollateralMap,
    /// Balances of the accounts at the creation of the subnet.
    ///
    /// These accounts will pre-fund the subnet after it's created.
    pub balances: BalanceMap,
    /// Nodes that participate in running the chain of this subnet.
    pub nodes: NodeMap,
    /// Relayers that submit bottom-up checkpoints to the parent subnet.
    pub relayers: RelayerMap,
    /// Child subnets under this parent.
    ///
    /// The subnet ID exists so we can find the outcome of existing deployments in the log.
    pub subnets: SubnetMap,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Node {
    /// Indicate whether this is a validator node or a full node.
    pub mode: NodeMode,
    /// Indicate whether to run the Ethereum API.
    pub ethapi: bool,
    /// The nodes from which CometBFT should bootstrap itself.
    pub seed_nodes: Vec<NodeId>,
    /// The parent node that the top-down syncer follows;
    /// or leave it empty if the parent is CalibrationNet.
    pub parent_node: Option<NodeId>,
}

/// The mode in which CometBFT is running.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeMode {
    /// A node able to create and sign blocks.
    Validator(AccountId),
    /// A node which runs consensus and executes blocks, but doesn't have a validator key.
    Full,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Relayer {
    /// The account which will pay for the submission on the parent subnet.
    pub submitter: AccountId,
    /// The node which the relayer is following on the subnet.
    pub follow_node: NodeId,
    /// The node where the relayer submits the checkpoints;
    /// or leave it empty if the parent is CalibrationNet.
    pub submit_node: Option<NodeId>,
}

#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;

    use super::Manifest;

    #[quickcheck]
    fn manifest_json(value0: Manifest) {
        let repr = serde_json::to_string(&value0).expect("failed to encode");
        let value1: Manifest = serde_json::from_str(&repr)
            .map_err(|e| format!("{e}; {repr}"))
            .expect("failed to decode JSON");

        assert_eq!(value1, value0)
    }

    #[quickcheck]
    fn manifest_yaml(value0: Manifest) {
        let repr = serde_yaml::to_string(&value0).expect("failed to encode");
        let value1: Manifest = serde_yaml::from_str(&repr)
            .map_err(|e| format!("{e}; {repr}"))
            .expect("failed to decode");

        assert_eq!(value1, value0)
    }
}
