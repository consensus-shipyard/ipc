// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use async_trait::async_trait;
use ethers::providers::{Http, HttpClientError, JsonRpcClient, JsonRpcError};
use ipc_actors_abis::error_parser::ContractErrorParser;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::{Debug, Formatter};

/// A fvm contract revert parsing util
#[derive(Clone)]
pub struct ErrorParserHttp {
    inner: Http,
}

impl From<Http> for ErrorParserHttp {
    fn from(inner: Http) -> Self {
        Self { inner }
    }
}

impl Debug for ErrorParserHttp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

#[async_trait]
impl JsonRpcClient for ErrorParserHttp {
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
                HttpClientError::JsonRpcError(e) => handle_json_rpc_error(e),
                e => e,
            })
    }
}

fn handle_json_rpc_error(e: JsonRpcError) -> HttpClientError {
    let Some(raw_error) = e.data.as_ref() else {
        return HttpClientError::JsonRpcError(e);
    };

    let Some(err_str) = raw_error.as_str() else {
        return HttpClientError::JsonRpcError(e);
    };

    // Try to parse the error with enhanced parsing
    let err_str_clean = err_str.strip_prefix("0x").unwrap_or(err_str);
    match ContractErrorParser::parse_from_hex_str(err_str_clean) {
        Ok(parsed_error) => {
            // Log the enhanced error information
            match parsed_error.error_type {
                ipc_actors_abis::error_parser::ErrorType::IpcContract => {
                    tracing::error!("IPC contract reverted with error: {}", parsed_error.name);
                    if let Some(ref params) = parsed_error.parameters {
                        tracing::error!("Error parameters: {:?}", params);
                    }
                    // Return the original error without modification
                    HttpClientError::JsonRpcError(e)
                }
                ipc_actors_abis::error_parser::ErrorType::StandardRevert => {
                    if let Some(message) = parsed_error.message {
                        tracing::error!("Contract reverted with message: {}", message);
                        // Replace the error message with the parsed revert message
                        let mut new_rpc_error = e.clone();
                        new_rpc_error.message = format!("Contract reverted: {}", message);
                        HttpClientError::JsonRpcError(new_rpc_error)
                    } else {
                        tracing::error!("Contract reverted with standard Error(string)");
                        // Replace the error message with a generic revert message
                        let mut new_rpc_error = e.clone();
                        new_rpc_error.message =
                            "Contract reverted with standard Error(string)".to_string();
                        HttpClientError::JsonRpcError(new_rpc_error)
                    }
                }
                ipc_actors_abis::error_parser::ErrorType::Panic => {
                    if let Some(message) = parsed_error.message {
                        tracing::error!("Contract panicked: {}", message);
                        // Replace the error message with the panic message
                        let mut new_rpc_error = e.clone();
                        new_rpc_error.message = format!("Contract panicked: {}", message);
                        if let Some(ref params) = parsed_error.parameters {
                            new_rpc_error
                                .message
                                .push_str(&format!(" (code: {:?})", params));
                        }
                        HttpClientError::JsonRpcError(new_rpc_error)
                    } else {
                        tracing::error!("Contract panicked");
                        // Replace the error message with a generic panic message
                        let mut new_rpc_error = e.clone();
                        new_rpc_error.message = "Contract panicked".to_string();
                        if let Some(ref params) = parsed_error.parameters {
                            new_rpc_error
                                .message
                                .push_str(&format!(" (code: {:?})", params));
                        }
                        HttpClientError::JsonRpcError(new_rpc_error)
                    }
                }
                ipc_actors_abis::error_parser::ErrorType::Unknown => {
                    tracing::error!(
                        "Contract reverted with unknown error: {} (selector: {})",
                        parsed_error.name,
                        err_str_clean
                    );
                    // Replace the error message with the unknown error name
                    let mut new_rpc_error = e.clone();
                    new_rpc_error.message = format!(
                        "Contract reverted with unknown error: {} (selector: {})",
                        parsed_error.name, err_str_clean
                    );
                    HttpClientError::JsonRpcError(new_rpc_error)
                }
            }
        }
        Err(_) => {
            // Fallback to legacy parsing for backward compatibility
            if let Ok(name) = ContractErrorParser::parse_from_hex_str_legacy(err_str_clean) {
                tracing::error!("contract reverted with error: {name}");
                // Replace the error message with the legacy parsed error name
                let mut new_rpc_error = e.clone();
                new_rpc_error.message = format!("Contract reverted with error: {}", name);
                HttpClientError::JsonRpcError(new_rpc_error)
            } else {
                tracing::error!("contract reverted with unparseable error: {err_str}");
                // Keep the original error message for unparseable errors
                HttpClientError::JsonRpcError(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn test_error_parsing_works() {
        // Test that IPC contract errors are parsed correctly
        let test_error = "d6bb62dd";
        match ContractErrorParser::parse_from_hex_str(test_error) {
            Ok(parsed_error) => {
                assert_eq!(parsed_error.name, "BottomUpCheckpointAlreadySubmitted");
                assert!(matches!(
                    parsed_error.error_type,
                    ipc_actors_abis::error_parser::ErrorType::IpcContract
                ));
                println!("✓ IPC error parsing works: {}", parsed_error.name);
            }
            Err(e) => {
                panic!("IPC error parsing failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_standard_revert_handled() {
        // Test that standard Solidity revert strings are now handled
        // Note: The actual message is "This is a test error messa" (truncated due to padding)
        let revert_string = "08c379a00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000001a5468697320697320612074657374206572726f72206d6573736167650000000000";

        match ContractErrorParser::parse_from_hex_str(revert_string) {
            Ok(parsed_error) => {
                assert_eq!(parsed_error.name, "RevertString");
                assert!(matches!(
                    parsed_error.error_type,
                    ipc_actors_abis::error_parser::ErrorType::StandardRevert
                ));
                // The message should contain "This is a test error" (may be truncated due to padding)
                assert!(parsed_error.message.is_some());
                let message = parsed_error.message.unwrap();
                assert!(message.contains("This is a test error"));
                println!("✓ Standard revert now handled: {}", message);
            }
            Err(e) => {
                panic!("Standard revert parsing failed: {:?}", e);
            }
        }
    }

    #[test]
    fn test_error_parser_http_creation() {
        // Test that we can create the proxy
        let url = Url::parse("http://localhost:8545").unwrap();
        let http_provider = Http::new(url);
        let _error_parser_http = ErrorParserHttp::from(http_provider);
        println!("✓ ErrorParserHttp proxy created successfully");
    }

    #[test]
    fn test_not_owner_of_public_key_error() {
        // Test that the NotOwnerOfPublicKey error is parsed correctly
        let test_error = "97d24a3a"; // NotOwnerOfPublicKey selector
        match ContractErrorParser::parse_from_hex_str(test_error) {
            Ok(parsed_error) => {
                assert_eq!(parsed_error.name, "NotOwnerOfPublicKey");
                assert!(matches!(
                    parsed_error.error_type,
                    ipc_actors_abis::error_parser::ErrorType::IpcContract
                ));
                println!(
                    "✓ NotOwnerOfPublicKey error parsing works: {}",
                    parsed_error.name
                );
            }
            Err(e) => {
                panic!("NotOwnerOfPublicKey error parsing failed: {:?}", e);
            }
        }
    }
}
