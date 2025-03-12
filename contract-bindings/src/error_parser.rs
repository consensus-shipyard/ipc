use crate::error_map::MAP;
use anyhow::anyhow;
use ethers::utils::hex;

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

        if error.decode(bytes).is_ok() {
            return Ok(Some(error.name.clone()));
        }

        Ok(None)
    }

    pub fn parse_from_hex_str(err: &str) -> anyhow::Result<Option<String>> {
        let bytes = hex::decode(err)?;
        Self::parse_from_bytes(bytes.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use crate::ContractErrorParser;
    use ethers::utils::hex;

    #[test]
    fn parse_error_ok() {
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
}
