#[macro_export]
macro_rules! gen_contract_error_mapping {
    ($([$snake_case:tt, $abi:tt]),* $(,)?) => {
        pub mod error_parser {
            use anyhow::anyhow;
            use ethers::utils::hex;

            lazy_static::lazy_static! {
                static ref MAP: std::collections::BTreeMap<String, ethers::abi::ethabi::AbiError> = {
                    let mut errors: std::collections::BTreeMap<String, Vec<ethers::abi::ethabi::AbiError>> = Default::default();

                    $(
                        errors.extend($crate::gen::$snake_case::$snake_case::$abi.errors.clone());
                    )*
                    // the above `errors` is actually indexed by name, now index by selector
                    let mut selector_indexed = std::collections::BTreeMap::default();
                    for (_, v) in errors.iter() {
                        for i in v {
                            let selector = ethers::utils::hex::encode(&i.signature().0[0..4]);
                            selector_indexed.insert(selector, i.clone());
                        }
                    }
                    selector_indexed
                };
            }

            const SOLIDITY_SELECTOR_BYTE_SIZE: usize = 4;

            pub struct ContractErrorParser {}

            impl ContractErrorParser {
                pub fn parse_from_bytes(bytes: &[u8]) -> anyhow::Result<Option<String>> {
                    if bytes.len() < SOLIDITY_SELECTOR_BYTE_SIZE {
                        return Err(anyhow!("error bytes too short: {}", hex::encode(bytes)));
                    }

                    let str = hex::encode(&bytes[0..4]);

                    let Some(error) = MAP.get(&str) else {
                        return Ok(None);
                    };

                    if let Err(e) = error.decode(bytes) {
                        tracing::warn!("contract error selector found: {str}, but decode failed: {e}");
                    }

                    Ok(Some(error.name.clone()))
                }

                pub fn parse_from_hex_str(err: &str) -> anyhow::Result<Option<String>> {
                    let bytes = hex::decode(err)?;
                    Self::parse_from_bytes(bytes.as_slice())
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::error_parser::ContractErrorParser;
    use ethers::utils::hex;

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
