// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_vm_genesis::Genesis;

use crate::cmd;
use crate::options::GenesisNewArgs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

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

    let mut file = File::create(genesis_file)?;

    write!(file, "{}", json)?;

    Ok(())
  }
}
