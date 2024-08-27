// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use ambassador::Delegate;
use cid::Cid;
use fvm::call_manager::CallManager;
use fvm::gas::Gas;
use fvm::kernel::prelude::*;
use fvm::kernel::{
    ActorOps, CryptoOps, DebugOps, EventOps, IpldBlockOps, MessageOps, NetworkOps, RandomnessOps,
    SelfOps, SendOps, SyscallHandler, UpgradeOps,
};
use fvm::kernel::{ClassifyResult, Result};
use fvm::syscalls::Linker;
use fvm::DefaultKernel;
use fvm_ipld_blockstore::Blockstore;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::randomness::RANDOMNESS_LENGTH;
use fvm_shared::sys::out::network::NetworkContext;
use fvm_shared::sys::out::vm::MessageContext;
use fvm_shared::{address::Address, econ::TokenAmount, ActorID, MethodNum};

#[derive(Delegate)]
#[delegate(ActorOps, where = "C: CallManager")]
#[delegate(SendOps < K >, generics = "K", where = "K: Kernel")]
#[delegate(UpgradeOps < K >, generics = "K", where = "K: Kernel")]
#[delegate(IpldBlockOps, where = "C: CallManager")]
#[delegate(CryptoOps, where = "C: CallManager")]
#[delegate(DebugOps, where = "C: CallManager")]
#[delegate(EventOps, where = "C: CallManager")]
#[delegate(MessageOps, where = "C: CallManager")]
#[delegate(NetworkOps, where = "C: CallManager")]
#[delegate(RandomnessOps, where = "C: CallManager")]
#[delegate(SelfOps, where = "C: CallManager")]
pub struct ObjectStoreKernel<C>(pub DefaultKernel<C>);

pub trait ObjectStoreOps {
    fn block_add(&mut self, cid: Cid, data: &[u8]) -> Result<()>;
}

impl<C> ObjectStoreOps for ObjectStoreKernel<C>
where
    C: CallManager,
{
    /// Directly add a block, skipping gas and reachability checks.
    fn block_add(&mut self, cid: Cid, data: &[u8]) -> Result<()> {
        self.0
            .call_manager
            .blockstore()
            .put_keyed(&cid, data)
            .or_fatal()?;
        self.0.blocks.mark_reachable(&cid);
        Ok(())
    }
}

impl<K> SyscallHandler<K> for ObjectStoreKernel<K::CallManager>
where
    K: Kernel
        + ActorOps
        + SendOps
        + UpgradeOps
        + IpldBlockOps
        + CryptoOps
        + DebugOps
        + EventOps
        + MessageOps
        + NetworkOps
        + RandomnessOps
        + SelfOps
        + ObjectStoreOps,
{
    fn link_syscalls(linker: &mut Linker<K>) -> anyhow::Result<()> {
        DefaultKernel::<K::CallManager>::link_syscalls(linker)?;
        linker.link_syscall(
            crate::SYSCALL_MODULE_NAME,
            crate::CIDRM_SYSCALL_FUNCTION_NAME,
            crate::cid_rm,
        )?;
        linker.link_syscall(
            crate::SYSCALL_MODULE_NAME,
            crate::HASHRM_SYSCALL_FUNCTION_NAME,
            crate::hash_rm,
        )?;

        Ok(())
    }
}

impl<C> Kernel for ObjectStoreKernel<C>
where
    C: CallManager,
{
    type CallManager = C;
    type Limiter = <DefaultKernel<C> as Kernel>::Limiter;

    fn into_inner(self) -> (Self::CallManager, BlockRegistry)
    where
        Self: Sized,
    {
        self.0.into_inner()
    }

    fn new(
        mgr: C,
        blocks: BlockRegistry,
        caller: ActorID,
        actor_id: ActorID,
        method: MethodNum,
        value_received: TokenAmount,
        read_only: bool,
    ) -> Self {
        ObjectStoreKernel(DefaultKernel::new(
            mgr,
            blocks,
            caller,
            actor_id,
            method,
            value_received,
            read_only,
        ))
    }

    fn machine(&self) -> &<Self::CallManager as CallManager>::Machine {
        self.0.machine()
    }

    fn limiter_mut(&mut self) -> &mut Self::Limiter {
        self.0.limiter_mut()
    }

    fn gas_available(&self) -> Gas {
        self.0.gas_available()
    }

    fn charge_gas(&self, name: &str, compute: Gas) -> Result<GasTimer> {
        self.0.charge_gas(name, compute)
    }
}
