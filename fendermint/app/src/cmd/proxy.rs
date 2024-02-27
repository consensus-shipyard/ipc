use std::{convert::Infallible, net::SocketAddr};
use std::future::Future;
use std::num::ParseIntError;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::anyhow;
use async_tempfile::TempFile;
use bytes::{Buf, Bytes};
use cid::Cid;
use futures_util::{Stream, StreamExt};
use fvm_shared::BLOCK_GAS_LIMIT;
use fvm_shared::econ::TokenAmount;
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use num_traits::Zero;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tendermint::block::Height;
use tendermint::Hash;
use thiserror::Error;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Mutex;
use tokio_util::compat::TokioAsyncReadCompatExt;
use warp::{Filter, http::StatusCode, Rejection, Reply};
use warp::http::{HeaderMap, HeaderValue};

use fendermint_actor_objectstore::Object;
use fendermint_rpc::client::FendermintClient;
use fendermint_rpc::message::GasParams;
use fendermint_rpc::tx::{CallClient, TxClient};
use fendermint_vm_message::query::FvmQueryHeight;

use crate::cmd;
use crate::cmd::proxy::ProxyError::RangeHeaderInvalid;
use crate::options::{
    proxy::{ProxyArgs, ProxyCommands},
    rpc::TransArgs,
};

use super::rpc::{BroadcastResponse, gas_params, TransClient};

const MAX_EVENT_LENGTH: u64 = 1024 * 500; // Limit to 500KiB for now

cmd! {
    ProxyArgs(self) {
        let client = FendermintClient::new_http(self.url.clone(), self.proxy_url.clone())?;
        match self.command.clone() {
            ProxyCommands::Start { args } => {
                let seq = args.sequence;
                let nonce = Arc::new(Mutex::new(seq));

                // Admin routes
                let health_route = warp::path!("health")
                    .and(warp::get()).and_then(health);

                // Object Store routes
                let upload_route = warp::path!("v1" / "os" / String)
                    .and(warp::put())
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(with_nonce(nonce.clone()))
                    .and(warp::header::optional::<u64>("X-DataRepo-GasLimit"))
                    .and(warp::body::stream())
                    .and_then(handle_os_upload);
                let add_route = warp::path!("v1" / "os" / String / Cid)
                    .and(warp::put())
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(with_nonce(nonce.clone()))
                    .and(warp::header::optional::<u64>("X-DataRepo-GasLimit"))
                    .and_then(handle_os_add);
                let delete_route = warp::path!("v1" / "os" / String)
                    .and(warp::delete())
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(with_nonce(nonce.clone()))
                    .and(warp::header::optional::<u64>("X-DataRepo-GasLimit"))
                    .and_then(handle_os_delete);
                let get_route = warp::path!("v1" / "os" / String)
                    .and(warp::get())
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(warp::query::<HeightQuery>())
                    .and(warp::header::optional::<String>("Range"))
                    .and_then(handle_os_get);
                let list_route = warp::path!("v1" / "os")
                    .and(warp::get())
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(warp::query::<HeightQuery>())
                    .and_then(handle_os_list);

                // Accumulator routes
                let push_route = warp::path!("v1" / "acc")
                    .and(warp::put())
                    .and(warp::body::content_length_limit(MAX_EVENT_LENGTH))
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(with_nonce(nonce))
                    .and(warp::header::optional::<u64>("X-DataRepo-GasLimit"))
                    .and(warp::body::bytes())
                    .and_then(handle_acc_push);
                let root_route = warp::path!("v1" / "acc")
                    .and(warp::get())
                    .and(with_client(client))
                    .and(with_args(args))
                    .and(warp::query::<HeightQuery>())
                    .and_then(handle_acc_root);

                let router = health_route
                    .or(upload_route)
                    .or(add_route)
                    .or(delete_route)
                    .or(get_route)
                    .or(list_route)
                    .or(push_route)
                    .or(root_route)
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

async fn handle_os_upload(
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

    let mut tmp = TempFile::new().await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to create tmp file: {}", e),
        })
    })?;
    while let Some(buf) = body.next().await {
        let mut buf = buf.unwrap();
        while buf.remaining() > 0 {
            let chunk = buf.chunk().to_owned();
            let chunk_len = chunk.len();
            tmp.write_all(&chunk).await.map_err(|e| {
                Rejection::from(BadRequest {
                    message: format!("failed to write chunk: {}", e),
                })
            })?;
            tmp.flush().await.map_err(|e| {
                Rejection::from(BadRequest {
                    message: format!("failed to flush chunk: {}", e),
                })
            })?;
            buf.advance(chunk_len);
        }
    }
    tmp.rewind().await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to rewind: {}", e),
        })
    })?;

    let ipfs = IpfsClient::default();
    let add = ipfs_api_backend_hyper::request::Add {
        chunker: Some("size-1048576"),
        pin: Some(false),
        raw_leaves: Some(true),
        cid_version: Some(1),
        hash: Some("blake2b-256"),
        ..Default::default()
    };

    let res = ipfs
        .add_async_with_options(tmp.compat(), add)
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("failed to add file: {}", e),
            })
        })?;
    let cid = Cid::try_from(res.hash).map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to parse cid: {}", e),
        })
    })?;

    let res = os_put(client, args, key, cid).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("put error: {}", e),
        })
    })?;

    *nonce_lck += 1;
    Ok(warp::reply::json(&res))
}

async fn handle_os_add(
    key: String,
    content: Cid,
    client: FendermintClient,
    mut args: TransArgs,
    nonce: Arc<Mutex<u64>>,
    gas_limit: Option<u64>,
) -> Result<impl Reply, Rejection> {
    let mut nonce_lck = nonce.lock().await;
    args.sequence = *nonce_lck;
    args.gas_limit = gas_limit.unwrap_or_else(|| BLOCK_GAS_LIMIT);

    let res = os_put(client, args, key, content).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("put error: {}", e),
        })
    })?;

    *nonce_lck += 1;
    Ok(warp::reply::json(&res))
}

async fn handle_os_delete(
    key: String,
    client: FendermintClient,
    mut args: TransArgs,
    nonce: Arc<Mutex<u64>>,
    gas_limit: Option<u64>,
) -> Result<impl Reply, Rejection> {
    let mut nonce_lck = nonce.lock().await;
    args.sequence = *nonce_lck;
    args.gas_limit = gas_limit.unwrap_or_else(|| BLOCK_GAS_LIMIT);

    let res = os_delete(client, args, key).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("delete error: {}", e),
        })
    })?;

    *nonce_lck += 1;
    Ok(warp::reply::json(&res))
}

fn get_range_params(range: String, size: u64) -> Result<(u64, u64), ProxyError> {
    let range: Vec<String> = range
        .replace("bytes=", "")
        .split("-")
        .map(|n| n.to_string())
        .collect();
    if range.len() != 2 {
        return Err(RangeHeaderInvalid);
    }
    let (start, end): (u64, u64) = match (range[0].len() > 0, range[1].len() > 0) {
        (true, true) => (range[0].parse::<u64>()?, range[1].parse::<u64>()?),
        (true, false) => (range[0].parse::<u64>()?, size - 1),
        (false, true) => {
            let last = range[1].parse::<u64>()?;
            if last > size {
                (0, size - 1)
            } else {
                (size - last, size - 1)
            }
        }
        (false, false) => (0, size - 1),
    };
    if start > end || end >= size {
        return Err(RangeHeaderInvalid);
    }
    Ok((start, end))
}

#[derive(Debug, Error)]
enum ProxyError {
    #[error("error parsing range header: `{0}`")]
    RangeHeaderParseError(ParseIntError),
    #[error("invalid range header")]
    RangeHeaderInvalid,
}

impl From<ParseIntError> for ProxyError {
    fn from(err: ParseIntError) -> Self {
        ProxyError::RangeHeaderParseError(err)
    }
}

async fn handle_os_get(
    key: String,
    client: FendermintClient,
    args: TransArgs,
    hq: HeightQuery,
    range: Option<String>,
) -> Result<impl Reply, Rejection> {
    let res = os_get(client, args, key, hq.height.unwrap_or_else(|| 0))
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("get error: {}", e),
            })
        })?;

    match res {
        Some(obj) => {
            let value = Cid::try_from(obj.value).map_err(|e| {
                Rejection::from(BadRequest {
                    message: format!("failed to decode value: {}", e),
                })
            })?;
            if !obj.resolved {
                return Err(Rejection::from(BadRequest {
                    message: "object is not resolved".to_string(),
                }));
            }
            let val = value.to_string();

            let ipfs = IpfsClient::default();
            let stat = ipfs.object_stat(&val).await.map_err(|e| {
                Rejection::from(BadRequest {
                    message: format!("failed to stat object: {}", e),
                })
            })?;

            let (stream, start, end, len) = match range {
                Some(range) => {
                    let (start, end) = get_range_params(range, stat.data_size).map_err(|e| {
                        Rejection::from(BadRequest {
                            message: format!("failed to get range params: {}", e),
                        })
                    })?;
                    let len = end - start + 1;
                    (
                        ipfs.cat_range(&val, start as usize, len as usize),
                        start,
                        end,
                        len,
                    )
                }
                None => (ipfs.cat(&val), 0, stat.data_size - 1, stat.data_size),
            };
            let body = warp::hyper::Body::wrap_stream(stream);
            let mut response = warp::reply::Response::new(body);

            if len < stat.data_size {
                *response.status_mut() = StatusCode::PARTIAL_CONTENT;
            }

            let mut header_map = HeaderMap::new();
            header_map.insert("Accept-Ranges", HeaderValue::from_str("bytes").unwrap());
            header_map.insert(
                "Content-Range",
                HeaderValue::from_str(&format!("bytes {}-{}/{}", start, end, len)).unwrap(),
            );
            header_map.insert("Content-Length", HeaderValue::from(len));

            let headers = response.headers_mut();
            headers.extend(header_map);

            Ok(response)
        }
        None => Err(Rejection::from(NotFound)),
    }
}

async fn handle_os_list(
    client: FendermintClient,
    args: TransArgs,
    hq: HeightQuery,
) -> Result<impl Reply, Rejection> {
    let res = os_list(client, args, hq.height.unwrap_or_else(|| 0))
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("list error: {}", e),
            })
        })?;

    let list = res
        .unwrap_or_default()
        .iter()
        .map(|v| -> Result<Value, Rejection> {
            let key = core::str::from_utf8(&v.0).unwrap_or_default().to_string();
            let value = Cid::try_from(v.1.value.clone()).map_err(|e| {
                Rejection::from(BadRequest {
                    message: format!("failed to decode value: {}", e),
                })
            })?;
            Ok(json!({"key": key, "value": value.to_string(), "resolved": v.1.resolved}))
        })
        .collect::<Result<Vec<Value>, Rejection>>()?;

    Ok(warp::reply::json(&list))
}

async fn handle_acc_push(
    client: FendermintClient,
    mut args: TransArgs,
    nonce: Arc<Mutex<u64>>,
    gas_limit: Option<u64>,
    body: Bytes,
) -> Result<impl Reply, Rejection> {
    let mut nonce_lck = nonce.lock().await;
    args.sequence = *nonce_lck;
    args.gas_limit = gas_limit.unwrap_or_else(|| BLOCK_GAS_LIMIT);

    let res = acc_push(client.clone(), args.clone(), body)
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("push error: {}", e),
            })
        })?;

    *nonce_lck += 1;
    Ok(warp::reply::json(&res))
}

async fn handle_acc_root(
    client: FendermintClient,
    args: TransArgs,
    hq: HeightQuery,
) -> Result<impl Reply, Rejection> {
    let res = acc_root(client, args, hq.height.unwrap_or_else(|| 0))
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("root error: {}", e),
            })
        })?;

    let json = json!({"root": res.unwrap_or_default().to_string()});
    Ok(warp::reply::json(&json))
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
    pub state: Option<String>,
}

impl Txn {
    fn pending(hash: Hash) -> Self {
        Txn {
            status: TxnStatus::Pending,
            hash,
            height: None,
            gas_used: 0,
            state: None,
        }
    }

    fn committed(hash: Hash, height: Height, gas_used: i64, state: Cid) -> Self {
        Txn {
            status: TxnStatus::Committed,
            hash,
            height: Some(height),
            gas_used,
            state: Some(state.to_string()),
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

async fn os_put(
    client: FendermintClient,
    args: TransArgs,
    key: String,
    content: Cid,
) -> anyhow::Result<Txn> {
    broadcast(client, args, |mut client, value, gas_params| {
        Box::pin(async move { client.os_put(key, content, value, gas_params).await })
    })
    .await
}

async fn os_delete(client: FendermintClient, args: TransArgs, key: String) -> anyhow::Result<Txn> {
    broadcast(client, args, |mut client, value, gas_params| {
        Box::pin(async move { client.os_delete(key, value, gas_params).await })
    })
    .await
}

async fn os_get(
    client: FendermintClient,
    args: TransArgs,
    key: String,
    height: u64,
) -> anyhow::Result<Option<Object>> {
    let mut client = TransClient::new(client, &args)?;
    let gas_params = gas_params(&args);
    let h = FvmQueryHeight::from(height);

    let res = client
        .inner
        .os_get_call(key, TokenAmount::default(), gas_params, h)
        .await?;

    Ok(res.return_data)
}

async fn os_list(
    client: FendermintClient,
    args: TransArgs,
    height: u64,
) -> anyhow::Result<Option<Vec<(Vec<u8>, Object)>>> {
    let mut client = TransClient::new(client, &args)?;
    let gas_params = gas_params(&args);
    let h = FvmQueryHeight::from(height);

    let res = client
        .inner
        .os_list_call(TokenAmount::default(), gas_params, h)
        .await?;

    Ok(res.return_data)
}

async fn acc_push(client: FendermintClient, args: TransArgs, event: Bytes) -> anyhow::Result<Txn> {
    broadcast(client, args, |mut client, value, gas_params| {
        Box::pin(async move { client.acc_push(event, value, gas_params).await })
    })
    .await
}

async fn acc_root(
    client: FendermintClient,
    args: TransArgs,
    height: u64,
) -> anyhow::Result<Option<Cid>> {
    let mut client = TransClient::new(client, &args)?;
    let gas_params = gas_params(&args);
    let h = FvmQueryHeight::from(height);

    let res = client
        .inner
        .acc_root_call(TokenAmount::default(), gas_params, h)
        .await?;

    Ok(res.return_data)
}
