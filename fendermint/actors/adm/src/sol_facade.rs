use fil_actors_runtime::runtime::Runtime;
use fil_actors_runtime::{actor_error, ActorError};
use fvm_ipld_encoding::tuple::{Deserialize_tuple, Serialize_tuple};
use fvm_ipld_encoding::{strict_bytes, tuple::*};
use fvm_shared::address::Address;
use recall_sol_facade::machine as sol;
use recall_sol_facade::machine::{listBuckets_0Call, listBuckets_1Call, Calls};
use recall_sol_facade::types::{Address as SolAddress, SolCall, SolInterface, H160};
use std::collections::HashMap;

use crate::{CreateExternalParams, CreateExternalReturn, Kind, ListMetadataParams, Metadata};

pub fn can_handle(input_data: &InputData) -> bool {
    Calls::valid_selector(input_data.selector())
}

pub fn parse_input(input: &InputData) -> Result<Calls, ActorError> {
    Calls::abi_decode_raw(input.selector(), input.calldata(), true)
        .map_err(|e| actor_error!(illegal_argument, format!("invalid call: {}", e)))
}

impl AbiCallRuntime for sol::createBucket_0Call {
    type Params = CreateExternalParams;
    type Returns = CreateExternalReturn;
    type Output = Vec<u8>;

    fn params(&self, rt: &impl Runtime) -> Self::Params {
        CreateExternalParams {
            owner: rt.message().caller(),
            kind: Kind::Bucket,
            metadata: HashMap::default(),
        }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        let address = returns
            .robust_address
            .map(|address| H160::try_from(address).unwrap_or_default())
            .unwrap_or_default();
        let address: SolAddress = address.into();
        Self::abi_encode_returns(&(address,))
    }
}

impl AbiCall for sol::createBucket_1Call {
    type Params = CreateExternalParams;
    type Returns = CreateExternalReturn;
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        let owner: Address = H160::from(self.owner).into();
        let mut metadata = HashMap::with_capacity(self.metadata.len());
        for kv in self.metadata.clone() {
            metadata.insert(kv.key, kv.value);
        }
        CreateExternalParams { owner, kind: Kind::Bucket, metadata }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        let address = returns
            .robust_address
            .map(|address| H160::try_from(address).unwrap_or_default())
            .unwrap_or_default();
        let address: SolAddress = address.into();
        Self::abi_encode_returns(&(address,))
    }
}

impl AbiCall for sol::createBucket_2Call {
    type Params = CreateExternalParams;
    type Returns = CreateExternalReturn;
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        let owner: Address = H160::from(self.owner).into();
        CreateExternalParams { owner, kind: Kind::Bucket, metadata: HashMap::default() }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        let address = returns
            .robust_address
            .map(|address| H160::try_from(address).unwrap_or_default())
            .unwrap_or_default();
        let address: SolAddress = address.into();
        Self::abi_encode_returns(&(address,))
    }
}

impl AbiCallRuntime for listBuckets_0Call {
    type Params = ListMetadataParams;
    type Returns = Vec<Metadata>;
    type Output = Vec<u8>;

    fn params(&self, rt: &impl Runtime) -> Self::Params {
        ListMetadataParams { owner: rt.message().caller() }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        let machines: Vec<sol::Machine> = returns
            .iter()
            .map(|m| sol::Machine {
                kind: sol_kind(m.kind),
                addr: H160::try_from(m.address).unwrap_or_default().into(),
                metadata: m
                    .metadata
                    .iter()
                    .map(|(k, v)| sol::KeyValue { key: k.clone(), value: v.clone() })
                    .collect(),
            })
            .collect();
        Self::abi_encode_returns(&(machines,))
    }
}

impl AbiCall for listBuckets_1Call {
    type Params = ListMetadataParams;
    type Returns = Vec<Metadata>;
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        ListMetadataParams { owner: H160::from(self.owner).into() }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        let machines: Vec<sol::Machine> = returns
            .iter()
            .map(|m| sol::Machine {
                kind: sol_kind(m.kind),
                addr: H160::try_from(m.address).unwrap_or_default().into(),
                metadata: m
                    .metadata
                    .iter()
                    .map(|(k, v)| sol::KeyValue { key: k.clone(), value: v.clone() })
                    .collect(),
            })
            .collect();
        Self::abi_encode_returns(&(machines,))
    }
}

fn sol_kind(kind: Kind) -> u8 {
    match kind {
        Kind::Bucket => 0,
        Kind::Timehub => 1,
    }
}

// --- Copied from recall_actor_sdk --- //

#[derive(Default, Serialize_tuple, Deserialize_tuple)]
#[serde(transparent)]
pub struct InvokeContractParams {
    #[serde(with = "strict_bytes")]
    pub input_data: Vec<u8>,
}

#[derive(Serialize_tuple, Deserialize_tuple)]
#[serde(transparent)]
pub struct InvokeContractReturn {
    #[serde(with = "strict_bytes")]
    pub output_data: Vec<u8>,
}

/// EVM call with selector (first 4 bytes) and calldata (remaining bytes)
pub struct InputData(Vec<u8>);

impl InputData {
    pub fn selector(&self) -> [u8; 4] {
        let mut selector = [0u8; 4];
        selector.copy_from_slice(&self.0[0..4]);
        selector
    }

    pub fn calldata(&self) -> &[u8] {
        &self.0[4..]
    }
}

impl TryFrom<InvokeContractParams> for InputData {
    type Error = ActorError;

    fn try_from(value: InvokeContractParams) -> Result<Self, Self::Error> {
        if value.input_data.len() < 4 {
            return Err(ActorError::illegal_argument("input too short".to_string()));
        }
        Ok(InputData(value.input_data))
    }
}

pub trait AbiCall {
    type Params;
    type Returns;
    type Output;
    fn params(&self) -> Self::Params;
    fn returns(&self, returns: Self::Returns) -> Self::Output;
}

pub trait AbiCallRuntime {
    type Params;
    type Returns;
    type Output;
    fn params(&self, rt: &impl fil_actors_runtime::runtime::Runtime) -> Self::Params;
    fn returns(&self, returns: Self::Returns) -> Self::Output;
}

#[derive(Debug, Clone)]
pub struct AbiEncodeError {
    message: String,
}

impl From<anyhow::Error> for AbiEncodeError {
    fn from(error: anyhow::Error) -> Self {
        Self { message: format!("failed to abi encode {}", error) }
    }
}

impl From<String> for AbiEncodeError {
    fn from(message: String) -> Self {
        Self { message }
    }
}

impl From<ActorError> for AbiEncodeError {
    fn from(error: ActorError) -> Self {
        Self { message: format!("{}", error) }
    }
}

impl From<AbiEncodeError> for ActorError {
    fn from(error: AbiEncodeError) -> Self {
        actor_error!(serialization, error.message)
    }
}
