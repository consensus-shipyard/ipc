use std::{
    borrow::Cow,
    fmt::Display,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT
pub mod manifest;
pub mod materializer;
pub mod testnet;

#[cfg(feature = "arb")]
mod arb;

/// An ID identifying a resource within its parent.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceId(Cow<'static, str>);

impl Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", self.0)
    }
}

impl From<&'static str> for ResourceId {
    fn from(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for ResourceId {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl From<&ResourceId> for ResourceId {
    fn from(value: &Self) -> Self {
        value.clone()
    }
}

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
pub struct ResourceName(pub PathBuf);

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
pub struct AccountName(ResourceName);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SubnetName(ResourceName);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodeName(ResourceName);

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelayerName(ResourceName);

impl NodeName {
    pub fn is_in_subnet(&self, subnet_name: &SubnetName) -> bool {
        subnet_name.0.is_prefix_of(&self.0)
    }
}

impl AccountName {
    pub fn new(account_id: &AccountId) -> Self {
        Self(ResourceName::from("/account").join_id(&account_id))
    }
}

impl SubnetName {
    pub fn root() -> Self {
        Self(ResourceName::from("/root"))
    }

    pub fn subnet<S: Into<SubnetId>>(&self, subnet_id: S) -> Self {
        Self(self.0.join("subnets").join_id(&subnet_id.into()))
    }

    pub fn node<T: Into<NodeId>>(&self, node_id: T) -> NodeName {
        NodeName(self.0.join("nodes").join_id(&node_id.into()))
    }

    pub fn relayer<T: Into<RelayerId>>(&self, relayer_id: T) -> RelayerName {
        RelayerName(self.0.join("relayers").join_id(&relayer_id.into()))
    }

    pub fn is_root(&self) -> bool {
        match self.path().parent() {
            None => true,
            Some(p) => p.ends_with("/"),
        }
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

    /// All the subnet names from the root to the parent of the subnet.
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
            .cloned()
            .skip(1)
            .chain(std::iter::once(self.clone()))
            .collect::<Vec<_>>();

        ss0.into_iter().zip(ss1.into_iter()).collect()
    }

    fn path(&self) -> &Path {
        &self.0 .0
    }
}

#[cfg(test)]
mod tests {
    use crate::SubnetName;

    #[test]
    fn test_subnet_parent() {
        let root = SubnetName::root();
        assert_eq!(root.parent(), None);

        let sub = root.subnet("foo");
        assert_eq!(sub.parent(), Some(root));
    }

    #[test]
    fn test_subnet_ancestors() {
        let subnet = SubnetName::root().subnet("foo").subnet("bar");
        assert_eq!(
            subnet.ancestors(),
            vec![SubnetName::root(), SubnetName::root().subnet("foo")]
        );
    }

    #[test]
    fn test_subnet_ancestor_hops() {
        let root = SubnetName::root();
        let foo = root.subnet("foo");
        let bar = foo.subnet("bar");
        assert_eq!(bar.ancestor_hops(), vec![(root, foo.clone()), (foo, bar)]);
    }

    #[test]
    fn test_node_subnet() {
        let subnet = SubnetName::root().subnet("foo");
        let node = subnet.node("node-1");

        assert!(node.is_in_subnet(&subnet));
    }
}
