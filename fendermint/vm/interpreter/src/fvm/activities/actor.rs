// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fvm::executor::{ApplyKind, ApplyRet, Executor};
use fvm_shared::clock::ChainEpoch;
use fendermint_actor_activity_tracker::{ValidatorSummary};
use fendermint_vm_actor_interface::system;
use crate::fvm::activities::{ActivitySummary, BlockMined, ValidatorActivityTracker};
use crate::fvm::FvmMessage;
use fendermint_vm_actor_interface::activity::ACTIVITY_TRACKER_ACTOR_ADDR;
use fendermint_vm_actor_interface::eam::EthAddress;

pub struct ActorActivityTracker<'a, E> {
    pub(crate) executor: &'a mut E,
    pub(crate) epoch: ChainEpoch,
}

impl <'a, E: Executor> ValidatorActivityTracker for ActorActivityTracker<'a, E> {
    type ValidatorSummaryDetail = ValidatorSummary;

    fn track_block_mined(&mut self, block: BlockMined) -> anyhow::Result<()> {
        let params = fendermint_actor_activity_tracker::BlockedMinedParams {
            validator: fvm_shared::address::Address::from(EthAddress::from(block.validator)),
        };

        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: ACTIVITY_TRACKER_ACTOR_ADDR,
            sequence: self.epoch as u64,
            // exclude this from gas restriction
            gas_limit: i64::MAX as u64,
            method_num: fendermint_actor_activity_tracker::Method::BlockMined as u64,
            params: fvm_ipld_encoding::RawBytes::serialize(params)?,
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };

        self.apply_implicit_message(msg)?;
        Ok(())
    }

    fn get_activities_summary(&self) -> anyhow::Result<ActivitySummary<Self::ValidatorSummaryDetail>> {
        // let msg = FvmMessage {
        //     from: system::SYSTEM_ACTOR_ADDR,
        //     to: ACTIVITY_TRACKER_ACTOR_ADDR,
        //     sequence: self.epoch as u64,
        //     // exclude this from gas restriction
        //     gas_limit: i64::MAX as u64,
        //     method_num: fendermint_actor_activity_tracker::Method::GetActivities as u64,
        //     params: fvm_ipld_encoding::RawBytes::default(),
        //     value: Default::default(),
        //     version: Default::default(),
        //     gas_fee_cap: Default::default(),
        //     gas_premium: Default::default(),
        // };
        //
        // let apply_ret = self.apply_implicit_message(msg)?;
        // let r =
        //     fvm_ipld_encoding::from_slice::<GetActivitiesResult>(&apply_ret.msg_receipt.return_data)
        //         .context("failed to parse validator activities")?;
        // Ok(ActivitySummary {
        //     block_range: (r.start_height, self.epoch),
        //     details: r.activities,
        // })
        todo!()
    }

    fn purge_activities(&mut self) -> anyhow::Result<()> {
        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: ACTIVITY_TRACKER_ACTOR_ADDR,
            sequence: self.epoch as u64,
            // exclude this from gas restriction
            gas_limit: i64::MAX as u64,
            method_num: fendermint_actor_activity_tracker::Method::PurgeActivities as u64,
            params: fvm_ipld_encoding::RawBytes::default(),
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };

        self.apply_implicit_message(msg)?;
        Ok(())
    }
}

impl <'a, E: Executor> ActorActivityTracker<'a, E> {
    fn apply_implicit_message<>(
        &mut self,
        msg: FvmMessage,
    ) -> anyhow::Result<ApplyRet> {
        let raw_length = fvm_ipld_encoding::to_vec(&msg).map(|bz| bz.len())?;
        let apply_ret = self.executor.execute_message(msg, ApplyKind::Implicit, raw_length)?;

        if let Some(err) = apply_ret.failure_info {
            anyhow::bail!("failed to apply activity tracker messages: {}", err)
        } else {
            Ok(apply_ret)
        }
    }
}
