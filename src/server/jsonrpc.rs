use crate::server::request::JSONRPCRequest;
use crate::server::response::{JSONRPCErrorResponse, JSONRPCResultResponse};
use bytes::Bytes;
use warp::http::StatusCode;
use warp::reject::Reject;
use warp::reply::with_status;
use warp::{Filter, Rejection, Reply};
use crate::config::{JSON_RPC_VERSION, Server as JsonRPCServerConfig};
use crate::config::JSON_RPC_ENDPOINT;

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
///     let n = JsonRPCServer::new(config.server);
///     n.run().await;
/// }
/// ```
pub struct JsonRPCServer {
    config: JsonRPCServerConfig,
}

impl JsonRPCServer {
    pub fn new(config: JsonRPCServerConfig) -> Self {
        Self { config }
    }

    /// Runs the node in the current thread
    pub async fn run(&self) {
        log::info!("IPC agent rpc node listening at {:?}", self.config.json_rpc_address);
        warp::serve(json_rpc_filter()).run(self.config.json_rpc_address).await;
    }
}

// Internal implementations

/// Create the json_rpc filter. The filter does the following:
/// - Listen to POST requests on the DEFAULT_JSON_RPC_ENDPOINT
/// - Extract the body of the request.
/// - Pass it to to the json_rpc_filter to deserialize into a jsonrpc request.
fn json_rpc_filter() -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Copy {
    warp::post()
        .and(warp::path(JSON_RPC_ENDPOINT))
        .and(warp::body::bytes())
        .and_then(to_json_rpc_request)
        .and_then(handle_request)
        .recover(handle_rejection)
}

// Filter that deserializes the body of the request into a jsonrpc request.
async fn to_json_rpc_request(bytes: Bytes) -> Result<JSONRPCRequest, warp::Rejection> {
    serde_json::from_slice::<JSONRPCRequest>(bytes.as_ref()).map_err(|e| {
        log::debug!("cannot deserialize {bytes:?} due to {e:?}");
        warp::reject::custom(InvalidParameter)
    })
}

/// Main function responsible for handling and routing jsonrpc requests to the right underlying handler according to the method
async fn handle_request(json_rpc_request: JSONRPCRequest) -> Result<impl Reply, warp::Rejection> {
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

    let response = JSONRPCResultResponse::new(id, ());
    Ok(warp::reply::json(&response))
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
    use crate::server::jsonrpc::{json_rpc_filter, JSONRPCResultResponse};
    use crate::server::request::JSONRPCRequest;
    use crate::config::JSON_RPC_VERSION;
    use warp::http::StatusCode;
    use crate::config::JSON_RPC_ENDPOINT;

    #[tokio::test]
    async fn test_json_rpc_filter_works() {
        let filter = json_rpc_filter();

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
        assert_eq!(v.result, ());
    }

    #[tokio::test]
    async fn test_json_rpc_filter_cannot_parse_param() {
        let filter = json_rpc_filter();

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
        let filter = json_rpc_filter();

        let value = warp::test::request()
            .method("POST")
            .path("/random")
            .json(&())
            .reply(&filter)
            .await;

        assert_eq!(StatusCode::NOT_FOUND, value.status());
    }
}
