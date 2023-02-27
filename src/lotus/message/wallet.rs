use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumString};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Display, EnumString, AsRefStr)]
pub enum WalletKeyType {
    #[strum(serialize = "bls")]
    BLS,
    #[strum(serialize = "secp256k1")]
    Secp256k1,
    #[strum(serialize = "secp256k1-ledger")]
    Secp256k1Ledger,
}

pub type WalletListResponse = Vec<String>;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::lotus::message::wallet::WalletKeyType;

    #[test]
    fn test_key_types() {
        let t = WalletKeyType::Secp256k1;
        assert_eq!(t.as_ref(), "secp256k1");

        let t = WalletKeyType::from_str(t.as_ref()).unwrap();
        assert_eq!(t, WalletKeyType::Secp256k1);
    }
}
