// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::Path;

use anyhow::Context;
use ethers::{core::rand::Rng, types::H160};
use fendermint_crypto::{to_b64, PublicKey, SecretKey};
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_genesis::Genesis;
use fvm_shared::address::Address;
use ipc_api::subnet_id::SubnetID;

use crate::{AccountName, SubnetName};

/// Type family of all the things a [Materializer] can create.
///
/// Kept separate from the [Materializer] so that we can wrap one in another
/// and pass the same types along.
pub trait Materials {
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
}

pub struct DefaultDeployment {
    pub name: SubnetName,
    pub gateway: H160,
    pub registry: H160,
}

pub struct DefaultGenesis {
    pub name: SubnetName,
    /// In-memory representation of the `genesis.json` file.
    pub genesis: Genesis,
}

pub struct DefaultSubnet {
    pub name: SubnetName,
    /// ID allocated to the subnet during creation.
    pub subnet_id: SubnetID,
}

#[derive(PartialEq, Eq)]
pub struct DefaultAccount {
    pub name: AccountName,
    pub secret_key: SecretKey,
    pub public_key: PublicKey,
}

impl PartialOrd for DefaultAccount {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DefaultAccount {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl DefaultAccount {
    pub fn eth_addr(&self) -> EthAddress {
        EthAddress::from(self.public_key)
    }

    /// We assume that all accounts that interact with IPC are ethereum accounts.
    pub fn fvm_addr(&self) -> Address {
        self.eth_addr().into()
    }

    pub fn get_or_create<R: Rng>(
        rng: &mut R,
        root: &Path,
        name: &AccountName,
    ) -> anyhow::Result<Self> {
        let dir = root.join(name.path());
        let sk = dir.join("secret.hex");

        let (sk, pk, is_new) = if sk.exists() {
            let sk = std::fs::read_to_string(sk).context("failed to read private key")?;
            let sk = hex::decode(sk).context("cannot decode hex private key")?;
            let sk = SecretKey::try_from(sk).context("failed to parse secret key")?;
            let pk = sk.public_key();
            (sk, pk, false)
        } else {
            let sk = SecretKey::random(rng);
            let pk = sk.public_key();
            (sk, pk, true)
        };

        let acc = Self {
            name: name.clone(),
            secret_key: sk,
            public_key: pk,
        };

        if is_new {
            let sk = acc.secret_key.serialize();
            let pk = acc.public_key.serialize();

            export(&dir, "secret", "b64", to_b64(sk.as_ref()))?;
            export(&dir, "secret", "hex", hex::encode(pk))?;
            export(&dir, "public", "b64", to_b64(sk.as_ref()))?;
            export(&dir, "public", "hex", hex::encode(pk))?;
            export(&dir, "eth-addr", "", acc.eth_addr().to_string())?;
            export(&dir, "fvm-addr", "", acc.fvm_addr().to_string())?;
        }

        Ok(acc)
    }
}

/// Write some content to a file.
pub fn export(
    output_dir: impl AsRef<Path>,
    name: &str,
    ext: &str,
    contents: impl AsRef<str>,
) -> anyhow::Result<()> {
    let file_name = if ext.is_empty() {
        name.into()
    } else {
        format!("{name}.{ext}")
    };
    let output_path = output_dir.as_ref().join(file_name);
    std::fs::write(output_path, contents.as_ref())?;
    Ok(())
}
