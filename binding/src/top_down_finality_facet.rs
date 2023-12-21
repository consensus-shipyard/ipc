pub use top_down_finality_facet::*;
/// This module was auto-generated with ethers-rs Abigen.
/// More information at: <https://github.com/gakonst/ethers-rs>
#[allow(
    clippy::enum_variant_names,
    clippy::too_many_arguments,
    clippy::upper_case_acronyms,
    clippy::type_complexity,
    dead_code,
    non_camel_case_types,
)]
pub mod top_down_finality_facet {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("applyFinalityChanges"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "applyFinalityChanges",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("commitParentFinality"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "commitParentFinality",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("finality"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct ParentFinality"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "hasCommittedBefore",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("previousFinality"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct ParentFinality"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("storeValidatorChanges"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "storeValidatorChanges",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("changeRequests"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Address,
                                                        ],
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                ],
                                            ),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct StakingChangeRequest[]",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
            ]),
            events: ::std::collections::BTreeMap::new(),
            errors: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("AddressShouldBeValidator"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "AddressShouldBeValidator",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CannotConfirmFutureChanges"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotConfirmFutureChanges",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidConfigurationNumber"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidConfigurationNumber",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NotSystemActor"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("NotSystemActor"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OldConfigurationNumber"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OldConfigurationNumber",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("PQDoesNotContainAddress"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "PQDoesNotContainAddress",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("PQEmpty"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("PQEmpty"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ParentFinalityAlreadyCommitted"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ParentFinalityAlreadyCommitted",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
            ]),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static TOPDOWNFINALITYFACET_ABI: ::ethers::contract::Lazy<
        ::ethers::core::abi::Abi,
    > = ::ethers::contract::Lazy::new(__abi);
    pub struct TopDownFinalityFacet<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for TopDownFinalityFacet<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for TopDownFinalityFacet<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for TopDownFinalityFacet<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for TopDownFinalityFacet<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(TopDownFinalityFacet))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> TopDownFinalityFacet<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    TOPDOWNFINALITYFACET_ABI.clone(),
                    client,
                ),
            )
        }
        ///Calls the contract's `applyFinalityChanges` (0x0df14461) function
        pub fn apply_finality_changes(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([13, 241, 68, 97], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `commitParentFinality` (0x11196974) function
        pub fn commit_parent_finality(
            &self,
            finality: ParentFinality,
        ) -> ::ethers::contract::builders::ContractCall<M, (bool, ParentFinality)> {
            self.0
                .method_hash([17, 25, 105, 116], (finality,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `storeValidatorChanges` (0xe49a547d) function
        pub fn store_validator_changes(
            &self,
            change_requests: ::std::vec::Vec<StakingChangeRequest>,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([228, 154, 84, 125], change_requests)
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for TopDownFinalityFacet<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `AddressShouldBeValidator` with signature `AddressShouldBeValidator()` and selector `0x2a55ca53`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "AddressShouldBeValidator", abi = "AddressShouldBeValidator()")]
    pub struct AddressShouldBeValidator;
    ///Custom Error type `CannotConfirmFutureChanges` with signature `CannotConfirmFutureChanges()` and selector `0x0815540a`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "CannotConfirmFutureChanges",
        abi = "CannotConfirmFutureChanges()"
    )]
    pub struct CannotConfirmFutureChanges;
    ///Custom Error type `InvalidConfigurationNumber` with signature `InvalidConfigurationNumber()` and selector `0x6ae94ca4`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "InvalidConfigurationNumber",
        abi = "InvalidConfigurationNumber()"
    )]
    pub struct InvalidConfigurationNumber;
    ///Custom Error type `NotSystemActor` with signature `NotSystemActor()` and selector `0xf0d97f3b`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "NotSystemActor", abi = "NotSystemActor()")]
    pub struct NotSystemActor;
    ///Custom Error type `OldConfigurationNumber` with signature `OldConfigurationNumber()` and selector `0x6e8d7c4a`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "OldConfigurationNumber", abi = "OldConfigurationNumber()")]
    pub struct OldConfigurationNumber;
    ///Custom Error type `PQDoesNotContainAddress` with signature `PQDoesNotContainAddress()` and selector `0xf2755e37`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "PQDoesNotContainAddress", abi = "PQDoesNotContainAddress()")]
    pub struct PQDoesNotContainAddress;
    ///Custom Error type `PQEmpty` with signature `PQEmpty()` and selector `0x40d9b011`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(name = "PQEmpty", abi = "PQEmpty()")]
    pub struct PQEmpty;
    ///Custom Error type `ParentFinalityAlreadyCommitted` with signature `ParentFinalityAlreadyCommitted()` and selector `0x2a75b082`
    #[derive(
        Clone,
        ::ethers::contract::EthError,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[etherror(
        name = "ParentFinalityAlreadyCommitted",
        abi = "ParentFinalityAlreadyCommitted()"
    )]
    pub struct ParentFinalityAlreadyCommitted;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum TopDownFinalityFacetErrors {
        AddressShouldBeValidator(AddressShouldBeValidator),
        CannotConfirmFutureChanges(CannotConfirmFutureChanges),
        InvalidConfigurationNumber(InvalidConfigurationNumber),
        NotSystemActor(NotSystemActor),
        OldConfigurationNumber(OldConfigurationNumber),
        PQDoesNotContainAddress(PQDoesNotContainAddress),
        PQEmpty(PQEmpty),
        ParentFinalityAlreadyCommitted(ParentFinalityAlreadyCommitted),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for TopDownFinalityFacetErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded) = <AddressShouldBeValidator as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AddressShouldBeValidator(decoded));
            }
            if let Ok(decoded) = <CannotConfirmFutureChanges as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CannotConfirmFutureChanges(decoded));
            }
            if let Ok(decoded) = <InvalidConfigurationNumber as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidConfigurationNumber(decoded));
            }
            if let Ok(decoded) = <NotSystemActor as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NotSystemActor(decoded));
            }
            if let Ok(decoded) = <OldConfigurationNumber as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OldConfigurationNumber(decoded));
            }
            if let Ok(decoded) = <PQDoesNotContainAddress as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PQDoesNotContainAddress(decoded));
            }
            if let Ok(decoded) = <PQEmpty as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PQEmpty(decoded));
            }
            if let Ok(decoded) = <ParentFinalityAlreadyCommitted as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ParentFinalityAlreadyCommitted(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for TopDownFinalityFacetErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::AddressShouldBeValidator(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotConfirmFutureChanges(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidConfigurationNumber(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotSystemActor(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OldConfigurationNumber(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PQDoesNotContainAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PQEmpty(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ParentFinalityAlreadyCommitted(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for TopDownFinalityFacetErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <AddressShouldBeValidator as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CannotConfirmFutureChanges as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidConfigurationNumber as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotSystemActor as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OldConfigurationNumber as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <PQDoesNotContainAddress as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <PQEmpty as ::ethers::contract::EthError>::selector() => true,
                _ if selector
                    == <ParentFinalityAlreadyCommitted as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for TopDownFinalityFacetErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AddressShouldBeValidator(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotConfirmFutureChanges(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidConfigurationNumber(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotSystemActor(element) => ::core::fmt::Display::fmt(element, f),
                Self::OldConfigurationNumber(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PQDoesNotContainAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PQEmpty(element) => ::core::fmt::Display::fmt(element, f),
                Self::ParentFinalityAlreadyCommitted(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for TopDownFinalityFacetErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<AddressShouldBeValidator> for TopDownFinalityFacetErrors {
        fn from(value: AddressShouldBeValidator) -> Self {
            Self::AddressShouldBeValidator(value)
        }
    }
    impl ::core::convert::From<CannotConfirmFutureChanges>
    for TopDownFinalityFacetErrors {
        fn from(value: CannotConfirmFutureChanges) -> Self {
            Self::CannotConfirmFutureChanges(value)
        }
    }
    impl ::core::convert::From<InvalidConfigurationNumber>
    for TopDownFinalityFacetErrors {
        fn from(value: InvalidConfigurationNumber) -> Self {
            Self::InvalidConfigurationNumber(value)
        }
    }
    impl ::core::convert::From<NotSystemActor> for TopDownFinalityFacetErrors {
        fn from(value: NotSystemActor) -> Self {
            Self::NotSystemActor(value)
        }
    }
    impl ::core::convert::From<OldConfigurationNumber> for TopDownFinalityFacetErrors {
        fn from(value: OldConfigurationNumber) -> Self {
            Self::OldConfigurationNumber(value)
        }
    }
    impl ::core::convert::From<PQDoesNotContainAddress> for TopDownFinalityFacetErrors {
        fn from(value: PQDoesNotContainAddress) -> Self {
            Self::PQDoesNotContainAddress(value)
        }
    }
    impl ::core::convert::From<PQEmpty> for TopDownFinalityFacetErrors {
        fn from(value: PQEmpty) -> Self {
            Self::PQEmpty(value)
        }
    }
    impl ::core::convert::From<ParentFinalityAlreadyCommitted>
    for TopDownFinalityFacetErrors {
        fn from(value: ParentFinalityAlreadyCommitted) -> Self {
            Self::ParentFinalityAlreadyCommitted(value)
        }
    }
    ///Container type for all input parameters for the `applyFinalityChanges` function with signature `applyFinalityChanges()` and selector `0x0df14461`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "applyFinalityChanges", abi = "applyFinalityChanges()")]
    pub struct ApplyFinalityChangesCall;
    ///Container type for all input parameters for the `commitParentFinality` function with signature `commitParentFinality((uint256,bytes32))` and selector `0x11196974`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "commitParentFinality",
        abi = "commitParentFinality((uint256,bytes32))"
    )]
    pub struct CommitParentFinalityCall {
        pub finality: ParentFinality,
    }
    ///Container type for all input parameters for the `storeValidatorChanges` function with signature `storeValidatorChanges(((uint8,bytes,address),uint64)[])` and selector `0xe49a547d`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "storeValidatorChanges",
        abi = "storeValidatorChanges(((uint8,bytes,address),uint64)[])"
    )]
    pub struct StoreValidatorChangesCall {
        pub change_requests: ::std::vec::Vec<StakingChangeRequest>,
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum TopDownFinalityFacetCalls {
        ApplyFinalityChanges(ApplyFinalityChangesCall),
        CommitParentFinality(CommitParentFinalityCall),
        StoreValidatorChanges(StoreValidatorChangesCall),
    }
    impl ::ethers::core::abi::AbiDecode for TopDownFinalityFacetCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <ApplyFinalityChangesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ApplyFinalityChanges(decoded));
            }
            if let Ok(decoded) = <CommitParentFinalityCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CommitParentFinality(decoded));
            }
            if let Ok(decoded) = <StoreValidatorChangesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::StoreValidatorChanges(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for TopDownFinalityFacetCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::ApplyFinalityChanges(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CommitParentFinality(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::StoreValidatorChanges(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for TopDownFinalityFacetCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::ApplyFinalityChanges(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CommitParentFinality(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::StoreValidatorChanges(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<ApplyFinalityChangesCall> for TopDownFinalityFacetCalls {
        fn from(value: ApplyFinalityChangesCall) -> Self {
            Self::ApplyFinalityChanges(value)
        }
    }
    impl ::core::convert::From<CommitParentFinalityCall> for TopDownFinalityFacetCalls {
        fn from(value: CommitParentFinalityCall) -> Self {
            Self::CommitParentFinality(value)
        }
    }
    impl ::core::convert::From<StoreValidatorChangesCall> for TopDownFinalityFacetCalls {
        fn from(value: StoreValidatorChangesCall) -> Self {
            Self::StoreValidatorChanges(value)
        }
    }
    ///Container type for all return fields from the `applyFinalityChanges` function with signature `applyFinalityChanges()` and selector `0x0df14461`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ApplyFinalityChangesReturn(pub u64);
    ///Container type for all return fields from the `commitParentFinality` function with signature `commitParentFinality((uint256,bytes32))` and selector `0x11196974`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct CommitParentFinalityReturn {
        pub has_committed_before: bool,
        pub previous_finality: ParentFinality,
    }
    ///`ParentFinality(uint256,bytes32)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct ParentFinality {
        pub height: ::ethers::core::types::U256,
        pub block_hash: [u8; 32],
    }
    ///`StakingChange(uint8,bytes,address)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct StakingChange {
        pub op: u8,
        pub payload: ::ethers::core::types::Bytes,
        pub validator: ::ethers::core::types::Address,
    }
    ///`StakingChangeRequest((uint8,bytes,address),uint64)`
    #[derive(
        Clone,
        ::ethers::contract::EthAbiType,
        ::ethers::contract::EthAbiCodec,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    pub struct StakingChangeRequest {
        pub change: StakingChange,
        pub configuration_number: u64,
    }
}
