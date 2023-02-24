//! This file contains the various response types to be used byt the lotus api.

use cid::Cid;
use fvm_shared::address::Address;
use fvm_shared::econ::TokenAmount;
use fvm_shared::MethodNum;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use anyhow::anyhow;
use serde_json::Value;
use strum::{AsRefStr, Display, EnumString};

/// Exec actor parameters
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ExecParams {
    pub code_cid: Cid,
    pub constructor_params: Vec<u8>,
}

/// Install actor parameters
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct InstallActorParams {
    pub code: Vec<u8>,
}

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

/// Helper struct to interact with lotus node
#[derive(Deserialize, Serialize, Debug)]
pub struct CIDMap {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "/")]
    pub cid: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct StateWaitMsgResponse {
    #[allow(dead_code)]
    message: CIDMap,
    #[allow(dead_code)]
    receipt: Receipt,
    #[allow(dead_code)]
    tip_set: Vec<CIDMap>,
    #[allow(dead_code)]
    height: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ReadStateResponse<State> {
    #[allow(dead_code)]
    pub balance: String,
    #[allow(dead_code)]
    pub code: CIDMap,
    #[allow(dead_code)]
    pub state: State,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Receipt {
    #[allow(dead_code)]
    exit_code: u32,
    #[allow(dead_code)]
    r#return: String,
    #[allow(dead_code)]
    gas_used: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct MpoolPushMessageResponse {
    pub message: MpoolPushMessageResponseInner,
    #[serde(rename = "CID")]
    pub cid: CIDMap,
}

/// The internal message payload that node rpc sends for `MpoolPushMessageResponse`.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct MpoolPushMessageResponseInner {
    to: String,
    from: String,
    pub value: String,
    pub method: MethodNum,
    pub params: String,

    pub nonce: u64,
    #[serde(rename = "GasLimit")]
    pub gas_limit: u64,
    #[serde(rename = "GasFeeCap")]
    pub gas_fee_cap: String,
    #[serde(rename = "GasPremium")]
    pub gas_premium: String,
    pub version: u16,

    #[serde(rename = "CID")]
    pub cid: CIDMap,
}

impl MpoolPushMessageResponseInner {
    pub fn get_root_cid(&self) -> Option<Cid> {
        self.cid
            .cid
            .as_ref()
            .map(|s| Cid::from_str(s).expect("server sent invalid cid"))
    }

    pub fn to(&self) -> anyhow::Result<Address> {
        Ok(Address::from_str(&self.to)?)
    }

    pub fn from(&self) -> anyhow::Result<Address> {
        Ok(Address::from_str(&self.from)?)
    }
}

pub struct MpoolPushMessage {
    pub to: Address,
    pub from: Address,
    pub value: TokenAmount,
    pub method: MethodNum,
    pub params: Vec<u8>,

    pub nonce: Option<u64>,
    pub gas_limit: Option<TokenAmount>,
    pub gas_fee_cap: Option<TokenAmount>,
    pub gas_premium: Option<TokenAmount>,
    pub cid: Option<Cid>,
    pub version: Option<u16>,
    pub max_fee: Option<TokenAmount>,
}

impl MpoolPushMessage {
    pub fn new(to: Address, from: Address, method: MethodNum, params: Vec<u8>) -> Self {
        MpoolPushMessage {
            to,
            from,
            method,
            params,
            value: TokenAmount::from_atto(0),
            nonce: None,
            gas_limit: None,
            gas_fee_cap: None,
            gas_premium: None,
            cid: None,
            version: None,
            max_fee: None,
        }
    }
}

/// A simplified struct representing a `ChainHead` response that does not decode the `blocks` field.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ChainHeadResponse {
    #[allow(dead_code)]
    pub cids: Vec<CIDMap>,
    #[allow(dead_code)]
    pub blocks: Vec<Value>,
    #[allow(dead_code)]
    pub height: u64,
}

impl TryFrom<CIDMap> for Cid {
    type Error = anyhow::Error;

    fn try_from(cid_map: CIDMap) -> Result<Self, Self::Error> {
        let cid_option: Option<Cid> = cid_map.into();
        cid_option.ok_or(anyhow!("cid not found"))
    }
}

impl From<CIDMap> for Option<Cid> {
    fn from(m: CIDMap) -> Self {
        m.cid
            .map(|cid| Cid::from_str(&cid).expect("invalid cid str"))
    }
}

impl From<Option<Cid>> for CIDMap {
    fn from(c: Option<Cid>) -> Self {
        c.map(|cid| CIDMap {
            cid: Some(cid.to_string()),
        })
        .unwrap_or(CIDMap { cid: None })
    }
}

impl From<Cid> for CIDMap {
    fn from(c: Cid) -> Self {
        CIDMap {
            cid: Some(c.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::lotus::message::WalletKeyType;
    use std::str::FromStr;

    #[test]
    fn test_key_types() {
        let t = WalletKeyType::Secp256k1;
        assert_eq!(t.as_ref(), "secp256k1");

        let t = WalletKeyType::from_str(t.as_ref()).unwrap();
        assert_eq!(t, WalletKeyType::Secp256k1);
    }
}
