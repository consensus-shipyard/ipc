use crate::cmd;
use crate::cmd::key::read_secret_key;
use crate::options::proxy::{BroadcastMode, ProxyArgs, ProxyCommands, TransArgs};
use async_trait::async_trait;
use bytes::Bytes;
use fendermint_rpc::client::{BoundFendermintClient, FendermintClient};
use fendermint_rpc::message::{GasParams, MessageFactory};
use fendermint_rpc::tx::{
    AsyncResponse, BoundClient, CallClient, CommitResponse, SyncResponse, TxAsync, TxClient,
    TxCommit, TxSync,
};
use fendermint_vm_actor_interface::tableland::ExecuteReturn;
use fendermint_vm_core::chainid;
use fendermint_vm_message::chain::ChainMessage;
use fvm_shared::econ::TokenAmount;
use fvm_shared::BLOCK_GAS_LIMIT;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::future::Future;
use std::pin::Pin;
use std::{convert::Infallible, net::SocketAddr};
use tendermint::abci::response::DeliverTx;
use tendermint::block::Height;
use tendermint_rpc::HttpClient;
use warp::{http::StatusCode, Filter, Rejection, Reply};

const MAX_BODY_LENGTH: u64 = 100 * 1024 * 1024;

cmd! {
    ProxyArgs(self) {
        let client = FendermintClient::new_http(self.url.clone(), self.proxy_url.clone())?;
        match self.command.clone() {
            ProxyCommands::Start { args } => {
                let health_route = warp::path!("health")
                    .and(warp::get()).and_then(health);
                let execute_route = warp::path!("v1" / "execute")
                    .and(warp::post())
                    .and(warp::body::content_length_limit(MAX_BODY_LENGTH))
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(warp::body::bytes())
                    .and_then(execute);
                let query_route = warp::path!("v1" / "query")
                    .and(warp::post())
                    .and(warp::body::content_length_limit(MAX_BODY_LENGTH))
                    .and(with_client(client))
                    .and(with_args(args))
                    .and(warp::body::bytes())
                    .and_then(query);

                let router = health_route
                    .or(execute_route)
                    .or(query_route)
                    .with(warp::cors().allow_any_origin()
                        .allow_headers(vec!["Content-Type"])
                        .allow_methods(vec!["POST", "GET"]))
                    .recover(handle_rejection);

                let saddr: SocketAddr = self.bind.parse().expect("Unable to parse server address");
                println!("Server started at {}", self.bind);
                Ok(warp::serve(router).run(saddr).await)
            },
        }
    }
}

fn with_client(
    client: FendermintClient,
) -> impl Filter<Extract = (FendermintClient,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

fn with_args(args: TransArgs) -> impl Filter<Extract = (TransArgs,), Error = Infallible> + Clone {
    warp::any().map(move || args.clone())
}

pub async fn health() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::reply())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteRequest {
    pub stmts: String,
    pub sequence: u64,
    #[serde(default = "block_gas_limit")]
    pub gas_limit: u64,
}

pub async fn execute(
    client: FendermintClient,
    args: TransArgs,
    body: Bytes,
) -> Result<impl Reply, Rejection> {
    let mut req = serde_json::from_slice::<ExecuteRequest>(&body).map_err(|e| {
        warp::reject::custom(ErrorMessage::new(
            StatusCode::BAD_REQUEST.as_u16(),
            format!("execute error: {}", e),
        ))
    })?;
    let stmts = req
        .stmts
        .trim_end_matches(";")
        .split(";")
        .map(|p| p.to_string())
        .collect::<Vec<String>>();
    if req.gas_limit == 0 {
        req.gas_limit = BLOCK_GAS_LIMIT
    }

    let res = tableland_execute(client, args, req.sequence, req.gas_limit, stmts)
        .await
        .map_err(|e| {
            warp::reject::custom(ErrorMessage::new(
                StatusCode::BAD_REQUEST.as_u16(),
                format!("execute error: {}", e),
            ))
        })?;

    Ok(warp::reply::json(&res))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryRequest {
    pub stmt: String,
    #[serde(default)]
    pub height: u64,
    #[serde(default = "block_gas_limit")]
    pub gas_limit: u64,
}

pub async fn query(
    client: FendermintClient,
    args: TransArgs,
    body: Bytes,
) -> Result<impl Reply, Rejection> {
    let mut req = serde_json::from_slice::<QueryRequest>(&body).map_err(|e| {
        warp::reject::custom(ErrorMessage::new(
            StatusCode::BAD_REQUEST.as_u16(),
            format!("query error: {}", e),
        ))
    })?;
    if req.gas_limit == 0 {
        req.gas_limit = BLOCK_GAS_LIMIT
    }

    let res = tableland_query(client, args, req.height, req.gas_limit, req.stmt)
        .await
        .map_err(|e| {
            warp::reject::custom(ErrorMessage::new(
                StatusCode::BAD_REQUEST.as_u16(),
                format!("query error: {}", e),
            ))
        })?;

    Ok(warp::reply::json(&res))
}

#[derive(Clone, Debug, Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

impl warp::reject::Reject for ErrorMessage {}

impl ErrorMessage {
    fn new(code: u16, message: String) -> Self {
        ErrorMessage { code, message }
    }
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (
            StatusCode::PAYLOAD_TOO_LARGE,
            "Payload too large".to_string(),
        )
    } else {
        let ferr = format!("{:?}", err);
        let status = if ferr.contains("code:") {
            StatusCode::BAD_REQUEST
        } else {
            eprintln!("unhandled error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        };
        (status, ferr)
    };

    let reply = warp::reply::json(&ErrorMessage::new(code.as_u16(), message));
    let reply = warp::reply::with_header(reply, "Access-Control-Allow-Origin", "*");
    Ok(warp::reply::with_status(reply, code))
}

/// Create a client, make a call to Tendermint with a closure, then maybe extract some JSON
/// depending on the return value, finally return the result in JSON.
async fn broadcast<F, T, G>(
    client: FendermintClient,
    args: TransArgs,
    sequence: u64,
    gas_limit: u64,
    f: F,
    g: G,
) -> anyhow::Result<Value>
where
    F: FnOnce(
        TransClient,
        TokenAmount,
        GasParams,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<BroadcastResponse<T>>> + Send>>,
    G: FnOnce(T) -> Value,
    T: Sync + Send,
{
    let client = TransClient::new(client, &args, sequence)?;
    let gas_params = gas_params(&args, gas_limit);
    let res = f(client, TokenAmount::default(), gas_params).await?;
    Ok(match res {
        BroadcastResponse::Async(res) => json!({"response": res.response}),
        BroadcastResponse::Sync(res) => json!({"response": res.response}),
        BroadcastResponse::Commit(res) => {
            let return_data = res.return_data.map(g).unwrap_or(Value::Null);
            json!({"response": res.response, "return_data": return_data})
        }
    })
}

async fn tableland_execute(
    client: FendermintClient,
    args: TransArgs,
    sequence: u64,
    gas_limit: u64,
    stmts: Vec<String>,
) -> anyhow::Result<Value> {
    broadcast(
        client,
        args,
        sequence,
        gas_limit,
        |mut client, value, gas_params| {
            Box::pin(async move { client.tableland_execute(stmts, value, gas_params).await })
        },
        |ret: ExecuteReturn| json!(ret),
    )
    .await
}

async fn tableland_query(
    client: FendermintClient,
    args: TransArgs,
    height: u64,
    gas_limit: u64,
    stmt: String,
) -> anyhow::Result<Value> {
    let mut client = TransClient::new(client, &args, 0)?;
    let gas_params = gas_params(&args, gas_limit);
    let mut h = None;
    if height > 0 {
        h = Some(Height::try_from(height)?)
    }

    let res = client
        .inner
        .tableland_query_call(stmt, TokenAmount::default(), gas_params, h)
        .await?;

    Ok(json!(res.return_data))
}

pub enum BroadcastResponse<T> {
    Async(AsyncResponse<T>),
    Sync(SyncResponse<T>),
    Commit(CommitResponse<T>),
}

impl fendermint_rpc::tx::BroadcastMode for BroadcastMode {
    type Response<T> = BroadcastResponse<T>;
}

struct TransClient {
    inner: BoundFendermintClient<HttpClient>,
    broadcast_mode: BroadcastMode,
}

impl TransClient {
    pub fn new(client: FendermintClient, args: &TransArgs, sequence: u64) -> anyhow::Result<Self> {
        let sk = read_secret_key(&args.secret_key)?;
        let chain_id = chainid::from_str_hashed(&args.chain_name)?;
        let mf = MessageFactory::new(sk, sequence, chain_id)?;
        let client = client.bind(mf);
        let client = Self {
            inner: client,
            broadcast_mode: args.broadcast_mode,
        };
        Ok(client)
    }
}

impl BoundClient for TransClient {
    fn message_factory_mut(&mut self) -> &mut MessageFactory {
        self.inner.message_factory_mut()
    }
}

#[async_trait]
impl TxClient<BroadcastMode> for TransClient {
    async fn perform<F, T>(&self, msg: ChainMessage, f: F) -> anyhow::Result<BroadcastResponse<T>>
    where
        F: FnOnce(&DeliverTx) -> anyhow::Result<T> + Sync + Send,
        T: Sync + Send,
    {
        match self.broadcast_mode {
            BroadcastMode::Async => {
                let res = TxClient::<TxAsync>::perform(&self.inner, msg, f).await?;
                Ok(BroadcastResponse::Async(res))
            }
            BroadcastMode::Sync => {
                let res = TxClient::<TxSync>::perform(&self.inner, msg, f).await?;
                Ok(BroadcastResponse::Sync(res))
            }
            BroadcastMode::Commit => {
                let res = TxClient::<TxCommit>::perform(&self.inner, msg, f).await?;
                Ok(BroadcastResponse::Commit(res))
            }
        }
    }
}

fn gas_params(args: &TransArgs, gas_limit: u64) -> GasParams {
    GasParams {
        gas_limit,
        gas_fee_cap: args.gas_fee_cap.clone(),
        gas_premium: args.gas_premium.clone(),
    }
}

fn block_gas_limit() -> u64 {
    BLOCK_GAS_LIMIT
}
