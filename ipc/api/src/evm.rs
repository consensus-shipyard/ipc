// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Type conversion for IPC Agent struct with solidity contract struct

use crate::address::IPCAddress;
use crate::checkpoint::BottomUpMsgBatch;
use crate::checkpoint::{
    consensus, BottomUpBatchCommitment, BottomUpCheckpoint, CompressedActivityRollup,
};
use crate::cross::{IpcEnvelope, IpcMsgKind};
use crate::staking::PowerChange;
use crate::staking::PowerChangeRequest;
use crate::subnet::{Asset, AssetKind};
use crate::subnet_id::SubnetID;
use crate::{eth_to_fil_amount, ethers_address_to_fil_address};
use anyhow::anyhow;
use ethers::types::U256;
use fvm_shared::address::{Address, Payload};
use fvm_shared::clock::ChainEpoch;
use fvm_shared::econ::TokenAmount;
use ipc_actors_abis::{
    checkpointing_facet, gateway_getter_facet, gateway_manager_facet, gateway_messenger_facet,
    lib_gateway, register_subnet_facet, subnet_actor_activity_facet,
    subnet_actor_checkpointing_facet, subnet_actor_diamond, subnet_actor_getter_facet,
    top_down_finality_facet, xnet_messaging_facet,
};

/// The type conversion for IPC structs to evm solidity contracts. We need this convenient macro because
/// the abigen is creating the same struct but under different modules. This save a lot of
/// code.
macro_rules! base_type_conversion {
    ($module:ident) => {
        impl TryFrom<&SubnetID> for $module::SubnetID {
            type Error = anyhow::Error;

            fn try_from(subnet: &SubnetID) -> Result<Self, Self::Error> {
                Ok($module::SubnetID {
                    root: subnet.root_id(),
                    route: subnet_id_to_evm_addresses(subnet)?,
                })
            }
        }

        impl TryFrom<$module::SubnetID> for SubnetID {
            type Error = anyhow::Error;

            fn try_from(value: $module::SubnetID) -> Result<Self, Self::Error> {
                let children = value
                    .route
                    .iter()
                    .map(ethers_address_to_fil_address)
                    .collect::<anyhow::Result<Vec<_>>>()?;
                Ok(SubnetID::new(value.root, children))
            }
        }
    };
}

/// Implement the cross network message types. To use this macro, make sure the $module has already
/// implemented the base types.
macro_rules! cross_msg_types {
    ($module:ident) => {
        impl TryFrom<IPCAddress> for $module::Ipcaddress {
            type Error = anyhow::Error;

            fn try_from(value: IPCAddress) -> Result<Self, Self::Error> {
                Ok($module::Ipcaddress {
                    subnet_id: $module::SubnetID::try_from(&value.subnet()?)?,
                    raw_address: $module::FvmAddress::try_from(value.raw_addr()?)?,
                })
            }
        }

        impl TryFrom<$module::Ipcaddress> for IPCAddress {
            type Error = anyhow::Error;

            fn try_from(value: $module::Ipcaddress) -> Result<Self, Self::Error> {
                let addr = Address::try_from(value.raw_address)?;
                let i = IPCAddress::new(&SubnetID::try_from(value.subnet_id)?, &addr)?;
                Ok(i)
            }
        }

        impl TryFrom<IpcEnvelope> for $module::IpcEnvelope {
            type Error = anyhow::Error;

            fn try_from(value: IpcEnvelope) -> Result<Self, Self::Error> {
                let val = fil_to_eth_amount(&value.value)?;

                let c = $module::IpcEnvelope {
                    kind: value.kind as u8,
                    from: $module::Ipcaddress::try_from(value.from).map_err(|e| {
                        anyhow!("cannot convert `from` ipc address msg due to: {e:}")
                    })?,
                    to: $module::Ipcaddress::try_from(value.to)
                        .map_err(|e| anyhow!("cannot convert `to`` ipc address due to: {e:}"))?,
                    value: val,
                    local_nonce: value.local_nonce,
                    original_nonce: value.original_nonce,
                    message: ethers::core::types::Bytes::from(value.message),
                };
                Ok(c)
            }
        }

        impl TryFrom<$module::IpcEnvelope> for IpcEnvelope {
            type Error = anyhow::Error;

            fn try_from(value: $module::IpcEnvelope) -> Result<Self, Self::Error> {
                let s = IpcEnvelope {
                    from: IPCAddress::try_from(value.from)?,
                    to: IPCAddress::try_from(value.to)?,
                    value: eth_to_fil_amount(&value.value)?,
                    kind: IpcMsgKind::try_from(value.kind)?,
                    message: value.message.to_vec(),
                    local_nonce: value.local_nonce,
                    original_nonce: value.original_nonce,
                };
                Ok(s)
            }
        }
    };
}

/// The type conversion between different bottom up checkpoint definition in ethers and sdk
macro_rules! bottom_up_checkpoint_conversion {
    ($module:ident) => {
        impl TryFrom<consensus::AggregatedStats> for $module::AggregatedStats {
            type Error = anyhow::Error;

            fn try_from(c: consensus::AggregatedStats) -> Result<Self, Self::Error> {
                Ok($module::AggregatedStats {
                    total_active_validators: c.total_active_validators,
                    total_num_blocks_committed: c.total_num_blocks_committed,
                })
            }
        }

        impl TryFrom<CompressedActivityRollup> for $module::CompressedActivityRollup {
            type Error = anyhow::Error;

            fn try_from(c: CompressedActivityRollup) -> Result<Self, Self::Error> {
                Ok($module::CompressedActivityRollup {
                    consensus: c.consensus.try_into()?,
                })
            }
        }

        impl From<$module::CompressedActivityRollup> for CompressedActivityRollup {
            fn from(value: $module::CompressedActivityRollup) -> Self {
                CompressedActivityRollup {
                    consensus: consensus::CompressedSummary {
                        stats: consensus::AggregatedStats {
                            total_active_validators: value.consensus.stats.total_active_validators,
                            total_num_blocks_committed: value
                                .consensus
                                .stats
                                .total_num_blocks_committed,
                        },
                        data_root_commitment: value.consensus.data_root_commitment.to_vec(),
                    },
                }
            }
        }

        impl TryFrom<consensus::CompressedSummary> for $module::CompressedSummary {
            type Error = anyhow::Error;

            fn try_from(c: consensus::CompressedSummary) -> Result<Self, Self::Error> {
                Ok($module::CompressedSummary {
                    stats: c
                        .stats
                        .try_into()
                        .map_err(|_| anyhow!("cannot convert aggregated stats"))?,
                    data_root_commitment: c
                        .data_root_commitment
                        .try_into()
                        .map_err(|_| anyhow!("cannot convert bytes32"))?,
                })
            }
        }

        impl TryFrom<BottomUpCheckpoint> for $module::BottomUpCheckpoint {
            type Error = anyhow::Error;

            fn try_from(checkpoint: BottomUpCheckpoint) -> Result<Self, Self::Error> {
                Ok($module::BottomUpCheckpoint {
                    subnet_id: $module::SubnetID::try_from(&checkpoint.subnet_id)?,
                    block_height: ethers::core::types::U256::from(checkpoint.block_height),
                    block_hash: vec_to_bytes32(checkpoint.block_hash)?,
                    next_configuration_number: checkpoint.next_configuration_number,
                    msgs: checkpoint.msgs.try_into()?,
                    activity: checkpoint.activity_rollup.try_into()?,
                })
            }
        }

        impl TryFrom<$module::BottomUpCheckpoint> for BottomUpCheckpoint {
            type Error = anyhow::Error;

            fn try_from(value: $module::BottomUpCheckpoint) -> Result<Self, Self::Error> {
                Ok(BottomUpCheckpoint {
                    subnet_id: SubnetID::try_from(value.subnet_id)?,
                    block_height: value.block_height.as_u128() as ChainEpoch,
                    block_hash: value.block_hash.to_vec(),
                    next_configuration_number: value.next_configuration_number,
                    msgs: value.msgs.into(),
                    activity_rollup: value.activity.into(),
                })
            }
        }

        impl TryFrom<BottomUpBatchCommitment> for $module::Commitment {
            type Error = anyhow::Error;

            fn try_from(value: BottomUpBatchCommitment) -> Result<Self, Self::Error> {
                Ok($module::Commitment {
                    total_num_msgs: value.total_num_msgs,
                    msgs_root: value
                        .msgs_root
                        .try_into()
                        .map_err(|_| anyhow!("cannot convert bytes32"))?,
                })
            }
        }

        impl From<$module::Commitment> for BottomUpBatchCommitment {
            fn from(value: $module::Commitment) -> Self {
                BottomUpBatchCommitment {
                    total_num_msgs: value.total_num_msgs,
                    msgs_root: value.msgs_root.to_vec(),
                }
            }
        }
    };
}

/// The type conversion between different bottom up message batch definition in ethers and sdk
macro_rules! bottom_up_msg_batch_conversion {
    ($module:ident) => {
        impl TryFrom<BottomUpMsgBatch> for $module::BottomUpMsgBatch {
            type Error = anyhow::Error;

            fn try_from(batch: BottomUpMsgBatch) -> Result<Self, Self::Error> {
                Ok($module::BottomUpMsgBatch {
                    subnet_id: $module::SubnetID::try_from(&batch.subnet_id)?,
                    block_height: ethers::core::types::U256::from(batch.block_height),
                    msgs: batch
                        .msgs
                        .into_iter()
                        .map($module::IpcEnvelope::try_from)
                        .collect::<Result<Vec<_>, _>>()?,
                })
            }
        }
    };
}

/// The type conversion between different asset token types
macro_rules! asset_conversion {
    ($module:ident) => {
        impl TryFrom<Asset> for $module::Asset {
            type Error = anyhow::Error;

            fn try_from(value: Asset) -> Result<Self, Self::Error> {
                let token_address = if let Some(token_address) = value.token_address {
                    payload_to_evm_address(token_address.payload())?
                } else {
                    ethers::types::Address::zero()
                };

                Ok(Self {
                    kind: value.kind as u8,
                    token_address,
                })
            }
        }

        impl TryFrom<$module::Asset> for Asset {
            type Error = anyhow::Error;

            fn try_from(value: $module::Asset) -> Result<Self, Self::Error> {
                let token_address = if value.token_address == ethers::types::Address::zero() {
                    None
                } else {
                    Some(ethers_address_to_fil_address(&value.token_address)?)
                };

                Ok(Self {
                    kind: AssetKind::try_from(value.kind)?,
                    token_address,
                })
            }
        }
    };
}

base_type_conversion!(xnet_messaging_facet);
base_type_conversion!(subnet_actor_getter_facet);
base_type_conversion!(gateway_manager_facet);
base_type_conversion!(subnet_actor_checkpointing_facet);
base_type_conversion!(gateway_getter_facet);
base_type_conversion!(gateway_messenger_facet);
base_type_conversion!(lib_gateway);
base_type_conversion!(subnet_actor_activity_facet);
base_type_conversion!(checkpointing_facet);

cross_msg_types!(gateway_getter_facet);
cross_msg_types!(xnet_messaging_facet);
cross_msg_types!(gateway_messenger_facet);
cross_msg_types!(lib_gateway);
cross_msg_types!(subnet_actor_checkpointing_facet);
cross_msg_types!(checkpointing_facet);

bottom_up_checkpoint_conversion!(checkpointing_facet);
bottom_up_checkpoint_conversion!(gateway_getter_facet);
bottom_up_checkpoint_conversion!(subnet_actor_checkpointing_facet);
bottom_up_msg_batch_conversion!(gateway_getter_facet);

asset_conversion!(subnet_actor_diamond);
asset_conversion!(register_subnet_facet);
asset_conversion!(subnet_actor_getter_facet);

impl TryFrom<u8> for AssetKind {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(AssetKind::Native),
            1 => Ok(AssetKind::ERC20),
            _ => Err(anyhow!("invalid kind {value}")),
        }
    }
}

/// Convert the ipc SubnetID type to a vec of evm addresses. It extracts all the children addresses
/// in the subnet id and turns them as a vec of evm addresses.
pub fn subnet_id_to_evm_addresses(
    subnet: &SubnetID,
) -> anyhow::Result<Vec<ethers::types::Address>> {
    let children = subnet.children();
    children
        .iter()
        .map(|addr| payload_to_evm_address(addr.payload()))
        .collect::<anyhow::Result<_>>()
}

/// Util function to convert Fil address payload to evm address. Only delegated address is supported.
pub fn payload_to_evm_address(payload: &Payload) -> anyhow::Result<ethers::types::Address> {
    match payload {
        Payload::Delegated(delegated) => {
            let slice = delegated.subaddress();
            Ok(ethers::types::Address::from_slice(&slice[0..20]))
        }
        _ => Err(anyhow!("address provided is not delegated")),
    }
}

/// Converts a Fil TokenAmount into an ethers::U256 amount.
pub fn fil_to_eth_amount(amount: &TokenAmount) -> anyhow::Result<U256> {
    let str = amount.atto().to_string();
    Ok(U256::from_dec_str(&str)?)
}

impl TryFrom<PowerChange> for top_down_finality_facet::PowerChange {
    type Error = anyhow::Error;

    fn try_from(value: PowerChange) -> Result<Self, Self::Error> {
        Ok(top_down_finality_facet::PowerChange {
            op: value.op as u8,
            payload: ethers::core::types::Bytes::from(value.payload),
            validator: payload_to_evm_address(value.validator.payload())?,
        })
    }
}

impl TryFrom<PowerChangeRequest> for top_down_finality_facet::PowerChangeRequest {
    type Error = anyhow::Error;

    fn try_from(value: PowerChangeRequest) -> Result<Self, Self::Error> {
        Ok(top_down_finality_facet::PowerChangeRequest {
            change: top_down_finality_facet::PowerChange::try_from(value.change)?,
            configuration_number: value.configuration_number,
        })
    }
}

pub fn vec_to_bytes32(v: Vec<u8>) -> anyhow::Result<[u8; 32]> {
    if v.len() != 32 {
        return Err(anyhow!("invalid length"));
    }

    let mut r = [0u8; 32];
    r.copy_from_slice(&v);

    Ok(r)
}

#[cfg(test)]
mod tests {
    use crate::evm::subnet_id_to_evm_addresses;
    use crate::subnet_id::SubnetID;
    use fvm_shared::address::Address;
    use ipc_types::EthAddress;
    use std::str::FromStr;

    #[test]
    fn test_subnet_id_to_evm_addresses() {
        let eth_addr = EthAddress::from_str("0x0000000000000000000000000000000000000000").unwrap();
        let addr = Address::from(eth_addr);
        let addr2 = Address::from_str("f410ffzyuupbyl2uiucmzr3lu3mtf3luyknthaz4xsrq").unwrap();

        let id = SubnetID::new(0, vec![addr, addr2]);

        let addrs = subnet_id_to_evm_addresses(&id).unwrap();

        let a =
            ethers::types::Address::from_str("0x0000000000000000000000000000000000000000").unwrap();
        let b =
            ethers::types::Address::from_str("0x2e714a3c385ea88a09998ed74db265dae9853667").unwrap();

        assert_eq!(addrs, vec![a, b]);
    }
}
