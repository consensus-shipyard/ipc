// Copyright 2026 Recall Contributors
// SPDX-License-Identifier: MIT

use anyhow::{Context, Result};
use clap::{Args, Subcommand, ValueEnum};
use fendermint_actor_blobs_shared::execution::{
    GetJobParams, JobStatus, ListJobsParams, ListJobsReturn, CREATE_JOB_SELECTOR,
};
use fendermint_actor_blobs_shared::method::Method;
use fendermint_actor_blobs_shared::BLOBS_ACTOR_ADDR;
use ethers::abi::{encode as abi_encode, Token};
use ethers::types::U256 as EthU256;
use fendermint_rpc::message::{GasParams, SignedMessageFactory};
use fendermint_rpc::tx::{TxClient, TxCommit};
use fendermint_rpc::{FendermintClient, QueryClient};
use fendermint_vm_actor_interface::eam::EthAddress as FvmEthAddress;
use fendermint_vm_actor_interface::system;
use fendermint_vm_message::query::FvmQueryHeight;
use fvm_ipld_encoding::RawBytes;
use fvm_shared::address::Address;
use fvm_shared::bigint::Zero;
use fvm_shared::chainid::ChainID;
use fvm_shared::econ::TokenAmount;
use fvm_shared::message::Message;
use std::path::PathBuf;
use std::str::FromStr;
use tendermint_rpc::Url;

use crate::GlobalArguments;

#[derive(Debug, Args)]
#[command(name = "exec", about = "execution job commands")]
#[command(args_conflicts_with_subcommands = true)]
pub(crate) struct ExecCommandsArgs {
    #[command(subcommand)]
    command: Commands,
}

impl ExecCommandsArgs {
    pub async fn handle(&self, _global: &GlobalArguments) -> anyhow::Result<()> {
        match &self.command {
            Commands::Submit(args) => submit_job(args).await,
            Commands::List(args) => list_jobs(args).await,
            Commands::Status(args) => status_job(args).await,
        }
    }
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Submit(SubmitJobArgs),
    List(ListJobsArgs),
    Status(StatusJobArgs),
}

#[derive(Debug, Clone, ValueEnum)]
enum JobStatusArg {
    Pending,
    Claimed,
    Running,
    Succeeded,
    Failed,
    TimedOut,
}

impl From<JobStatusArg> for JobStatus {
    fn from(value: JobStatusArg) -> Self {
        match value {
            JobStatusArg::Pending => JobStatus::Pending,
            JobStatusArg::Claimed => JobStatus::Claimed,
            JobStatusArg::Running => JobStatus::Running,
            JobStatusArg::Succeeded => JobStatus::Succeeded,
            JobStatusArg::Failed => JobStatus::Failed,
            JobStatusArg::TimedOut => JobStatus::TimedOut,
        }
    }
}

#[derive(Debug, Args)]
pub(crate) struct SubmitJobArgs {
    #[arg(long, default_value = "http://localhost:26657")]
    rpc_url: String,
    #[arg(long, env = "SECRET_KEY_FILE", required = true)]
    secret_key_file: PathBuf,
    #[arg(long)]
    binary_ref: String,
    #[arg(long = "input-ref", help = "Raw input reference (passed as-is to input_refs)")]
    input_refs: Vec<String>,
    /// ipc:// URI for input file (e.g. ipc://t0123/data.txt). Executor downloads
    /// the file and sets IPC_INPUT_N env var pointing to the local path.
    #[arg(long = "input")]
    inputs: Vec<String>,
    /// ipc:// URI for output destination (e.g. ipc://t0123/result.txt). Executor
    /// uploads the file written to IPC_OUTPUT_FILE_N after the job completes.
    #[arg(long = "output")]
    outputs: Vec<String>,
    #[arg(long = "arg")]
    args: Vec<String>,
    #[arg(long = "env", help = "Extra env var in KEY=VALUE format")]
    env: Vec<String>,
    #[arg(long, default_value = "300")]
    timeout_secs: u64,
}

#[derive(Debug, Args)]
pub(crate) struct ListJobsArgs {
    #[arg(long, default_value = "http://localhost:26657")]
    rpc_url: String,
    #[arg(long)]
    status: Option<JobStatusArg>,
    #[arg(long, default_value = "20")]
    limit: u32,
}

#[derive(Debug, Args)]
pub(crate) struct StatusJobArgs {
    #[arg(long, default_value = "http://localhost:26657")]
    rpc_url: String,
    #[arg(long)]
    id: u64,
}

async fn submit_job(args: &SubmitJobArgs) -> Result<()> {
    let rpc_url = Url::from_str(&args.rpc_url).context("failed to parse RPC URL")?;
    let client = FendermintClient::new_http(rpc_url, None).context("failed to create client")?;

    let sk = SignedMessageFactory::read_secret_key(&args.secret_key_file)
        .context("failed to read secret key")?;
    let pk = sk.public_key();
    let from_eth = FvmEthAddress::new_secp256k1(&pk.serialize())
        .context("failed to derive delegated address from secret key")?;
    let from_f410 =
        Address::new_delegated(10, &from_eth.0).context("failed to create f410 address")?;

    let sequence = get_sequence_opt(&client, &from_f410)
        .await
        .context("failed to get delegated account sequence")?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "delegated sender {} not found on-chain; fund/initialize delegated account",
                from_f410,
            )
        })?;
    let chain_id = client
        .state_params(FvmQueryHeight::default())
        .await
        .context("failed to get state params")?
        .value
        .chain_id;

    let mf = SignedMessageFactory::new(sk, from_f410, sequence, ChainID::from(chain_id));
    let mut tx_client = client.bind(mf);

    // Merge --input-ref and --input into input_refs. --input values are ipc:// URIs
    // that the executor will download before running the binary.
    let mut all_input_refs = args.input_refs.clone();
    all_input_refs.extend(args.inputs.iter().cloned());

    // Build env: explicit --env pairs + IPC_OUTPUT_N entries from --output flags.
    let mut env_pairs = parse_env_pairs(&args.env)?;
    for (i, output_uri) in args.outputs.iter().enumerate() {
        env_pairs.push((format!("IPC_OUTPUT_{}", i), output_uri.clone()));
    }

    let calldata = encode_create_job_calldata(
        &args.binary_ref,
        &all_input_refs,
        &args.args,
        &env_pairs,
        args.timeout_secs,
    );

    let gas_params = GasParams {
        gas_limit: 10_000_000,
        gas_fee_cap: TokenAmount::from_atto(1_000_000),
        gas_premium: TokenAmount::from_atto(100_000),
    };
    let res = TxClient::<TxCommit>::fevm_invoke(
        &mut tx_client,
        BLOBS_ACTOR_ADDR,
        calldata.into(),
        TokenAmount::zero(),
        gas_params,
    )
    .await
    .context("failed to send CreateJob transaction")?;

    if res.response.check_tx.code.is_err() {
        let log = &res.response.check_tx.log;
        anyhow::bail!(
            "CreateJob check_tx rejected: code={:?} log={}",
            res.response.check_tx.code,
            if log.is_empty() { "<empty>" } else { log.as_str() },
        );
    }
    if res.response.deliver_tx.code.is_err() {
        let log = &res.response.deliver_tx.log;
        let info = &res.response.deliver_tx.info;
        anyhow::bail!(
            "CreateJob deliver_tx failed: code={:?} log={} info={}",
            res.response.deliver_tx.code,
            if log.is_empty() { "<empty>" } else { log.as_str() },
            if info.is_empty() { "<empty>" } else { info.as_str() },
        );
    }

    println!("Job submitted successfully");
    println!("  tx_hash: {}", res.response.hash);
    if !all_input_refs.is_empty() {
        println!("  inputs:");
        for r in &all_input_refs {
            println!("    - {}", r);
        }
    }
    if !args.outputs.is_empty() {
        println!("  outputs:");
        for o in &args.outputs {
            println!("    - {}", o);
        }
    }
    Ok(())
}

async fn list_jobs(args: &ListJobsArgs) -> Result<()> {
    let rpc_url = Url::from_str(&args.rpc_url).context("failed to parse RPC URL")?;
    let client = FendermintClient::new_http(rpc_url, None).context("failed to create client")?;

    let params = ListJobsParams {
        status: args.status.clone().map(Into::into),
        limit: args.limit,
    };
    let jobs = query_list_jobs(&client, params).await?;
    println!("Found {} jobs", jobs.jobs.len());
    for job in jobs.jobs {
        println!(
            "- id={} status={:?} creator={} claimed_by={}",
            job.id,
            job.status,
            job.creator,
            job.claimed_by
                .map(|a| a.to_string())
                .unwrap_or_else(|| "-".to_string())
        );
    }
    Ok(())
}

async fn status_job(args: &StatusJobArgs) -> Result<()> {
    let rpc_url = Url::from_str(&args.rpc_url).context("failed to parse RPC URL")?;
    let client = FendermintClient::new_http(rpc_url, None).context("failed to create client")?;
    let maybe = query_get_job(&client, args.id).await?;
    match maybe {
        Some(job) => {
            println!("Job {}", job.id);
            println!("  status:      {:?}", job.status);
            println!("  creator:     {}", job.creator);
            println!(
                "  claimed_by:  {}",
                job.claimed_by
                    .map(|a| a.to_string())
                    .unwrap_or_else(|| "-".to_string())
            );
            println!("  binary_ref:  {}", job.binary_ref);
            if !job.args.is_empty() {
                println!("  args:        {:?}", job.args);
            }
            if !job.env.is_empty() {
                println!("  env:");
                for (k, v) in &job.env {
                    println!("    {}={}", k, v);
                }
            }
            println!("  timeout:     {}s", job.timeout_secs);
            println!("  created:     epoch {}", job.created_epoch);
            if let Some(ep) = job.started_epoch {
                println!("  started:     epoch {}", ep);
            }
            if let Some(ep) = job.completed_epoch {
                println!("  completed:   epoch {}", ep);
            }
            if !job.input_refs.is_empty() {
                println!("  input_refs:");
                for r in &job.input_refs {
                    println!("    - {}", r);
                }
            }
            if !job.output_refs.is_empty() {
                println!("  output_refs:");
                for r in &job.output_refs {
                    println!("    - {}", r);
                }
            }
            if let Some(hash) = &job.output_commitment {
                println!("  output_hash: 0x{}", hex::encode(hash.0));
            }
            if let Some(code) = job.exit_code {
                println!("  exit_code:   {}", code);
            }
            if let Some(err) = &job.error {
                println!("  error:       {}", err);
            }
        }
        None => println!("Job {} not found", args.id),
    }
    Ok(())
}

async fn get_sequence_opt(client: &impl QueryClient, addr: &Address) -> Result<Option<u64>> {
    let state = client
        .actor_state(addr, FvmQueryHeight::default())
        .await
        .context("failed to get actor state")?;
    match state.value {
        Some((_id, state)) => Ok(Some(state.sequence)),
        None => Ok(None),
    }
}

fn parse_env_pairs(items: &[String]) -> Result<Vec<(String, String)>> {
    items
        .iter()
        .map(|item| {
            let (k, v) = item
                .split_once('=')
                .ok_or_else(|| anyhow::anyhow!("invalid --env entry '{}', expected KEY=VALUE", item))?;
            Ok((k.to_string(), v.to_string()))
        })
        .collect()
}

fn encode_create_job_calldata(
    binary_ref: &str,
    input_refs: &[String],
    args: &[String],
    env: &[(String, String)],
    timeout_secs: u64,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + 256);
    out.extend_from_slice(&CREATE_JOB_SELECTOR);
    let env_flat: Vec<String> = env.iter().map(|(k, v)| format!("{k}={v}")).collect();
    let encoded = abi_encode(&[
        Token::String(binary_ref.to_string()),
        Token::Array(input_refs.iter().cloned().map(Token::String).collect()),
        Token::Array(args.iter().cloned().map(Token::String).collect()),
        Token::Array(env_flat.into_iter().map(Token::String).collect()),
        Token::Uint(EthU256::from(timeout_secs)),
    ]);
    out.extend_from_slice(&encoded);
    out
}

async fn query_list_jobs(client: &impl QueryClient, params: ListJobsParams) -> Result<ListJobsReturn> {
    let params = RawBytes::serialize(params).context("failed to serialize ListJobs params")?;
    let msg = Message {
        version: Default::default(),
        from: system::SYSTEM_ACTOR_ADDR,
        to: BLOBS_ACTOR_ADDR,
        sequence: 0,
        value: TokenAmount::zero(),
        method_num: Method::ListJobs as u64,
        params,
        gas_limit: 10_000_000_000,
        gas_fee_cap: TokenAmount::zero(),
        gas_premium: TokenAmount::zero(),
    };
    let response = client
        .call(msg, FvmQueryHeight::default())
        .await
        .context("failed to execute ListJobs query")?;
    if response.value.code.is_err() {
        anyhow::bail!("ListJobs query failed: {}", response.value.info);
    }
    let return_data = fendermint_rpc::response::decode_data(&response.value.data)
        .context("failed to decode ListJobs response data")?;
    let jobs = fvm_ipld_encoding::from_slice::<ListJobsReturn>(&return_data)
        .context("failed to decode ListJobs return type")?;
    Ok(jobs)
}

async fn query_get_job(
    client: &impl QueryClient,
    id: u64,
) -> Result<Option<fendermint_actor_blobs_shared::execution::ExecutionJob>> {
    let params = RawBytes::serialize(GetJobParams { id }).context("failed to serialize GetJob params")?;
    let msg = Message {
        version: Default::default(),
        from: system::SYSTEM_ACTOR_ADDR,
        to: BLOBS_ACTOR_ADDR,
        sequence: 0,
        value: TokenAmount::zero(),
        method_num: Method::GetJob as u64,
        params,
        gas_limit: 10_000_000_000,
        gas_fee_cap: TokenAmount::zero(),
        gas_premium: TokenAmount::zero(),
    };
    let response = client
        .call(msg, FvmQueryHeight::default())
        .await
        .context("failed to execute GetJob query")?;
    if response.value.code.is_err() {
        anyhow::bail!("GetJob query failed: {}", response.value.info);
    }
    let return_data = fendermint_rpc::response::decode_data(&response.value.data)
        .context("failed to decode GetJob response data")?;
    let job =
        fvm_ipld_encoding::from_slice::<Option<fendermint_actor_blobs_shared::execution::ExecutionJob>>(
            &return_data,
        )
        .context("failed to decode GetJob return type")?;
    Ok(job)
}
