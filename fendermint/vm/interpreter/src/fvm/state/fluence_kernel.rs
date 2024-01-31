use fvm::call_manager::CallManager;
use fvm::gas::Gas;
use fvm::kernel::prelude::*;
use fvm::kernel::Result;
use fvm::kernel::{
    ActorOps, CryptoOps, DebugOps, EventOps, IpldBlockOps, MessageOps, NetworkOps, RandomnessOps,
    SelfOps, SendOps, SyscallHandler, UpgradeOps,
};
use fvm::syscalls::Linker;
use fvm_shared::clock::ChainEpoch;
use fvm_shared::randomness::RANDOMNESS_LENGTH;
use fvm_shared::sys::out::network::NetworkContext;
use fvm_shared::sys::out::vm::MessageContext;
use fvm_shared::{address::Address, econ::TokenAmount, ActorID, MethodNum};

use fvm::DefaultKernel;

use ambassador::Delegate;
use cid::Cid;

#[derive(Delegate)]
#[delegate(IpldBlockOps, where = "C: CallManager")]
#[delegate(ActorOps, where = "C: CallManager")]
#[delegate(CryptoOps, where = "C: CallManager")]
#[delegate(DebugOps, where = "C: CallManager")]
#[delegate(EventOps, where = "C: CallManager")]
#[delegate(MessageOps, where = "C: CallManager")]
#[delegate(NetworkOps, where = "C: CallManager")]
#[delegate(RandomnessOps, where = "C: CallManager")]
#[delegate(SelfOps, where = "C: CallManager")]
#[delegate(SendOps<K>, generics = "K", where = "K: Kernel")]
#[delegate(UpgradeOps<K>, generics = "K", where = "K: Kernel")]
pub struct FluenceKernelImpl<C>(pub DefaultKernel<C>);

impl<K> SyscallHandler<K> for FluenceKernelImpl<K::CallManager>
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
        + SelfOps,
{
    fn link_syscalls(linker: &mut Linker<K>) -> anyhow::Result<()> {
        use fluence_fendermint_syscall::{SYSCALL_FUNCTION_NAME, SYSCALL_MODULE_NAME};

        DefaultKernel::<K::CallManager>::link_syscalls(linker)?;
        linker.link_syscall(
            SYSCALL_MODULE_NAME,
            SYSCALL_FUNCTION_NAME,
            fluence_fendermint_syscall::run_randomx,
        )?;

        Ok(())
    }
}

impl<C> Kernel for FluenceKernelImpl<C>
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
        FluenceKernelImpl(DefaultKernel::new(
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
