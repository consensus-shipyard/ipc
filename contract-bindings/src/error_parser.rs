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
/// Extends the provided error map with errors from a contract’s error collection.
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
            let selector = const_hex::hex::encode(&e.signature().0[0..SOLIDITY_SELECTOR_BYTE_SIZE]);
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

pub struct ContractErrorParser {}

impl ContractErrorParser {
    pub fn parse_from_bytes(bytes: &[u8]) -> Result<String, ParseContractError> {
        if bytes.len() < SOLIDITY_SELECTOR_BYTE_SIZE {
            return Err(ParseContractError::ErrorBytesTooShort);
        }

        let selector = const_hex::hex::encode(&bytes[0..4]);

        let Some(error) = crate::gen::MAP.get(&selector) else {
            return Err(ParseContractError::ErrorNotFound { selector });
        };
        if let Err(e) = error.decode(bytes) {
            tracing::warn!("contract error selector found: {selector}, but decode failed: {e}");
        }

        Ok(error.name.clone())
    }

    pub fn parse_from_hex_str(err: &str) -> Result<String, ParseContractError> {
        let bytes = const_hex::hex::decode(err)
            .map_err(|e| ParseContractError::ErrorNotHexStr(e.to_string()))?;
        Self::parse_from_bytes(bytes.as_slice())
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

        assert_eq!(
            ContractErrorParser::parse_from_bytes(err_bytes.as_ref()).unwrap(),
            "BottomUpCheckpointAlreadySubmitted".to_string()
        );

        // selector for "FunctionNotFound" error
        let err_bytes =
            hex::decode("5416eb98611941f900000000000000000000000000000000000000000000000000000000")
                .unwrap();

        assert_eq!(
            ContractErrorParser::parse_from_bytes(err_bytes.as_ref()).unwrap(),
            "FunctionNotFound".to_string()
        );
    }

    #[test]
    fn test_parse_error_not_found() {
        // a random error selector
        let err_bytes = hex::decode("a6bb62dd").unwrap();

        assert_eq!(
            ContractErrorParser::parse_from_bytes(err_bytes.as_ref()),
            Err(ParseContractError::ErrorNotFound {
                selector: "a6bb62dd".to_string()
            })
        );
    }
}
