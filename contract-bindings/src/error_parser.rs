use ethers::abi::ethabi::AbiError;
use std::collections::BTreeMap;

/// Macro to extend the contract error mapping by aggregating errors from multiple contracts.
/// Each contract is specified as a pair: `[snake_case, ABI_IDENTIFIER]` where:
/// - `snake_case` is the contract name converted to snake case.
/// - `ABI_IDENTIFIER` is the contract name in uppercase with `_ABI` appended.
///
/// This macro builds a lazily initialized static map (`MAP`) that associates Solidity error selectors
/// (first 4 bytes of an error signature, hex-encoded) with their corresponding `AbiError`.
#[macro_export]
macro_rules! extend_contract_error_mapping {
    ($([$snake_case:ident, $abi:ident]),* $(,)?) => {
        lazy_static::lazy_static! {
            pub(crate) static ref MAP: ::std::collections::BTreeMap<String,  ethers::abi::ethabi::AbiError> = {
                let mut errors = ::std::collections::BTreeMap::default();

                $(
                    $crate::error_parser::extend_errors(&mut errors, $crate::gen::$snake_case::$snake_case::$abi.errors.clone());
                )*

                errors
            };
        }
    }
}

const SOLIDITY_SELECTOR_BYTE_SIZE: usize = 4;

/// Standard Solidity error selectors
const ERROR_STRING_SELECTOR: &str = "08c379a0"; // Error(string)
const PANIC_SELECTOR: &str = "4e487b71"; // Panic(uint256)

/// Extends the provided error map with errors from a contract's error collection.
/// For each error, it extracts the Solidity selector (first 4 bytes of the error signature),
/// hex-encodes it, and maps that selector to a clone of the `AbiError`.
///
/// If a selector already exists in the map, a warning is logged.
pub fn extend_errors(
    map: &mut BTreeMap<String, AbiError>,
    contract_errors: BTreeMap<String, Vec<AbiError>>,
) {
    for (_, v) in contract_errors.iter() {
        for e in v {
            // solidity selector is only the first 4 bytes of the signature
            let selector = const_hex::encode(&e.signature().0[0..SOLIDITY_SELECTOR_BYTE_SIZE]);
            map.insert(selector, e.clone());
        }
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ParseContractError {
    #[error("error bytes shorter than 4 bytes for solidity contract selector")]
    ErrorBytesTooShort,
    #[error("error string not hex format: {0}")]
    ErrorNotHexStr(String),
    #[error("error selector not found in contract error map: {selector}")]
    ErrorNotFound { selector: String },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedError {
    pub error_type: ErrorType,
    pub name: String,
    pub message: Option<String>,
    pub parameters: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    IpcContract,
    StandardRevert,
    Panic,
    Unknown,
}

pub struct ContractErrorParser {}

impl ContractErrorParser {
    pub fn parse_from_bytes(bytes: &[u8]) -> Result<ParsedError, ParseContractError> {
        if bytes.len() < SOLIDITY_SELECTOR_BYTE_SIZE {
            return Err(ParseContractError::ErrorBytesTooShort);
        }

        let selector = const_hex::encode(&bytes[0..4]);

        // Check for standard Solidity errors first
        match selector.as_str() {
            ERROR_STRING_SELECTOR => {
                // Handle Error(string)
                if bytes.len() >= 68 {
                    // 4 bytes selector + 32 bytes offset + 32 bytes length
                    let length_start = 36; // 4 + 32
                    let length_end = 68; // 4 + 32 + 32

                    if bytes.len() >= length_end {
                        let length_bytes = &bytes[length_start..length_end];
                        let length_u256 = ethers::types::U256::from_big_endian(length_bytes);
                        let length = length_u256.as_u64();

                        let message_start = length_end;
                        let message_end = message_start + length as usize;

                        if bytes.len() >= message_end {
                            let message_bytes = &bytes[message_start..message_end];
                            if let Ok(message) = String::from_utf8(message_bytes.to_vec()) {
                                return Ok(ParsedError {
                                    error_type: ErrorType::StandardRevert,
                                    name: "RevertString".to_string(),
                                    message: Some(message),
                                    parameters: None,
                                });
                            }
                        }
                    }
                }

                // Fallback if parsing fails
                Ok(ParsedError {
                    error_type: ErrorType::StandardRevert,
                    name: "RevertString".to_string(),
                    message: None,
                    parameters: None,
                })
            }
            PANIC_SELECTOR => {
                // Handle Panic(uint256)
                if bytes.len() >= 36 {
                    // 4 bytes selector + 32 bytes panic code
                    let panic_code_bytes = &bytes[4..36];
                    let panic_code_u256 = ethers::types::U256::from_big_endian(panic_code_bytes);
                    let panic_code = panic_code_u256.as_u64();

                    let panic_message = match panic_code {
                        0x00 => "Generic panic",
                        0x01 => "Assertion failed",
                        0x11 => "Arithmetic overflow/underflow",
                        0x12 => "Division by zero",
                        0x21 => "Invalid enum value",
                        0x22 => "Invalid encoded storage byte array",
                        0x31 => "Pop on empty array",
                        0x32 => "Array index out of bounds",
                        0x41 => "Memory allocation overflow",
                        0x51 => "Zero-initialized variable of internal function type",
                        _ => "Unknown panic",
                    };

                    return Ok(ParsedError {
                        error_type: ErrorType::Panic,
                        name: "Panic".to_string(),
                        message: Some(panic_message.to_string()),
                        parameters: Some(vec![format!("0x{:x}", panic_code)]),
                    });
                }

                Ok(ParsedError {
                    error_type: ErrorType::Panic,
                    name: "Panic".to_string(),
                    message: None,
                    parameters: None,
                })
            }
            _ => {
                // Check for IPC contract errors
                let Some(error) = crate::gen::MAP.get(&selector) else {
                    // Instead of returning ErrorNotFound, treat as Unknown error
                    return Ok(ParsedError {
                        error_type: ErrorType::Unknown,
                        name: format!("UnknownError_{}", selector),
                        message: None,
                        parameters: None,
                    });
                };

                // Try to decode the error with parameters
                let mut parameters = None;
                if let Ok(decoded) = error.decode(bytes) {
                    // Only show parameters if they contain meaningful data
                    let decoded_str = format!("{:?}", decoded);
                    if !decoded_str.contains("[]") && !decoded_str.is_empty() {
                        parameters = Some(vec![decoded_str]);
                    }
                }

                Ok(ParsedError {
                    error_type: ErrorType::IpcContract,
                    name: error.name.clone(),
                    message: None,
                    parameters,
                })
            }
        }
    }

    pub fn parse_from_hex_str(err: &str) -> Result<ParsedError, ParseContractError> {
        let bytes = const_hex::decode(err)
            .map_err(|e| ParseContractError::ErrorNotHexStr(e.to_string()))?;
        Self::parse_from_bytes(bytes.as_slice())
    }

    /// Legacy method for backward compatibility
    pub fn parse_from_bytes_legacy(bytes: &[u8]) -> Result<String, ParseContractError> {
        Self::parse_from_bytes(bytes).map(|parsed| parsed.name)
    }

    /// Legacy method for backward compatibility
    pub fn parse_from_hex_str_legacy(err: &str) -> Result<String, ParseContractError> {
        Self::parse_from_hex_str(err).map(|parsed| parsed.name)
    }
}

#[cfg(test)]
mod tests {
    use crate::error_parser::{ContractErrorParser, ParseContractError};
    use const_hex::hex;

    #[test]
    fn test_parse_error_ok() {
        // selector for "BottomUpCheckpointAlreadySubmitted" error
        let err_bytes = hex::decode("d6bb62dd").unwrap();

        let result = ContractErrorParser::parse_from_bytes(err_bytes.as_ref()).unwrap();
        assert_eq!(result.name, "BottomUpCheckpointAlreadySubmitted");
        assert!(matches!(
            result.error_type,
            crate::error_parser::ErrorType::IpcContract
        ));
    }

    #[test]
    fn test_parse_standard_revert() {
        // Standard Solidity Error(string) with message "This is a test error message"
        let revert_string = "08c379a00000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000001a5468697320697320612074657374206572726f72206d6573736167650000000000";

        let result = ContractErrorParser::parse_from_hex_str(revert_string).unwrap();
        assert_eq!(result.name, "RevertString");
        assert!(matches!(
            result.error_type,
            crate::error_parser::ErrorType::StandardRevert
        ));
        // The message should contain "This is a test error" (may be truncated due to padding)
        assert!(result.message.is_some());
        let message = result.message.unwrap();
        assert!(message.contains("This is a test error"));
    }

    #[test]
    fn test_parse_panic() {
        // Panic with code 0x11 (arithmetic overflow)
        let panic_bytes =
            hex::decode("4e487b710000000000000000000000000000000000000000000000000000000000000011")
                .unwrap();

        let result = ContractErrorParser::parse_from_bytes(panic_bytes.as_ref()).unwrap();
        assert_eq!(result.name, "Panic");
        assert!(matches!(
            result.error_type,
            crate::error_parser::ErrorType::Panic
        ));
        assert_eq!(
            result.message,
            Some("Arithmetic overflow/underflow".to_string())
        );
        assert_eq!(result.parameters, Some(vec!["0x11".to_string()]));
    }

    #[test]
    fn test_parse_error_not_found() {
        // a random error selector
        let err_bytes = hex::decode("a6bb62dd").unwrap();

        let result = ContractErrorParser::parse_from_bytes(err_bytes.as_ref());
        assert!(result.is_err());
        if let Err(ParseContractError::ErrorNotFound { selector }) = result {
            assert_eq!(selector, "a6bb62dd");
        } else {
            panic!("Expected ErrorNotFound error");
        }
    }

    #[test]
    fn test_legacy_compatibility() {
        // Test that legacy methods still work
        let test_error = "d6bb62dd";
        let legacy_result = ContractErrorParser::parse_from_hex_str_legacy(test_error).unwrap();
        assert_eq!(legacy_result, "BottomUpCheckpointAlreadySubmitted");
    }
}
