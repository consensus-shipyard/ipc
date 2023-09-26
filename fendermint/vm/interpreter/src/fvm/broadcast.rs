// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use anyhow::{anyhow, bail, Context};
use ethers::types as et;
use fvm_shared::{address::Address, chainid::ChainID, econ::TokenAmount, BLOCK_GAS_LIMIT};
use num_traits::Zero;
use tendermint_rpc::Client;

use fendermint_crypto::SecretKey;
use fendermint_rpc::message::GasParams;
use fendermint_rpc::query::QueryClient;
use fendermint_rpc::tx::{CallClient, TxClient, TxCommit};
use fendermint_rpc::{client::FendermintClient, message::MessageFactory};
use fendermint_vm_message::query::FvmQueryHeight;

/// Broadcast transactions to Tendermint.
///
/// This is typically something only active validators would want to do
/// from within Fendermint as part of the block lifecycle, for example
/// to submit their signatures to the ledger.
///
/// The broadcaster encapsulates the tactics for figuring out the nonce,
/// the gas limit, potential retries, etc.
#[derive(Clone)]
pub struct Broadcaster<C> {
    client: FendermintClient<C>,
    secret_key: SecretKey,
    addr: Address,
    gas_fee_cap: TokenAmount,
    gas_premium: TokenAmount,
}

impl<C> Broadcaster<C>
where
    C: Client + Clone + Send + Sync,
{
    pub fn new(
        client: C,
        secret_key: SecretKey,
        gas_fee_cap: TokenAmount,
        gas_premium: TokenAmount,
    ) -> Self {
        let client = FendermintClient::new(client);
        // TODO: We could use f410 addresses to send the transaction, but the `MessageFactory` assumes f1.
        let addr = Address::new_secp256k1(&secret_key.public_key().serialize())
            .expect("public key is 65 bytes");
        Self {
            client,
            secret_key,
            addr,
            gas_fee_cap,
            gas_premium,
        }
    }

    pub async fn fevm_invoke(
        &self,
        contract: Address,
        calldata: et::Bytes,
        chain_id: ChainID,
    ) -> anyhow::Result<()> {
        let sequence = self
            .sequence()
            .await
            .context("failed to get broadcaster sequence")?;

        let factory = MessageFactory::new(self.secret_key.clone(), sequence, chain_id)
            .context("failed to create MessageFactory")?;

        // Using the bound client as a one-shot transaction sender.
        let mut client = self.client.clone().bind(factory);

        // TODO: Maybe we should implement something like the Ethereum facade for estimating fees?
        // I don't want to call the Ethereum API directly (it would be one more dependency).
        // Another option is for Fendermint to recognise transactions coming from validators
        // and always put them into the block to facilitate checkpointing.
        let mut gas_params = GasParams {
            gas_limit: BLOCK_GAS_LIMIT,
            gas_fee_cap: self.gas_fee_cap.clone(),
            gas_premium: self.gas_premium.clone(),
        };

        // Not expecting to send any tokens to the contracts.
        let value = TokenAmount::zero();

        let gas_estimate = client
            .fevm_estimate_gas(
                contract,
                calldata.0.clone(),
                value.clone(),
                gas_params.clone(),
                FvmQueryHeight::Committed,
            )
            .await
            .context("failed to estimate gas")?;

        if gas_estimate.value.exit_code.is_success() {
            gas_params.gas_limit = gas_estimate.value.gas_limit;
        } else {
            bail!(
                "failed to estimate gas: {} - {}",
                gas_estimate.value.exit_code,
                gas_estimate.value.info
            );
        }

        let res =
            TxClient::<TxCommit>::fevm_invoke(&mut client, contract, calldata.0, value, gas_params)
                .await
                .context("failed to invoke contract")?;

        if res.response.check_tx.code.is_err() {
            bail!(
                "broadcasted transaction failed during check: {} - {}",
                res.response.check_tx.code.value(),
                res.response.check_tx.info
            );
        } else if res.response.deliver_tx.code.is_err() {
            // TODO: Retry?
            bail!(
                "broadcasted transaction failed during deliver: {} - {}",
                res.response.deliver_tx.code.value(),
                res.response.deliver_tx.info
            );
        } else {
            Ok(())
        }
    }

    /// Fetch the current nonce to be used in the next message.
    async fn sequence(&self) -> anyhow::Result<u64> {
        let res = self
            .client
            .actor_state(&self.addr, FvmQueryHeight::Committed)
            .await
            .context("failed to get broadcaster actor state")?;

        match res.value {
            Some((_, state)) => Ok(state.sequence),
            None => Err(anyhow!("broadcaster actor {} cannot be found", self.addr)),
        }
    }
}
