// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Performs dry run for the ipc provider instead of directly submitter the txn on chain

use crate::manager::evm::dry_run::EvmDryRun;
use crate::manager::fvm::FvmDryRun;
use crate::preflight::Preflight;
use anyhow::anyhow;
use fvm_shared::address::Address;
use ipc_api::subnet::ConstructParams;

/// The network type the dry run performs in
#[derive(Debug, Clone, strum::EnumString)]
pub enum Network {
    #[strum(serialize = "Evm", serialize = "evm")]
    Evm,
    #[strum(serialize = "Fvm", serialize = "fvm")]
    Fvm,
}

pub struct IPCDryRunProvider {
    preflight: Preflight,
    inner: Inner,
}

enum Inner {
    Evm(EvmDryRun),
    Fvm(FvmDryRun),
}

impl IPCDryRunProvider {
    pub fn evm(preflight: Preflight, evm: EvmDryRun) -> Self {
        Self {
            preflight,
            inner: Inner::Evm(evm),
        }
    }

    pub fn fvm(preflight: Preflight, fvm: FvmDryRun) -> Self {
        Self {
            preflight,
            inner: Inner::Fvm(fvm),
        }
    }

    pub fn create_subnet(
        &self,
        from: Option<Address>,
        params: ConstructParams,
    ) -> anyhow::Result<()> {
        let params = self.preflight.create_subnet(params)?;
        let from = self.fill_signer(from)?;

        let json = match &self.inner {
            Inner::Evm(d) => serde_json::to_string(&d.create_subnet(&from, params)?)?,
            Inner::Fvm(d) => serde_json::to_string(&d.create_subnet(&from, params)?)?,
        };

        println!("txn to run: \n{}", json);

        Ok(())
    }

    fn fill_signer(&self, from: Option<Address>) -> anyhow::Result<Address> {
        if let Some(addr) = from {
            return Ok(addr);
        }

        self.preflight
            .get_default_signer()?
            .ok_or_else(|| anyhow!("no from address specified"))
    }
}
