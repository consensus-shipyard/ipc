// Copyright 2024 Hoku Contributors
// Copyright 2022-2024 Protocol Labs
// SPDX-License-Identifier: Apache-2.0, MIT

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use fil_actors_evm_shared::{address::EthAddress, uints::U256};
use fil_actors_runtime::{actor_error, runtime::Runtime, ActorError, EAM_ACTOR_ID};
use fvm_ipld_encoding::IPLD_RAW;
use fvm_shared::{
    address::{Address, Payload},
    event::{ActorEvent, Flags},
};
use multihash::{Hasher, Keccak256};

/// The event key for the Ethereum log data.
const EVENT_DATA_KEY: &str = "d";

/// The event keys for the Ethereum log topics.
const EVENT_TOPIC_KEYS: &[&str] = &["t1", "t2", "t3", "t4"];

/// Trait for types that compose an EVM event.
pub trait EvmEventable {
    /// The EVM param type to use for this type.
    const PARAM_TYPE: EvmParamType;

    /// Returns the EVM param type.
    fn evm_type(&self) -> EvmParamType {
        Self::PARAM_TYPE
    }

    /// Retuns bytes that will be written to the EVM event.
    fn encode(&self) -> Result<Vec<u8>, ActorError>;
}

impl EvmEventable for [u8; 32] {
    const PARAM_TYPE: EvmParamType = EvmParamType::Bytes32;

    fn encode(&self) -> Result<Vec<u8>, ActorError> {
        Ok(self.to_vec())
    }
}

impl EvmEventable for U256 {
    const PARAM_TYPE: EvmParamType = EvmParamType::Uint256;

    fn encode(&self) -> Result<Vec<u8>, ActorError> {
        let mut buf = vec![0u8; 32];
        self.to_big_endian(&mut buf);
        Ok(buf)
    }
}

impl EvmEventable for Address {
    const PARAM_TYPE: EvmParamType = EvmParamType::Address;

    fn encode(&self) -> Result<Vec<u8>, ActorError> {
        Ok(to_eth_address(self)?.to_fixed_bytes().to_vec())
    }
}

fn to_eth_address(addr: &Address) -> Result<H160, ActorError> {
    match addr.payload() {
        Payload::Delegated(d) if d.namespace() == EAM_ACTOR_ID && d.subaddress().len() == 20 => {
            Ok(H160::from_slice(d.subaddress()))
        }
        Payload::ID(id) => Ok(H160::from_slice(&EthAddress::from_id(*id).0)),
        _ => Err(actor_error!(illegal_argument; "not an Ethereum address: {}", addr)),
    }
}

/// Fixed-size uninterpreted hash type with 20 bytes (160 bits) size.
struct H160([u8; 20]);

impl H160 {
    fn from_slice(slice: &[u8]) -> Self {
        if slice.len() != 20 {
            panic!("slice length must be exactly 20 bytes");
        }
        let mut buf = [0u8; 20];
        buf.copy_from_slice(slice);
        H160(buf)
    }

    fn to_fixed_bytes(&self) -> [u8; 20] {
        self.0
    }
}

impl fmt::Debug for H160 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "H160({:?})", &self.0)
    }
}

impl EvmEventable for Vec<u8> {
    const PARAM_TYPE: EvmParamType = EvmParamType::Bytes;

    fn encode(&self) -> Result<Vec<u8>, ActorError> {
        Ok(self.to_vec())
    }
}

impl EvmEventable for String {
    const PARAM_TYPE: EvmParamType = EvmParamType::String;

    fn encode(&self) -> Result<Vec<u8>, ActorError> {
        Ok(self.as_bytes().to_vec())
    }
}

impl EvmEventable for &str {
    const PARAM_TYPE: EvmParamType = EvmParamType::String;

    fn encode(&self) -> Result<Vec<u8>, ActorError> {
        Ok(self.as_bytes().to_vec())
    }
}

/// A list of possible EVM event param types.
#[derive(Clone, Copy, Debug)]
pub enum EvmParamType {
    Bytes32,
    Uint256,
    Address,
    Bytes,
    String,
}

impl EvmParamType {
    fn fixed_size(&self) -> bool {
        match self {
            EvmParamType::Bytes32 => true,
            EvmParamType::Uint256 => true,
            EvmParamType::Address => true,
            EvmParamType::Bytes => false,
            EvmParamType::String => false,
        }
    }
}

impl fmt::Display for EvmParamType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            EvmParamType::Bytes32 => "bytes32",
            EvmParamType::Uint256 => "uint256",
            EvmParamType::Address => "address",
            EvmParamType::Bytes => "bytes",
            EvmParamType::String => "string",
        };
        write!(f, "{}", name)
    }
}

impl FromStr for EvmParamType {
    type Err = ActorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bytes32" => Ok(EvmParamType::Bytes32),
            "uint256" => Ok(EvmParamType::Uint256),
            "address" => Ok(EvmParamType::Address),
            "bytes" => Ok(EvmParamType::Bytes),
            "string" => Ok(EvmParamType::String),
            _ => Err(actor_error!(illegal_argument; "invalid EVM param in signature: {}", s)),
        }
    }
}

/// Helper struct for managing event params.
#[derive(Clone, Debug)]
struct Entry {
    evm_type: EvmParamType,
    indexed: bool,
    value: Result<Vec<u8>, ActorError>,
}

/// An EVM event builder.
#[derive(Clone, Debug)]
pub struct EventBuilder {
    signature: &'static str,
    entries: Vec<Entry>,
}

impl EventBuilder {
    /// Creates a new EVM event builder.
    /// Signature must match the provided params, e.g., "Transfer(address,address,uint256)".
    pub fn new(signature: &'static str) -> Self {
        Self {
            signature,
            entries: Vec::new(),
        }
    }

    /// Adds a non-indexed parameter and returns the builder for chaining.
    pub fn param<T: EvmEventable>(mut self, value: T) -> Self {
        self.param_mut(value, false);
        self
    }

    /// Adds an indexed parameter and returns the builder for chaining.
    pub fn param_indexed<T: EvmEventable>(mut self, value: T) -> Self {
        self.param_mut(value, true);
        self
    }

    /// Adds a parameter to the event.
    fn param_mut<T: EvmEventable>(&mut self, value: T, index: bool) {
        self.entries.push(Entry {
            evm_type: value.evm_type(),
            indexed: index,
            value: value.encode(),
        });
    }

    /// Returns an actor event ready to emit (consuming self).
    pub fn build(&self) -> Result<ActorEvent, ActorError> {
        let mut entries: Vec<fvm_shared::event::Entry> = Vec::with_capacity(self.entries.len());
        let mut data_entries = vec![];

        // The first topic is the event signature
        let sig = self.evm_sig()?;
        let entry = to_fvm_entry(EVENT_TOPIC_KEYS[0], &sig);
        entries.push(entry);

        // Collect topics
        let mut t = 1;
        for e in self.entries.iter() {
            if e.indexed && t < EVENT_TOPIC_KEYS.len() {
                let value = e.value.as_ref().map_err(|e| e.to_owned())?;
                let value = to_evm_word(value, e.evm_type.fixed_size())?;
                let entry = to_fvm_entry(EVENT_TOPIC_KEYS[t], &value);
                entries.push(entry);
                t += 1;
            } else {
                data_entries.push(e);
            }
        }

        // Collect unindexed params as data
        if !data_entries.is_empty() {
            let value = to_evm_data(data_entries, entries.len())?;
            let entry = to_fvm_entry(EVENT_DATA_KEY, &value);
            entries.push(entry);
        }

        Ok(entries.into())
    }

    /// Build and emit the event.
    pub fn emit(&self, rt: &impl Runtime) -> Result<(), ActorError> {
        let event = self.build()?;
        rt.emit_event(&event)
    }

    /// Returns a hash corresponding to the implied Ethereum event signature.
    fn evm_sig(&self) -> Result<Vec<u8>, ActorError> {
        let types = self
            .entries
            .iter()
            .map(|e| e.evm_type.to_string())
            .collect::<Vec<_>>()
            .join(",");
        validate_sig(&self.signature, &types)?;
        let mut hasher = Keccak256::default();
        hasher.update(self.signature.as_bytes());
        Ok(hasher.finalize().to_vec())
    }
}

/// Validates that the user-provided event signature matches the implied event signature from
/// the added params.
fn validate_sig(sig: &str, types: &str) -> Result<(), ActorError> {
    // Extract params from the provided signature
    let sig_params = if let Some(start) = sig.find('(') {
        sig.rfind(')').map(|end| &sig[start + 1..end])
    } else {
        None
    }
    .ok_or_else(|| actor_error!(illegal_argument; "invalid event signature: {}", sig))?;

    // Validate params
    if !sig_params.is_empty() {
        let parsed: Vec<&str> = sig_params.split(',').map(|s| s.trim()).collect();
        for p in parsed {
            p.parse::<EvmParamType>()?;
        }
    }

    // Check if params match
    if sig_params != types {
        Err(
            actor_error!(illegal_argument; "invalid event signature for params '{}': {}", types, sig),
        )
    } else {
        Ok(())
    }
}

/// Returns an FVM event entry for the given key and value.
fn to_fvm_entry(key: &str, value: &Vec<u8>) -> fvm_shared::event::Entry {
    fvm_shared::event::Entry {
        flags: Flags::FLAG_INDEXED_ALL,
        key: (*key).to_owned(),
        codec: IPLD_RAW,
        value: value.to_owned(),
    }
}

/// Pads or hashes data to an EVM "word", which is 32 bytes.
/// Fixed-sized types are left-padded.
/// Dynamic types are right-padded if their length is less than 32 bytes.
/// Dynamic types are hashed if their length is greater than 32 bytes.
fn to_evm_word(data: &[u8], fixed_size: bool) -> Result<Vec<u8>, ActorError> {
    let len = data.len();
    match len.cmp(&32) {
        Ordering::Greater => {
            let mut hasher = Keccak256::default();
            hasher.update(data);
            Ok(hasher.finalize().to_vec())
        }
        Ordering::Less => {
            let mut buf = vec![0u8; 32];
            if fixed_size {
                buf[32 - len..].copy_from_slice(data);
            } else {
                buf[..len].copy_from_slice(data);
            }
            Ok(buf)
        }
        Ordering::Equal => Ok(data.to_owned()),
    }
}

/// Returns the "data" portion of an EVM event, which contains all non-indexed params.
/// See https://docs.soliditylang.org/en/latest/abi-spec.html.
fn to_evm_data(entries: Vec<&Entry>, num_topics: usize) -> Result<Vec<u8>, ActorError> {
    let mut head = Vec::new();
    let mut tail = Vec::new();
    let data_offset = (entries.len() + num_topics) * 32;

    for entry in entries {
        let encoded = entry.value.as_ref().map_err(|e| e.to_owned())?;
        if entry.evm_type.fixed_size() {
            // Fixed-sized types go directly into the head
            head.extend(to_evm_word(encoded, true)?);
        } else {
            // Dynamic-sized types get offset written to head
            let offset = ((data_offset + tail.len()) as u64).to_be_bytes();
            head.extend(to_evm_word(&offset, true)?);
            // Encoded length is added to the tail
            let length = (encoded.len() as u64).to_be_bytes();
            tail.extend(to_evm_word(&length, true)?);
            // Add encoded data to the tail and pad to 32-byte boundary
            tail.extend(encoded);
            while tail.len() % 32 != 0 {
                tail.push(0);
            }
        }
    }

    // Combine head and tail to form the final encoded data field
    head.extend(tail);
    Ok(head)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct BuildTest {
        signature: &'static str,
        arrays: Vec<([u8; 32], bool)>,
        uints: Vec<(U256, bool)>,
        addresses: Vec<(Address, bool)>,
        bytes: Vec<(Vec<u8>, bool)>,
        strings: Vec<(&'static str, bool)>,
        result: BuildTestResult,
    }

    impl BuildTest {
        fn build(&self, builder: &mut EventBuilder) {
            for v in self.arrays.iter() {
                builder.param_mut(v.0, v.1);
            }
            for v in self.uints.iter() {
                builder.param_mut(v.0, v.1);
            }
            for v in self.addresses.iter() {
                builder.param_mut(v.0, v.1);
            }
            for v in self.bytes.iter() {
                builder.param_mut(v.0.clone(), v.1);
            }
            for v in self.strings.iter() {
                builder.param_mut(v.0, v.1);
            }
        }
    }

    #[derive(Clone)]
    enum BuildTestResult {
        Success(Vec<&'static str>, Option<&'static str>),
        Failure(&'static str),
    }

    impl Default for BuildTestResult {
        fn default() -> Self {
            Self::Failure("")
        }
    }

    fn to_delegated_address(hex: &str) -> Address {
        let eth_addr = hex::decode(hex).unwrap();
        Address::new_delegated(EAM_ACTOR_ID, &eth_addr).unwrap()
    }

    /// This mimics how the IPC Ethereum API builds an EVM log from an actor event.  
    fn get_topics_and_data(event: &ActorEvent) -> (Vec<String>, Option<String>) {
        let mut topics = vec![];
        let mut data = None;

        for entry in event.entries.iter() {
            match entry.key.as_str() {
                "t1" | "t2" | "t3" | "t4" => {
                    if entry.value.len() != 32 {
                        panic!("invalid topic length");
                    }
                    let i = entry.key[1..].parse::<usize>().unwrap().saturating_sub(1);
                    while topics.len() <= i {
                        topics.push(String::new());
                    }
                    topics[i] = hex::encode(&entry.value);
                }
                "d" => data = Some(hex::encode(&entry.value)),
                _ => {
                    panic!("invalid event key");
                }
            }
        }

        if topics.is_empty() {
            panic!("no topics");
        } else if topics.len() > 4 {
            panic!("too many topics");
        }

        (topics, data)
    }

    #[test]
    fn test_varied_event_building() {
        let tests = vec![
            // Minimal signature
            BuildTest {
                signature: "TestEvent()",
                result: BuildTestResult::Success(
                    vec!["24ec1d3ff24c2f6ff210738839dbc339cd45a5294d85c79361016243157aae7b"],
                    None,
                ),
                ..Default::default()
            },
            // Single fixed-size parameter
            BuildTest {
                signature: "TestEvent(bytes32)",
                arrays: vec![([1u8; 32], false)],
                result: BuildTestResult::Success(
                    vec!["2d87364d1542bf89b684ede9ddff45aed45971c6f05deaca687bd3d1b6caf1c3"],
                    Some("0101010101010101010101010101010101010101010101010101010101010101"),
                ),
                ..Default::default()
            },
            // Single dynamic-size parameter
            BuildTest {
                signature: "TestEvent(bytes)",
                bytes: vec![(vec![1u8, 2u8, 3u8], false)],
                result: BuildTestResult::Success(
                    vec!["f6909898a012721c5ec53dd666c9114f4e7a1dd8548777674de87fb03a191791"],
                    Some("000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000030102030000000000000000000000000000000000000000000000000000000000"),
                ),
                ..Default::default()
            },
            // All parameter types, non-indexed
            BuildTest {
                signature: "TestEvent(bytes32,uint256,address,bytes,string)",
                arrays: vec![([2u8; 32], false)],
                uints: vec![(U256::from(42), false)],
                addresses: vec![(
                    to_delegated_address("bd770416a3345f91e4b34576cb804a576fa48eb1"),
                    false,
                )],
                bytes: vec![(vec![4u8, 5u8, 6u8], false)],
                strings: vec![("NonIndexedString", false)],
                result: BuildTestResult::Success(
                    vec!["b72a26b25954e89192a98625d6edc5cd3fa3c662f7c1ef861aab9b23df9d1fe2"],
                    Some("0202020202020202020202020202020202020202020202020202020202020202000000000000000000000000000000000000000000000000000000000000002a000000000000000000000000bd770416a3345f91e4b34576cb804a576fa48eb100000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000003040506000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000104e6f6e496e6465786564537472696e6700000000000000000000000000000000"),
                ),
            },
            // All parameter types, indexed
            BuildTest {
                signature: "TestEvent(bytes32,uint256,address,bytes,string)",
                arrays: vec![([3u8; 32], true)],
                uints: vec![(U256::from(56), true)],
                addresses: vec![(
                    to_delegated_address("9a5b23dcd30a7c5e4b4e2b645daf6fae22991d2e"),
                    true,
                )],
                bytes: vec![(vec![7u8, 8u8, 9u8], true)],
                strings: vec![("IndexedString", true)],
                result: BuildTestResult::Success(
                    vec![
                        "b72a26b25954e89192a98625d6edc5cd3fa3c662f7c1ef861aab9b23df9d1fe2",
                        "0303030303030303030303030303030303030303030303030303030303030303",
                        "0000000000000000000000000000000000000000000000000000000000000038",
                        "0000000000000000000000009a5b23dcd30a7c5e4b4e2b645daf6fae22991d2e",
                    ],
                    Some("00000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000030708090000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000d496e6465786564537472696e6700000000000000000000000000000000000000")
                ),
            },
            // Mixed indexed and non-indexed parameters
            BuildTest {
                signature: "TestEvent(bytes32,uint256,address,bytes,string)",
                arrays: vec![([4u8; 32], true)],
                uints: vec![(U256::from(99), false)],
                addresses: vec![(
                    to_delegated_address("bd770416a3345f91e4b34576cb804a576fa48eb1"),
                    true,
                )],
                bytes: vec![(vec![10u8, 11u8], false)],
                strings: vec![("MixIndexString", true)],
                result: BuildTestResult::Success(
                    vec![
                        "b72a26b25954e89192a98625d6edc5cd3fa3c662f7c1ef861aab9b23df9d1fe2",
                        "0404040404040404040404040404040404040404040404040404040404040404",
                        "000000000000000000000000bd770416a3345f91e4b34576cb804a576fa48eb1",
                        "4d6978496e646578537472696e67000000000000000000000000000000000000",
                    ],
                    Some("000000000000000000000000000000000000000000000000000000000000006300000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000020a0b000000000000000000000000000000000000000000000000000000000000")
                ),
            },
            // Long dynamic bytes
            BuildTest {
                signature: "TestEvent(bytes)",
                bytes: vec![(vec![8u8; 50], false)],
                result: BuildTestResult::Success(
                    vec!["f6909898a012721c5ec53dd666c9114f4e7a1dd8548777674de87fb03a191791"],
                    Some("0000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000003208080808080808080808080808080808080808080808080808080808080808080808080808080808080808080808080808080000000000000000000000000000"),
                ),
                ..Default::default()
            },
            // Duplicate parameter types
            BuildTest {
                signature: "TestEvent(bytes32,bytes32)",
                arrays: vec![([11u8; 32], false), ([22u8; 32], false)],
                result: BuildTestResult::Success(
                    vec!["2754c7b9bece642d18818e09362252e9facd40e888069214ad5632005ae926c5"],
                    Some("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b1616161616161616161616161616161616161616161616161616161616161616"),
                ),
                ..Default::default()
            },
            // Boundary test for string size
            BuildTest {
                signature: "TestEvent(string,string,string)",
                strings: vec![
                    ("", false),
                    ("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", false),
                    ("BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB", false),
                ],
                result: BuildTestResult::Success(
                    vec!["95a72f5b88e67c971ee47c73b6c54216e4593de4bff257e240e25fac0e2b4d87"],
                    Some("000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020414141414141414141414141414141414141414141414141414141414141414100000000000000000000000000000000000000000000000000000000000000644242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424242424200000000000000000000000000000000000000000000000000000000"),
                ),
                ..Default::default()
            },
            // Dynamic and fixed parameters together
            BuildTest {
                signature: "TestEvent(bytes32,uint256,bytes,string)",
                arrays: vec![([4u8; 32], false)],
                uints: vec![(U256::from(500), true)],
                bytes: vec![(vec![12u8, 13u8], false)],
                strings: vec![("Dynamic and fixed mix", false)],
                result: BuildTestResult::Success(
                    vec![
                        "965b0af6b3ec4fbaa75bbae132c3aeae19f3b41cb3de924ebf3c13d30496bd21",
                        "00000000000000000000000000000000000000000000000000000000000001f4"
                    ],
                    Some("040404040404040404040404040404040404040404040404040404040404040400000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000020c0d000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001544796e616d696320616e64206669786564206d69780000000000000000000000"),
                ),
                ..Default::default()
            },
            // Nested dynamic data
            BuildTest {
                signature: "TestEvent(bytes,bytes,string)",
                bytes: vec![
                    (vec![0xaa, 0xbb, 0xcc, 0xdd], false),
                    (vec![0, 1, 2, 3, 4, 5, 6], false),
                ],
                strings: vec![("Nested dynamics", false)],
                result: BuildTestResult::Success(
                    vec![
                        "d4076ecbc8e3b2b464a7f67febe552c291c5b850397d59cd3db7a3947bf309b5",
                    ],
                    Some("000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000004aabbccdd0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000070001020304050600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000f4e65737465642064796e616d6963730000000000000000000000000000000000"),
                ),
                ..Default::default()
            },
            // Zero uint values
            BuildTest {
                signature: "TestEvent(uint256,uint256)",
                uints: vec![(U256::from(0), false), (U256::from(0), true)],
                result: BuildTestResult::Success(
                    vec![
                        "f3ca124a697ba07e8c5e80bebcfcc48991fc16a63170e8a9206e30508960d003",
                        "0000000000000000000000000000000000000000000000000000000000000000",
                    ],
                    Some("0000000000000000000000000000000000000000000000000000000000000000"),
                ),
                ..Default::default()
            },
            // Max address encoding
            BuildTest {
                signature: "TestEvent(address)",
                addresses: vec![(
                    to_delegated_address(&"ff".repeat(20)), // Maximum Ethereum address of 20 bytes
                    false,
                )],
                result: BuildTestResult::Success(
                    vec!["ab77f9000c19702a713e62164a239e3764dde2ba5265c7551f9a49e0d304530d"],
                    Some("000000000000000000000000ffffffffffffffffffffffffffffffffffffffff"),
                ),
                ..Default::default()
            },
            // Empty signature
            BuildTest {
                signature: "",
                arrays: vec![([5u8; 32], true)],
                result: BuildTestResult::Failure("invalid event signature: "),
                ..Default::default()
            },
            // Invalid signature
            BuildTest {
                signature: "InvalidSignature(address,uint256,string", // Invalid signature format
                arrays: vec![],
                result: BuildTestResult::Failure("invalid event signature: InvalidSignature(address,uint256,string"),
                ..Default::default()
            },
            // Empty parameters
            BuildTest {
                signature: "TestEvent(bytes32,uint256,address,bytes,string)",
                result: BuildTestResult::Failure("invalid event signature for params '': TestEvent(bytes32,uint256,address,bytes,string)"),
                ..Default::default()
            },
            // Invalid paramaters
            BuildTest {
                signature: "InvalidParam(foo)", // Invalid parameter
                arrays: vec![],
                result: BuildTestResult::Failure("invalid EVM param in signature: foo"),
                ..Default::default()
            },
            // Invalid address encoding
            BuildTest {
                signature: "TestEvent(address)",
                addresses: vec![(
                    Address::new_delegated(EAM_ACTOR_ID, &[0u8; 10]).unwrap(), // Invalid length
                    false,
                )],
                result: BuildTestResult::Failure("not an Ethereum address: f410faaaaaaaaaaaaaaaa2lano7y"),
                ..Default::default()
            },
        ];

        // Loop through each test case
        for test in tests.iter() {
            let expected = test.result.clone();
            let mut builder = EventBuilder::new(test.signature);
            test.build(&mut builder);
            let result = builder.build();
            match expected {
                BuildTestResult::Success(expected_topics, expected_data) => {
                    assert!(result.is_ok());
                    let event = result.unwrap();
                    let (topics, data) = get_topics_and_data(&event);
                    assert_eq!(topics, expected_topics);
                    assert_eq!(data, expected_data.map(|d| d.to_owned()));
                }
                BuildTestResult::Failure(msg) => {
                    assert!(result.is_err());
                    assert_eq!(result.unwrap_err().msg(), msg);
                }
            }
        }
    }
}
