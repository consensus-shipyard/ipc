// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

//! Credit subcommand for buying and querying storage credits.

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use clap::{Args, Subcommand};
use num_traits::Zero;
use std::path::PathBuf;

use fendermint_actor_blobs_shared::{
    accounts::account::Account,
    credit::BuyCreditParams,
    method::Method as BlobsMethod,
    BLOBS_ACTOR_ADDR,
};
use fendermint_rpc::client::FendermintClient;
use fendermint_rpc::message::{GasParams, SignedMessageFactory};
use fendermint_rpc::tx::{TxClient, TxCommit};
use fendermint_rpc::QueryClient;
use fendermint_vm_message::query::FvmQueryHeight;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address;
use fvm_shared::chainid::ChainID;
use fvm_shared::econ::TokenAmount;

use crate::commands::storage::bucket;
use crate::commands::storage::config::StorageConfig;
use crate::{CommandLineHandler, GlobalArguments};

#[derive(Debug, Args)]
#[command(name = "credit", about = "Buy and query storage credits")]
pub struct CreditCommandArgs {
    #[command(subcommand)]
    command: CreditCommands,
}

#[derive(Debug, Subcommand)]
pub enum CreditCommands {
    /// Buy storage credits by sending tokens to the blobs actor
    Buy(BuyCreditArgs),
    /// Get account credit information
    Info(CreditInfoArgs),
}

impl CreditCommandArgs {
    pub async fn handle(&self, global: &GlobalArguments) -> anyhow::Result<()> {
        match &self.command {
            CreditCommands::Buy(args) => BuyCredit::handle(global, args).await,
            CreditCommands::Info(args) => CreditInfo::handle(global, args).await,
        }
    }
}

// ---------------------------------------------------------------------------
// Buy
// ---------------------------------------------------------------------------

#[derive(Debug, Args)]
pub struct BuyCreditArgs {
    /// Amount of tokens to spend (in FIL/ether units, e.g. 0.1)
    #[arg(value_name = "AMOUNT")]
    pub amount: f64,

    /// Recipient address (defaults to the operator key address)
    #[arg(long)]
    pub to: Option<String>,

    /// Storage config file
    #[arg(long)]
    pub config: Option<PathBuf>,
}

pub struct BuyCredit;

#[async_trait]
impl CommandLineHandler for BuyCredit {
    type Arguments = BuyCreditArgs;

    async fn handle(_global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        if args.amount <= 0.0 {
            return Err(anyhow!("Amount must be positive"));
        }

        let config_path = args.config.clone().unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap()
                .join(".ipc")
                .join("storage_default.yaml")
        });

        let config = if config_path.exists() {
            StorageConfig::load(&config_path)?
        } else {
            return Err(anyhow!(
                "Storage config not found at {}. Run 'ipc-cli storage init' first.",
                config_path.display()
            ));
        };

        let fm_client = FendermintClient::new_http(config.tendermint_rpc_url.parse()?, None)?;

        let chain_id = bucket::query_chain_id(&fm_client)
            .await
            .context("Failed to query chain ID")?;

        let secret_key = SignedMessageFactory::read_secret_key(&config.secret_key_file)?;
        let addr = Address::new_secp256k1(&secret_key.public_key().serialize())?;

        let state = fm_client
            .actor_state(&addr, FvmQueryHeight::default())
            .await
            .context("Failed to get actor state")?;
        let sequence = state.value.map(|(_, s)| s.sequence).unwrap_or(0);

        let mf = SignedMessageFactory::new(secret_key, addr, sequence, ChainID::from(chain_id));
        let mut bound_client = fm_client.bind(mf);

        // Determine recipient
        let recipient = if let Some(ref to_str) = args.to {
            crate::require_fil_addr_from_str(to_str)?
        } else {
            addr
        };

        let params = BuyCreditParams(recipient);
        let params_bytes =
            RawBytes::serialize(params).context("Failed to serialize BuyCreditParams")?;

        // Convert amount to TokenAmount (nano precision)
        let value = crate::f64_to_token_amount(args.amount)?;

        let gas_params = GasParams {
            gas_limit: 10_000_000_000,
            gas_fee_cap: TokenAmount::from_atto(100),
            gas_premium: TokenAmount::from_atto(100),
        };

        println!("Buying credit for {} with {} FIL...", recipient, args.amount);

        let res = TxClient::<TxCommit>::transaction(
            &mut bound_client,
            BLOBS_ACTOR_ADDR,
            BlobsMethod::BuyCredit as u64,
            params_bytes,
            value,
            gas_params,
        )
        .await
        .context("Failed to send BuyCredit transaction")?;

        if res.response.check_tx.code.is_err() {
            return Err(anyhow!(
                "BuyCredit check_tx failed: {}",
                res.response.check_tx.log
            ));
        }

        if res.response.deliver_tx.code.is_err() {
            return Err(anyhow!(
                "BuyCredit deliver_tx failed: {}",
                res.response.deliver_tx.log
            ));
        }

        println!("Credit purchased successfully for {}", recipient);

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Info
// ---------------------------------------------------------------------------

#[derive(Debug, Args)]
pub struct CreditInfoArgs {
    /// Account address to query (defaults to operator key address)
    #[arg(long)]
    pub address: Option<String>,

    /// Storage config file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Output in JSON format
    #[arg(long)]
    pub json: bool,
}

pub struct CreditInfo;

#[async_trait]
impl CommandLineHandler for CreditInfo {
    type Arguments = CreditInfoArgs;

    async fn handle(_global: &GlobalArguments, args: &Self::Arguments) -> Result<()> {
        let config_path = args.config.clone().unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap()
                .join(".ipc")
                .join("storage_default.yaml")
        });

        let config = if config_path.exists() {
            StorageConfig::load(&config_path)?
        } else {
            return Err(anyhow!(
                "Storage config not found at {}. Run 'ipc-cli storage init' first.",
                config_path.display()
            ));
        };

        let fm_client = FendermintClient::new_http(config.tendermint_rpc_url.parse()?, None)?;

        // Determine the address to query
        let query_addr = if let Some(ref addr_str) = args.address {
            crate::require_fil_addr_from_str(addr_str)?
        } else {
            let secret_key = SignedMessageFactory::read_secret_key(&config.secret_key_file)?;
            Address::new_secp256k1(&secret_key.public_key().serialize())?
        };

        // Query the GetAccount method on the blobs actor
        let params_bytes =
            RawBytes::serialize(query_addr).context("Failed to serialize address")?;

        let msg = fvm_shared::message::Message {
            version: Default::default(),
            from: fendermint_vm_actor_interface::system::SYSTEM_ACTOR_ADDR,
            to: BLOBS_ACTOR_ADDR,
            sequence: 0,
            value: TokenAmount::zero(),
            method_num: BlobsMethod::GetAccount as u64,
            params: params_bytes,
            gas_limit: 10_000_000_000,
            gas_fee_cap: TokenAmount::zero(),
            gas_premium: TokenAmount::zero(),
        };

        let response = fm_client
            .call(msg, FvmQueryHeight::default())
            .await
            .context("Failed to query GetAccount")?;

        if response.value.code.is_err() {
            return Err(anyhow!(
                "GetAccount query failed: {}",
                response.value.info
            ));
        }

        let return_data = fendermint_rpc::response::decode_data(&response.value.data)
            .context("Failed to decode response data")?;

        let account: Option<Account> = fvm_ipld_encoding::from_slice(&return_data)
            .context("Failed to decode Account")?;

        match account {
            Some(acct) => {
                if args.json {
                    let output = serde_json::json!({
                        "address": query_addr.to_string(),
                        "capacity_used": acct.capacity_used,
                        "credit_free": acct.credit_free.atto().to_string(),
                        "credit_committed": acct.credit_committed.atto().to_string(),
                        "credit_sponsor": acct.credit_sponsor.map(|a| a.to_string()),
                        "last_debit_epoch": acct.last_debit_epoch,
                        "max_ttl": acct.max_ttl,
                        "gas_allowance": acct.gas_allowance.atto().to_string(),
                        "approvals_to": acct.approvals_to.len(),
                        "approvals_from": acct.approvals_from.len(),
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                } else {
                    println!("Account: {}", query_addr);
                    println!("  Capacity used:    {} bytes", acct.capacity_used);
                    println!("  Credit free:      {}", acct.credit_free);
                    println!("  Credit committed: {}", acct.credit_committed);
                    if let Some(sponsor) = &acct.credit_sponsor {
                        println!("  Credit sponsor:   {}", sponsor);
                    }
                    println!("  Last debit epoch: {}", acct.last_debit_epoch);
                    println!("  Max TTL:          {} epochs", acct.max_ttl);
                    println!("  Gas allowance:    {}", acct.gas_allowance);
                    println!(
                        "  Approvals:        {} outgoing, {} incoming",
                        acct.approvals_to.len(),
                        acct.approvals_from.len()
                    );
                }
            }
            None => {
                println!("No account found for {}", query_addr);
            }
        }

        Ok(())
    }
}
