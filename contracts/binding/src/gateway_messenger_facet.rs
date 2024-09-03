pub use gateway_messenger_facet::*;
/// This module was auto-generated with ethers-rs Abigen.
/// More information at: <https://github.com/gakonst/ethers-rs>
#[allow(
    clippy::enum_variant_names,
    clippy::too_many_arguments,
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    dead_code,
    non_camel_case_types
)]
pub mod gateway_messenger_facet {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("propagate"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("propagate"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("msgCid"),
                            kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("bytes32"),
                            ),
                        },],
                        outputs: ::std::vec![],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::Payable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("sendContractXnetMessage"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("sendContractXnetMessage",),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("envelope"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ::ethers::core::abi::ethabi::ParamType::Array(
                                            ::std::boxed::Box::new(
                                                ::ethers::core::abi::ethabi::ParamType::Address,
                                            ),
                                        ),
                                    ],),
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                        ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    ],),
                                ],),
                                ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ::ethers::core::abi::ethabi::ParamType::Array(
                                            ::std::boxed::Box::new(
                                                ::ethers::core::abi::ethabi::ParamType::Address,
                                            ),
                                        ),
                                    ],),
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                        ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    ],),
                                ],),
                                ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                ::ethers::core::abi::ethabi::ParamType::Bytes,
                            ],),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("struct IpcEnvelope"),
                            ),
                        },],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("committed"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ::ethers::core::abi::ethabi::ParamType::Array(
                                            ::std::boxed::Box::new(
                                                ::ethers::core::abi::ethabi::ParamType::Address,
                                            ),
                                        ),
                                    ],),
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                        ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    ],),
                                ],),
                                ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ::ethers::core::abi::ethabi::ParamType::Array(
                                            ::std::boxed::Box::new(
                                                ::ethers::core::abi::ethabi::ParamType::Address,
                                            ),
                                        ),
                                    ],),
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                        ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    ],),
                                ],),
                                ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                ::ethers::core::abi::ethabi::ParamType::Bytes,
                            ],),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("struct IpcEnvelope"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::Payable,
                    },],
                ),
            ]),
            events: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("NewBottomUpMsgBatch"),
                    ::std::vec![::ethers::core::abi::ethabi::Event {
                        name: ::std::borrow::ToOwned::to_owned("NewBottomUpMsgBatch",),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::EventParam {
                            name: ::std::borrow::ToOwned::to_owned("epoch"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                            indexed: true,
                        },],
                        anonymous: false,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NewTopDownMessage"),
                    ::std::vec![::ethers::core::abi::ethabi::Event {
                        name: ::std::borrow::ToOwned::to_owned("NewTopDownMessage"),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("subnet"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                indexed: true,
                            },
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("message"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Address,
                                                ),
                                            ),
                                        ],),
                                        ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                        ],),
                                    ],),
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Address,
                                                ),
                                            ),
                                        ],),
                                        ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                        ],),
                                    ],),
                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                ],),
                                indexed: false,
                            },
                        ],
                        anonymous: false,
                    },],
                ),
            ]),
            errors: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("CallFailed"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("CallFailed"),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CannotSendCrossMsgToItself"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("CannotSendCrossMsgToItself",),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InsufficientFunds"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("InsufficientFunds"),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidXnetMessage"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("InvalidXnetMessage"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("reason"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("enum InvalidXnetMessageReason",),
                            ),
                        },],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("MethodNotAllowed"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("MethodNotAllowed"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("reason"),
                            kind: ::ethers::core::abi::ethabi::ParamType::String,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("string"),
                            ),
                        },],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NotRegisteredSubnet"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("NotRegisteredSubnet",),
                        inputs: ::std::vec![],
                    },],
                ),
            ]),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static GATEWAYMESSENGERFACET_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(__abi);
    pub struct GatewayMessengerFacet<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for GatewayMessengerFacet<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for GatewayMessengerFacet<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for GatewayMessengerFacet<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for GatewayMessengerFacet<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(GatewayMessengerFacet))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> GatewayMessengerFacet<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                GATEWAYMESSENGERFACET_ABI.clone(),
                client,
            ))
        }
        ///Calls the contract's `propagate` (0x25bf0db6) function
        pub fn propagate(
            &self,
            msg_cid: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([37, 191, 13, 182], msg_cid)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `sendContractXnetMessage` (0x3eeb723f) function
        pub fn send_contract_xnet_message(
            &self,
            envelope: IpcEnvelope,
        ) -> ::ethers::contract::builders::ContractCall<M, IpcEnvelope> {
            self.0
                .method_hash([62, 235, 114, 63], (envelope,))
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `NewBottomUpMsgBatch` event
        pub fn new_bottom_up_msg_batch_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, NewBottomUpMsgBatchFilter>
        {
            self.0.event()
        }
        ///Gets the contract's `NewTopDownMessage` event
        pub fn new_top_down_message_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, NewTopDownMessageFilter>
        {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, GatewayMessengerFacetEvents>
        {
            self.0
                .event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
        for GatewayMessengerFacet<M>
    {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `CallFailed` with signature `CallFailed()` and selector `0x3204506f`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[etherror(name = "CallFailed", abi = "CallFailed()")]
    pub struct CallFailed;
    ///Custom Error type `CannotSendCrossMsgToItself` with signature `CannotSendCrossMsgToItself()` and selector `0xbcccd7fc`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[etherror(
        name = "CannotSendCrossMsgToItself",
        abi = "CannotSendCrossMsgToItself()"
    )]
    pub struct CannotSendCrossMsgToItself;
    ///Custom Error type `InsufficientFunds` with signature `InsufficientFunds()` and selector `0x356680b7`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[etherror(name = "InsufficientFunds", abi = "InsufficientFunds()")]
    pub struct InsufficientFunds;
    ///Custom Error type `InvalidXnetMessage` with signature `InvalidXnetMessage(uint8)` and selector `0xbc0f01cf`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[etherror(name = "InvalidXnetMessage", abi = "InvalidXnetMessage(uint8)")]
    pub struct InvalidXnetMessage {
        pub reason: u8,
    }
    ///Custom Error type `MethodNotAllowed` with signature `MethodNotAllowed(string)` and selector `0x015538b1`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[etherror(name = "MethodNotAllowed", abi = "MethodNotAllowed(string)")]
    pub struct MethodNotAllowed {
        pub reason: ::std::string::String,
    }
    ///Custom Error type `NotRegisteredSubnet` with signature `NotRegisteredSubnet()` and selector `0xe991abd0`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[etherror(name = "NotRegisteredSubnet", abi = "NotRegisteredSubnet()")]
    pub struct NotRegisteredSubnet;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum GatewayMessengerFacetErrors {
        CallFailed(CallFailed),
        CannotSendCrossMsgToItself(CannotSendCrossMsgToItself),
        InsufficientFunds(InsufficientFunds),
        InvalidXnetMessage(InvalidXnetMessage),
        MethodNotAllowed(MethodNotAllowed),
        NotRegisteredSubnet(NotRegisteredSubnet),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for GatewayMessengerFacetErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) =
                <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded) = <CallFailed as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::CallFailed(decoded));
            }
            if let Ok(decoded) =
                <CannotSendCrossMsgToItself as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::CannotSendCrossMsgToItself(decoded));
            }
            if let Ok(decoded) = <InsufficientFunds as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::InsufficientFunds(decoded));
            }
            if let Ok(decoded) =
                <InvalidXnetMessage as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::InvalidXnetMessage(decoded));
            }
            if let Ok(decoded) = <MethodNotAllowed as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::MethodNotAllowed(decoded));
            }
            if let Ok(decoded) =
                <NotRegisteredSubnet as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NotRegisteredSubnet(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for GatewayMessengerFacetErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::CallFailed(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::CannotSendCrossMsgToItself(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InsufficientFunds(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::InvalidXnetMessage(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MethodNotAllowed(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::NotRegisteredSubnet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for GatewayMessengerFacetErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector == <CallFailed as ::ethers::contract::EthError>::selector() => true,
                _ if selector
                    == <CannotSendCrossMsgToItself as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <InsufficientFunds as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <InvalidXnetMessage as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector == <MethodNotAllowed as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotRegisteredSubnet as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for GatewayMessengerFacetErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::CallFailed(element) => ::core::fmt::Display::fmt(element, f),
                Self::CannotSendCrossMsgToItself(element) => ::core::fmt::Display::fmt(element, f),
                Self::InsufficientFunds(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidXnetMessage(element) => ::core::fmt::Display::fmt(element, f),
                Self::MethodNotAllowed(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotRegisteredSubnet(element) => ::core::fmt::Display::fmt(element, f),
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for GatewayMessengerFacetErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<CallFailed> for GatewayMessengerFacetErrors {
        fn from(value: CallFailed) -> Self {
            Self::CallFailed(value)
        }
    }
    impl ::core::convert::From<CannotSendCrossMsgToItself> for GatewayMessengerFacetErrors {
        fn from(value: CannotSendCrossMsgToItself) -> Self {
            Self::CannotSendCrossMsgToItself(value)
        }
    }
    impl ::core::convert::From<InsufficientFunds> for GatewayMessengerFacetErrors {
        fn from(value: InsufficientFunds) -> Self {
            Self::InsufficientFunds(value)
        }
    }
    impl ::core::convert::From<InvalidXnetMessage> for GatewayMessengerFacetErrors {
        fn from(value: InvalidXnetMessage) -> Self {
            Self::InvalidXnetMessage(value)
        }
    }
    impl ::core::convert::From<MethodNotAllowed> for GatewayMessengerFacetErrors {
        fn from(value: MethodNotAllowed) -> Self {
            Self::MethodNotAllowed(value)
        }
    }
    impl ::core::convert::From<NotRegisteredSubnet> for GatewayMessengerFacetErrors {
        fn from(value: NotRegisteredSubnet) -> Self {
            Self::NotRegisteredSubnet(value)
        }
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethevent(name = "NewBottomUpMsgBatch", abi = "NewBottomUpMsgBatch(uint256)")]
    pub struct NewBottomUpMsgBatchFilter {
        #[ethevent(indexed)]
        pub epoch: ::ethers::core::types::U256,
    }
    #[derive(
        Clone,
        ::ethers::contract::EthEvent,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethevent(
        name = "NewTopDownMessage",
        abi = "NewTopDownMessage(address,(uint8,((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint64,uint256,bytes))"
    )]
    pub struct NewTopDownMessageFilter {
        #[ethevent(indexed)]
        pub subnet: ::ethers::core::types::Address,
        pub message: IpcEnvelope,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum GatewayMessengerFacetEvents {
        NewBottomUpMsgBatchFilter(NewBottomUpMsgBatchFilter),
        NewTopDownMessageFilter(NewTopDownMessageFilter),
    }
    impl ::ethers::contract::EthLogDecode for GatewayMessengerFacetEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = NewBottomUpMsgBatchFilter::decode_log(log) {
                return Ok(GatewayMessengerFacetEvents::NewBottomUpMsgBatchFilter(
                    decoded,
                ));
            }
            if let Ok(decoded) = NewTopDownMessageFilter::decode_log(log) {
                return Ok(GatewayMessengerFacetEvents::NewTopDownMessageFilter(
                    decoded,
                ));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for GatewayMessengerFacetEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::NewBottomUpMsgBatchFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::NewTopDownMessageFilter(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<NewBottomUpMsgBatchFilter> for GatewayMessengerFacetEvents {
        fn from(value: NewBottomUpMsgBatchFilter) -> Self {
            Self::NewBottomUpMsgBatchFilter(value)
        }
    }
    impl ::core::convert::From<NewTopDownMessageFilter> for GatewayMessengerFacetEvents {
        fn from(value: NewTopDownMessageFilter) -> Self {
            Self::NewTopDownMessageFilter(value)
        }
    }
    ///Container type for all input parameters for the `propagate` function with signature `propagate(bytes32)` and selector `0x25bf0db6`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(name = "propagate", abi = "propagate(bytes32)")]
    pub struct PropagateCall {
        pub msg_cid: [u8; 32],
    }
    ///Container type for all input parameters for the `sendContractXnetMessage` function with signature `sendContractXnetMessage((uint8,((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint64,uint256,bytes))` and selector `0x3eeb723f`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    #[ethcall(
        name = "sendContractXnetMessage",
        abi = "sendContractXnetMessage((uint8,((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint64,uint256,bytes))"
    )]
    pub struct SendContractXnetMessageCall {
        pub envelope: IpcEnvelope,
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum GatewayMessengerFacetCalls {
        Propagate(PropagateCall),
        SendContractXnetMessage(SendContractXnetMessageCall),
    }
    impl ::ethers::core::abi::AbiDecode for GatewayMessengerFacetCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <PropagateCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Propagate(decoded));
            }
            if let Ok(decoded) =
                <SendContractXnetMessageCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::SendContractXnetMessage(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for GatewayMessengerFacetCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::Propagate(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::SendContractXnetMessage(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for GatewayMessengerFacetCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::Propagate(element) => ::core::fmt::Display::fmt(element, f),
                Self::SendContractXnetMessage(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<PropagateCall> for GatewayMessengerFacetCalls {
        fn from(value: PropagateCall) -> Self {
            Self::Propagate(value)
        }
    }
    impl ::core::convert::From<SendContractXnetMessageCall> for GatewayMessengerFacetCalls {
        fn from(value: SendContractXnetMessageCall) -> Self {
            Self::SendContractXnetMessage(value)
        }
    }
    ///Container type for all return fields from the `sendContractXnetMessage` function with signature `sendContractXnetMessage((uint8,((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint64,uint256,bytes))` and selector `0x3eeb723f`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct SendContractXnetMessageReturn {
        pub committed: IpcEnvelope,
    }
    ///`FvmAddress(uint8,bytes)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct FvmAddress {
        pub addr_type: u8,
        pub payload: ::ethers::core::types::Bytes,
    }
    ///`Ipcaddress((uint64,address[]),(uint8,bytes))`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct Ipcaddress {
        pub subnet_id: SubnetID,
        pub raw_address: FvmAddress,
    }
    ///`IpcEnvelope(uint8,((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint64,uint256,bytes)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct IpcEnvelope {
        pub kind: u8,
        pub to: Ipcaddress,
        pub from: Ipcaddress,
        pub nonce: u64,
        pub value: ::ethers::core::types::U256,
        pub message: ::ethers::core::types::Bytes,
    }
    ///`SubnetID(uint64,address[])`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash,
    )]
    pub struct SubnetID {
        pub root: u64,
        pub route: ::std::vec::Vec<::ethers::core::types::Address>,
    }
}
