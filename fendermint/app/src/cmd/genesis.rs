// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, Context};
use fvm_shared::address::Address;
use libsecp256k1::PublicKey;
use std::path::PathBuf;

use fendermint_vm_genesis::{Account, Actor, ActorAddr, ActorMeta, Genesis, Multisig};

use crate::cmd;
use crate::options::{GenesisAddAccountArgs, GenesisAddMultisigArgs, GenesisNewArgs};

use super::keygen::b64_to_public;

cmd! {
  GenesisNewArgs(self, genesis_file: PathBuf) {
    let genesis = Genesis {
      network_name: self.network_name.clone(),
      network_version: self.network_version,
      base_fee: self.base_fee.clone(),
      validators: Vec::new(),
      accounts: Vec::new()
    };

    let json = serde_json::to_string_pretty(&genesis)?;
    std::fs::write(genesis_file, json)?;

    Ok(())
  }
}

cmd! {
  GenesisAddAccountArgs(self, genesis_file: PathBuf) {
    add_account(&genesis_file, self)
  }
}

cmd! {
  GenesisAddMultisigArgs(self, genesis_file: PathBuf) {
    add_multisig(&genesis_file, self)
  }
}

fn add_account(genesis_file: &PathBuf, args: &GenesisAddAccountArgs) -> anyhow::Result<()> {
    update_genesis(genesis_file, |mut genesis| {
        let pk = read_public_key(&args.public_key)?;
        let addr = Address::new_secp256k1(&pk.serialize())?;
        let meta = ActorMeta::Account(Account {
            owner: ActorAddr(addr),
        });
        if genesis.accounts.iter().any(|a| a.meta == meta) {
            return Err(anyhow!("account already exists in the genesis file"));
        }
        let actor = Actor {
            meta,
            balance: args.balance.clone(),
        };
        genesis.accounts.push(actor);
        Ok(genesis)
    })
}

fn add_multisig(genesis_file: &PathBuf, args: &GenesisAddMultisigArgs) -> anyhow::Result<()> {
    update_genesis(genesis_file, |mut genesis| {
        let mut signers = Vec::new();
        for p in &args.public_key {
            let pk = read_public_key(p)?;
            let addr = ActorAddr(Address::new_secp256k1(&pk.serialize())?);
            if signers.contains(&addr) {
                return Err(anyhow!("duplicated signer: {}", p.to_string_lossy()));
            }
            signers.push(addr);
        }

        if signers.is_empty() {
            return Err(anyhow!("there needs to be at least one signer"));
        }
        if signers.len() < args.threshold as usize {
            return Err(anyhow!("threshold cannot be higher than number of signers"));
        }
        if args.threshold == 0 {
            return Err(anyhow!("threshold must be positive"));
        }

        let ms = Multisig {
            signers,
            threshold: args.threshold,
            vesting_duration: args.vesting_duration,
            vesting_start: args.vesting_start,
        };

        let actor = Actor {
            meta: ActorMeta::Multisig(ms),
            balance: args.balance.clone(),
        };

        genesis.accounts.push(actor);

        Ok(genesis)
    })
}

fn read_public_key(public_key: &PathBuf) -> anyhow::Result<PublicKey> {
    let b64 = std::fs::read_to_string(public_key).context("failed to read public key")?;
    let pk = b64_to_public(&b64).context("public key from base64")?;
    Ok(pk)
}

fn update_genesis<F>(genesis_file: &PathBuf, f: F) -> anyhow::Result<()>
where
    F: FnOnce(Genesis) -> anyhow::Result<Genesis>,
{
    let json = std::fs::read_to_string(genesis_file).context("failed to read genesis")?;
    let genesis = serde_json::from_str::<Genesis>(&json).context("failed to parse genesis")?;
    let genesis = f(genesis)?;
    let json = serde_json::to_string_pretty(&genesis)?;
    std::fs::write(genesis_file, json)?;
    Ok(())
}
