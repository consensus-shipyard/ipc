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
        let mut us = if upgrade_file.exists() {
            // load existing upgrade schedule
            UpgradeSchedule::from_file(&upgrade_file)?
        } else {
            // create an empty upgrade schedule
            UpgradeSchedule::new().to_file(&upgrade_file)?;
            UpgradeSchedule::new()
        };

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
