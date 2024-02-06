use std::{
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
pub struct ResourceId(pub String);

impl Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}'", self.0)
    }
}

impl From<&str> for ResourceId {
    fn from(value: &str) -> Self {
        Self(value.into())
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
    pub fn join(&self, s: &str) -> Self {
        Self(self.0.join(s))
    }

    pub fn join_id(&self, id: &ResourceId) -> Self {
        self.join(&id.0)
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

impl AccountName {
    pub fn new(account_id: &AccountId) -> Self {
        Self(ResourceName::from("/account").join_id(&account_id))
    }
}

impl SubnetName {
    pub fn root() -> Self {
        Self(ResourceName::from("/root"))
    }

    pub fn node(&self, node_id: &NodeId) -> NodeName {
        NodeName(self.0.join("nodes").join_id(node_id))
    }

    pub fn subnet(&self, subnet_id: &SubnetId) -> Self {
        Self(self.0.join("subnets").join_id(subnet_id))
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

    fn path(&self) -> &Path {
        &self.0 .0
    }
}

#[cfg(test)]
mod tests {
    use crate::{SubnetId, SubnetName};

    #[test]
    fn test_subnet_parent() {
        let root = SubnetName::root();
        assert_eq!(root.parent(), None);

        let sub = root.subnet(&SubnetId::from("foo"));
        assert_eq!(sub.parent(), Some(root));
    }
}
