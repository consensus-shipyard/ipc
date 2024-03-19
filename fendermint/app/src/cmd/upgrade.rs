// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::path::PathBuf;

use fendermint_app_options::upgrade::AddUpgrade;
use fendermint_app_options::upgrade::UpgradeArgs;
use fendermint_app_options::upgrade::UpgradeCommands;
use fendermint_vm_interpreter::fvm::upgrades::UpgradeSchedule;
use fendermint_vm_message::ipc::UpgradeInfo;

use crate::cmd;

cmd! {
    UpgradeArgs(self) {
        let upgrade_file = self.upgrade_file.clone();
        match &self.command {
            UpgradeCommands::AddUpgrade(args) => args.exec(upgrade_file).await,
        }
    }
}

cmd! {
    AddUpgrade(self, upgrade_file: PathBuf) {
        let mut us = UpgradeSchedule::get_or_create(&upgrade_file)?;
        us.add(UpgradeInfo::new(
            self.height.try_into().unwrap(),
            self.new_app_version,
            self.cometbft_version.clone(),
            self.required,
        ))?;

        us.to_file(upgrade_file)?;

        Ok(())
  }
}
