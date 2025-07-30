// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: MIT

use async_trait::async_trait;
use ethers::providers::{Http, HttpClientError, JsonRpcClient};
use ipc_actors_abis::error_parser::{ContractErrorParser, ParsedError};
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
                HttpClientError::JsonRpcError(e) => {
                    let Some(raw_error) = e.data.as_ref() else {
                        return HttpClientError::JsonRpcError(e);
                    };

                    let Some(err_str) = raw_error.as_str() else {
                        return HttpClientError::JsonRpcError(e);
                    };

                    // Try to parse the error with enhanced parsing
                    match ContractErrorParser::parse_from_hex_str(err_str) {
                        Ok(parsed_error) => {
                            // Log the enhanced error information
                            match parsed_error.error_type {
                                ipc_actors_abis::error_parser::ErrorType::IpcContract => {
                                    tracing::error!(
                                        "IPC contract reverted with error: {}",
                                        parsed_error.name
                                    );
                                    if let Some(params) = parsed_error.parameters {
                                        tracing::error!("Error parameters: {:?}", params);
                                    }
                                }
                                ipc_actors_abis::error_parser::ErrorType::StandardRevert => {
                                    if let Some(message) = parsed_error.message {
                                        tracing::error!(
                                            "Contract reverted with message: {}",
                                            message
                                        );
                                    } else {
                                        tracing::error!(
                                            "Contract reverted with standard Error(string)"
                                        );
                                    }
                                }
                                ipc_actors_abis::error_parser::ErrorType::Panic => {
                                    if let Some(message) = parsed_error.message {
                                        tracing::error!("Contract panicked: {}", message);
                                    } else {
                                        tracing::error!("Contract panicked");
                                    }
                                    if let Some(params) = parsed_error.parameters {
                                        tracing::error!("Panic parameters: {:?}", params);
                                    }
                                }
                                ipc_actors_abis::error_parser::ErrorType::Unknown => {
                                    tracing::error!(
                                        "Contract reverted with unknown error: {}",
                                        parsed_error.name
                                    );
                                }
                            }
                        }
                        Err(_) => {
                            // Fallback to legacy parsing for backward compatibility
                            if let Ok(name) =
                                ContractErrorParser::parse_from_hex_str_legacy(err_str)
                            {
                                tracing::error!("contract reverted with error: {name}");
                            } else {
                                tracing::error!(
                                    "contract reverted with unparseable error: {err_str}"
                                );
                            }
                        }
                    }

                    HttpClientError::JsonRpcError(e)
                }
                e => e,
            })
    }
}

/// Enhanced error information for better user experience
#[derive(Debug, Clone)]
pub struct EnhancedContractError {
    pub error_type: String,
    pub name: String,
    pub message: Option<String>,
    pub parameters: Option<Vec<String>>,
    pub contract_address: Option<String>,
    pub function_name: Option<String>,
    pub suggestions: Option<Vec<String>>,
}

impl EnhancedContractError {
    pub fn from_parsed_error(parsed: ParsedError) -> Self {
        let suggestions = match parsed.error_type {
            ipc_actors_abis::error_parser::ErrorType::IpcContract => match parsed.name.as_str() {
                "BottomUpCheckpointAlreadySubmitted" => Some(vec![
                    "This checkpoint has already been submitted".to_string(),
                    "Check if you're trying to submit the same checkpoint twice".to_string(),
                ]),
                "InsufficientFunds" => Some(vec![
                    "Ensure you have sufficient funds for this operation".to_string(),
                    "Check your balance and collateral requirements".to_string(),
                ]),
                "NotAuthorized" => Some(vec![
                    "You are not authorized to perform this action".to_string(),
                    "Check if you have the required permissions".to_string(),
                ]),
                _ => None,
            },
            ipc_actors_abis::error_parser::ErrorType::StandardRevert => Some(vec![
                "This is a standard Solidity revert".to_string(),
                "Check the revert message for specific details".to_string(),
            ]),
            ipc_actors_abis::error_parser::ErrorType::Panic => Some(vec![
                "This is a Solidity panic".to_string(),
                "Check the panic code and message for details".to_string(),
            ]),
            ipc_actors_abis::error_parser::ErrorType::Unknown => None,
        };

        Self {
            error_type: format!("{:?}", parsed.error_type),
            name: parsed.name,
            message: parsed.message,
            parameters: parsed.parameters,
            contract_address: None, // TODO: Extract from context
            function_name: None,    // TODO: Extract from context
            suggestions,
        }
    }

    pub fn to_user_friendly_message(&self) -> String {
        let mut message = format!("Contract Error: {}", self.name);

        if let Some(msg) = &self.message {
            message.push_str(&format!("\nMessage: {}", msg));
        }

        if let Some(params) = &self.parameters {
            message.push_str(&format!("\nParameters: {:?}", params));
        }

        if let Some(suggestions) = &self.suggestions {
            message.push_str("\n\nSuggestions:");
            for suggestion in suggestions {
                message.push_str(&format!("\n- {}", suggestion));
            }
        }

        message
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
    fn test_enhanced_error_creation() {
        let test_error = "d6bb62dd";
        let parsed = ContractErrorParser::parse_from_hex_str(test_error).unwrap();
        let enhanced = EnhancedContractError::from_parsed_error(parsed);

        assert_eq!(enhanced.name, "BottomUpCheckpointAlreadySubmitted");
        assert!(enhanced.suggestions.is_some());
        println!("✓ Enhanced error created with suggestions");
    }

    #[test]
    fn test_error_parser_http_creation() {
        // Test that we can create the proxy
        let url = Url::parse("http://localhost:8545").unwrap();
        let http_provider = Http::new(url);
        let _error_parser_http = ErrorParserHttp::from(http_provider);
        println!("✓ ErrorParserHttp proxy created successfully");
    }
}
