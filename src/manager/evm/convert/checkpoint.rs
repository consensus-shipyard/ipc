// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT

use crate::checkpoint::{NativeBottomUpCheckpoint, NativeChildCheck};
use crate::manager::evm::convert::{eth_to_fil_amount, fil_to_eth_amount};
use crate::manager::evm::manager::{
    gateway_router_facet, subnet_actor_getter_facet, subnet_actor_manager_facet,
};
use anyhow::anyhow;
use fvm_shared::clock::ChainEpoch;
use ipc_gateway::checkpoint::BatchCrossMsgs;
use ipc_gateway::{CrossMsg, TopDownCheckpoint};
use ipc_sdk::subnet_id::SubnetID;

impl TryFrom<TopDownCheckpoint> for gateway_router_facet::TopDownCheckpoint {
    type Error = anyhow::Error;

    fn try_from(value: TopDownCheckpoint) -> Result<Self, Self::Error> {
        Ok(Self {
            epoch: value.epoch as u64,
            top_down_msgs: value
                .top_down_msgs
                .into_iter()
                .map(gateway_router_facet::CrossMsg::try_from)
                .collect::<anyhow::Result<Vec<_>>>()?,
        })
    }
}

/// Derive the conversion for bottom up checkpoint
macro_rules! bottom_up_checkpoint {
    ($module:ident) => {
        impl TryFrom<NativeChildCheck> for $module::ChildCheck {
            type Error = anyhow::Error;

            fn try_from(value: NativeChildCheck) -> Result<Self, Self::Error> {
                let vec_to_array = |v: Vec<u8>| {
                    let bytes = if v.len() > 32 {
                        log::warn!("child check more than 32 bytes, taking only first 32 bytes");
                        &v[0..32]
                    } else {
                        &v
                    };

                    let mut array = [0u8; 32];
                    array.copy_from_slice(bytes);

                    array
                };
                let checks: Vec<[u8; 32]> = value
                    .checks
                    .into_iter()
                    .map(vec_to_array)
                    .collect::<Vec<_>>();
                Ok(Self {
                    source: $module::SubnetID::try_from(&value.source)?,
                    checks,
                })
            }
        }

        impl TryFrom<$module::ChildCheck> for NativeChildCheck {
            type Error = anyhow::Error;

            fn try_from(value: $module::ChildCheck) -> Result<Self, Self::Error> {
                let checks = value.checks.into_iter().map(|v| v.to_vec()).collect();
                Ok(Self {
                    source: SubnetID::try_from(value.source)?,
                    checks,
                })
            }
        }

        impl TryFrom<$module::BottomUpCheckpoint> for NativeBottomUpCheckpoint {
            type Error = anyhow::Error;

            fn try_from(value: $module::BottomUpCheckpoint) -> Result<Self, Self::Error> {
                let children = value
                    .children
                    .into_iter()
                    .map(NativeChildCheck::try_from)
                    .collect::<anyhow::Result<_>>()?;

                let cross_msgs = value
                    .cross_msgs
                    .into_iter()
                    .map(|i| {
                        CrossMsg::try_from(i)
                            .map_err(|e| anyhow!("cannot convert cross msg due to: {e:}"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let b = NativeBottomUpCheckpoint {
                    source: SubnetID::try_from(value.source)?,
                    proof: Some(value.proof.to_vec()),
                    epoch: value.epoch as ChainEpoch,
                    prev_check: Some(value.prev_hash.to_vec()),
                    children,
                    cross_msgs: BatchCrossMsgs {
                        cross_msgs: Some(cross_msgs),
                        fee: eth_to_fil_amount(&value.fee)?,
                    },
                    sig: vec![],
                };
                Ok(b)
            }
        }

        impl TryFrom<NativeBottomUpCheckpoint> for $module::BottomUpCheckpoint {
            type Error = anyhow::Error;

            fn try_from(value: NativeBottomUpCheckpoint) -> Result<Self, Self::Error> {
                let cross_msgs = value
                    .cross_msgs
                    .cross_msgs
                    .unwrap_or_default()
                    .into_iter()
                    .map(|i| {
                        $module::CrossMsg::try_from(i)
                            .map_err(|e| anyhow!("cannot convert cross msg due to: {e:}"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let children = value
                    .children
                    .into_iter()
                    .map(|i| {
                        $module::ChildCheck::try_from(i)
                            .map_err(|e| anyhow!("cannot convert child check due to: {e:}"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                let mut prev_hash = [0u8; 32];
                if let Some(v) = &value.prev_check {
                    prev_hash.copy_from_slice(v);
                }

                let proof = if let Some(v) = value.proof {
                    ethers::core::types::Bytes::from(v)
                } else {
                    ethers::core::types::Bytes::default()
                };

                let b = $module::BottomUpCheckpoint {
                    source: $module::SubnetID::try_from(&value.source)?,
                    epoch: value.epoch as u64,
                    fee: fil_to_eth_amount(&value.cross_msgs.fee)?,
                    cross_msgs,
                    children,
                    prev_hash,
                    proof,
                };
                Ok(b)
            }
        }
    };
}

bottom_up_checkpoint!(subnet_actor_manager_facet);
bottom_up_checkpoint!(subnet_actor_getter_facet);
