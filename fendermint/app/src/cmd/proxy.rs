// Copyright 2024 Textile
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::str::FromStr;
use std::{
    convert::Infallible, future::Future, net::ToSocketAddrs, num::ParseIntError, pin::Pin,
    sync::Arc,
};

use anyhow::anyhow;
use async_tempfile::TempFile;
use base64::{engine::general_purpose, Engine};
use bytes::{Buf, Bytes};
use cid::multihash::{Blake2bHasher, Hasher};
use cid::Cid;
use ethers::core::types::{self as et};
use fendermint_actor_machine::{Metadata, WriteAccess};
use fendermint_actor_objectstore::{Object, ObjectKind, ObjectList, ObjectListItem};
use fendermint_rpc::QueryClient;
use fendermint_vm_message::conv::from_fvm::to_eth_tokens;
use fendermint_vm_message::signed::SignedMessage;
use futures_util::{Stream, StreamExt};
use fvm_ipld_encoding::strict_bytes::ByteBuf;
use fvm_shared::{address::Address, econ::TokenAmount, ActorID, BLOCK_GAS_LIMIT};
use ipfs_api_backend_hyper::request::Add;
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
    filters::multipart::Part,
    http::{HeaderMap, HeaderValue, StatusCode},
    path::Tail,
    Filter, Rejection, Reply,
};

use fendermint_actor_accumulator::PushReturn;
use fendermint_actor_objectstore::{DeleteParams, GetParams, ListParams, PutParams};
use fendermint_app_options::rpc::BroadcastMode;
use fendermint_app_settings::proxy::ProxySettings;
use fendermint_rpc::tx::BoundClient;
use fendermint_rpc::{
    client::FendermintClient,
    message::GasParams,
    tx::{CallClient, TxClient},
};
use fendermint_vm_actor_interface::adm;
use fendermint_vm_message::query::FvmQueryHeight;
use fvm_shared::chainid::ChainID;

use super::rpc::{gas_params, BroadcastResponse, TransClient};
use crate::cmd;
use crate::options::{
    proxy::{ProxyArgs, ProxyCommands},
    rpc::TransArgs,
};

const MAX_OBJECT_LENGTH: u64 = 1024 * 1024 * 1024;
const MAX_INTERNAL_OBJECT_LENGTH: u64 = 1024;
const MAX_EVENT_LENGTH: u64 = 1024 * 500; // Limit to 500KiB for now

cmd! {
    ProxyArgs(self, settings: ProxySettings) {
        match self.command.clone() {
            ProxyCommands::Run { tendermint_url, ipfs_addr, args} => {
                let client = FendermintClient::new_http(tendermint_url, None)?;
                let ipfs = IpfsClient::from_multiaddr_str(&ipfs_addr)?;
                let ipfs_adapter = Ipfs { inner: ipfs.clone() };

                let seq = args.sequence;
                let nonce = Arc::new(Mutex::new(seq));

                // Admin routes
                let health_route = warp::path!("health")
                    .and(warp::get()).and_then(health);

                // Machines
                let machines_create_os_route = warp::path!("v1" / "machines" / "objectstores")
                    .and(warp::post())
                    .and(warp::query::<WriteAccessQuery>())
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(with_nonce(nonce.clone()))
                    .and(warp::header::optional::<u64>("X-ADM-GasLimit"))
                    .and(warp::header::optional::<BroadcastMode>("X-ADM-BroadcastMode"))
                    .and_then(handle_machines_create_os);
                let machines_create_acc_route = warp::path!("v1" / "machines" / "accumulators")
                    .and(warp::post())
                    .and(warp::query::<WriteAccessQuery>())
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(with_nonce(nonce.clone()))
                    .and(warp::header::optional::<u64>("X-ADM-GasLimit"))
                    .and(warp::header::optional::<BroadcastMode>("X-ADM-BroadcastMode"))
                    .and_then(handle_machines_create_acc);
                let machines_get_route = warp::path!("v1" / "machines" / Address)
                    .and(warp::query::<HeightQuery>())
                    .and(warp::get())
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and_then(handle_machines_get);

                // Accounts
                let accounts_get_machines_route = warp::path!("v1" / "accounts" / Address / "machines")
                    .and(warp::query::<HeightQuery>())
                    .and(warp::get())
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and_then(handle_accounts_get_machines);

                // Objectstore routes
                let os_object_route = warp::path!("v1" / "object" )
                .and(warp::put())
                .and(with_client(client.clone()))
                .and(with_ipfs_adapter(ipfs_adapter.clone()))
                .and(warp::multipart::form().max_length(MAX_OBJECT_LENGTH))
                .and_then(handle_object_upload);

                let os_upload_route = warp::path!("v1" / "objectstores" / Address / ..)
                    .and(warp::put())
                    .and(warp::path::tail())
                    .and(warp::body::stream())
                    .and(warp::body::content_length_limit(MAX_OBJECT_LENGTH))
                    .and(warp::header::<u64>("Content-Length"))
                    .and(with_client(client.clone()))
                    .and(with_ipfs(ipfs.clone()))
                    .and(with_args(args.clone()))
                    .and(with_nonce(nonce.clone()))
                    .and(warp::header::optional::<u64>("X-ADM-GasLimit"))
                    .and(warp::header::optional::<BroadcastMode>("X-ADM-BroadcastMode"))
                    .and_then(handle_os_upload);

                let os_delete_route = warp::path!("v1" / "objectstores" / Address / ..)
                    .and(warp::delete())
                    .and(warp::path::tail())
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(with_nonce(nonce.clone()))
                    .and(warp::header::optional::<u64>("X-ADM-GasLimit"))
                    .and(warp::header::optional::<BroadcastMode>("X-ADM-BroadcastMode"))
                    .and_then(handle_os_delete);
                let os_get_or_list_route = warp::path!("v1" / "objectstores" / Address / ..)
                    .and(
                        warp::get().or(warp::head()).unify()
                    )
                    .and(warp::path::tail())
                    .and(warp::query::<HeightQuery>())
                    .and(warp::query::<ListQuery>())
                    .and(warp::header::optional::<String>("Range"))
                    .and(with_client(client.clone()))
                    .and(with_ipfs(ipfs.clone()))
                    .and(with_args(args.clone()))
                    .and_then(handle_os_get_or_list);

                // Accumulator routes
                let acc_push_route = warp::path!("v1" / "accumulators" / Address)
                    .and(warp::put())
                    .and(warp::body::content_length_limit(MAX_EVENT_LENGTH))
                    .and(warp::body::bytes())
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and(with_nonce(nonce))
                    .and(warp::header::optional::<u64>("X-ADM-GasLimit"))
                    .and(warp::header::optional::<BroadcastMode>("X-ADM-BroadcastMode"))
                    .and_then(handle_acc_push);
                let acc_get_at_route = warp::path!("v1" / "accumulators" / Address / "get")
                    .and(warp::get())
                    .and(warp::query::<GetQuery>())
                    .and(warp::query::<HeightQuery>())
                    .and(with_client(client.clone()))
                    .and(with_args(args.clone()))
                    .and_then(handle_acc_get_at);
                let acc_root_route = warp::path!("v1" / "accumulators" / Address)
                    .and(warp::get())
                    .and(warp::query::<HeightQuery>())
                    .and(with_client(client))
                    .and(with_args(args))
                    .and_then(handle_acc_root);

                let router = health_route
                    .or(machines_create_os_route)
                    .or(machines_create_acc_route)
                    .or(machines_get_route)
                    .or(accounts_get_machines_route)
                    .or(os_upload_route)
                    .or(os_object_route)
                    .or(os_delete_route)
                    .or(os_get_or_list_route)
                    .or(acc_push_route)
                    .or(acc_get_at_route)
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

fn with_ipfs_adapter<I: IpfsApiAdapter + Clone + Send>(
    client: I,
) -> impl Filter<Extract = (I,), Error = Infallible> + Clone {
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

fn get_os_key(tail: Tail) -> Result<Vec<u8>, Rejection> {
    let key = tail.as_str();
    match key.is_empty() {
        true => Err(Rejection::from(BadRequest {
            message: "empty key".into(),
        })),
        false => Ok(key.into()),
    }
}

#[derive(Serialize, Deserialize)]
struct WriteAccessQuery {
    pub write_access: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct HeightQuery {
    pub height: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct ListQuery {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Serialize, Deserialize)]
struct GetQuery {
    pub index: u64,
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

#[derive(Serialize, Deserialize, Debug)]
struct UploadQuery {
    pub msg: String,
    pub chain_id: u64,
}

pub trait IpfsApiAdapter {
    async fn add_file(
        &self,
        temp_file: TempFile,
        signed_msg: &SignedMessage,
    ) -> anyhow::Result<String>;
}

#[derive(Clone)]
pub struct Ipfs {
    inner: IpfsClient,
}

impl IpfsApiAdapter for Ipfs {
    async fn add_file(
        &self,
        mut temp_file: TempFile,
        signed_msg: &SignedMessage,
    ) -> anyhow::Result<String> {
        let temp_file_clone = temp_file.try_clone().await?;
        let cid_from_msg = match &signed_msg.object {
            Some(object) => object.value,
            None => return Err(anyhow!("missing cid in signed message")),
        };

        // Only chunk and hash - do not write to disk
        let res = self
            .inner
            .add_async_with_options(
                temp_file_clone.compat(),
                Add {
                    chunker: Some("size-1048576"),
                    only_hash: Some(true),
                    pin: Some(false),
                    ..Default::default()
                },
            )
            .await?;

        // Check if the computed CID matches the one in the signed message
        // It is important to verify the CID separately from the signature
        // because the signature is over the CID, it is unaware of the actual
        // data. Hence, we should verify that the CID of the underlying data
        // matches the one in the signed message
        let ipfs_cid = Cid::try_from(res.hash)?;
        if ipfs_cid != cid_from_msg {
            return Err(anyhow!(
                "computed cid {:?} does not match {:?}",
                ipfs_cid,
                cid_from_msg
            ));
        }

        // Actually add the file to IPFS
        temp_file.rewind().await?;
        let res = self
            .inner
            .add_async_with_options(
                temp_file.compat(),
                Add {
                    chunker: Some("size-1048576"),
                    pin: Some(false),
                    ..Default::default()
                },
            )
            .await?;
        let cid = Cid::try_from(res.hash)?;
        Ok(cid.to_string())
    }
}

struct ObjectVerifier {
    hasher: Blake2bHasher<32>,
    size: usize,
    signed_msg: Option<SignedMessage>,
    chain_id: ChainID,
    temp_file: TempFile,
}

impl ObjectVerifier {
    async fn read_chain_id(&mut self, form_part: Part) -> anyhow::Result<()> {
        let value = form_part
            .stream()
            .fold(Vec::new(), |mut vec, data| async move {
                let data = data.unwrap();
                vec.extend_from_slice(&data.chunk());
                vec
            })
            .await;
        let text =
            String::from_utf8(value.clone()).map_err(|_| anyhow!("cannot parse chain id"))?;
        let int: u64 = text.parse().map_err(|_| anyhow!("cannot parse chain_id"))?;
        self.chain_id = ChainID::from(int);

        Ok(())
    }

    async fn read_msg(&mut self, form_part: Part) -> anyhow::Result<()> {
        let value = form_part
            .stream()
            .fold(Vec::new(), |mut vec, data| async move {
                let data = data.unwrap();
                vec.extend_from_slice(&data.chunk());
                vec
            })
            .await;

        let signed_msg = general_purpose::URL_SAFE
            .decode(value)
            .map_err(|e| anyhow!("Failed to decode b64 encoded message: {}", e))
            .and_then(|b64_decoded| {
                fvm_ipld_encoding::from_slice::<SignedMessage>(&b64_decoded)
                    .map_err(|e| anyhow!("Failed to deserialize signed message: {}", e))
            })?;

        self.signed_msg = Some(signed_msg);

        Ok(())
    }

    async fn read_object(&mut self, form_part: Part) -> anyhow::Result<()> {
        let mut part_stream = form_part.stream();

        while let Some(data) = part_stream.next().await {
            let mut data = data.unwrap();
            while data.remaining() > 0 {
                let chunk = data.chunk().to_owned();
                let chunk_len = chunk.len();
                self.temp_file.write_all(&chunk).await.unwrap();
                self.temp_file.flush().await.unwrap();
                data.advance(chunk_len);
                self.hasher.update(&chunk);
                self.size += chunk_len;
            }
        }
        self.temp_file
            .rewind()
            .await
            .map_err(|e| anyhow!("failed to rewind temporary file: {}", e))?;

        Ok(())
    }

    async fn read_form(mut form_parts: warp::multipart::FormData) -> anyhow::Result<Self> {
        let temp_file = TempFile::new()
            .await
            .map_err(|e| anyhow!("failed to create temporary file: {}", e))?;

        let mut verifier = ObjectVerifier {
            hasher: Blake2bHasher::default(),
            size: 0,
            signed_msg: None,
            chain_id: ChainID::from(0),
            temp_file,
        };

        while let Some(part) = form_parts.next().await {
            let part = part.map_err(|_| anyhow!("cannot read form data"))?;
            match part.name() {
                "chain_id" => {
                    verifier.read_chain_id(part).await?;
                }
                "msg" => {
                    verifier.read_msg(part).await?;
                }
                "upload" => {
                    verifier.read_object(part).await?;
                }
                _ => {
                    return Err(anyhow!("unknown form field"));
                }
            }
        }

        Ok(verifier)
    }
}

impl ObjectVerifier {
    fn verify_signature(&self) -> anyhow::Result<()> {
        match &self.signed_msg {
            Some(signed_msg) => {
                signed_msg.verify(&self.chain_id)?;
            }
            None => return Err(anyhow!("missing signed message")),
        }

        Ok(())
    }

    async fn ensure_balance<F: QueryClient>(&mut self, client: &F) -> anyhow::Result<()> {
        match &self.signed_msg {
            Some(signed_msg) => {
                let from = signed_msg.message.from;
                let height = FvmQueryHeight::Committed;
                let actor_state = client.actor_state(&from, height).await?;
                let balance = match actor_state.value {
                    Some((_, state)) => to_eth_tokens(&state.balance).unwrap(),
                    None => et::U256::zero(),
                };

                // (todo): make cost_per_byte a configurable constant
                let cost_per_byte = et::U256::from(1_000_000_000u128);
                let required_balance = cost_per_byte * self.size;
                if balance < required_balance {
                    return Err(anyhow!("insufficient balance"));
                }
            }
            None => return Err(anyhow!("missing signed message")),
        }

        Ok(())
    }
}

async fn health() -> Result<impl Reply, Rejection> {
    Ok(warp::reply::reply())
}

async fn handle_object_upload<F: QueryClient, I: IpfsApiAdapter>(
    client: F,
    ipfs: I,
    form_parts: warp::multipart::FormData,
) -> Result<impl Reply, Rejection> {
    let mut verifier = ObjectVerifier::read_form(form_parts).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to read form: {}", e),
        })
    })?;

    // Verify the signature
    verifier.verify_signature().map_err(|e| {
        Rejection::from(BadRequest {
            message: e.to_string(),
        })
    })?;

    // Ensure the sender has enough balance
    verifier.ensure_balance(&client).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("failed to ensure balance: {}", e),
        })
    })?;

    // Add the data to IPFS

    let signed_msg = verifier.signed_msg.as_ref().unwrap();
    let cid = ipfs
        .add_file(verifier.temp_file, signed_msg)
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("failed to add file: {}", e),
            })
        })?;

    Ok(cid.to_string())
}

async fn handle_machines_create_os(
    write_access_query: WriteAccessQuery,
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

    let write_access = write_access_query
        .write_access
        .unwrap_or("onlyowner".into());
    let write_access = WriteAccess::from_str(&write_access).map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("machines create os error: {}", e),
        })
    })?;

    let res = os_create(client, args, write_access).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("machines create os error: {}", e),
        })
    })?;

    *nonce_lck += 1;
    Ok(warp::reply::json(&res))
}

async fn handle_machines_create_acc(
    write_access_query: WriteAccessQuery,
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

    let write_access = write_access_query
        .write_access
        .unwrap_or("onlyowner".into());
    let write_access = WriteAccess::from_str(&write_access).map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("machines create acc error: {}", e),
        })
    })?;

    let res = acc_create(client, args, write_access).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("machines create acc error: {}", e),
        })
    })?;

    *nonce_lck += 1;
    Ok(warp::reply::json(&res))
}

async fn handle_accounts_get_machines(
    owner: Address,
    height_query: HeightQuery,
    client: FendermintClient,
    args: TransArgs,
) -> Result<impl Reply, Rejection> {
    let height = height_query.height.unwrap_or(0);

    let res = list_machines(client, args, Some(owner), height)
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("accounts get machines error: {}", e),
            })
        })?;

    let list = res.unwrap_or_default();
    let machines = list
        .iter()
        .map(|m| json!({"address": m.address.to_string(), "kind": m.kind.to_string()}))
        .collect::<Vec<Value>>();

    let json = json!({"machines": machines});
    Ok(warp::reply::json(&json))
}

async fn handle_machines_get(
    address: Address,
    height_query: HeightQuery,
    client: FendermintClient,
    args: TransArgs,
) -> Result<impl Reply, Rejection> {
    let height = height_query.height.unwrap_or(0);

    let res = get_machine(client, args, address, height)
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("machines get error: {}", e),
            })
        })?;

    let json = json!({"kind": res.kind.to_string(), "owner": res.owner.to_string()});
    Ok(warp::reply::json(&json))
}

// Objectstore handlers

#[allow(clippy::too_many_arguments)]
async fn handle_os_upload(
    address: Address,
    path: Tail,
    mut body: impl Stream<Item = Result<impl Buf, warp::Error>> + Unpin + Send + Sync,
    size: u64,
    client: FendermintClient,
    ipfs: IpfsClient,
    mut args: TransArgs,
    nonce: Arc<Mutex<u64>>,
    gas_limit: Option<u64>,
    broadcast_mode: Option<BroadcastMode>,
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

    let key = get_os_key(path)?;

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

        PutParams {
            key,
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

        PutParams {
            key,
            kind: ObjectKind::Internal(ByteBuf(collected)),
            overwrite: true,
        }
    };

    let res = os_put(client, args, address, params).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("objectstore upload error: {}", e),
        })
    })?;

    *nonce_lck += 1;
    Ok(warp::reply::json(&res))
}

async fn handle_os_delete(
    address: Address,
    path: Tail,
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

    let key = get_os_key(path)?;

    let res = os_delete(client, args, address, DeleteParams { key })
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("objectstore delete error: {}", e),
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

#[allow(clippy::too_many_arguments)]
async fn handle_os_get_or_list(
    address: Address,
    tail: Tail,
    height_query: HeightQuery,
    list_query: ListQuery,
    range: Option<String>,
    client: FendermintClient,
    ipfs: IpfsClient,
    args: TransArgs,
) -> Result<impl Reply, Rejection> {
    let path = tail.as_str();
    if path.is_empty() || path.ends_with('/') {
        return handle_os_list(address, path, height_query, list_query, client, args).await;
    }

    let key: Vec<u8> = path.into();
    let height = height_query.height.unwrap_or(0);

    let res = os_get(client, args, address, GetParams { key }, height)
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("objectstore get error: {}", e),
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

async fn handle_os_list(
    address: Address,
    mut prefix: &str,
    height_query: HeightQuery,
    list_query: ListQuery,
    client: FendermintClient,
    args: TransArgs,
) -> Result<warp::reply::Response, Rejection> {
    if prefix == "/" {
        prefix = "";
    }
    let params = ListParams {
        prefix: prefix.into(),
        delimiter: "/".into(),
        offset: list_query.offset.unwrap_or(0),
        limit: list_query.limit.unwrap_or(0),
    };
    let height = height_query.height.unwrap_or(0);

    let res = os_list(client, args, address, params, height)
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("objectstore list error: {}", e),
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

    let list = json!({"objects": objects, "common_prefixes": common_prefixes});
    let list = serde_json::to_vec(&list).unwrap();
    let mut header_map = HeaderMap::new();
    header_map.insert("Content-Length", HeaderValue::from(list.len()));
    header_map.insert("Content-Type", HeaderValue::from_static("application/json"));
    let body = warp::hyper::Body::from(list);
    let mut response = warp::reply::Response::new(body);
    let headers = response.headers_mut();
    headers.extend(header_map);

    Ok(response)
}

// Accumulator handlers

async fn handle_acc_push(
    address: Address,
    body: Bytes,
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

    let res = acc_push(client.clone(), args.clone(), address, body)
        .await
        .map_err(|e| {
            Rejection::from(BadRequest {
                message: format!("accumulator push error: {}", e),
            })
        })?;

    *nonce_lck += 1;

    let data = res.data.map(|pr| {
        json!( {
            "root": pr.root.to_string(),
            "index": pr.index,
        })
    });
    let res_human_readable = Txn {
        status: res.status,
        hash: res.hash,
        height: res.height,
        gas_used: res.gas_used,
        data,
    };
    Ok(warp::reply::json(&res_human_readable))
}

async fn handle_acc_get_at(
    address: Address,
    get_query: GetQuery,
    height_query: HeightQuery,
    client: FendermintClient,
    args: TransArgs,
) -> Result<impl Reply, Rejection> {
    let res = acc_get_at(
        client,
        args,
        address,
        get_query.index,
        height_query.height.unwrap_or(0),
    )
    .await
    .map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("root error: {}", e),
        })
    })?;

    let str = String::from_utf8(res.unwrap_or_default()).unwrap();
    let json = json!({"value": str});
    Ok(warp::reply::json(&json))
}

async fn handle_acc_root(
    address: Address,
    height_query: HeightQuery,
    client: FendermintClient,
    args: TransArgs,
) -> Result<impl Reply, Rejection> {
    let height = height_query.height.unwrap_or(0);
    let res = acc_root(client, args, address, height).await.map_err(|e| {
        Rejection::from(BadRequest {
            message: format!("accumulator root error: {}", e),
        })
    })?;

    let json = json!({"root": res.unwrap_or_default().to_string()});
    Ok(warp::reply::json(&json))
}

// Rejection handlers

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

// Transaction handling

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

// RPC methods

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReturnPretty {
    pub actor_id: ActorID,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub robust_address: Option<String>,
}

async fn os_create(
    client: FendermintClient,
    args: TransArgs,
    write_access: WriteAccess,
) -> anyhow::Result<Txn<CreateReturnPretty>> {
    let tx = broadcast(client, args, |mut client, value, gas_params| {
        Box::pin(async move { client.os_create(write_access, value, gas_params).await })
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
) -> anyhow::Result<Option<Vec<adm::Metadata>>> {
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

async fn get_machine(
    client: FendermintClient,
    args: TransArgs,
    address: Address,
    height: u64,
) -> anyhow::Result<Metadata> {
    let mut client = TransClient::new(client, &args)?;
    let gas_params = gas_params(&args);
    let h = FvmQueryHeight::from(height);

    let res = client
        .inner
        .get_machine_call(address, TokenAmount::default(), gas_params, h)
        .await?;

    Ok(res.return_data.expect("metadata exists"))
}

async fn os_put(
    client: FendermintClient,
    args: TransArgs,
    address: Address,
    params: PutParams,
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
    params: DeleteParams,
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
    params: GetParams,
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
    params: ListParams,
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
    write_access: WriteAccess,
) -> anyhow::Result<Txn<CreateReturnPretty>> {
    let tx = broadcast(client, args, |mut client, value, gas_params| {
        Box::pin(async move { client.acc_create(write_access, value, gas_params).await })
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
) -> anyhow::Result<Txn<PushReturn>> {
    broadcast(client, args, |mut client, value, gas_params| {
        Box::pin(async move { client.acc_push(address, event, value, gas_params).await })
    })
    .await
}

async fn acc_get_at(
    client: FendermintClient,
    args: TransArgs,
    address: Address,
    index: u64,
    height: u64,
) -> anyhow::Result<Option<Vec<u8>>> {
    let mut client = TransClient::new(client, &args)?;
    let gas_params = gas_params(&args);
    let h = FvmQueryHeight::from(height);

    let res = client
        .inner
        .acc_get_at_call(address, TokenAmount::default(), gas_params, h, index)
        .await?;

    Ok(res.return_data)
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

#[cfg(test)]
mod tests {
    use super::*;
    use cid::multihash::{Code, MultihashDigest};
    use ethers::core::k256::ecdsa::SigningKey;
    use ethers::core::rand::{rngs::StdRng, SeedableRng};
    use fendermint_actor_objectstore::{ObjectKind, PutParams};
    use fendermint_rpc::FendermintClient;
    use fendermint_vm_message::conv::from_eth::to_fvm_address;
    use fvm_ipld_encoding::RawBytes;
    use tendermint_rpc::{Method, MockClient, MockRequestMethodMatcher};

    pub struct IpfsMocked {
        _inner: IpfsClient,
    }

    impl IpfsApiAdapter for IpfsMocked {
        async fn add_file(
            &self,
            _temp_file: TempFile,
            _signed_msg: &SignedMessage,
        ) -> anyhow::Result<String> {
            Ok("Qm123".to_string())
        }
    }

    // Used to mocking Actor State
    const ABCI_QUERY_RESPONSE: &str = r#"{
        "jsonrpc": "2.0",
        "id": "",
        "result": {
         "response": {
             "code": 0,
             "log": "",
             "info": "",
             "index": "0",
             "key": "GGQ=",
             "value": "pWRjb2Rl2CpYJwABVaDkAiB4ZQKaqaSEiu8tIb2Ef7bIWOoxPeNkAEljZabMaAMlaGVzdGF0ZdgqWCcAAXGg5AIgRbDPwiDO7Ft8HGLE1Bk9OOTrpI6IFXKc51+cCrDkwcBoc2VxdWVuY2UAZ2JhbGFuY2VKADY1ya3F3qAAAHFkZWxlZ2F0ZWRfYWRkcmVzc1YECqf8cO8ArRpoWzmcqFtkasWDXCqx",
             "proof": null,
             "height": "8",
             "codespace": ""
           }
        }
     }"#;

    fn form_body(
        boundary: &str,
        serialized_signed_message_b64: &str,
        external_object: &[u8],
    ) -> Vec<u8> {
        let mut body = Vec::new();
        body.extend_from_slice(
            format!(
                "\
            --{0}\r\n\
            content-disposition: form-data; name=\"chain_id\"\r\n\r\n\
            314159\r\n\
            --{0}\r\n\
            content-disposition: form-data; name=\"msg\"\r\n\r\n\
            {1}\r\n\
            --{0}\r\n\
            ",
                boundary, serialized_signed_message_b64
            )
            .as_bytes(),
        );
        body.extend_from_slice(
            format!(
                "Content-Disposition: form-data; name=\"upload\"; filename=\"example.bin\"\r\n\
                Content-Type: application/octet-stream\r\n\r\n",
            )
            .as_bytes(),
        );
        body.extend_from_slice(&external_object);
        body.extend_from_slice(format!("\r\n--{0}--\r\n", boundary).as_bytes());
        body
    }

    async fn multipart_form(
        serialized_signed_message_b64: &str,
        external_object: &[u8],
    ) -> warp::multipart::FormData {
        let boundary = "--abcdef1234--";
        let body = form_body(boundary, serialized_signed_message_b64, external_object);
        warp::test::request()
            .method("PUT")
            .header("content-length", body.len())
            .header(
                "content-type",
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(body)
            .filter(&warp::multipart::form())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_handle_object_upload() {
        let matcher = MockRequestMethodMatcher::default()
            .map(Method::AbciQuery, Ok(ABCI_QUERY_RESPONSE.to_string()));
        let client = FendermintClient::new(MockClient::new(matcher).0);
        let ipfs = IpfsMocked {
            _inner: IpfsClient::default(),
        };

        let key = b"key";
        let external_object = b"hello world".as_ref();
        let digest = Code::Blake2b256.digest(external_object);
        let object_cid = Cid::new_v1(fvm_ipld_encoding::IPLD_RAW, digest);
        let params = PutParams {
            key: key.to_vec(),
            kind: ObjectKind::External(object_cid),
            overwrite: true,
        };
        let params = RawBytes::serialize(params).unwrap();
        let to = Address::new_id(90);
        let object = fendermint_vm_message::signed::Object::new(key.to_vec(), object_cid, to);

        let sk = fendermint_crypto::SecretKey::random(&mut StdRng::from_entropy());
        let signing_key = SigningKey::from_slice(sk.serialize().as_ref()).unwrap();
        let from_address = ethers::core::utils::secret_key_to_address(&signing_key);
        let message = fvm_shared::message::Message {
            version: Default::default(),
            from: to_fvm_address(from_address),
            to,
            sequence: 0,
            value: TokenAmount::from_atto(0),
            method_num: fendermint_actor_objectstore::Method::PutObject as u64,
            params,
            gas_limit: 3000000,
            gas_fee_cap: TokenAmount::from_atto(0),
            gas_premium: TokenAmount::from_atto(0),
        };
        let chain_id = fvm_shared::chainid::ChainID::from(314159);
        let signed = fendermint_vm_message::signed::SignedMessage::new_secp256k1(
            message,
            Some(object),
            &sk,
            &chain_id,
        )
        .unwrap();

        let serialized_signed_message = fvm_ipld_encoding::to_vec(&signed).unwrap();
        let serialized_signed_message_b64 =
            general_purpose::URL_SAFE.encode(&serialized_signed_message);

        let multipart_form = multipart_form(&serialized_signed_message_b64, external_object).await;
        let reply = handle_object_upload(client, ipfs, multipart_form)
            .await
            .unwrap();
        let response = reply.into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
