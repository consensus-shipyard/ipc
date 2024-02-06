use super::rpc::{gas_params, BroadcastResponse, TransClient};
use crate::cmd;
use crate::options::{
    proxy::{ProxyArgs, ProxyCommands},
    rpc::TransArgs,
};
use anyhow::anyhow;
use bytes::Buf;
use bytes::Bytes;
use cid::Cid;
use fendermint_rpc::client::FendermintClient;
use fendermint_rpc::message::GasParams;
use fendermint_rpc::tx::{CallClient, TxClient};
use fendermint_vm_message::query::FvmQueryHeight;
use futures_util::{Stream, StreamExt};
use fvm_shared::econ::TokenAmount;
use fvm_shared::BLOCK_GAS_LIMIT;
use num_traits::Zero;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::{convert::Infallible, net::SocketAddr};
use tendermint::block::Height;
use tendermint::Hash;
use tokio::sync::Mutex;
use warp::{http::StatusCode, Filter, Rejection, Reply};

const MAX_BODY_LENGTH: u64 = 1024 * 1024 * 1024;

cmd! {
    ProxyArgs(self) {
        let client = FendermintClient::new_http(self.url.clone(), self.proxy_url.clone())?;
        match self.command.clone() {
            ProxyCommands::Start { args } => {
                let seq = args.sequence;
                let nonce = Arc::new(Mutex::new(seq));

                let health_route = warp::path!("health")
                    .and(warp::get()).and_then(health);
                let add_route = warp::path!("v1" / "os" / String)
                    .and(warp::put())
                    .and(warp::body::content_length_limit(MAX_BODY_LENGTH))
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(with_nonce(nonce.clone()))
                    .and(warp::header::optional::<u64>("X-DataRepo-GasLimit"))
                    .and(warp::body::stream())
                    .and_then(handle_add);
                let delete_route = warp::path!("v1" / "os" / String)
                    .and(warp::delete())
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(with_nonce(nonce))
                    .and(warp::header::optional::<u64>("X-DataRepo-GasLimit"))
                    .and_then(handle_delete);
                let get_route = warp::path!("v1" / "os" / String)
                    .and(warp::get())
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(warp::query::<HeightQuery>())
                    .and_then(handle_get);
                let list_route = warp::path!("v1" / "os")
                    .and(warp::get())
                    .and(with_client(client))
                    .and(with_args(args))
                    .and(warp::query::<HeightQuery>())
                    .and_then(handle_list);

                let router = health_route
                    .or(add_route)
                    .or(delete_route)
                    .or(get_route)
                    .or(list_route)
                    .with(warp::cors().allow_any_origin()
                        .allow_headers(vec!["Content-Type"])
                        .allow_methods(vec!["PUT", "DEL", "GET"]))
                    .recover(handle_rejection);

                let saddr: SocketAddr = self.bind.parse().expect("Unable to parse server address");
                println!("Server started at {} with nonce {}", self.bind, seq);
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

fn with_nonce(
    nonce: Arc<Mutex<u64>>,
) -> impl Filter<Extract = (Arc<Mutex<u64>>,), Error = Infallible> + Clone {
    warp::any().map(move || nonce.clone())
}

#[derive(Serialize, Deserialize)]
struct HeightQuery {
    pub height: Option<u64>,
}

async fn health() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::reply())
}

async fn handle_add(
    key: String,
    client: FendermintClient,
    mut args: TransArgs,
    nonce: Arc<Mutex<u64>>,
    gas_limit: Option<u64>,
    mut body: impl Stream<Item = Result<impl Buf, warp::Error>> + Unpin + Send + Sync,
) -> Result<impl Reply, Rejection> {
    let mut nonce_lck = nonce.lock().await;
    args.sequence = *nonce_lck;
    args.gas_limit = gas_limit.unwrap_or_else(|| BLOCK_GAS_LIMIT);

    let mut res: Vec<Txn> = vec![];
    while let Some(buf) = body.next().await {
        let mut buf = buf.unwrap();
        while buf.remaining() > 0 {
            // Note(sander): chunk seems to max out at 504KiB, but we'll want to double-check against some limit
            let chunk = buf.chunk().to_owned();
            let chunk_len = chunk.len();
            let txn = if res.is_empty() {
                datarepo_put(
                    client.clone(),
                    args.clone(),
                    key.clone(),
                    Bytes::from(chunk),
                )
                .await
                .map_err(|e| {
                    Rejection::from(BadRequest {
                        message: format!("put error: {}", e),
                    })
                })?
            } else {
                datarepo_append(
                    client.clone(),
                    args.clone(),
                    key.clone(),
                    Bytes::from(chunk),
                )
                .await
                .map_err(|e| {
                    Rejection::from(BadRequest {
                        message: format!("append error: {}", e),
                    })
                })?
            };
            res.push(txn);

            buf.advance(chunk_len);
            *nonce_lck += 1;
            args.sequence = *nonce_lck;
        }
    }

    Ok(warp::reply::json(&res))
}

async fn handle_delete(
    key: String,
    client: FendermintClient,
    mut args: TransArgs,
    nonce: Arc<Mutex<u64>>,
    gas_limit: Option<u64>,
) -> Result<impl Reply, Rejection> {
    let mut nonce_lck = nonce.lock().await;
    args.sequence = *nonce_lck;
    args.gas_limit = gas_limit.unwrap_or_else(|| BLOCK_GAS_LIMIT);

    let res = datarepo_delete(client.clone(), args.clone(), key.clone())
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("delete error: {}", e),
            })
        })?;

    *nonce_lck += 1;
    Ok(warp::reply::json(&res))
}

async fn handle_get(
    key: String,
    client: FendermintClient,
    args: TransArgs,
    hq: HeightQuery,
) -> Result<impl Reply, Rejection> {
    let res = datarepo_get(client, args, key, hq.height.unwrap_or_else(|| 0))
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("get error: {}", e),
            })
        })?;

    match res {
        Some(obj) => Ok(obj),
        None => Err(Rejection::from(NotFound)),
    }
}

async fn handle_list(
    client: FendermintClient,
    args: TransArgs,
    hq: HeightQuery,
) -> Result<impl Reply, Rejection> {
    let res = datarepo_list(client, args, hq.height.unwrap_or_else(|| 0))
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("list error: {}", e),
            })
        })?;

    let list: Vec<String> = res
        .unwrap_or_default()
        .iter()
        .map(|v| core::str::from_utf8(v).unwrap_or_default().to_string())
        .collect();

    Ok(warp::reply::json(&list))
}

#[derive(Clone, Debug)]
struct BadRequest {
    message: String,
}

impl warp::reject::Reject for BadRequest {}

#[derive(Debug)]
struct NotFound;

impl warp::reject::Reject for NotFound {}

#[derive(Clone, Debug, Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let (code, message) = if err.is_not_found() || err.find::<NotFound>().is_some() {
        (StatusCode::NOT_FOUND, "Not Found".to_string())
    } else if let Some(e) = err.find::<BadRequest>() {
        let err = e.to_owned();
        (StatusCode::BAD_REQUEST, err.message)
    } else if err.find::<warp::reject::PayloadTooLarge>().is_some() {
        (
            StatusCode::PAYLOAD_TOO_LARGE,
            "Payload too large".to_string(),
        )
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", err))
    };

    let reply = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message,
    });
    let reply = warp::reply::with_header(reply, "Access-Control-Allow-Origin", "*");
    Ok(warp::reply::with_status(reply, code))
}

#[derive(Clone, Debug, Serialize)]
enum TxnStatus {
    Pending,
    Committed,
}

#[derive(Clone, Debug, Serialize)]
struct Txn {
    pub status: TxnStatus,
    pub hash: Hash,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<Height>,
    #[serde(skip_serializing_if = "i64::is_zero")]
    pub gas_used: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_root: Option<String>,
}

impl Txn {
    fn pending(hash: Hash) -> Self {
        Txn {
            status: TxnStatus::Pending,
            hash,
            height: None,
            gas_used: 0,
            repo_root: None,
        }
    }

    fn committed(hash: Hash, height: Height, gas_used: i64, repo_root: Cid) -> Self {
        Txn {
            status: TxnStatus::Committed,
            hash,
            height: Some(height),
            gas_used,
            repo_root: Some(repo_root.to_string()),
        }
    }
}

/// Create a client, make a call to Tendermint with a closure, then maybe extract some JSON
/// depending on the return value, finally return the result in JSON.
async fn broadcast<F>(client: FendermintClient, args: TransArgs, f: F) -> anyhow::Result<Txn>
where
    F: FnOnce(
        TransClient,
        TokenAmount,
        GasParams,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<BroadcastResponse<Cid>>> + Send>>,
{
    let client = TransClient::new(client, &args)?;
    let gas_params = gas_params(&args);
    let res = f(client, TokenAmount::default(), gas_params).await?;
    Ok(match res {
        BroadcastResponse::Async(res) => Txn::pending(res.response.hash),
        BroadcastResponse::Sync(res) => {
            if res.response.code.is_err() {
                return Err(anyhow!(res.response.log));
            }
            Txn::pending(res.response.hash)
        }
        BroadcastResponse::Commit(res) => {
            if res.response.check_tx.code.is_err() {
                return Err(anyhow!(res.response.check_tx.log));
            } else if res.response.deliver_tx.code.is_err() {
                return Err(anyhow!(res.response.deliver_tx.log));
            }
            Txn::committed(
                res.response.hash,
                res.response.height,
                res.response.deliver_tx.gas_used,
                res.return_data.unwrap_or_default(),
            )
        }
    })
}

async fn datarepo_put(
    client: FendermintClient,
    args: TransArgs,
    key: String,
    content: Bytes,
) -> anyhow::Result<Txn> {
    broadcast(client, args, |mut client, value, gas_params| {
        Box::pin(async move { client.datarepo_put(key, content, value, gas_params).await })
    })
    .await
}

async fn datarepo_append(
    client: FendermintClient,
    args: TransArgs,
    key: String,
    content: Bytes,
) -> anyhow::Result<Txn> {
    broadcast(client, args, |mut client, value, gas_params| {
        Box::pin(async move {
            client
                .datarepo_append(key, content, value, gas_params)
                .await
        })
    })
    .await
}

async fn datarepo_delete(
    client: FendermintClient,
    args: TransArgs,
    key: String,
) -> anyhow::Result<Txn> {
    broadcast(client, args, |mut client, value, gas_params| {
        Box::pin(async move { client.datarepo_delete(key, value, gas_params).await })
    })
    .await
}

async fn datarepo_get(
    client: FendermintClient,
    args: TransArgs,
    key: String,
    height: u64,
) -> anyhow::Result<Option<Vec<u8>>> {
    let mut client = TransClient::new(client, &args)?;
    let gas_params = gas_params(&args);
    let h = FvmQueryHeight::from(height);

    let res = client
        .inner
        .datarepo_get_call(key, TokenAmount::default(), gas_params, h)
        .await?;

    Ok(res.return_data)
}

async fn datarepo_list(
    client: FendermintClient,
    args: TransArgs,
    height: u64,
) -> anyhow::Result<Option<Vec<Vec<u8>>>> {
    let mut client = TransClient::new(client, &args)?;
    let gas_params = gas_params(&args);
    let h = FvmQueryHeight::from(height);

    let res = client
        .inner
        .datarepo_list_call(TokenAmount::default(), gas_params, h)
        .await?;

    Ok(res.return_data)
}
