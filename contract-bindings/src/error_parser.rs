use ethers::abi::ethabi::AbiError;
use std::collections::BTreeMap;

#[macro_export]
macro_rules! extend_contract_error_mapping {
    ($([$snake_case:tt, $abi:tt]),* $(,)?) => {
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

pub fn extend_errors(
    map: &mut BTreeMap<String, AbiError>,
    contract_errors: BTreeMap<String, Vec<AbiError>>,
) {
    for (_, v) in contract_errors.iter() {
        for i in v {
            // solidity selector is only the first 4 bytes of the signature
            let selector = const_hex::hex::encode(&i.signature().0[0..SOLIDITY_SELECTOR_BYTE_SIZE]);
            map.insert(selector, i.clone());
        }
    }
}

pub struct ContractErrorParser {}

impl ContractErrorParser {
    pub fn parse_from_bytes(bytes: &[u8]) -> anyhow::Result<Option<String>> {
        if bytes.len() < SOLIDITY_SELECTOR_BYTE_SIZE {
            return Err(anyhow::anyhow!(
                "error bytes too short: {}",
                const_hex::hex::encode(bytes)
            ));
        }

        let str = const_hex::hex::encode(&bytes[0..4]);

        let Some(error) = crate::gen::MAP.get(&str) else {
            return Ok(None);
        };
        if let Err(e) = error.decode(bytes) {
            tracing::warn!("contract error selector found: {str}, but decode failed: {e}");
        }

        Ok(Some(error.name.clone()))
    }

    pub fn parse_from_hex_str(err: &str) -> anyhow::Result<Option<String>> {
        let bytes = const_hex::hex::decode(err)?;
        Self::parse_from_bytes(bytes.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use crate::error_parser::ContractErrorParser;
    use const_hex::hex;

    #[test]
    fn test_parse_error_ok() {
        // selector for "BottomUpCheckpointAlreadySubmitted" error
        let err_bytes = hex::decode("d6bb62dd").unwrap();

        assert_eq!(
            ContractErrorParser::parse_from_bytes(err_bytes.as_ref()).unwrap(),
            Some("BottomUpCheckpointAlreadySubmitted".into())
        );

        // selector for "FunctionNotFound" error
        let err_bytes =
            hex::decode("5416eb98611941f900000000000000000000000000000000000000000000000000000000")
                .unwrap();

        assert_eq!(
            ContractErrorParser::parse_from_bytes(err_bytes.as_ref()).unwrap(),
            Some("FunctionNotFound".into())
        );
    }

    #[test]
    fn test_parse_error_not_found() {
        // a random error selector
        let err_bytes = hex::decode("a6bb62dd").unwrap();

        assert_eq!(
            ContractErrorParser::parse_from_bytes(err_bytes.as_ref()).unwrap(),
            None
        );
    }
}
