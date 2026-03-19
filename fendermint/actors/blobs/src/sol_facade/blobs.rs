// Copyright 2022-2024 Recall Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use fendermint_actor_blobs_shared::{
    blobs::{
        AddBlobParams, Blob, BlobStatus, DeleteBlobParams, GetBlobParams, OverwriteBlobParams,
        TrimBlobExpiriesParams,
    },
    bytes::B256,
    operators::OperatorInfo,
    GetStatsReturn,
};
use fil_actors_runtime::{actor_error, runtime::Runtime, ActorError};
use fvm_shared::{address::Address, clock::ChainEpoch};
use ipc_storage_actor_sdk::evm::TryIntoEVMEvent;
pub use ipc_storage_sol_facade::blobs::Calls;
use ipc_storage_sol_facade::{
    blobs as sol,
    primitives::U256,
    types::{BigUintWrapper, SolCall, SolInterface, H160},
};
use num_traits::Zero;

use crate::sol_facade::{AbiCall, AbiCallRuntime, AbiEncodeError};

// ----- Events ----- //

pub struct BlobAdded<'a> {
    pub subscriber: Address,
    pub hash: &'a B256,
    pub size: u64,
    pub expiry: ChainEpoch,
    pub bytes_used: u64,
}

impl TryIntoEVMEvent for BlobAdded<'_> {
    type Target = sol::Events;

    fn try_into_evm_event(self) -> Result<Self::Target, anyhow::Error> {
        let subscriber: H160 = self.subscriber.try_into()?;
        Ok(sol::Events::BlobAdded(sol::BlobAdded {
            subscriber: subscriber.into(),
            hash: self.hash.0.into(),
            size: U256::from(self.size),
            expiry: U256::from(self.expiry),
            bytesUsed: U256::from(self.bytes_used),
        }))
    }
}

pub struct BlobPending<'a> {
    pub subscriber: Address,
    pub hash: &'a B256,
    pub source: &'a B256,
}
impl TryIntoEVMEvent for BlobPending<'_> {
    type Target = sol::Events;
    fn try_into_evm_event(self) -> Result<sol::Events, anyhow::Error> {
        let subscriber: H160 = self.subscriber.try_into()?;
        Ok(sol::Events::BlobPending(sol::BlobPending {
            subscriber: subscriber.into(),
            hash: self.hash.0.into(),
            sourceId: self.source.0.into(),
        }))
    }
}

pub struct BlobFinalized<'a> {
    pub subscriber: Address,
    pub hash: &'a B256,
    pub resolved: bool,
}
impl TryIntoEVMEvent for BlobFinalized<'_> {
    type Target = sol::Events;
    fn try_into_evm_event(self) -> Result<sol::Events, anyhow::Error> {
        let subscriber: H160 = self.subscriber.try_into()?;
        Ok(sol::Events::BlobFinalized(sol::BlobFinalized {
            subscriber: subscriber.into(),
            hash: self.hash.0.into(),
            resolved: self.resolved,
        }))
    }
}

pub struct BlobDeleted<'a> {
    pub subscriber: Address,
    pub hash: &'a B256,
    pub size: u64,
    pub bytes_released: u64,
}
impl TryIntoEVMEvent for BlobDeleted<'_> {
    type Target = sol::Events;
    fn try_into_evm_event(self) -> Result<sol::Events, anyhow::Error> {
        let subscriber: H160 = self.subscriber.try_into()?;
        Ok(sol::Events::BlobDeleted(sol::BlobDeleted {
            subscriber: subscriber.into(),
            hash: self.hash.0.into(),
            size: U256::from(self.size),
            bytesReleased: U256::from(self.bytes_released),
        }))
    }
}

// ----- Calls ----- //

pub fn can_handle(input_data: &ipc_storage_actor_sdk::evm::InputData) -> bool {
    Calls::valid_selector(input_data.selector())
}

pub fn parse_input(input: &ipc_storage_actor_sdk::evm::InputData) -> Result<Calls, ActorError> {
    Calls::abi_decode_raw(input.selector(), input.calldata(), true)
        .map_err(|e| actor_error!(illegal_argument, format!("invalid call: {}", e)))
}

pub const REGISTER_NODE_OPERATOR_SELECTOR: [u8; 4] = [0x71, 0x3b, 0x10, 0xcf];
pub const GET_OPERATOR_INFO_SELECTOR: [u8; 4] = [0x27, 0xd9, 0xab, 0x5d];
pub const GET_ACTIVE_OPERATORS_SELECTOR: [u8; 4] = [0x64, 0xbd, 0xc6, 0x7e];

pub struct RegisterNodeOperatorInvokeCall {
    pub bls_pubkey: Vec<u8>,
    pub rpc_url: String,
}

pub struct GetOperatorInfoInvokeCall {
    pub address: Address,
}

pub fn is_register_node_operator_call(input: &ipc_storage_actor_sdk::evm::InputData) -> bool {
    input.selector() == REGISTER_NODE_OPERATOR_SELECTOR
}

pub fn is_get_operator_info_call(input: &ipc_storage_actor_sdk::evm::InputData) -> bool {
    input.selector() == GET_OPERATOR_INFO_SELECTOR
}

pub fn is_get_active_operators_call(input: &ipc_storage_actor_sdk::evm::InputData) -> bool {
    input.selector() == GET_ACTIVE_OPERATORS_SELECTOR
}

pub fn parse_register_node_operator_input(
    input: &ipc_storage_actor_sdk::evm::InputData,
) -> Result<RegisterNodeOperatorInvokeCall, ActorError> {
    let calldata = input.calldata();
    if calldata.len() < 64 {
        return Err(actor_error!(illegal_argument, "invalid call: input too short"));
    }

    let bls_offset = decode_offset(calldata, 0)?;
    let rpc_offset = decode_offset(calldata, 32)?;

    let bls_pubkey = decode_dynamic_bytes(calldata, bls_offset)?;
    let rpc_bytes = decode_dynamic_bytes(calldata, rpc_offset)?;
    let rpc_url = String::from_utf8(rpc_bytes)
        .map_err(|e| actor_error!(illegal_argument, format!("invalid call: bad UTF-8: {}", e)))?;

    Ok(RegisterNodeOperatorInvokeCall {
        bls_pubkey,
        rpc_url,
    })
}

pub fn parse_get_operator_info_input(
    input: &ipc_storage_actor_sdk::evm::InputData,
) -> Result<GetOperatorInfoInvokeCall, ActorError> {
    let calldata = input.calldata();
    if calldata.len() < 32 {
        return Err(actor_error!(illegal_argument, "invalid call: input too short"));
    }
    let word = &calldata[0..32];
    if word[..12].iter().any(|b| *b != 0) {
        return Err(actor_error!(
            illegal_argument,
            "invalid call: malformed address"
        ));
    }
    let address: Address = H160::from_slice(&word[12..32]).into();
    Ok(GetOperatorInfoInvokeCall { address })
}

pub fn encode_get_operator_info_output(info: Option<OperatorInfo>) -> Result<Vec<u8>, ActorError> {
    let (bls_pubkey, rpc_url, active) = if let Some(info) = info {
        (info.bls_pubkey, info.rpc_url.into_bytes(), info.active)
    } else {
        (Vec::new(), Vec::new(), false)
    };

    let bls_section = encode_dynamic_bytes(&bls_pubkey);
    let rpc_section = encode_dynamic_bytes(&rpc_url);

    let head_size = 32 * 3;
    let bls_offset = head_size;
    let rpc_offset = head_size + bls_section.len();

    let mut output = Vec::with_capacity(head_size + bls_section.len() + rpc_section.len());
    output.extend_from_slice(&abi_word_from_usize(bls_offset));
    output.extend_from_slice(&abi_word_from_usize(rpc_offset));
    output.extend_from_slice(&abi_word_from_bool(active));
    output.extend_from_slice(&bls_section);
    output.extend_from_slice(&rpc_section);
    Ok(output)
}

pub fn encode_get_active_operators_output(operators: Vec<Address>) -> Result<Vec<u8>, ActorError> {
    let mut operators_section = Vec::with_capacity(32 + operators.len() * 32);
    operators_section.extend_from_slice(&abi_word_from_usize(operators.len()));
    for operator in operators {
        let h160 = H160::try_from(operator).map_err(|e| {
            actor_error!(
                illegal_argument,
                format!("failed to encode operator address: {}", e)
            )
        })?;
        operators_section.extend_from_slice(&abi_word_from_address(h160));
    }

    let mut output = Vec::with_capacity(32 + operators_section.len());
    output.extend_from_slice(&abi_word_from_usize(32));
    output.extend_from_slice(&operators_section);
    Ok(output)
}

fn decode_offset(calldata: &[u8], at: usize) -> Result<usize, ActorError> {
    let end = at + 32;
    if end > calldata.len() {
        return Err(actor_error!(illegal_argument, "invalid call: malformed offset"));
    }
    let word = &calldata[at..end];
    if word[..24].iter().any(|b| *b != 0) {
        return Err(actor_error!(
            illegal_argument,
            "invalid call: offset too large"
        ));
    }
    let mut n = [0u8; 8];
    n.copy_from_slice(&word[24..32]);
    Ok(u64::from_be_bytes(n) as usize)
}

fn decode_dynamic_bytes(calldata: &[u8], offset: usize) -> Result<Vec<u8>, ActorError> {
    if offset + 32 > calldata.len() {
        return Err(actor_error!(
            illegal_argument,
            "invalid call: dynamic offset out of bounds"
        ));
    }

    let len = decode_offset(calldata, offset)?;
    let start = offset + 32;
    let end = start
        .checked_add(len)
        .ok_or_else(|| actor_error!(illegal_argument, "invalid call: overflow"))?;

    if end > calldata.len() {
        return Err(actor_error!(
            illegal_argument,
            "invalid call: dynamic value out of bounds"
        ));
    }

    Ok(calldata[start..end].to_vec())
}

fn abi_word_from_usize(value: usize) -> [u8; 32] {
    let mut word = [0u8; 32];
    word[24..32].copy_from_slice(&(value as u64).to_be_bytes());
    word
}

fn abi_word_from_bool(value: bool) -> [u8; 32] {
    let mut word = [0u8; 32];
    word[31] = u8::from(value);
    word
}

fn abi_word_from_address(value: H160) -> [u8; 32] {
    let mut word = [0u8; 32];
    word[12..32].copy_from_slice(&value.to_fixed_bytes());
    word
}

fn encode_dynamic_bytes(value: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(32 + padded_32_len(value.len()));
    out.extend_from_slice(&abi_word_from_usize(value.len()));
    out.extend_from_slice(value);
    let padding = padded_32_len(value.len()) - value.len();
    out.extend(std::iter::repeat(0u8).take(padding));
    out
}

fn padded_32_len(size: usize) -> usize {
    if size == 0 { 0 } else { size.div_ceil(32) * 32 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn address_word(bytes20: [u8; 20]) -> [u8; 32] {
        let mut word = [0u8; 32];
        word[12..32].copy_from_slice(&bytes20);
        word
    }

    #[test]
    fn parses_get_operator_info_input_address() {
        let addr = [0x11u8; 20];
        let mut input = Vec::new();
        input.extend_from_slice(&GET_OPERATOR_INFO_SELECTOR);
        input.extend_from_slice(&address_word(addr));
        let input =
            ipc_storage_actor_sdk::evm::InputData::try_from(ipc_storage_actor_sdk::evm::InvokeContractParams {
                input_data: input,
            })
            .expect("valid input");

        let parsed = parse_get_operator_info_input(&input).expect("parse succeeds");
        let expected = Address::new_delegated(10, &addr).expect("delegated");
        assert_eq!(parsed.address, expected);
    }

    #[test]
    fn encodes_get_active_operators_output_as_address_array() {
        let id = Address::new_id(66);
        let delegated = Address::new_delegated(10, &[0x22; 20]).expect("delegated");

        let encoded = encode_get_active_operators_output(vec![id, delegated]).expect("encode");

        assert_eq!(&encoded[0..32], &abi_word_from_usize(32));
        assert_eq!(&encoded[32..64], &abi_word_from_usize(2));

        let id_h160 = H160::try_from(id).expect("id to h160");
        let delegated_h160 = H160::try_from(delegated).expect("delegated to h160");
        assert_eq!(&encoded[64..96], &abi_word_from_address(id_h160));
        assert_eq!(&encoded[96..128], &abi_word_from_address(delegated_h160));
    }

    #[test]
    fn encodes_get_operator_info_output_tuple() {
        let info = OperatorInfo {
            bls_pubkey: vec![1, 2, 3, 4],
            rpc_url: "http://127.0.0.1:8081".to_string(),
            active: true,
        };
        let encoded = encode_get_operator_info_output(Some(info)).expect("encode");

        assert_eq!(&encoded[0..32], &abi_word_from_usize(96));
        let bls_section_len = 32 + 32; // len + padded data for 4 bytes
        assert_eq!(
            &encoded[32..64],
            &abi_word_from_usize(96 + bls_section_len)
        );
        assert_eq!(&encoded[64..96], &abi_word_from_bool(true));
    }
}

fn blob_status_as_solidity_enum(blob_status: BlobStatus) -> u8 {
    match blob_status {
        BlobStatus::Added => 0,
        BlobStatus::Pending => 1,
        BlobStatus::Resolved => 2,
        BlobStatus::Failed => 3,
    }
}

impl AbiCallRuntime for sol::addBlobCall {
    type Params = Result<AddBlobParams, AbiEncodeError>;
    type Returns = ();
    type Output = Vec<u8>;
    fn params(&self, rt: &impl Runtime) -> Self::Params {
        let sponsor: Option<Address> = H160::from(self.sponsor).as_option().map(|a| a.into());
        let source = B256(self.source.into());
        let hash = B256(self.blobHash.into());
        let metadata_hash = B256(self.metadataHash.into());
        let subscription_id = self.subscriptionId.clone().try_into()?;
        let size = self.size;
        let ttl = if self.ttl.is_zero() {
            None
        } else {
            Some(self.ttl as ChainEpoch)
        };
        let from = rt.message().caller();
        Ok(AddBlobParams {
            sponsor,
            source,
            hash,
            metadata_hash,
            id: subscription_id,
            size,
            ttl,
            from,
            data_shards: self.dataShards,
            parity_shards: self.parityShards,
        })
    }
    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&returns)
    }
}

impl AbiCallRuntime for sol::deleteBlobCall {
    type Params = Result<DeleteBlobParams, AbiEncodeError>;
    type Returns = ();
    type Output = Vec<u8>;
    fn params(&self, rt: &impl Runtime) -> Self::Params {
        let subscriber = H160::from(self.subscriber).as_option().map(|a| a.into());
        let hash = B256(self.blobHash.into());
        let subscription_id = self.subscriptionId.clone().try_into()?;
        let from = rt.message().caller();
        Ok(DeleteBlobParams {
            sponsor: subscriber,
            hash,
            id: subscription_id,
            from,
        })
    }
    fn returns(&self, _: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&())
    }
}

impl AbiCall for sol::getBlobCall {
    type Params = Result<GetBlobParams, AbiEncodeError>;
    type Returns = Option<Blob>;
    type Output = Result<Vec<u8>, AbiEncodeError>;
    fn params(&self) -> Self::Params {
        let blob_hash = B256(self.blobHash.into());
        Ok(GetBlobParams(blob_hash))
    }
    fn returns(&self, blob: Self::Returns) -> Self::Output {
        let blob = if let Some(blob) = blob {
            sol::Blob {
                size: blob.size,
                metadataHash: blob.metadata_hash.0.into(),
                status: blob_status_as_solidity_enum(blob.status),
                subscriptions: blob
                    .subscribers
                    .iter()
                    .map(|(subscription_id, expiry)| sol::Subscription {
                        expiry: *expiry as u64,
                        subscriptionId: subscription_id.clone().into(),
                    })
                    .collect(),
            }
        } else {
            sol::Blob {
                size: 0,
                metadataHash: [0u8; 32].into(),
                status: blob_status_as_solidity_enum(BlobStatus::Failed),
                subscriptions: Vec::default(),
            }
        };
        Ok(Self::abi_encode_returns(&(blob,)))
    }
}

impl AbiCall for sol::getStatsCall {
    type Params = ();
    type Returns = GetStatsReturn;
    type Output = Vec<u8>;
    fn params(&self) -> Self::Params {}
    fn returns(&self, stats: Self::Returns) -> Self::Output {
        let subnet_stats = sol::SubnetStats {
            balance: BigUintWrapper::from(stats.balance).into(),
            capacityFree: stats.capacity_free,
            capacityUsed: stats.capacity_used,
            creditSold: BigUintWrapper::from(stats.credit_sold).into(),
            creditCommitted: BigUintWrapper::from(stats.credit_committed).into(),
            creditDebited: BigUintWrapper::from(stats.credit_debited).into(),
            tokenCreditRate: BigUintWrapper(stats.token_credit_rate.rate().clone()).into(),
            numAccounts: stats.num_accounts,
            numBlobs: stats.num_blobs,
            numAdded: stats.num_added,
            bytesAdded: stats.bytes_added,
            numResolving: stats.num_resolving,
            bytesResolving: stats.bytes_resolving,
        };
        Self::abi_encode_returns(&(subnet_stats,))
    }
}

impl AbiCallRuntime for sol::overwriteBlobCall {
    type Params = Result<OverwriteBlobParams, AbiEncodeError>;
    type Returns = ();
    type Output = Vec<u8>;
    fn params(&self, rt: &impl Runtime) -> Self::Params {
        let old_hash = B256(self.oldHash.into());
        let sponsor = H160::from(self.sponsor).as_option().map(|a| a.into());
        let source = B256(self.source.into());
        let hash = B256(self.blobHash.into());
        let metadata_hash = B256(self.metadataHash.into());
        let subscription_id = self.subscriptionId.clone().try_into()?;
        let size = self.size;
        let ttl = if self.ttl.is_zero() {
            None
        } else {
            Some(self.ttl as ChainEpoch)
        };
        let from = rt.message().caller();
        Ok(OverwriteBlobParams {
            old_hash,
            add: AddBlobParams {
                sponsor,
                source,
                hash,
                metadata_hash,
                id: subscription_id,
                size,
                ttl,
                from,
                data_shards: self.dataShards,
                parity_shards: self.parityShards,
            },
        })
    }
    fn returns(&self, returns: Self::Returns) -> Self::Output {
        Self::abi_encode_returns(&returns)
    }
}

impl AbiCall for sol::trimBlobExpiriesCall {
    type Params = TrimBlobExpiriesParams;
    type Returns = (u32, Option<B256>);
    type Output = Vec<u8>;

    fn params(&self) -> Self::Params {
        let limit = self.limit;
        let limit = if limit.is_zero() { None } else { Some(limit) };
        let hash: [u8; 32] = self.startingHash.into();
        let hash = if hash == [0; 32] {
            None
        } else {
            Some(B256(hash))
        };
        TrimBlobExpiriesParams {
            subscriber: H160::from(self.subscriber).into(),
            limit,
            starting_hash: hash,
        }
    }

    fn returns(&self, returns: Self::Returns) -> Self::Output {
        let next_key = returns.1;
        let next_key = next_key.unwrap_or_default();
        let cursor = sol::TrimBlobExpiries {
            processed: returns.0,
            nextKey: next_key.0.into(),
        };
        Self::abi_encode_returns(&(cursor,))
    }
}
