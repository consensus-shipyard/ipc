// Copyright 2025 Recall Contributors
// Copyright 2021-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::ops::{Deref, DerefMut};
use std::result::Result as StdResult;

use anyhow::{anyhow, bail, Context, Result};
use cid::Cid;
use fendermint_actor_blobs_shared::params::{GetGasAllowanceParams, UpdateGasAllowanceParams};
use fendermint_actor_blobs_shared::state::GasAllowance;
use fendermint_vm_actor_interface::{
    blobs::{BLOBS_ACTOR_ADDR, BLOBS_ACTOR_ID},
    eam::EAM_ACTOR_ID,
    system::SYSTEM_ACTOR_ADDR,
};
use fvm::call_manager::{backtrace, Backtrace, CallManager, Entrypoint, InvocationResult};
use fvm::engine::EnginePool;
use fvm::executor::{ApplyFailure, ApplyKind, ApplyRet, Executor};
use fvm::gas::{Gas, GasCharge};
use fvm::kernel::{Block, ClassifyResult, Context as _, ExecutionError, Kernel};
use fvm::machine::{Machine, BURNT_FUNDS_ACTOR_ID, REWARD_ACTOR_ID};
use fvm::trace::ExecutionTrace;
use fvm_ipld_encoding::{RawBytes, CBOR};
use fvm_shared::{
    address::{Address, Payload},
    econ::TokenAmount,
    error::{ErrorNumber, ExitCode},
    event::StampedEvent,
    message::Message,
    receipt::Receipt,
    ActorID, IPLD_RAW, METHOD_SEND,
};
use num_traits::Zero;
use tracing::debug;

mod outputs;

use crate::outputs::{GasAmounts, GasOutputs};

/// The default [`Executor`].
///
/// # Warning
///
/// Message execution might run out of stack and crash (the entire process) if it doesn't have at
/// least 64MiB of stack space. If you can't guarantee 64MiB of stack space, wrap this executor in
/// a [`ThreadedExecutor`][super::ThreadedExecutor].
pub struct RecallExecutor<K: Kernel> {
    engine_pool: EnginePool,
    // If the inner value is `None,` it means the machine got poisoned and is unusable.
    machine: Option<<K::CallManager as CallManager>::Machine>,
}

impl<K: Kernel> Deref for RecallExecutor<K> {
    type Target = <K::CallManager as CallManager>::Machine;

    fn deref(&self) -> &Self::Target {
        self.machine.as_ref().expect("machine poisoned")
    }
}

impl<K: Kernel> DerefMut for RecallExecutor<K> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.machine.as_mut().expect("machine poisoned")
    }
}

impl<K> Executor for RecallExecutor<K>
where
    K: Kernel,
{
    type Kernel = K;

    /// This is the entrypoint to execute a message.
    fn execute_message(
        &mut self,
        msg: Message,
        apply_kind: ApplyKind,
        raw_length: usize,
    ) -> Result<ApplyRet> {
        // Validate if the message was correct, charge for it, and extract some preliminary data.
        let (sender_id, sponsor_id, gas_costs, inclusion_cost) =
            match self.preflight_message(&msg, apply_kind, raw_length)? {
                Ok(res) => res,
                Err(apply_ret) => return Ok(apply_ret),
            };

        struct MachineExecRet {
            result: fvm::kernel::Result<InvocationResult>,
            gas_used: u64,
            backtrace: Backtrace,
            exec_trace: ExecutionTrace,
            events_root: Option<Cid>,
            events: Vec<StampedEvent>, // TODO consider removing if nothing in the client ends up using it.
        }

        // Pre-resolve the message receiver's address, if known.
        let receiver_id = self
            .state_tree()
            .lookup_id(&msg.to)
            .context("failure when looking up message receiver")?;

        // Filecoin caps the premium plus the base-fee at the fee-cap.
        // We expose the _effective_ premium to the user.
        let effective_premium = msg
            .gas_premium
            .clone()
            .min(&msg.gas_fee_cap - &self.context().base_fee)
            .max(TokenAmount::zero());

        // Acquire an engine from the pool. This may block if there are concurrently executing
        // messages inside other executors sharing the same pool.
        let engine = self.engine_pool.acquire();

        // Apply the message.
        let ret = self.map_machine(|machine| {
            // We're processing a chain message, so the sender is the origin of the call stack.
            let mut cm = K::CallManager::new(
                machine,
                engine,
                msg.gas_limit,
                sender_id,
                msg.from,
                receiver_id,
                msg.to,
                msg.sequence,
                effective_premium,
            );
            // This error is fatal because it should have already been accounted for inside
            // preflight_message.
            if let Err(e) = cm.charge_gas(inclusion_cost) {
                let (_, machine) = cm.finish();
                return (Err(e), machine);
            }

            let params = (!msg.params.is_empty()).then(|| {
                Block::new(
                    if msg.method_num == METHOD_SEND {
                        // Method zero params are "arbitrary bytes", so we'll just count them as
                        // raw.
                        //
                        // This won't actually affect anything (because no code will see these
                        // parameters), but it's more correct and makes me happier.
                        //
                        // NOTE: this _may_ start to matter once we start _validating_ ipld (m2.2).
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

            let result = cm.with_transaction(|cm| {
                // Invoke the message. We charge for the return value internally if the call-stack depth
                // is 1.
                cm.call_actor::<K>(
                    sender_id,
                    msg.to,
                    Entrypoint::Invoke(msg.method_num),
                    params,
                    &msg.value,
                    None,
                    false,
                )
            });

            let (res, machine) = match cm.finish() {
                (Ok(res), machine) => (res, machine),
                (Err(err), machine) => return (Err(err), machine),
            };

            (
                Ok(MachineExecRet {
                    result,
                    gas_used: res.gas_used,
                    backtrace: res.backtrace,
                    exec_trace: res.exec_trace,
                    events_root: res.events_root,
                    events: res.events,
                }),
                machine,
            )
        })?;

        let MachineExecRet {
            result: res,
            gas_used,
            mut backtrace,
            exec_trace,
            events_root,
            events,
        } = ret;

        // Extract the exit code and build the result of the message application.
        let receipt = match res {
            Ok(InvocationResult { exit_code, value }) => {
                // Convert back into a top-level return "value". We throw away the codec here,
                // unfortunately.
                let return_data = value
                    .map(|blk| RawBytes::from(blk.data().to_vec()))
                    .unwrap_or_default();

                if exit_code.is_success() {
                    backtrace.clear();
                }
                Receipt {
                    exit_code,
                    return_data,
                    gas_used,
                    events_root,
                }
            }
            Err(ExecutionError::OutOfGas) => Receipt {
                exit_code: ExitCode::SYS_OUT_OF_GAS,
                return_data: Default::default(),
                gas_used,
                events_root,
            },
            Err(ExecutionError::Syscall(err)) => {
                // Errors indicate the message couldn't be dispatched at all
                // (as opposed to failing during execution of the receiving actor).
                // These errors are mapped to exit codes that persist on chain.
                let exit_code = match err.1 {
                    ErrorNumber::InsufficientFunds => ExitCode::SYS_INSUFFICIENT_FUNDS,
                    ErrorNumber::NotFound => ExitCode::SYS_INVALID_RECEIVER,
                    _ => ExitCode::SYS_ASSERTION_FAILED,
                };

                backtrace.begin(backtrace::Cause::from_syscall("send", "send", err));
                Receipt {
                    exit_code,
                    return_data: Default::default(),
                    gas_used,
                    events_root,
                }
            }
            Err(ExecutionError::Fatal(err)) => {
                // We produce a receipt with SYS_ASSERTION_FAILED exit code, and
                // we consume the full gas amount so that, in case of a network-
                // wide fatal errors, all nodes behave deterministically.
                //
                // We set the backtrace from the fatal error to aid diagnosis.
                // Note that we use backtrace#set_cause instead of backtrace#begin
                // because we want to retain the propagation chain that we've
                // accumulated on the way out.
                let err = err.context(format!(
                    "[from={}, to={}, seq={}, m={}, h={}]",
                    msg.from,
                    msg.to,
                    msg.sequence,
                    msg.method_num,
                    self.context().epoch,
                ));
                backtrace.set_cause(backtrace::Cause::from_fatal(err));
                Receipt {
                    exit_code: ExitCode::SYS_ASSERTION_FAILED,
                    return_data: Default::default(),
                    gas_used: msg.gas_limit,
                    events_root,
                }
            }
        };

        let failure_info = if backtrace.is_empty() || receipt.exit_code.is_success() {
            None
        } else {
            Some(ApplyFailure::MessageBacktrace(backtrace))
        };

        match apply_kind {
            ApplyKind::Explicit => self.finish_message(
                sender_id,
                sponsor_id,
                msg,
                receipt,
                failure_info,
                gas_costs,
                exec_trace,
                events,
            ),
            ApplyKind::Implicit => Ok(ApplyRet {
                msg_receipt: receipt,
                penalty: TokenAmount::zero(),
                miner_tip: TokenAmount::zero(),
                base_fee_burn: TokenAmount::zero(),
                over_estimation_burn: TokenAmount::zero(),
                refund: TokenAmount::zero(),
                gas_refund: 0,
                gas_burned: 0,
                failure_info,
                exec_trace,
                events,
            }),
        }
    }

    /// Flush the state-tree to the underlying blockstore.
    fn flush(&mut self) -> Result<Cid> {
        let k = (**self).flush()?;
        Ok(k)
    }
}

impl<K> RecallExecutor<K>
where
    K: Kernel,
{
    /// Create a new [`RecallExecutor`] for executing messages on the [`Machine`].
    pub fn new(
        engine_pool: EnginePool,
        machine: <K::CallManager as CallManager>::Machine,
    ) -> Result<Self> {
        // Skip preloading all builtin actors when testing.
        #[cfg(not(any(test, feature = "testing")))]
        {
            // Preload any uncached modules.
            // This interface works for now because we know all actor CIDs
            // ahead of time, but with user-supplied code, we won't have that
            // guarantee.
            engine_pool.acquire().preload_all(
                machine.blockstore(),
                machine.builtin_actors().builtin_actor_codes(),
            )?;
        }
        Ok(Self {
            engine_pool,
            machine: Some(machine),
        })
    }

    /// Consume consumes the executor and returns the Machine. If the Machine had
    /// been poisoned during execution, the Option will be None.
    pub fn into_machine(self) -> Option<<K::CallManager as CallManager>::Machine> {
        self.machine
    }

    // TODO: The return type here is very strange because we have three cases:
    //  1. Continue: Return sender ID, & gas.
    //  2. Short-circuit: Return ApplyRet.
    //  3. Fail: Return an error.
    //  We could use custom types, but that would be even more annoying.
    fn preflight_message(
        &mut self,
        msg: &Message,
        apply_kind: ApplyKind,
        raw_length: usize,
    ) -> Result<StdResult<(ActorID, Option<ActorID>, GasAmounts, GasCharge), ApplyRet>> {
        msg.check().or_fatal()?;

        // TODO We don't like having price lists _inside_ the FVM, but passing
        //  these across the boundary is also a no-go.
        let pl = &self.context().price_list;

        let (inclusion_cost, miner_penalty_amount) = match apply_kind {
            ApplyKind::Implicit => (
                GasCharge::new("none", Gas::zero(), Gas::zero()),
                Default::default(),
            ),
            ApplyKind::Explicit => {
                let inclusion_cost = pl.on_chain_message(raw_length);
                let inclusion_total = inclusion_cost.total().round_up();

                // Verify the cost of the message is not over the message gas limit.
                if inclusion_total > msg.gas_limit {
                    return Ok(Err(ApplyRet::prevalidation_fail(
                        ExitCode::SYS_OUT_OF_GAS,
                        format!("Out of gas ({} > {})", inclusion_total, msg.gas_limit),
                        &self.context().base_fee * inclusion_total,
                    )));
                }

                let miner_penalty_amount = &self.context().base_fee * msg.gas_limit;
                (inclusion_cost, miner_penalty_amount)
            }
        };

        // Load sender actor state.
        let sender_id = match self
            .state_tree()
            .lookup_id(&msg.from)
            .with_context(|| format!("failed to lookup actor {}", &msg.from))?
        {
            Some(id) => id,
            None => {
                return Ok(Err(ApplyRet::prevalidation_fail(
                    ExitCode::SYS_SENDER_INVALID,
                    "Sender invalid",
                    miner_penalty_amount,
                )));
            }
        };

        if apply_kind == ApplyKind::Implicit {
            return Ok(Ok((sender_id, None, GasAmounts::default(), inclusion_cost)));
        }

        let mut sender_state = match self
            .state_tree()
            .get_actor(sender_id)
            .with_context(|| format!("failed to lookup actor {}", &msg.from))?
        {
            Some(act) => act,
            None => {
                return Ok(Err(ApplyRet::prevalidation_fail(
                    ExitCode::SYS_SENDER_INVALID,
                    "Sender invalid",
                    miner_penalty_amount,
                )));
            }
        };

        // Sender is valid if it is:
        // - an account actor
        // - an Ethereum Externally Owned Address
        // - a placeholder actor that has an f4 address in the EAM's namespace

        let mut sender_is_valid = self.builtin_actors().is_account_actor(&sender_state.code)
            || self
                .builtin_actors()
                .is_ethaccount_actor(&sender_state.code);

        if self.builtin_actors().is_placeholder_actor(&sender_state.code) &&
            sender_state.sequence == 0 &&
            sender_state
                .delegated_address
                .map(|a| matches!(a.payload(), Payload::Delegated(da) if da.namespace() == EAM_ACTOR_ID))
                .unwrap_or(false) {
            sender_is_valid = true;
            sender_state.code = *self.builtin_actors().get_ethaccount_code();
        }

        if !sender_is_valid {
            return Ok(Err(ApplyRet::prevalidation_fail(
                ExitCode::SYS_SENDER_INVALID,
                "Send not from valid sender",
                miner_penalty_amount,
            )));
        };

        // Check sequence is correct
        if msg.sequence != sender_state.sequence {
            return Ok(Err(ApplyRet::prevalidation_fail(
                ExitCode::SYS_SENDER_STATE_INVALID,
                format!(
                    "Actor sequence invalid: {} != {}",
                    msg.sequence, sender_state.sequence
                ),
                miner_penalty_amount,
            )));
        };

        sender_state.sequence += 1;

        // Get sender's gas allowance for gas fees.
        let gas_allowance = self.get_gas_allowance(msg.from)?;

        // Pre-resolve the message sponsor's address, if known.
        let sponsor_id = if let Some(sponsor) = gas_allowance.sponsor {
            self.state_tree()
                .lookup_id(&sponsor)
                .context("failure when looking up message sponsor")?
        } else {
            None
        };

        // Ensure from actor has enough balance to cover the gas cost of the message.
        let total_gas_allowance = gas_allowance.total();
        let total_gas_cost: TokenAmount = msg.gas_fee_cap.clone() * msg.gas_limit;
        let sender_balance = sender_state.balance.clone();
        if &total_gas_allowance + &sender_balance < total_gas_cost {
            return Ok(Err(ApplyRet::prevalidation_fail(
                ExitCode::SYS_SENDER_STATE_INVALID,
                format!(
                    "Actor allowance plus balance less than needed: {} + {} < {}",
                    total_gas_allowance, sender_state.balance, total_gas_cost
                ),
                miner_penalty_amount,
            )));
        }
        let gas_costs = if total_gas_allowance.is_zero() {
            // The sender is responsible for the entire gas cost
            sender_state.deduct_funds(&total_gas_cost)?;
            GasAmounts::new(total_gas_cost, TokenAmount::zero(), TokenAmount::zero())
        } else {
            // Use the sender's gas allowance from the source actor
            let mut source_state =
                match self
                    .state_tree()
                    .get_actor(BLOBS_ACTOR_ID)
                    .with_context(|| {
                        format!(
                            "failed to lookup gas source actor with id {}",
                            BLOBS_ACTOR_ID
                        )
                    })? {
                    Some(act) => act,
                    None => {
                        return Ok(Err(ApplyRet::prevalidation_fail(
                            ExitCode::SYS_SENDER_INVALID,
                            "Sender invalid",
                            miner_penalty_amount,
                        )));
                    }
                };

            // Check the source balance
            if source_state.balance < total_gas_allowance {
                // This should not happen
                return Ok(Err(ApplyRet::prevalidation_fail(
                    ExitCode::SYS_SENDER_STATE_INVALID,
                    format!(
                        "Gas allowance source actor balance less than needed: {} < {}",
                        source_state.balance, total_gas_allowance
                    ),
                    miner_penalty_amount,
                )));
            }

            let gas_costs = if total_gas_allowance < total_gas_cost {
                // Deduct the entire allowance
                source_state.deduct_funds(&total_gas_allowance)?;
                // Deduct the remainder from sender
                let sender_gas_cost = &total_gas_cost - &total_gas_allowance;
                sender_state.deduct_funds(&sender_gas_cost)?;
                // Consume entire allowance
                GasAmounts::new(
                    sender_gas_cost,
                    gas_allowance.amount,
                    gas_allowance.sponsored_amount,
                )
            } else {
                // Deduct entire gas cost from source
                source_state.deduct_funds(&total_gas_cost)?;
                // Consume allowances
                let (gas_cost, sponsored_gas_cost) = if gas_allowance.sponsored_amount.is_zero() {
                    // Consume from own allowance
                    (total_gas_cost, TokenAmount::zero())
                } else {
                    // Prioritize sponsor allowance when consuming
                    if gas_allowance.sponsored_amount > total_gas_cost {
                        // Consume from sponsored allowance
                        (TokenAmount::zero(), total_gas_cost)
                    } else {
                        // Consume entire sponsored allowance
                        (
                            &total_gas_cost - &gas_allowance.sponsored_amount,
                            gas_allowance.sponsored_amount,
                        )
                    }
                };
                GasAmounts::new(TokenAmount::zero(), gas_cost, sponsored_gas_cost)
            };

            // Update the source actor in the state tree
            self.state_tree_mut()
                .set_actor(BLOBS_ACTOR_ID, source_state);
            gas_costs
        };

        // Update the sender actor in the state tree
        self.state_tree_mut().set_actor(sender_id, sender_state);

        // Debit gas costs (the unused amount will get refunded)
        self.update_gas_allowance(msg.from, None, -gas_costs.from_allowance.clone())?;
        self.update_gas_allowance(
            msg.from,
            gas_allowance.sponsor,
            -gas_costs.from_sponsor_allowance.clone(),
        )?;

        debug!(
            from_balance = ?gas_costs.from_balance,
            from_allowance = ?gas_costs.from_allowance,
            from_sponsor_allowance = ?gas_costs.from_sponsor_allowance,
            "calculated gas costs for tx from {} to {}",
            msg.from,
            msg.to
        );

        Ok(Ok((sender_id, sponsor_id, gas_costs, inclusion_cost)))
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_message(
        &mut self,
        sender_id: ActorID,
        sponsor_id: Option<ActorID>,
        msg: Message,
        receipt: Receipt,
        failure_info: Option<ApplyFailure>,
        gas_costs: GasAmounts,
        exec_trace: ExecutionTrace,
        events: Vec<StampedEvent>,
    ) -> Result<ApplyRet> {
        // NOTE: we don't support old network versions in the FVM, so we always burn.
        let gas_outputs = GasOutputs::compute(
            receipt.gas_used,
            msg.gas_limit,
            &self.context().base_fee,
            &msg.gas_fee_cap,
            &msg.gas_premium,
        );

        debug!(
            "gas outputs for tx from {} to {}: {:#?}",
            msg.from, msg.to, gas_outputs
        );

        let GasOutputs {
            base_fee_burn,
            over_estimation_burn,
            miner_penalty,
            miner_tip,
            refund,
            gas_refund,
            gas_burned,
        } = gas_outputs;

        let mut transfer_to_actor = |addr: ActorID, amt: &TokenAmount| -> Result<()> {
            if amt.is_negative() {
                return Err(anyhow!("attempted to transfer negative value into actor"));
            }
            if amt.is_zero() {
                return Ok(());
            }

            self.state_tree_mut()
                .mutate_actor(addr, |act| act.deposit_funds(amt).or_fatal())
                .context("failed to lookup actor for transfer")?;
            Ok(())
        };

        transfer_to_actor(BURNT_FUNDS_ACTOR_ID, &base_fee_burn)?;

        transfer_to_actor(REWARD_ACTOR_ID, &miner_tip)?;

        transfer_to_actor(BURNT_FUNDS_ACTOR_ID, &over_estimation_burn)?;

        let gas_refunds = gas_costs.refund(&refund);
        transfer_to_actor(sender_id, &gas_refunds.from_balance)?;
        transfer_to_actor(
            BLOBS_ACTOR_ID,
            &(&gas_refunds.from_allowance + &gas_refunds.from_sponsor_allowance),
        )?;

        debug!(
            balance_refund = ?gas_refunds.from_balance,
            gas_refund = ?gas_refunds.from_allowance,
            sponsor_gas_refund = ?gas_refunds.from_sponsor_allowance,
            "calculated gas refunds for tx from {} to {}",
            msg.from,
            msg.to
        );

        if (&base_fee_burn + &over_estimation_burn + &refund + &miner_tip) != gas_costs.total() {
            // Sanity check. This could be a fatal error.
            return Err(anyhow!("Gas handling math is wrong"));
        }

        // Refund gas difference
        self.update_gas_allowance(msg.from, None, gas_refunds.from_allowance)?;
        self.update_gas_allowance(
            msg.from,
            sponsor_id.map(Address::new_id),
            gas_refunds.from_sponsor_allowance,
        )?;

        Ok(ApplyRet {
            msg_receipt: receipt,
            penalty: miner_penalty,
            miner_tip,
            base_fee_burn,
            over_estimation_burn,
            refund,
            gas_refund,
            gas_burned,
            failure_info,
            exec_trace,
            events,
        })
    }

    fn map_machine<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(
            <K::CallManager as CallManager>::Machine,
        ) -> (T, <K::CallManager as CallManager>::Machine),
    {
        replace_with::replace_with_and_return(
            &mut self.machine,
            || None,
            |m| {
                let (ret, machine) = f(m.unwrap());
                (ret, Some(machine))
            },
        )
    }

    /// Returns the gas allowance for the sender.
    fn get_gas_allowance(&mut self, from: Address) -> Result<GasAllowance> {
        let params = RawBytes::serialize(GetGasAllowanceParams(from))?;

        let msg = Message {
            from: SYSTEM_ACTOR_ADDR,
            to: BLOBS_ACTOR_ADDR,
            sequence: 0, // irrelevant for implicit executions
            gas_limit: i64::MAX as u64,
            method_num: fendermint_actor_blobs_shared::Method::GetGasAllowance as u64,
            params,
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };

        let apply_ret = self.execute_message(msg, ApplyKind::Implicit, 0)?;
        if let Some(err) = apply_ret.failure_info {
            bail!("failed to get gas allowance for {}: {}", from, err);
        }

        fvm_ipld_encoding::from_slice::<GasAllowance>(&apply_ret.msg_receipt.return_data)
            .context("failed to parse gas allowance")
    }

    /// Updates gas allowance from the sender.
    fn update_gas_allowance(
        &mut self,
        from: Address,
        sponsor: Option<Address>,
        add_amount: TokenAmount,
    ) -> Result<()> {
        if add_amount.is_zero() {
            return Ok(());
        }

        let params = RawBytes::serialize(UpdateGasAllowanceParams {
            from,
            sponsor,
            add_amount: add_amount.clone(),
        })?;

        let msg = Message {
            from: SYSTEM_ACTOR_ADDR,
            to: BLOBS_ACTOR_ADDR,
            sequence: 0, // irrelevant for implicit executions
            gas_limit: i64::MAX as u64,
            method_num: fendermint_actor_blobs_shared::Method::UpdateGasAllowance as u64,
            params,
            value: Default::default(),
            version: Default::default(),
            gas_fee_cap: Default::default(),
            gas_premium: Default::default(),
        };

        let apply_ret = self.execute_message(msg, ApplyKind::Implicit, 0)?;
        if let Some(err) = apply_ret.failure_info {
            bail!(
                "failed to update gas allowance for {} (amount: {}; sponsor: {:?}): {}",
                from,
                add_amount,
                sponsor,
                err
            );
        }

        debug!(
            "updated gas allowance for {} (amount: {}; sponsor: {:?})",
            from, add_amount, sponsor
        );

        Ok(())
    }
}
