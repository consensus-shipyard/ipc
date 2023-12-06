// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

pub use self::network::*;
pub use self::shared::*;
pub use self::singletons::*;
use num_derive::FromPrimitive;

pub mod network;
pub mod shared;
pub mod singletons;
pub mod types;

pub const CALLER_TYPES_SIGNABLE: &[Type] = &[Type::Account, Type::Multisig];

/// Identifies the builtin actor types for usage with the
/// actor::resolve_builtin_actor_type syscall.
/// Note that there is a mirror of this enum in the FVM SDK src/actors/builtins.rs.
/// These must be kept in sync for the syscall to work correctly, without either side
/// importing the other.
#[derive(PartialEq, Eq, Clone, Copy, PartialOrd, Ord, FromPrimitive, Debug)]
#[repr(i32)]
pub enum Type {
    System = 1,
    Init = 2,
    Cron = 3,
    Account = 4,
    Power = 5,
    Miner = 6,
    Market = 7,
    PaymentChannel = 8,
    Multisig = 9,
    Reward = 10,
    VerifiedRegistry = 11,
    DataCap = 12,
}

impl Type {
    pub fn from_i32(u: i32) -> Self {
        match u {
            1 => Type::System,
            2 => Type::Init,
            3 => Type::Cron,
            4 => Type::Account,
            5 => Type::Power,
            6 => Type::Miner,
            7 => Type::Market,
            8 => Type::PaymentChannel,
            9 => Type::Multisig,
            10 => Type::Reward,
            11 => Type::VerifiedRegistry,
            _ => Type::DataCap,
        }
    }

    pub fn name(&self) -> &'static str {
        match *self {
            Type::System => "system",
            Type::Init => "init",
            Type::Cron => "cron",
            Type::Account => "account",
            Type::Power => "storagepower",
            Type::Miner => "storageminer",
            Type::Market => "storagemarket",
            Type::PaymentChannel => "paymentchannel",
            Type::Multisig => "multisig",
            Type::Reward => "reward",
            Type::VerifiedRegistry => "verifiedregistry",
            Type::DataCap => "datacap",
        }
    }
}
