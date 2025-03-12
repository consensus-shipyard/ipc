// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use async_trait::async_trait;
use ethers::providers::{Http, HttpClientError, JsonRpcClient};
use ipc_actors_abis::error_parser::ContractErrorParser;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::{Debug, Formatter};

/// A fvm contract revert parsing util
#[derive(Clone)]
pub struct FvmHttp {
    inner: Http,
}

impl From<Http> for FvmHttp {
    fn from(inner: Http) -> Self {
        Self { inner }
    }
}

impl Debug for FvmHttp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

#[async_trait]
impl JsonRpcClient for FvmHttp {
    type Error = <Http as JsonRpcClient>::Error;

    async fn request<T, R>(&self, method: &str, params: T) -> Result<R, Self::Error>
    where
        T: Debug + Serialize + Send + Sync,
        R: DeserializeOwned + Send,
    {
        self.inner
            .request(method, params)
            .await
            .map_err(|client_error| match client_error {
                HttpClientError::JsonRpcError(e) => {
                    let Some(raw_error) = e.data.as_ref() else {
                        return HttpClientError::JsonRpcError(e);
                    };

                    let Some(err_str) = raw_error.as_str() else {
                        return HttpClientError::JsonRpcError(e);
                    };

                    let Ok(Some(name)) = ContractErrorParser::parse_from_hex_str(err_str) else {
                        return HttpClientError::JsonRpcError(e);
                    };

                    tracing::error!("contract reverted with error: {name}");

                    HttpClientError::JsonRpcError(e)
                }
                e => e,
            })
    }
}
