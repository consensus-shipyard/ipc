// Copyright 2022-2023 Protocol Labs
// SPDX-License-Identifier: MIT
use crate::config::JSON_RPC_ENDPOINT;
use crate::config::{Config, JSON_RPC_VERSION};
use crate::server::request::JSONRPCRequest;
use crate::server::response::{JSONRPCError, JSONRPCErrorResponse, JSONRPCResultResponse};
use crate::server::Handlers;
use bytes::Bytes;
use std::sync::Arc;
use warp::http::StatusCode;
use warp::reject::Reject;
use warp::reply::with_status;
use warp::{Filter, Rejection, Reply};

type ArcHandlers = Arc<Handlers>;

/// The IPC JSON RPC node that contains all the methods and handlers. The underlying implementation
/// is using `warp`.
///
/// Note that currently only http json rpc is supported.
///
/// # Examples
/// ```no_run
/// use ipc_agent::config::Config;
/// use ipc_agent::server::jsonrpc::JsonRPCServer;
///
/// #[tokio::main]
/// async fn main() {
///     let config = Config::from_file("PATH TO YOUR CONFIG FILE").unwrap();
///     let n = JsonRPCServer::new(config);
///     n.run().await;
/// }
/// ```
pub struct JsonRPCServer {
    config: Config,
}

impl JsonRPCServer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Runs the node in the current thread
    pub async fn run(&self) {
        log::info!(
            "IPC agent rpc node listening at {:?}",
            self.config.server.json_rpc_address
        );

        let handlers = Arc::new(Handlers::new());
        warp::serve(json_rpc_filter(handlers))
            .run(self.config.server.json_rpc_address)
            .await;
    }
}

// Internal implementations

/// Create the json_rpc filter. The filter does the following:
/// - Listen to POST requests on the DEFAULT_JSON_RPC_ENDPOINT
/// - Extract the body of the request.
/// - Pass it to to the json_rpc_filter to deserialize into a jsonrpc request.
fn json_rpc_filter(
    handlers: ArcHandlers,
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path(JSON_RPC_ENDPOINT))
        .and(warp::body::bytes())
        .and_then(to_json_rpc_request)
        .and(with_handlers(handlers))
        .and_then(handle_request)
        .recover(handle_rejection)
}

fn with_handlers(
    handlers: ArcHandlers,
) -> impl Filter<Extract = (ArcHandlers,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || handlers.clone())
}

// Filter that deserializes the body of the request into a jsonrpc request.
async fn to_json_rpc_request(bytes: Bytes) -> Result<JSONRPCRequest, warp::Rejection> {
    serde_json::from_slice::<JSONRPCRequest>(bytes.as_ref()).map_err(|e| {
        log::debug!("cannot deserialize {bytes:?} due to {e:?}");
        warp::reject::custom(InvalidParameter)
    })
}

/// Main function responsible for handling and routing jsonrpc requests to the right underlying handler according to the method
async fn handle_request(
    json_rpc_request: JSONRPCRequest,
    handlers: ArcHandlers,
) -> Result<impl Reply, warp::Rejection> {
    log::debug!("received json rpc request = {:?}", json_rpc_request);

    let JSONRPCRequest {
        id,
        method,
        params,
        jsonrpc,
    } = json_rpc_request;

    if jsonrpc != JSON_RPC_VERSION {
        return Ok(warp::reply::json(&JSONRPCErrorResponse::invalid_request(
            id,
        )));
    }

    log::info!("received method = {method:?} and params = {params:?}");
    match handlers.handle(method, params).await {
        Ok(response) => Ok(warp::reply::json(&JSONRPCResultResponse::new(id, response))),
        Err(e) => {
            let error: JSONRPCError<()> = JSONRPCError {
                code: -1,
                data: None,
                message: e.to_string(),
            };
            Ok(warp::reply::json(&JSONRPCErrorResponse::new(id, error)))
        }
    }
}

/// The invalid parameter warp rejection error handling
#[derive(Debug)]
struct InvalidParameter;

impl Reject for InvalidParameter {}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, warp::Rejection> {
    if err.is_not_found() {
        Ok(with_status("NOT_FOUND", StatusCode::NOT_FOUND))
    } else if err.find::<InvalidParameter>().is_some() {
        Ok(with_status("BAD_REQUEST", StatusCode::BAD_REQUEST))
    } else {
        log::error!("unhandled rejection: {:?}", err);
        Ok(with_status(
            "INTERNAL_SERVER_ERROR",
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::config::{JSON_RPC_ENDPOINT, JSON_RPC_VERSION};
    use crate::server::jsonrpc::{json_rpc_filter, ArcHandlers, JSONRPCResultResponse};
    use crate::server::request::JSONRPCRequest;
    use crate::server::Handlers;
    use std::sync::Arc;
    use warp::http::StatusCode;

    fn get_test_handlers() -> ArcHandlers {
        Arc::new(Handlers::new())
    }

    #[tokio::test]
    #[ignore]
    async fn test_json_rpc_filter_works() {
        let filter = json_rpc_filter(get_test_handlers());

        let foo = "foo".to_string();
        let jsonrpc = String::from(JSON_RPC_VERSION);
        let id = 0;

        let req = JSONRPCRequest {
            id,
            jsonrpc: jsonrpc.clone(),
            method: foo.clone(),
            params: Default::default(),
        };

        let value = warp::test::request()
            .method("POST")
            .path(&format!("/{JSON_RPC_ENDPOINT:}"))
            .json(&req)
            .reply(&filter)
            .await;

        let v = serde_json::from_slice::<JSONRPCResultResponse<()>>(value.body()).unwrap();

        assert_eq!(v.id, id);
        assert_eq!(v.jsonrpc, jsonrpc);
        //assert_eq!(v.result, ());
    }

    #[tokio::test]
    async fn test_json_rpc_filter_cannot_parse_param() {
        let filter = json_rpc_filter(get_test_handlers());

        let value = warp::test::request()
            .method("POST")
            .path(&format!("/{JSON_RPC_ENDPOINT:}"))
            .json(&())
            .reply(&filter)
            .await;

        assert_eq!(StatusCode::BAD_REQUEST, value.status());
    }

    #[tokio::test]
    async fn test_json_rpc_filter_not_found() {
        let filter = json_rpc_filter(get_test_handlers());

        let value = warp::test::request()
            .method("POST")
            .path("/random")
            .json(&())
            .reply(&filter)
            .await;

        assert_eq!(StatusCode::NOT_FOUND, value.status());
    }
}
