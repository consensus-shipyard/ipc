// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use crate::fvm::activity::{ActivityDetails, ValidatorActivityTracker};
use crate::fvm::state::FvmExecState;
use crate::fvm::FvmMessage;
use anyhow::Context;
use fendermint_actor_activity_tracker::{GetActivitiesResult, ValidatorData};
use fendermint_vm_actor_interface::activity::ACTIVITY_TRACKER_ACTOR_ADDR;
use fendermint_vm_actor_interface::eam::EthAddress;
use fendermint_vm_actor_interface::system;
use fvm::executor::ApplyRet;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::address::Address;
use fvm_shared::clock::ChainEpoch;
use fendermint_crypto::PublicKey;

pub struct ActorActivityTracker<'a, DB: Blockstore + Clone + 'static> {
    pub(crate) executor: &'a mut FvmExecState<DB>,
    pub(crate) epoch: ChainEpoch,
}

impl<'a, DB: Blockstore + Clone + 'static> ValidatorActivityTracker
    for ActorActivityTracker<'a, DB>
{
    type ValidatorSummaryDetail = ValidatorData;

    fn record_block_committed(&mut self, validator: PublicKey) -> anyhow::Result<()> {
        let address: Address = EthAddress::from(validator).into();

        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: ACTIVITY_TRACKER_ACTOR_ADDR,
            sequence: 0, // irrelevant
            gas_limit: i64::MAX as u64, // exclude this from gas restriction
            method_num: fendermint_actor_activity_tracker::Method::RecordBlockCommitted as u64,
            params: fvm_ipld_encoding::RawBytes::serialize(address)?,
            ..Default::default()
        };

        self.apply_implicit_message(msg)?;
        Ok(())
    }

    fn commit_activity(
        &mut self,
    ) -> anyhow::Result<ActivityDetails<Self::ValidatorSummaryDetail>> {
        let msg = FvmMessage {
            from: system::SYSTEM_ACTOR_ADDR,
            to: ACTIVITY_TRACKER_ACTOR_ADDR,
            sequence: 0, // irrelevant
            gas_limit: i64::MAX as u64, // exclude this from gas restriction
            method_num: fendermint_actor_activity_tracker::Method::CommitActivity as u64,
            ..Default::default()
        };

        let (apply_ret, _) = self.executor.execute_implicit(msg)?;
        let r = fvm_ipld_encoding::from_slice::<GetActivitiesResult>(
            &apply_ret.msg_receipt.return_data,
        )
        .context("failed to parse validator activities")?;
        Ok(ActivityDetails::new(r.activities, r.cycle_start))
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

impl<'a, DB: Blockstore + Clone + 'static> ActorActivityTracker<'a, DB> {
    fn apply_implicit_message(&mut self, msg: FvmMessage) -> anyhow::Result<ApplyRet> {
        let (apply_ret, _) = self.executor.execute_implicit(msg)?;
        if let Some(err) = apply_ret.failure_info {
            anyhow::bail!("failed to apply activity tracker messages: {}", err)
        } else {
            Ok(apply_ret)
        }
    }
}
