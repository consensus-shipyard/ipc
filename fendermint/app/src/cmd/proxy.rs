// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::{
    convert::Infallible, future::Future, net::ToSocketAddrs, num::ParseIntError, pin::Pin,
    sync::Arc,
};

use anyhow::anyhow;
use async_tempfile::TempFile;
use bytes::{Buf, Bytes};
use cid::Cid;
use futures_util::{Stream, StreamExt};
use fvm_ipld_encoding::strict_bytes::ByteBuf;
use fvm_shared::{address::Address, econ::TokenAmount, ActorID, BLOCK_GAS_LIMIT};
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient, TryFromUri};
use num_traits::Zero;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tendermint::{block::Height, Hash};
use thiserror::Error;
use tokio::{
    io::{AsyncSeekExt, AsyncWriteExt},
    sync::Mutex,
};
use tokio_util::compat::TokioAsyncReadCompatExt;
use warp::{
    http::{HeaderMap, HeaderValue, StatusCode},
    Filter, Rejection, Reply,
};

use fendermint_actor_objectstore::{
    Object, ObjectDeleteParams, ObjectGetParams, ObjectKind, ObjectList, ObjectListItem,
    ObjectListParams, ObjectPutParams,
};
use fendermint_app_options::rpc::BroadcastMode;
use fendermint_app_settings::proxy::ProxySettings;
use fendermint_rpc::tx::BoundClient;
use fendermint_rpc::{
    client::FendermintClient,
    message::GasParams,
    tx::{CallClient, TxClient},
};
use fendermint_vm_actor_interface::adm::ListByOwnerReturn;
use fendermint_vm_message::query::FvmQueryHeight;

use crate::cmd;
use crate::options::{
    proxy::{ProxyArgs, ProxyCommands},
    rpc::TransArgs,
};

use super::rpc::{gas_params, BroadcastResponse, TransClient};

const MAX_OBJECT_LENGTH: u64 = 1024 * 1024 * 1024;
const MAX_INTERNAL_OBJECT_LENGTH: u64 = 1024;
const MAX_EVENT_LENGTH: u64 = 1024 * 500; // Limit to 500KiB for now

cmd! {
    ProxyArgs(self, settings: ProxySettings) {
        match self.command.clone() {
            ProxyCommands::Run { tendermint_url, ipfs_addr, args} => {
                let client = FendermintClient::new_http(tendermint_url, None)?;
                let ipfs = IpfsClient::from_multiaddr_str(&ipfs_addr)?;

                let seq = args.sequence;
                let nonce = Arc::new(Mutex::new(seq));

                // Admin routes
                let health_route = warp::path!("health")
                    .and(warp::get()).and_then(health);
                let list_machines_route = warp::path!("v1" / "machines")
                    .and(warp::get())
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(warp::query::<HeightQuery>())
                    .and_then(handle_list_machines);

                // Object Store routes
                let os_create_route = warp::path!("v1" / "os")
                    .and(warp::post())
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(with_nonce(nonce.clone()))
                    .and(warp::header::optional::<u64>("X-ADM-GasLimit"))
                    .and(warp::header::optional::<BroadcastMode>("X-ADM-BroadcastMode"))
                    .and_then(handle_os_create);
                let os_upload_route = warp::path!("v1" / "os" / Address / String)
                    .and(warp::put())
                    .and(warp::body::content_length_limit(MAX_OBJECT_LENGTH))
                    .and(with_client(client.clone()))
                    .and(with_ipfs(ipfs.clone()))
                    .and(with_args(args.clone()))
                    .and(with_nonce(nonce.clone()))
                    .and(warp::header::<u64>("Content-Length"))
                    .and(warp::header::optional::<u64>("X-ADM-GasLimit"))
                    .and(warp::header::optional::<BroadcastMode>("X-ADM-BroadcastMode"))
                    .and(warp::body::stream())
                    .and_then(handle_os_upload);
                let os_delete_route = warp::path!("v1" / "os" / Address / String)
                    .and(warp::delete())
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(with_nonce(nonce.clone()))
                    .and(warp::header::optional::<u64>("X-ADM-GasLimit"))
                    .and(warp::header::optional::<BroadcastMode>("X-ADM-BroadcastMode"))
                    .and_then(handle_os_delete);
                let os_get_route = warp::path!("v1" / "os" / Address / String)
                    .and(
                        warp::get().or(warp::head()).unify()
                    )
                    .and(with_client(client.clone()))
                    .and(with_ipfs(ipfs.clone()))
                    .and(with_args(args.clone()))
                    .and(warp::query::<HeightQuery>())
                    .and(warp::header::optional::<String>("Range"))
                    .and_then(handle_os_get);
                let os_list_route = warp::path!("v1" / "os" / Address)
                    .and(warp::get())
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(warp::query::<ListQuery>())
                    .and(warp::query::<HeightQuery>())
                    .and_then(handle_os_list);

                // Accumulator routes
                let acc_create_route = warp::path!("v1" / "acc")
                    .and(warp::post())
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(with_nonce(nonce.clone()))
                    .and(warp::header::optional::<u64>("X-ADM-GasLimit"))
                    .and(warp::header::optional::<BroadcastMode>("X-ADM-BroadcastMode"))
                    .and_then(handle_acc_create);
                let acc_push_route = warp::path!("v1" / "acc" / Address)
                    .and(warp::put())
                    .and(warp::body::content_length_limit(MAX_EVENT_LENGTH))
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(with_nonce(nonce))
                    .and(warp::header::optional::<u64>("X-ADM-GasLimit"))
                    .and(warp::header::optional::<BroadcastMode>("X-ADM-BroadcastMode"))
                    .and(warp::body::bytes())
                    .and_then(handle_acc_push);
                let acc_root_route = warp::path!("v1" / "acc" / Address)
                    .and(warp::get())
                    .and(with_client(client))
                    .and(with_args(args))
                    .and(warp::query::<HeightQuery>())
                    .and_then(handle_acc_root);

                let router = health_route
                    .or(list_machines_route)
                    .or(os_create_route)
                    .or(os_upload_route)
                    .or(os_delete_route)
                    .or(os_get_route)
                    .or(os_list_route)
                    .or(acc_create_route)
                    .or(acc_push_route)
                    .or(acc_root_route)
                    .with(warp::cors().allow_any_origin()
                        .allow_headers(vec!["Content-Type"])
                        .allow_methods(vec!["PUT", "DEL", "GET", "HEAD"]))
                    .recover(handle_rejection);

                if let Some(listen_addr) = settings.listen.to_socket_addrs()?.next() {
                    warp::serve(router).run(listen_addr).await;
                    Ok(())
                } else {
                    Err(anyhow!("failed to convert to any socket address"))
                }
            },
        }
    }
}

fn with_client(
    client: FendermintClient,
) -> impl Filter<Extract = (FendermintClient,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

fn with_ipfs(
    client: IpfsClient,
) -> impl Filter<Extract = (IpfsClient,), Error = Infallible> + Clone {
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

async fn handle_list_machines(
    client: FendermintClient,
    args: TransArgs,
    hq: HeightQuery,
) -> Result<impl Reply, Rejection> {
    let res = list_machines(client, args, None, hq.height.unwrap_or(0))
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("list error: {}", e),
            })
        })?;

    let list = res.unwrap_or_default();
    let machines = list
        .machines
        .iter()
        .map(|a| a.to_string())
        .collect::<Vec<String>>();

    let json = json!({"machines": machines});
    Ok(warp::reply::json(&json))
}

async fn handle_os_create(
    client: FendermintClient,
    mut args: TransArgs,
    nonce: Arc<Mutex<u64>>,
    gas_limit: Option<u64>,
    broadcast_mode: Option<BroadcastMode>,
) -> Result<impl Reply, Rejection> {
    let mut nonce_lck = nonce.lock().await;
    args.sequence = *nonce_lck;
    args.gas_limit = gas_limit.unwrap_or(BLOCK_GAS_LIMIT);
    args.broadcast_mode = broadcast_mode.unwrap_or(args.broadcast_mode);

    let res = os_create(client, args).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("create error: {}", e),
        })
    })?;

    *nonce_lck += 1;
    Ok(warp::reply::json(&res))
}

#[allow(clippy::too_many_arguments)]
async fn handle_os_upload(
    address: Address,
    key: String,
    client: FendermintClient,
    ipfs: IpfsClient,
    mut args: TransArgs,
    nonce: Arc<Mutex<u64>>,
    size: u64,
    gas_limit: Option<u64>,
    broadcast_mode: Option<BroadcastMode>,
    mut body: impl Stream<Item = Result<impl Buf, warp::Error>> + Unpin + Send + Sync,
) -> Result<impl Reply, Rejection> {
    let mut nonce_lck = nonce.lock().await;
    args.sequence = *nonce_lck;
    args.gas_limit = gas_limit.unwrap_or(BLOCK_GAS_LIMIT);
    args.broadcast_mode = broadcast_mode.unwrap_or(args.broadcast_mode);

    if size == 0 {
        return Err(Rejection::from(BadRequest {
            message: "empty body".into(),
        }));
    }

    let params = if size > MAX_INTERNAL_OBJECT_LENGTH {
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

        ObjectPutParams {
            key: key.into_bytes(),
            kind: ObjectKind::External(cid),
            overwrite: true,
        }
    } else {
        let mut collected: Vec<u8> = vec![];
        while let Some(buf) = body.next().await {
            let mut buf = buf.unwrap();
            while buf.remaining() > 0 {
                let chunk = buf.chunk();
                let chunk_len = chunk.len();
                collected.extend_from_slice(chunk);
                buf.advance(chunk_len);
            }
        }

        ObjectPutParams {
            key: key.into_bytes(),
            kind: ObjectKind::Internal(ByteBuf(collected)),
            overwrite: true,
        }
    };

    let res = os_put(client, args, address, params).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("put error: {}", e),
        })
    })?;

    *nonce_lck += 1;
    Ok(warp::reply::json(&res))
}

async fn handle_os_delete(
    address: Address,
    key: String,
    client: FendermintClient,
    mut args: TransArgs,
    nonce: Arc<Mutex<u64>>,
    gas_limit: Option<u64>,
    broadcast_mode: Option<BroadcastMode>,
) -> Result<impl Reply, Rejection> {
    let mut nonce_lck = nonce.lock().await;
    args.sequence = *nonce_lck;
    args.gas_limit = gas_limit.unwrap_or(BLOCK_GAS_LIMIT);
    args.broadcast_mode = broadcast_mode.unwrap_or(args.broadcast_mode);

    let params = ObjectDeleteParams {
        key: key.into_bytes(),
    };
    let res = os_delete(client, args, address, params)
        .await
        .map_err(|e| {
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
        .split('-')
        .map(|n| n.to_string())
        .collect();
    if range.len() != 2 {
        return Err(ProxyError::RangeHeaderInvalid);
    }
    let (start, end): (u64, u64) = match (!range[0].is_empty(), !range[1].is_empty()) {
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
        return Err(ProxyError::RangeHeaderInvalid);
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
    address: Address,
    key: String,
    client: FendermintClient,
    ipfs: IpfsClient,
    args: TransArgs,
    hq: HeightQuery,
    range: Option<String>,
) -> Result<impl Reply, Rejection> {
    let params = ObjectGetParams {
        key: key.into_bytes(),
    };
    let res = os_get(client, args, address, params, hq.height.unwrap_or(0))
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("get error: {}", e),
            })
        })?;

    match res {
        Some(obj) => {
            let (body, start, end, len, size) = match obj {
                Object::Internal(buf) => {
                    let size = buf.0.len() as u64;
                    match range {
                        Some(range) => {
                            let (start, end) = get_range_params(range, size).map_err(|e| {
                                Rejection::from(BadRequest {
                                    message: format!("failed to get range params: {}", e),
                                })
                            })?;
                            let len = end - start + 1;
                            (
                                warp::hyper::Body::from(
                                    buf.0[start as usize..=end as usize].to_vec(),
                                ),
                                start,
                                end,
                                len,
                                size,
                            )
                        }
                        None => (warp::hyper::Body::from(buf.0), 0, size - 1, size, size),
                    }
                }
                Object::External((buf, resolved)) => {
                    let cid = Cid::try_from(buf.0).map_err(|e| {
                        Rejection::from(BadRequest {
                            message: format!("failed to decode cid: {}", e),
                        })
                    })?;
                    let cid = cid.to_string();
                    if !resolved {
                        return Err(Rejection::from(BadRequest {
                            message: "object is not resolved".to_string(),
                        }));
                    }

                    let stat = ipfs
                        .files_stat(format!("/ipfs/{cid}").as_str())
                        .await
                        .map_err(|e| {
                            Rejection::from(BadRequest {
                                message: format!("failed to stat object: {}", e),
                            })
                        })?;
                    let size = stat.size;

                    match range {
                        Some(range) => {
                            let (start, end) = get_range_params(range, size).map_err(|e| {
                                Rejection::from(BadRequest {
                                    message: format!("failed to get range params: {}", e),
                                })
                            })?;
                            let len = end - start + 1;
                            (
                                warp::hyper::Body::wrap_stream(ipfs.cat_range(
                                    &cid,
                                    start as usize,
                                    len as usize,
                                )),
                                start,
                                end,
                                len,
                                size,
                            )
                        }
                        None => (
                            warp::hyper::Body::wrap_stream(ipfs.cat(&cid)),
                            0,
                            size - 1,
                            size,
                            size,
                        ),
                    }
                }
            };

            let mut response = warp::reply::Response::new(body);
            let mut header_map = HeaderMap::new();
            if len < size {
                *response.status_mut() = StatusCode::PARTIAL_CONTENT;
                header_map.insert(
                    "Content-Range",
                    HeaderValue::from_str(&format!("bytes {}-{}/{}", start, end, len)).unwrap(),
                );
            } else {
                header_map.insert("Accept-Ranges", HeaderValue::from_str("bytes").unwrap());
            }
            header_map.insert("Content-Length", HeaderValue::from(len));
            let headers = response.headers_mut();
            headers.extend(header_map);

            Ok(response)
        }
        None => Err(Rejection::from(NotFound)),
    }
}

#[derive(Serialize, Deserialize)]
struct ListQuery {
    pub prefix: Option<String>,
    pub delimiter: Option<String>,
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}

async fn handle_os_list(
    address: Address,
    client: FendermintClient,
    args: TransArgs,
    options: ListQuery,
    hq: HeightQuery,
) -> Result<impl Reply, Rejection> {
    let params = ObjectListParams {
        prefix: options.prefix.unwrap_or_default().into(),
        delimiter: options.delimiter.unwrap_or_default().into(),
        offset: options.offset.unwrap_or(0),
        limit: options.limit.unwrap_or(0),
    };
    let res = os_list(client, args, address, params, hq.height.unwrap_or(0))
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("list error: {}", e),
            })
        })?;

    let list = res.unwrap_or_default();
    let objects = list
        .objects
        .iter()
        .map(|v| {
            let key = core::str::from_utf8(&v.0).unwrap_or_default().to_string();
            match &v.1 {
                ObjectListItem::Internal((cid, size)) => {
                    json!({"key": key, "value": json!({"kind": "internal", "content": cid.to_string(), "size": size})})
                }
                ObjectListItem::External((cid, resolved)) => {
                    json!({"key": key, "value": json!({"kind": "external", "content": cid.to_string(), "resolved": resolved})})
                }
            }
        })
        .collect::<Vec<Value>>();
    let common_prefixes = list
        .common_prefixes
        .iter()
        .map(|v| Value::String(core::str::from_utf8(v).unwrap_or_default().to_string()))
        .collect::<Vec<Value>>();

    let json = json!({"objects": objects, "common_prefixes": common_prefixes});
    Ok(warp::reply::json(&json))
}

async fn handle_acc_create(
    client: FendermintClient,
    mut args: TransArgs,
    nonce: Arc<Mutex<u64>>,
    gas_limit: Option<u64>,
    broadcast_mode: Option<BroadcastMode>,
) -> Result<impl Reply, Rejection> {
    let mut nonce_lck = nonce.lock().await;
    args.sequence = *nonce_lck;
    args.gas_limit = gas_limit.unwrap_or(BLOCK_GAS_LIMIT);
    args.broadcast_mode = broadcast_mode.unwrap_or(args.broadcast_mode);

    let res = acc_create(client, args).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("create error: {}", e),
        })
    })?;

    *nonce_lck += 1;
    Ok(warp::reply::json(&res))
}

async fn handle_acc_push(
    address: Address,
    client: FendermintClient,
    mut args: TransArgs,
    nonce: Arc<Mutex<u64>>,
    gas_limit: Option<u64>,
    broadcast_mode: Option<BroadcastMode>,
    body: Bytes,
) -> Result<impl Reply, Rejection> {
    let mut nonce_lck = nonce.lock().await;
    args.sequence = *nonce_lck;
    args.gas_limit = gas_limit.unwrap_or(BLOCK_GAS_LIMIT);
    args.broadcast_mode = broadcast_mode.unwrap_or(args.broadcast_mode);

    let res = acc_push(client.clone(), args.clone(), address, body)
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
    address: Address,
    client: FendermintClient,
    args: TransArgs,
    hq: HeightQuery,
) -> Result<impl Reply, Rejection> {
    let res = acc_root(client, args, address, hq.height.unwrap_or(0))
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
struct Txn<D> {
    pub status: TxnStatus,
    pub hash: Hash,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<Height>,
    #[serde(skip_serializing_if = "i64::is_zero")]
    pub gas_used: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<D>,
}

impl<D> Txn<D> {
    fn pending(hash: Hash) -> Self {
        Txn {
            status: TxnStatus::Pending,
            hash,
            height: None,
            gas_used: 0,
            data: None,
        }
    }

    fn committed(hash: Hash, height: Height, gas_used: i64, data: Option<D>) -> Self {
        Txn {
            status: TxnStatus::Committed,
            hash,
            height: Some(height),
            gas_used,
            data,
        }
    }
}

async fn broadcast<F, D>(client: FendermintClient, args: TransArgs, f: F) -> anyhow::Result<Txn<D>>
where
    F: FnOnce(
        TransClient,
        TokenAmount,
        GasParams,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<BroadcastResponse<D>>> + Send>>,
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
                res.return_data,
            )
        }
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReturnPretty {
    pub actor_id: ActorID,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub robust_address: Option<String>,
}

async fn os_create(
    client: FendermintClient,
    args: TransArgs,
) -> anyhow::Result<Txn<CreateReturnPretty>> {
    let tx = broadcast(client, args, |mut client, value, gas_params| {
        Box::pin(async move { client.os_create(value, gas_params).await })
    })
    .await?;
    let data = tx.data.map(|data| CreateReturnPretty {
        actor_id: data.actor_id,
        robust_address: data.robust_address.map(|a| a.to_string()),
    });

    let tx_pretty: Txn<CreateReturnPretty> = Txn {
        status: tx.status,
        hash: tx.hash,
        height: tx.height,
        gas_used: tx.gas_used,
        data,
    };
    Ok(tx_pretty)
}

async fn list_machines(
    client: FendermintClient,
    args: TransArgs,
    owner: Option<Address>,
    height: u64,
) -> anyhow::Result<Option<ListByOwnerReturn>> {
    let mut client = TransClient::new(client, &args)?;
    let gas_params = gas_params(&args);
    let h = FvmQueryHeight::from(height);
    let owner = owner.unwrap_or(client.address());

    let res = client
        .inner
        .list_machines_call(owner, TokenAmount::default(), gas_params, h)
        .await?;

    Ok(res.return_data)
}

async fn os_put(
    client: FendermintClient,
    args: TransArgs,
    address: Address,
    params: ObjectPutParams,
) -> anyhow::Result<Txn<String>> {
    broadcast(client, args, |mut client, value, gas_params| {
        Box::pin(async move { client.os_put(address, params, value, gas_params).await })
    })
    .await
}

async fn os_delete(
    client: FendermintClient,
    args: TransArgs,
    address: Address,
    params: ObjectDeleteParams,
) -> anyhow::Result<Txn<String>> {
    broadcast(client, args, |mut client, value, gas_params| {
        Box::pin(async move { client.os_delete(address, params, value, gas_params).await })
    })
    .await
}

async fn os_get(
    client: FendermintClient,
    args: TransArgs,
    address: Address,
    params: ObjectGetParams,
    height: u64,
) -> anyhow::Result<Option<Object>> {
    let mut client = TransClient::new(client, &args)?;
    let gas_params = gas_params(&args);
    let h = FvmQueryHeight::from(height);

    let res = client
        .inner
        .os_get_call(address, params, TokenAmount::default(), gas_params, h)
        .await?;

    Ok(res.return_data)
}

async fn os_list(
    client: FendermintClient,
    args: TransArgs,
    address: Address,
    params: ObjectListParams,
    height: u64,
) -> anyhow::Result<Option<ObjectList>> {
    let mut client = TransClient::new(client, &args)?;
    let gas_params = gas_params(&args);
    let h = FvmQueryHeight::from(height);

    let res = client
        .inner
        .os_list_call(address, params, TokenAmount::default(), gas_params, h)
        .await?;

    Ok(res.return_data)
}

async fn acc_create(
    client: FendermintClient,
    args: TransArgs,
) -> anyhow::Result<Txn<CreateReturnPretty>> {
    let tx = broadcast(client, args, |mut client, value, gas_params| {
        Box::pin(async move { client.acc_create(value, gas_params).await })
    })
    .await?;
    let data = tx.data.map(|data| CreateReturnPretty {
        actor_id: data.actor_id,
        robust_address: data.robust_address.map(|a| a.to_string()),
    });

    let tx_pretty: Txn<CreateReturnPretty> = Txn {
        status: tx.status,
        hash: tx.hash,
        height: tx.height,
        gas_used: tx.gas_used,
        data,
    };
    Ok(tx_pretty)
}

async fn acc_push(
    client: FendermintClient,
    args: TransArgs,
    address: Address,
    event: Bytes,
) -> anyhow::Result<Txn<String>> {
    broadcast(client, args, |mut client, value, gas_params| {
        Box::pin(async move { client.acc_push(address, event, value, gas_params).await })
    })
    .await
}

async fn acc_root(
    client: FendermintClient,
    args: TransArgs,
    address: Address,
    height: u64,
) -> anyhow::Result<Option<String>> {
    let mut client = TransClient::new(client, &args)?;
    let gas_params = gas_params(&args);
    let h = FvmQueryHeight::from(height);

    let res = client
        .inner
        .acc_root_call(address, TokenAmount::default(), gas_params, h)
        .await?;

    Ok(res.return_data)
}
