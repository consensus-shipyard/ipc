// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::checkpoint::{NativeBottomUpCheckpoint, NativeChildCheck};
use cid::Cid;
use ipc_gateway::checkpoint::{CheckData, ChildCheck};
use ipc_gateway::BottomUpCheckpoint;
use primitives::TCid;

impl TryFrom<&NativeBottomUpCheckpoint> for BottomUpCheckpoint {
    type Error = anyhow::Error;

    fn try_from(value: &NativeBottomUpCheckpoint) -> Result<Self, Self::Error> {
        Ok(BottomUpCheckpoint {
            data: CheckData {
                source: value.source.clone(),
                proof: value.proof.clone().unwrap_or_default(),
                epoch: value.epoch,
                prev_check: Default::default(),
                children: value
                    .children
                    .iter()
                    .map(ChildCheck::try_from)
                    .collect::<anyhow::Result<_>>()?,
                cross_msgs: value.cross_msgs.clone(),
            },
            sig: value.sig.clone(),
        })
    }
}

impl From<ChildCheck> for NativeChildCheck {
    fn from(value: ChildCheck) -> Self {
        NativeChildCheck {
            source: value.source,
            checks: value
                .checks
                .into_iter()
                .map(|v| v.cid().to_bytes())
                .collect(),
        }
    }
}

impl TryFrom<&NativeChildCheck> for ChildCheck {
    type Error = anyhow::Error;

    fn try_from(value: &NativeChildCheck) -> Result<Self, Self::Error> {
        let mut checks = vec![];
        for check in &value.checks {
            checks.push(TCid::from(Cid::try_from(check.as_slice())?));
        }

        Ok(ChildCheck {
            source: value.source.clone(),
            checks,
        })
    }
}

impl TryFrom<BottomUpCheckpoint> for NativeBottomUpCheckpoint {
    type Error = anyhow::Error;

    fn try_from(value: BottomUpCheckpoint) -> Result<Self, Self::Error> {
        let checkpoint = NativeBottomUpCheckpoint {
            epoch: value.epoch(),
            prev_check: Some(value.prev_check().cid().to_bytes()),
            children: value
                .data
                .children
                .into_iter()
                .map(NativeChildCheck::from)
                .collect(),
            cross_msgs: value.data.cross_msgs,
            sig: value.sig,
            source: value.data.source,
            proof: Some(value.data.proof),
        };
        Ok(checkpoint)
    }
}
