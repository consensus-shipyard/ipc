// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

pub mod manifest;
pub mod materializer;
pub mod testnet;

#[cfg(feature = "arb")]
mod arb;

/// An ID identifying a resource within its parent.
#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceId(String);

impl<'de> Deserialize<'de> for ResourceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(|s| Self::from(s))
    }
}

impl Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", self.0)
    }
}

impl From<&str> for ResourceId {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}

impl From<String> for ResourceId {
    fn from(value: String) -> Self {
        Self(value.replace('/', "_"))
    }
}

impl From<&ResourceId> for ResourceId {
    fn from(value: &Self) -> Self {
        value.clone()
    }
}

/// A human readable name for a testnet.
pub type TestnetId = ResourceId;

/// A human readable name for an account.
pub type AccountId = ResourceId;

/// A human readable name for a subnet.
pub type SubnetId = ResourceId;

/// A human readable name for a node.
pub type NodeId = ResourceId;

/// A human readable name for a relayer.
pub type RelayerId = ResourceId;

/// The name of a resource consists of its ID and all the IDs of its ancestors
/// concatenated into a URL-like path.
///
/// See <https://cloud.google.com/apis/design/resource_names>
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceName(PathBuf);

impl ResourceName {
    fn join(&self, s: &str) -> Self {
        Self(self.0.join(s))
    }

    fn join_id(&self, id: &ResourceId) -> Self {
        self.join(&id.0)
    }

    pub fn is_prefix_of(&self, other: &ResourceName) -> bool {
        other.0.starts_with(&self.0)
    }
}

impl From<&str> for ResourceName {
    fn from(value: &str) -> Self {
        Self(PathBuf::from(value))
    }
}

impl Display for ResourceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", self.0.to_string_lossy())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct TestnetName(ResourceName);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccountName(ResourceName);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubnetName(ResourceName);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeName(ResourceName);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelayerName(ResourceName);

impl TestnetName {
    pub fn new<T: Into<TestnetId>>(id: T) -> Self {
        Self(ResourceName::from("/testnets").join_id(&id.into()))
    }

    pub fn account<T: Into<AccountId>>(&self, id: T) -> AccountName {
        AccountName(self.0.join("accounts").join_id(&id.into()))
    }

    pub fn root(&self) -> SubnetName {
        SubnetName(self.0.join("root"))
    }
}

impl NodeName {
    pub fn is_in_subnet(&self, subnet_name: &SubnetName) -> bool {
        subnet_name.0.is_prefix_of(&self.0)
    }
}

impl SubnetName {
    pub fn subnet<T: Into<SubnetId>>(&self, id: T) -> Self {
        Self(self.0.join("subnets").join_id(&id.into()))
    }

    pub fn node<T: Into<NodeId>>(&self, id: T) -> NodeName {
        NodeName(self.0.join("nodes").join_id(&id.into()))
    }

    pub fn relayer<T: Into<RelayerId>>(&self, id: T) -> RelayerName {
        RelayerName(self.0.join("relayers").join_id(&id.into()))
    }

    pub fn is_root(&self) -> bool {
        self.path().components().count() == 4 && self.path().ends_with("root")
    }

    pub fn parent(&self) -> Option<SubnetName> {
        if self.is_root() {
            None
        } else {
            let path = self
                .path()
                .parent()
                .and_then(|p| p.parent())
                .expect("invalid subnet path");

            Some(Self(ResourceName(path.into())))
        }
    }

    /// All the subnet names from the root to the parent of the subnet,
    /// excluding the subnet itself.
    pub fn ancestors(&self) -> Vec<SubnetName> {
        let mut ss = Vec::new();
        let mut p = self.parent();
        while let Some(s) = p {
            p = s.parent();
            ss.push(s);
        }
        ss.reverse();
        ss
    }

    /// parent->child hop pairs from the root to the subnet.
    pub fn ancestor_hops(&self) -> Vec<(SubnetName, SubnetName)> {
        let ss0 = self.ancestors();

        let ss1 = ss0
            .iter()
            .skip(1)
            .chain(std::iter::once(self))
            .cloned()
            .collect::<Vec<_>>();

        ss0.into_iter().zip(ss1).collect()
    }

    fn path(&self) -> &Path {
        &self.0 .0
    }
}

#[cfg(test)]
mod tests {
    use crate::TestnetName;

    #[test]
    fn test_subnet_parent() {
        let tn = TestnetName::new("example");
        let rn = tn.root();
        assert_eq!(rn.parent(), None);

        let sn = rn.subnet("foo");
        assert_eq!(sn.parent(), Some(rn));
    }

    #[test]
    fn test_subnet_ancestors() {
        let tn = TestnetName::new("example");
        let sn = tn.root().subnet("foo").subnet("bar");
        assert_eq!(sn.ancestors(), vec![tn.root(), tn.root().subnet("foo")]);
    }

    #[test]
    fn test_subnet_ancestor_hops() {
        let tn = TestnetName::new("example");
        let rn = tn.root();
        let foo = rn.subnet("foo");
        let bar = foo.subnet("bar");
        assert_eq!(bar.ancestor_hops(), vec![(rn, foo.clone()), (foo, bar)]);
    }

    #[test]
    fn test_node_subnet() {
        let tn = TestnetName::new("example");
        let sn = tn.root().subnet("foo");
        let node = sn.node("node-1");

        assert!(node.is_in_subnet(&sn));
    }
}
