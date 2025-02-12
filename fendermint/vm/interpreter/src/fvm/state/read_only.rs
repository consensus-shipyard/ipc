// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::anyhow;
use fvm::{DefaultKernel, Kernel};
use fvm::call_manager::{CallManager, DefaultCallManager, Entrypoint, InvocationResult};
use fvm::engine::{Engine, EnginePool};
use fvm::executor::{ApplyKind, ApplyRet};
use fvm::kernel::{Block, ClassifyResult, Context};
use fvm::machine::{DefaultMachine, Machine};
use fvm::state_tree::StateTree;
use fvm_ipld_blockstore::Blockstore;
use fvm_ipld_encoding::{CBOR, IPLD_RAW, RawBytes};
use fvm_shared::{ActorID, METHOD_SEND};
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use fvm_shared::error::ExitCode;
use fvm_shared::message::Message;
use fvm_shared::receipt::Receipt;
use num_traits::Zero;
use crate::fvm::externs::FendermintExterns;
use crate::fvm::state::exec::ExecResult;

type FvmKernel<DB> = DefaultKernel<DefaultCallManager<DefaultMachine<DB, FendermintExterns<DB>>>>;
type FvmMachine<DB> = DefaultMachine<DB, FendermintExterns<DB>>;

pub struct FvmReadOnlyExecutor<DB> where DB: Blockstore + Clone + 'static
{

    machine: Option<FvmMachine<DB>>,
    engine_pool: EnginePool,
}

impl <DB: Blockstore + Clone + 'static> FvmReadOnlyExecutor<DB> {
    pub fn new(machine: FvmMachine<DB>, engine_pool: EnginePool) -> Self {
        Self { machine: Some(machine), engine_pool }
    }

    pub fn into_inner(self) -> anyhow::Result<(FvmMachine<DB>, EnginePool)> {
        let machine = self.machine.ok_or_else(|| anyhow!("machine is poisoned"))?;
        Ok((machine, self.engine_pool))
    }

    /// Execute a read only message, the apply kind is by default implicit.
    pub fn exec_message(&mut self, msg: Message) -> anyhow::Result<ApplyRet> {
        let machine = self.machine.take().ok_or_else(|| anyhow!("machine is poisoned"))?;
        let engine = self.engine_pool.acquire();

        let (r, machine) = Self::exec_inner(machine, engine, msg);

        self.machine = Some(machine);
        r
    }

    fn process_message(machine: &FvmMachine<DB>, msg: &Message) -> anyhow::Result<(ActorID, Option<ActorID>, Option<Block>)> {
        msg.check().or_fatal()?;

        let sender_id = machine
            .state_tree()
            .lookup_id(&msg.from)
            .with_context(|| format!("failed to lookup actor {}", msg.from))?
            .ok_or_else(|| anyhow!("sender invalid"))?;

        let receiver_id = machine
            .state_tree()
            .lookup_id(&msg.to)
            .context("failure when looking up message receiver")?;

        let params = (!msg.params.is_empty()).then(|| {
            Block::new(
                if msg.method_num == METHOD_SEND {
                    IPLD_RAW
                } else {
                    // This is CBOR, not DAG_CBOR, because links sent from off-chain aren't
                    // reachable.
                    CBOR
                },
                msg.params.bytes(),
                // not DAG-CBOR, so we don't have to parse for links.
                Vec::new(),
            )
        });

        Ok((sender_id, receiver_id, params))
    }

    fn exec_inner(machine: FvmMachine<DB>, engine: Engine, msg: Message) -> (anyhow::Result<ApplyRet>, FvmMachine<DB>) {
        let (sender_id, receiver_id, params) = match Self::process_message(&machine, &msg) {
            Ok(v) => v,
            Err(e) => return (Err(e), machine),
        };

        let mut cm = DefaultCallManager::new(
            machine,
            engine,
            msg.gas_limit,
            sender_id,
            msg.from,
            receiver_id,
            msg.to,
            msg.sequence,
            TokenAmount::zero(),
        );

        cm.machine_mut().state_tree_mut().begin_transaction();

        let r = match cm.call_actor::<FvmKernel<DB>>(
            sender_id,
            msg.to,
            Entrypoint::Invoke(msg.method_num),
            params,
            &msg.value,
            None,
            false,
        ) {
            Ok(v) => v,
            Err(e) => return (Err(anyhow!("{}", e.to_string())), cm.finish().1),
        };

        // always revert
        if let Err(e) = cm.machine_mut().state_tree_mut().end_transaction(true) {
            return (Err(anyhow!("{}", e.to_string())), cm.finish().1);
        }

        let (res, machine) = match cm.finish() {
            (Ok(res), machine) => (res, machine),
            (Err(err), machine) => return (
                Err(anyhow!("{}", err.to_string())),
                machine
            ),
        };
        let return_data = r.value
            .map(|blk| RawBytes::from(blk.data().to_vec()))
            .unwrap_or_default();

        let receipt = Receipt {
            exit_code: r.exit_code,
            return_data,
            gas_used: res.gas_used,
            events_root: res.events_root,
        };

        (Ok(ApplyRet {
            msg_receipt: receipt,
            penalty: TokenAmount::zero(),
            miner_tip: TokenAmount::zero(),
            base_fee_burn: TokenAmount::zero(),
            over_estimation_burn: TokenAmount::zero(),
            refund: TokenAmount::zero(),
            gas_refund: 0,
            gas_burned: 0,
            failure_info: None,
            exec_trace: vec![],
            events: res.events,
        }), machine)

    }

}