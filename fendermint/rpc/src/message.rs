// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_vm_actor_interface::{eam, evm};
use fendermint_vm_message::{chain::ChainMessage, signed::SignedMessage};
use fvm_ipld_encoding::{BytesSer, RawBytes};
use fvm_shared::{address::Address, econ::TokenAmount, MethodNum, METHOD_SEND};
use libsecp256k1::{PublicKey, SecretKey};

/// Factory methods for signed transaction payload construction.
pub struct MessageFactory {
    sk: SecretKey,
    addr: Address,
    sequence: u64,
}

impl MessageFactory {
    pub fn new(sk: SecretKey, sequence: u64) -> anyhow::Result<Self> {
        let pk = PublicKey::from_secret_key(&sk);
        let addr = Address::new_secp256k1(&pk.serialize())?;
        Ok(Self { sk, addr, sequence })
    }

    /// Serialize a [`ChainMessage`] for inclusion in a Tendermint transaction.
    pub fn serialize(message: &ChainMessage) -> anyhow::Result<Vec<u8>> {
        Ok(fvm_ipld_encoding::to_vec(message)?)
    }

    /// Transfer tokens to another account.
    pub fn transfer(
        &mut self,
        to: Address,
        value: TokenAmount,
        gas_params: GasParams,
    ) -> anyhow::Result<ChainMessage> {
        self.transaction(to, METHOD_SEND, Default::default(), value, gas_params)
    }

    /// Send a message to an actor.
    pub fn transaction(
        &mut self,
        to: Address,
        method_num: MethodNum,
        params: RawBytes,
        value: TokenAmount,
        gas_params: GasParams,
    ) -> anyhow::Result<ChainMessage> {
        let message = fvm_shared::message::Message {
            version: Default::default(), // TODO: What does this do?
            from: self.addr,
            to,
            sequence: self.sequence,
            value,
            method_num,
            params,
            gas_limit: gas_params.gas_limit,
            gas_fee_cap: gas_params.gas_fee_cap,
            gas_premium: gas_params.gas_premium,
        };
        self.sequence += 1;
        let signed = SignedMessage::new_secp256k1(message, &self.sk)?;
        let chain = ChainMessage::Signed(Box::new(signed));
        Ok(chain)
    }

    /// Deploy a FEVM contract.
    pub fn fevm_create(
        &mut self,
        contract: RawBytes,
        constructor_args: RawBytes,
        value: TokenAmount,
        gas_params: GasParams,
    ) -> anyhow::Result<ChainMessage> {
        let initcode = [contract.to_vec(), constructor_args.to_vec()].concat();
        let initcode = RawBytes::serialize(BytesSer(&initcode))?;
        let message = self.transaction(
            eam::EAM_ACTOR_ADDR,
            eam::Method::CreateExternal as u64,
            initcode,
            value,
            gas_params,
        )?;
        Ok(message)
    }

    /// Invoke a method on a FEVM contract.
    pub fn fevm_invoke(
        &mut self,
        contract: Address,
        method: RawBytes,
        method_args: RawBytes,
        value: TokenAmount,
        gas_params: GasParams,
    ) -> anyhow::Result<ChainMessage> {
        let calldata = [method.to_vec(), method_args.to_vec()].concat();
        let calldata = RawBytes::serialize(BytesSer(&calldata))?;
        let message = self.transaction(
            contract,
            evm::Method::InvokeContract as u64,
            calldata,
            value,
            gas_params,
        )?;
        Ok(message)
    }
}

pub struct GasParams {
    /// Maximum amount of gas that can be charged.
    pub gas_limit: u64,
    /// Price of gas.
    ///
    /// Any discrepancy between this and the base fee is paid for
    /// by the validator who puts the transaction into the block.
    pub gas_fee_cap: TokenAmount,
    /// Gas premium.
    pub gas_premium: TokenAmount,
}
