pub use gateway_diamond::*;
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
pub mod gateway_diamond {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::Some(::ethers::core::abi::ethabi::Constructor {
                inputs: ::std::vec![
                    ::ethers::core::abi::ethabi::Param {
                        name: ::std::borrow::ToOwned::to_owned("_diamondCut"),
                        kind: ::ethers::core::abi::ethabi::ParamType::Array(
                            ::std::boxed::Box::new(
                                ::ethers::core::abi::ethabi::ParamType::Tuple(
                                    ::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Address,
                                        ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                        ::ethers::core::abi::ethabi::ParamType::Array(
                                            ::std::boxed::Box::new(
                                                ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                            ),
                                        ),
                                    ],
                                ),
                            ),
                        ),
                        internal_type: ::core::option::Option::Some(
                            ::std::borrow::ToOwned::to_owned(
                                "struct IDiamond.FacetCut[]",
                            ),
                        ),
                    },
                    ::ethers::core::abi::ethabi::Param {
                        name: ::std::borrow::ToOwned::to_owned("params"),
                        kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                            ::std::vec![
                                ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                ::ethers::core::abi::ethabi::ParamType::Uint(16usize),
                                ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                ::ethers::core::abi::ethabi::ParamType::Tuple(
                                    ::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ::ethers::core::abi::ethabi::ParamType::Array(
                                            ::std::boxed::Box::new(
                                                ::ethers::core::abi::ethabi::ParamType::Address,
                                            ),
                                        ),
                                    ],
                                ),
                                ::ethers::core::abi::ethabi::ParamType::Array(
                                    ::std::boxed::Box::new(
                                        ::ethers::core::abi::ethabi::ParamType::Tuple(
                                            ::std::vec![
                                                ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                ::ethers::core::abi::ethabi::ParamType::Address,
                                                ::ethers::core::abi::ethabi::ParamType::Bytes,
                                            ],
                                        ),
                                    ),
                                ),
                                ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                            ],
                        ),
                        internal_type: ::core::option::Option::Some(
                            ::std::borrow::ToOwned::to_owned(
                                "struct GatewayDiamond.ConstructorParams",
                            ),
                        ),
                    },
                ],
            }),
            functions: ::std::collections::BTreeMap::new(),
            events: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("ActiveValidatorCollateralUpdated"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ActiveValidatorCollateralUpdated",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("validator"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("newPower"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ActiveValidatorReplaced"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ActiveValidatorReplaced",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("oldValidator"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("newValidator"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("DiamondCut"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("DiamondCut"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("_diamondCut"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Address,
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Array(
                                                        ::std::boxed::Box::new(
                                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                                        ),
                                                    ),
                                                ],
                                            ),
                                        ),
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("_init"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("_calldata"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("MembershipUpdated"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("MembershipUpdated"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Address,
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                        ],
                                                    ),
                                                ),
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ],
                                    ),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NewActiveValidator"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("NewActiveValidator"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("validator"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("power"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NewWaitingValidator"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "NewWaitingValidator",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("validator"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("power"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OwnershipTransferred"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "OwnershipTransferred",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("oldOwner"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("newOwner"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "WaitingValidatorCollateralUpdated",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "WaitingValidatorCollateralUpdated",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("validator"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("newPower"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
            ]),
            errors: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned(
                        "CannotAddFunctionToDiamondThatAlreadyExists",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotAddFunctionToDiamondThatAlreadyExists",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CannotAddSelectorsToZeroAddress"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotAddSelectorsToZeroAddress",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_selectors"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4[]"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "CannotRemoveFunctionThatDoesNotExist",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotRemoveFunctionThatDoesNotExist",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CannotRemoveImmutableFunction"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotRemoveImmutableFunction",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "CannotReplaceFunctionThatDoesNotExists",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotReplaceFunctionThatDoesNotExists",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "CannotReplaceFunctionsFromFacetWithZeroAddress",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotReplaceFunctionsFromFacetWithZeroAddress",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_selectors"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4[]"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CannotReplaceImmutableFunction"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotReplaceImmutableFunction",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_selector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("FunctionNotFound"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("FunctionNotFound"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_functionSelector"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        4usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("IncorrectFacetCutAction"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "IncorrectFacetCutAction",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_action"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "enum IDiamond.FacetCutAction",
                                        ),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InitializationFunctionReverted"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InitializationFunctionReverted",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "_initializationContractAddress",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_calldata"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidMajorityPercentage"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidMajorityPercentage",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidSubmissionPeriod"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidSubmissionPeriod",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NoBytecodeAtAddress"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "NoBytecodeAtAddress",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_contractAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_message"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::String,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("string"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned(
                        "NoSelectorsProvidedForFacetForCut",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "NoSelectorsProvidedForFacetForCut",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_facetAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
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
                    ::std::borrow::ToOwned::to_owned(
                        "RemoveFacetAddressMustBeZeroAddress",
                    ),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "RemoveFacetAddressMustBeZeroAddress",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("_facetAddress"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                        },
                    ],
                ),
            ]),
            receive: true,
            fallback: true,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static GATEWAYDIAMOND_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(__abi);
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = b"`\x80\x80`@R4b\0\x0FeWb\0'(\x90\x818\x03\x80\x92b\0\0!\x82\x84b\0\x10\xB9V[\x829`@\x81\x83\x81\x01\x03\x12b\0\x0FeW\x80Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0FeW\x81\x01\x82\x82\x01`\x1F\x82\x01\x12\x15b\0\x0FeW\x80Qb\0\0^\x81b\0\x10\xDDV[\x91b\0\0n`@Q\x93\x84b\0\x10\xB9V[\x81\x83R` \x83\x01` \x81\x93`\x05\x1B\x83\x01\x01\x91\x86\x86\x01\x83\x11b\0\x0FeW` \x81\x01\x91[\x83\x83\x10b\0\x0F\x84WPPPP` \x83\x01Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0FeW`\xC0\x81\x85\x01\x86\x86\x01\x03\x12b\0\x0FeW`@Q\x93`\xC0\x85\x01`\x01`\x01`@\x1B\x03\x81\x11\x86\x82\x10\x17b\0\x07]W`@R\x80\x82\x01\x80Q\x86R` \x01Qa\xFF\xFF\x81\x16\x81\x03b\0\x0FeW` \x86\x01R`@\x82\x82\x01\x01Q`\xFF\x81\x16\x81\x03b\0\x0FeW`@\x86\x01R\x80\x82\x01``\x01Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0FeW\x82\x82\x01\x01`@\x81\x88\x84\x01\x03\x12b\0\x0FeW`@Q\x90b\0\x01F\x82b\0\x10\x9DV[\x80Q`\x01`\x01`@\x1B\x03\x81\x16\x81\x03b\0\x0FeW\x82R` \x81\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0FeW\x01\x87\x83\x01`\x1F\x82\x01\x12\x15b\0\x0FeW\x80Q\x90b\0\x01\x8D\x82b\0\x10\xDDV[\x91b\0\x01\x9D`@Q\x93\x84b\0\x10\xB9V[\x80\x83R` \x80\x84\x01\x91`\x05\x1B\x83\x01\x01\x91\x8A\x86\x01\x83\x11b\0\x0FeW` \x01\x90[\x82\x82\x10b\0\x0FiWPPP` \x82\x01R``\x86\x01R\x80\x82\x01`\x80\x01Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0FeW\x86\x82\x01`\x1F\x82\x85\x85\x01\x01\x01\x12\x15b\0\x0FeW\x80\x83\x83\x01\x01Qb\0\x02\n\x81b\0\x10\xDDV[\x91b\0\x02\x1A`@Q\x93\x84b\0\x10\xB9V[\x81\x83R` \x83\x01\x90\x89\x85\x01` \x84`\x05\x1B\x83\x89\x89\x01\x01\x01\x01\x11b\0\x0FeW` \x81\x87\x87\x01\x01\x01\x91[` \x84`\x05\x1B\x83\x89\x89\x01\x01\x01\x01\x83\x10b\0\x0E\x81WPPPP`\x80\x86\x01R\x01`\xA0\x90\x81\x01Q\x90\x84\x01R\x82Q\x15b\0\x0EoW`\xFF`@\x84\x01Q\x16`3\x81\x10\x90\x81\x15b\0\x0EcW[Pb\0\x0EQW\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD5\x80T3`\x01`\x01`\xA0\x1B\x03\x19\x82\x16\x81\x17\x90\x92U`@\x80Q`\x01`\x01`\xA0\x1B\x03\x90\x92\x16\x82R` \x82\x01\x92\x90\x92R\x7F\x8B\xE0\x07\x9CS\x16Y\x14\x13D\xCD\x1F\xD0\xA4\xF2\x84\x19I\x7F\x97\"\xA3\xDA\xAF\xE3\xB4\x18okdW\xE0\x91\x90\xA1`@Q`\x01`\x01`@\x1B\x03` \x82\x01\x90\x81\x11\x90\x82\x11\x17b\0\x07]W` \x81\x01`@R_\x81R\x82Q_[\x81\x81\x10b\0\x08\tWPP`@Q\x92``\x84\x01\x90``\x85RQ\x80\x91R`\x80\x84\x01\x90`\x80\x81`\x05\x1B\x86\x01\x01\x93\x91_\x90[\x82\x82\x10b\0\x07\xAEW\x87\x7F\x8F\xAAp\x87\x86q\xCC\xD2\x12\xD2\x07q\xB7\x95\xC5\n\xF8\xFD?\xF6\xCF'\xF4\xBD\xE5~]M\xE0\xAE\xB6s\x88\x80b\0\x03\x9B\x8A\x8A_` \x85\x01R\x83\x82\x03`@\x85\x01Rb\0\x11\xCCV[\x03\x90\xA1\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD4` \x90\x81R\x7F}\xFDH\xDD\x9D\xEF\0/\xA9\xB4\xA0[\xD6\xB7&\xA6\xC3\x13\xC3b\xD3\xF3\xE8A=zu \xF0\t\r%\x80T`\xFF\x19\x90\x81\x16`\x01\x90\x81\x17\x90\x92U\x7FM\x7FL\x8A/\xB5\xB3\\\xA3\xC2w\xC98\x88\xB4\x7F\x0F\")\xBD\xCC\xCFfPM\x1B\xA4\x8E\x88\xB8\x81d\x80T\x82\x16\x83\x17\x90UcH\xE2\xB0\x93`\xE0\x1B_R\x7FY\xBAM\xB4\xA2\x13\xE8\x16\x1D\xE5\x97\xB8\xC1\r\xB0\xE7\xE7\xBAZ\xCE\\&\x8E67\x9E$\x9Am-B\xC9\x80T\x82\x16\x90\x92\x17\x90\x91U`\x06\x80Tb\xFF\xFF\xFF\x19\x16b\x01\x01\n\x17\x90U``\x83\x01Q\x80Q`\x12\x80T`\x01`\x01`@\x1B\x03\x19\x16`\x01`\x01`@\x1B\x03\x92\x83\x16\x17\x90U\x92\x01Q\x80Q\x91\x92\x82\x11b\0\x07]Wh\x01\0\0\0\0\0\0\0\0\x82\x11b\0\x07]W` \x90`\x13T\x83`\x13U\x80\x84\x10b\0\x07\x8FW[P\x01`\x13_R` _ _[\x83\x81\x10b\0\x07qW\x85\x85\x81Q`\x01U`\xFF`@\x83\x01Q\x16\x90`\x04T\x16\x17`\x04U`\x01`\x0CU`\xA0\x81\x01Q`\x05U`\x05`\xC1\x1B`\x01\x80`\xC0\x1B\x03`\x03T\x16\x17`\x03U` \x81\x01Qb\xFF\xFF\0`\x14T\x91`\x08\x1B\x16\x90b\xFF\xFF\0\x19\x16\x17`\x14Uh\x01\0\0\0\0\0\0\0\x01`\x01\x80`\x80\x1B\x03\x19`\x1DT\x16\x17`\x1DU`\x80\x81\x01QQ_[\x81\x81\x10b\0\x05\x82Wb\0\x05r`\x80\x84\x01Q`@Q\x90b\0\x05d\x82b\0\x10\x9DV[\x81R_` \x82\x01Rb\0\x159V[`@Qa\x016\x90\x81b\0%\x92\x829\xF3[`\x01\x80`\xA0\x1B\x03` b\0\x05\x9B\x83`\x80\x87\x01Qb\0\x11EV[Q\x01Q\x16b\0\x05\xAF\x82`\x80\x86\x01Qb\0\x11EV[QQ\x90`@b\0\x05\xC4\x84`\x80\x88\x01Qb\0\x11EV[Q\x01Q`\x01`\x01`\xA0\x1B\x03\x82\x16_\x90\x81R`\x16` R`@\x90 `\x03\x90\x82Q\x91\x01\x91`\x01`\x01`@\x1B\x03\x82\x11b\0\x07]Wb\0\x06\r\x82b\0\x06\x06\x85Tb\0\x11\xF3V[\x85b\0\x12.V[` \x90`\x1F\x83\x11`\x01\x14b\0\x06\xE9W\x82b\0\x06\xD6\x95\x93`\x01\x98\x97\x95\x93b\0\x06J\x93_\x92b\0\x06\xDDW[PP\x81`\x01\x1B\x91_\x19\x90`\x03\x1B\x1C\x19\x16\x17\x90V[\x90U[`\x01`\x01`\xA0\x1B\x03\x81\x16_\x90\x81R`\x16` R`@\x90 `\x02\x01b\0\x06t\x83\x82Tb\0\x12xV[\x90Ub\0\x06\x9F\x82\x86b\0\x06\x97\x84`\x01\x80`\xA0\x1B\x03\x16_R`\x16` R`@_ \x90V[\x01Tb\0\x12xV[\x91\x82\x86b\0\x06\xBD\x84`\x01\x80`\xA0\x1B\x03\x16_R`\x16` R`@_ \x90V[\x01Ub\0\x06\xCE`\x15\x91\x82Tb\0\x12xV[\x90Ub\0\x19\xEFV[\x01b\0\x05DV[\x01Q\x90P\x8B\x80b\0\x066V[\x90\x83_R` _ \x91_[`\x1F\x19\x85\x16\x81\x10b\0\x07DWP\x92`\x01\x97\x96\x94\x92\x88\x92b\0\x06\xD6\x97\x95\x83`\x1F\x19\x81\x16\x10b\0\x07+W[PPP\x81\x1B\x01\x90Ub\0\x06MV[\x01Q_\x19`\xF8\x84`\x03\x1B\x16\x1C\x19\x16\x90U\x8A\x80\x80b\0\x07\x1DV[\x91\x92` `\x01\x81\x92\x86\x85\x01Q\x81U\x01\x94\x01\x92\x01b\0\x06\xF4V[cNH{q`\xE0\x1B_R`A`\x04R`$_\xFD[\x82Q`\x01`\x01`\xA0\x1B\x03\x16\x81\x83\x01U` \x90\x92\x01\x91`\x01\x01b\0\x04\xC5V[b\0\x07\xA7\x90`\x13_R\x84\x84_ \x91\x82\x01\x91\x01b\0\x11-V[\x85b\0\x04\xB9V[\x90\x91\x92\x94` \x80b\0\x07\xFA`\x01\x93`\x7F\x19\x8B\x82\x03\x01\x86R```@\x8BQ\x87\x80`\xA0\x1B\x03\x81Q\x16\x84R\x85\x81\x01Qb\0\x07\xE5\x81b\0\x11nV[\x86\x85\x01R\x01Q\x91\x81`@\x82\x01R\x01\x90b\0\x11\x8DV[\x97\x01\x92\x01\x92\x01\x90\x92\x91b\0\x03UV[`@b\0\x08\x17\x82\x87b\0\x11EV[Q\x01Q`\x01`\x01`\xA0\x1B\x03b\0\x08.\x83\x88b\0\x11EV[QQ\x16\x90\x80Q\x15b\0\x0E8W` b\0\x08H\x84\x89b\0\x11EV[Q\x01Qb\0\x08V\x81b\0\x11nV[b\0\x08a\x81b\0\x11nV[\x80b\0\ntWP\x81\x15b\0\nIWa\xFF\xFF_\x80Q` b\0&\xE8\x839\x81Q\x91RT\x16b\0\x08\xD0`@Qb\0\x08\x95\x81b\0\x10\x81V[`!\x81R\x7FdiamondCut: Add facet has no cod` \x82\x01R`e`\xF8\x1B`@\x82\x01R\x84b\0\x1F\x87V[\x81Q\x91_\x91[\x83\x83\x10b\0\x08\xEEWPPPPP`\x01\x90[\x01b\0\x03'V[`\x01`\x01`\xE0\x1B\x03\x19b\0\t\x03\x84\x84b\0\x11EV[Q\x16_\x81\x81R_\x80Q` b\0'\x08\x839\x81Q\x91R` R`@\x90 T\x90\x91\x90`\x01`\x01`\xA0\x1B\x03\x16b\0\n0Wb\0\t\xA2`@Qb\0\tC\x81b\0\x10\x9DV[\x87\x81Ra\xFF\xFF\x92\x90\x92\x16` \x80\x84\x01\x82\x81R_\x86\x81R_\x80Q` b\0'\x08\x839\x81Q\x91R\x90\x92R`@\x90\x91 \x93Q\x84T\x91Q`\x01`\x01`\xB0\x1B\x03\x19\x90\x92\x16`\x01`\x01`\xA0\x1B\x03\x91\x90\x91\x16\x17`\xA0\x91\x90\x91\x1Ba\xFF\xFF`\xA0\x1B\x16\x17\x90\x92UV[_\x80Q` b\0&\xE8\x839\x81Q\x91RT\x91h\x01\0\0\0\0\0\0\0\0\x83\x10\x15b\0\x07]Wb\0\n\x07\x90b\0\t\xEA`\x01\x94\x85\x81\x01_\x80Q` b\0&\xE8\x839\x81Q\x91RUb\0\x19\xC0V[\x90\x91\x90c\xFF\xFF\xFF\xFF\x83T\x91`\x03\x1B\x92`\xE0\x1C\x83\x1B\x92\x1B\x19\x16\x17\x90UV[a\xFF\xFF\x81\x14b\0\n\x1CW\x81\x01\x92\x01\x91b\0\x08\xD6V[cNH{q`\xE0\x1B_R`\x11`\x04R`$_\xFD[`@Qc\xEB\xBF]\x07`\xE0\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qc\x02\xB8\xDA\x07`\xE2\x1B\x81R` `\x04\x82\x01R\x90\x81\x90b\0\np\x90`$\x83\x01\x90b\0\x11\x8DV[\x03\x90\xFD[b\0\n\x7F\x81b\0\x11nV[`\x01\x81\x03b\0\x0B\xF3WP\x81\x15b\0\x0B\xCCWb\0\n\xE4`@Qb\0\n\xA2\x81b\0\x10\x81V[`(\x81R\x7FLibDiamondCut: Replace facet has` \x82\x01Rg no code`\xC0\x1B`@\x82\x01R\x83b\0\x1F\x87V[\x80Q\x90_[\x82\x81\x10b\0\n\xFEWPPPP`\x01\x90b\0\x08\xE7V[`\x01`\x01`\xE0\x1B\x03\x19b\0\x0B\x13\x82\x84b\0\x11EV[Q\x16_\x81\x81R_\x80Q` b\0'\x08\x839\x81Q\x91R` R`@\x90 T`\x01`\x01`\xA0\x1B\x03\x160\x81\x14b\0\x0B\xB3W\x85\x81\x14b\0\x0B\x9AW\x15b\0\x0B\x82W_\x90\x81R_\x80Q` b\0'\x08\x839\x81Q\x91R` R`@\x90 \x80T`\x01`\x01`\xA0\x1B\x03\x19\x16\x85\x17\x90U`\x01\x01b\0\n\xE9V[`$\x90`@Q\x90cty\xF99`\xE0\x1B\x82R`\x04\x82\x01R\xFD[`@Qc\x1A\xC6\xCE\x8D`\xE1\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qc)\x01\x80m`\xE1\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qc\xCD\x98\xA9o`\xE0\x1B\x81R` `\x04\x82\x01R\x90\x81\x90b\0\np\x90`$\x83\x01\x90b\0\x11\x8DV[b\0\x0B\xFE\x81b\0\x11nV[`\x02\x81\x03b\0\x0E\x16WP_\x80Q` b\0&\xE8\x839\x81Q\x91RT\x91\x80b\0\r\xFEWP\x80Q\x90_\x92[\x82\x84\x10b\0\x0C;WPPPP`\x01\x90b\0\x08\xE7V[`\x01`\x01`\xE0\x1B\x03\x19b\0\x0CP\x85\x84b\0\x11EV[Q\x16\x93\x84_R_\x80Q` b\0'\x08\x839\x81Q\x91R` R`@_ \x94`@Q\x95b\0\x0C|\x87b\0\x10\x9DV[T`\x01`\x01`\xA0\x1B\x03\x81\x16\x80\x88R`\xA0\x91\x90\x91\x1Ca\xFF\xFF\x16` \x88\x01R\x15b\0\r\xE6W\x85Q`\x01`\x01`\xA0\x1B\x03\x160\x14b\0\r\xCEW\x82\x15b\0\n\x1CW_\x19\x80\x93\x01\x95\x86a\xFF\xFF` \x83\x01Q\x16\x03b\0\rMW[P_\x80Q` b\0&\xE8\x839\x81Q\x91RT\x92\x83\x15b\0\r9W`\x01\x93\x01b\0\x0C\xF7\x81b\0\x19\xC0V[c\xFF\xFF\xFF\xFF\x82T\x91`\x03\x1B\x1B\x19\x16\x90U_\x80Q` b\0&\xE8\x839\x81Q\x91RU_R_\x80Q` b\0'\x08\x839\x81Q\x91R` R_`@\x81 U\x01\x92b\0\x0C&V[cNH{q`\xE0\x1B_R`1`\x04R`$_\xFD[b\0\r\xC7\x90a\xFF\xFF` b\0\rb\x8Ab\0\x19\xC0V[\x90T\x90`\x03\x1B\x1C`\xE0\x1B\x92b\0\r\x82\x84b\0\t\xEA\x85\x85\x85\x01Q\x16b\0\x19\xC0V[\x01Q`\x01`\x01`\xE0\x1B\x03\x19\x90\x92\x16_\x90\x81R_\x80Q` b\0'\x08\x839\x81Q\x91R` R`@\x90 \x80Ta\xFF\xFF`\xA0\x1B\x19\x16\x91\x90\x92\x16`\xA0\x1Ba\xFF\xFF`\xA0\x1B\x16\x17\x90UV[\x8Cb\0\x0C\xCFV[`$\x90`@Q\x90c\r\xF5\xFDa`\xE3\x1B\x82R`\x04\x82\x01R\xFD[`$\x90`@Q\x90cz\x08\xA2-`\xE0\x1B\x82R`\x04\x82\x01R\xFD[`$\x90`@Q\x90c\xD0\x91\xBC\x81`\xE0\x1B\x82R`\x04\x82\x01R\xFD[`@Qc?\xF4\xD2\x0F`\xE1\x1B\x81R`$\x91b\0\x0E1\x81b\0\x11nV[`\x04\x82\x01R\xFD[`@Qc\xE7g\xF9\x1F`\xE0\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qcu\xC3\xB4'`\xE0\x1B\x81R`\x04\x90\xFD[`d\x91P\x11\x84b\0\x02\x87V[`@Qc1/\x8E\x05`\xE0\x1B\x81R`\x04\x90\xFD[\x82Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0FeW`\x1F\x19\x91\x90\x87\x89\x01\x84\x01\x01``\x8D\x89\x01\x82\x90\x03\x84\x01\x12b\0\x0FeW`@Q\x90b\0\x0E\xBC\x82b\0\x10\x81V[` \x81\x01Q\x82Rb\0\x0E\xD1`@\x82\x01b\0\x10\xF5V[` \x83\x01R``\x81\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0FeW\x01\x91\x8D\x89\x01`?\x84\x01\x12\x15b\0\x0FeW` \x83\x01Q\x91`\x01`\x01`@\x1B\x03\x83\x11b\0\x07]W\x8Eb\0\x0F(` `@Q\x97`\x1F\x87\x01\x16\x01\x87b\0\x10\xB9V[\x83\x86R\x8A\x01`@\x84\x86\x01\x01\x11b\0\x0FeW\x84b\0\x0FR` \x96\x94\x87\x96`@\x88\x80\x98\x01\x91\x01b\0\x11\nV[`@\x82\x01R\x81R\x01\x93\x01\x92\x90Pb\0\x02BV[_\x80\xFD[` \x80\x91b\0\x0Fx\x84b\0\x10\xF5V[\x81R\x01\x91\x01\x90b\0\x01\xBCV[\x82Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0FeW\x82\x01``\x90\x81`\x1F\x19\x82\x8C\x8C\x01\x03\x01\x12b\0\x0FeW`@Q\x91b\0\x0F\xB9\x83b\0\x10\x81V[b\0\x0F\xC7` \x83\x01b\0\x10\xF5V[\x83R`@\x82\x01Q`\x03\x81\x10\x15b\0\x0FeW` \x84\x01R\x81\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0FeW\x01\x89\x89\x01`?\x82\x01\x12\x15b\0\x0FeW` \x81\x01Qb\0\x10\x0F\x81b\0\x10\xDDV[\x91b\0\x10\x1F`@Q\x93\x84b\0\x10\xB9V[\x81\x83R`@` \x84\x01\x92`\x05\x1B\x82\x01\x01\x90\x8C\x8C\x01\x82\x11b\0\x0FeW`@\x01\x91[\x81\x83\x10b\0\x10^WPPP`@\x82\x01R\x81R` \x92\x83\x01\x92\x01b\0\0\x90V[\x82Q`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x81\x03b\0\x0FeW\x81R` \x92\x83\x01\x92\x01b\0\x10?V[``\x81\x01\x90\x81\x10`\x01`\x01`@\x1B\x03\x82\x11\x17b\0\x07]W`@RV[`@\x81\x01\x90\x81\x10`\x01`\x01`@\x1B\x03\x82\x11\x17b\0\x07]W`@RV[`\x1F\x90\x91\x01`\x1F\x19\x16\x81\x01\x90`\x01`\x01`@\x1B\x03\x82\x11\x90\x82\x10\x17b\0\x07]W`@RV[`\x01`\x01`@\x1B\x03\x81\x11b\0\x07]W`\x05\x1B` \x01\x90V[Q\x90`\x01`\x01`\xA0\x1B\x03\x82\x16\x82\x03b\0\x0FeWV[_[\x83\x81\x10b\0\x11\x1CWPP_\x91\x01RV[\x81\x81\x01Q\x83\x82\x01R` \x01b\0\x11\x0CV[\x81\x81\x10b\0\x119WPPV[_\x81U`\x01\x01b\0\x11-V[\x80Q\x82\x10\x15b\0\x11ZW` \x91`\x05\x1B\x01\x01\x90V[cNH{q`\xE0\x1B_R`2`\x04R`$_\xFD[`\x03\x11\x15b\0\x11yWV[cNH{q`\xE0\x1B_R`!`\x04R`$_\xFD[\x90\x81Q\x80\x82R` \x80\x80\x93\x01\x93\x01\x91_[\x82\x81\x10b\0\x11\xADWPPPP\x90V[\x83Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x93\x81\x01\x93\x92\x81\x01\x92`\x01\x01b\0\x11\x9EV[\x90` \x91b\0\x11\xE7\x81Q\x80\x92\x81\x85R\x85\x80\x86\x01\x91\x01b\0\x11\nV[`\x1F\x01`\x1F\x19\x16\x01\x01\x90V[\x90`\x01\x82\x81\x1C\x92\x16\x80\x15b\0\x12#W[` \x83\x10\x14b\0\x12\x0FWV[cNH{q`\xE0\x1B_R`\"`\x04R`$_\xFD[\x91`\x7F\x16\x91b\0\x12\x03V[\x91\x90`\x1F\x81\x11b\0\x12>WPPPV[b\0\x12k\x92_R` _ \x90` `\x1F\x84\x01`\x05\x1C\x83\x01\x93\x10b\0\x12mW[`\x1F\x01`\x05\x1C\x01\x90b\0\x11-V[V[\x90\x91P\x81\x90b\0\x12]V[\x91\x90\x82\x01\x80\x92\x11b\0\n\x1CWV[\x90\x80\x82Q\x90\x81\x81R` \x80\x91\x01\x92\x81\x80\x84`\x05\x1B\x83\x01\x01\x95\x01\x93_\x91[\x84\x83\x10b\0\x12\xB4WPPPPPP\x90V[\x90\x91\x92\x93\x94\x95\x84\x80b\0\x12\xF8`\x01\x93`\x1F\x19\x86\x82\x03\x01\x87R\x8AQ\x90``\x90\x82Q\x81R\x86\x80`\xA0\x1B\x03\x85\x84\x01Q\x16\x85\x82\x01R\x81`@\x80\x94\x01Q\x93\x82\x01R\x01\x90b\0\x11\xCCV[\x98\x01\x93\x01\x93\x01\x91\x94\x93\x92\x90b\0\x12\xA3V[\x90\x80\x82\x14b\0\x13\xF1Wb\0\x13\x1E\x81Tb\0\x11\xF3V[\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x07]Wb\0\x13?\x82b\0\x06\x06\x85Tb\0\x11\xF3V[_\x90`\x1F\x83\x11`\x01\x14b\0\x13\x7FWb\0\x13o\x92\x91_\x91\x83b\0\x13sWPP\x81`\x01\x1B\x91_\x19\x90`\x03\x1B\x1C\x19\x16\x17\x90V[\x90UV[\x01T\x90P_\x80b\0\x066V[\x90`\x1F\x19\x83\x16\x91_R` \x91` _ \x90\x85_R` _ \x93_\x90[\x82\x82\x10b\0\x13\xD7WPP\x90\x84`\x01\x95\x94\x93\x92\x10b\0\x13\xBEW[PPP\x81\x1B\x01\x90UV[\x01T_\x19`\xF8\x84`\x03\x1B\x16\x1C\x19\x16\x90U_\x80\x80b\0\x13\xB4V[\x84\x95\x81\x92\x95\x85\x01T\x81U`\x01\x80\x91\x01\x96\x01\x94\x01\x90b\0\x13\x9BV[PPV[`\x07T\x81\x10\x15b\0\x11ZW`\x07_R`\x03` _ \x91\x02\x01\x90_\x90V[\x92\x91\x90b\0\x15&W\x80Q\x83U` \x80\x82\x01Q`\x01\x80\x86\x01\x80T`\x01`\x01`\xA0\x1B\x03\x19\x16`\x01`\x01`\xA0\x1B\x03\x93\x90\x93\x16\x92\x90\x92\x17\x90\x91U`@\x90\x92\x01Q\x80Q\x90\x94`\x02\x01\x92\x90\x91`\x01`\x01`@\x1B\x03\x83\x11b\0\x07]Wb\0\x14\x7F\x83b\0\x14x\x86Tb\0\x11\xF3V[\x86b\0\x12.V[` \x91`\x1F\x84\x11`\x01\x14b\0\x14\xC0WPP\x81\x90b\0\x13o\x93\x94\x95_\x92b\0\x14\xB4WPP\x81`\x01\x1B\x91_\x19\x90`\x03\x1B\x1C\x19\x16\x17\x90V[\x01Q\x90P_\x80b\0\x066V[_\x85\x81R` \x81 `\x1F\x19\x86\x16\x98\x90\x94\x93\x90\x92[\x89\x83\x10b\0\x15\x0EWPPP\x83`\x01\x95\x96\x97\x10b\0\x14\xF5WPPP\x81\x1B\x01\x90UV[\x01Q_\x19`\xF8\x84`\x03\x1B\x16\x1C\x19\x16\x90U_\x80\x80b\0\x13\xB4V[\x83\x85\x01Q\x86U\x94\x85\x01\x94\x93\x81\x01\x93\x91\x81\x01\x91b\0\x14\xD4V[cNH{q`\xE0\x1B_R_`\x04R`$_\xFD[`@Q\x90` \x91\x82\x81Rb\0\x15Z\x82Q`@\x85\x84\x01R``\x83\x01\x90b\0\x12\x86V[\x91\x7F~\xCD\xACH#4\xC3o\xCC\xBE7C\x18\xCF\xE7N\xA0\xC8\x18\x13\x94\x89\r\xDE\xC8\x94\xA1\x0F\x0F\xCCt\x81\x84\x82\x01\x92\x80\x84Q\x95`\x01\x80`@\x1B\x03\x80\x97\x16`@\x83\x01R\x03\x90\xA1\x82`\x08T\x16\x80b\0\x18AW[P`\x07\x91\x82T\x93h\x01\0\0\0\0\0\0\0\0\x94\x85\x81\x11b\0\x07]W`\tT\x81`\tU\x80\x82\x10b\0\x17\x8BW[P\x84_R\x86_ `\t_R\x87_ \x90_\x90[\x83\x82\x10b\0\x175WPPPP\x80`\x08T\x16\x91`\x01\x80`@\x1B\x03\x19\x92\x83`\nT\x16\x17`\nU\x83QQ\x93\x85T\x96_[\x86\x81\x10b\0\x16\xC7WPPPQ\x16\x90`\x08T\x16\x17`\x08U\x80\x83\x11b\0\x160W[PPPPV[\x82\x81\x10\x15b\0\x16*W\x81T\x80\x15b\0\r9W_\x19\x01\x90b\0\x16Q\x82b\0\x13\xF5V[\x92\x90\x92b\0\x15&W_\x83U`\x02`\x01\x93_\x85\x82\x01U\x01b\0\x16s\x81Tb\0\x11\xF3V[\x90\x81b\0\x16\x86W[PP\x83U\x01b\0\x160V[_\x91`\x1F\x80\x82\x11`\x01\x14b\0\x16\xA1WPPU[_\x80b\0\x16{V[\x91b\0\x16\xBF\x84\x92\x93\x82\x84R\x8B\x84 \x94\x01`\x05\x1C\x84\x01\x88\x85\x01b\0\x11-V[UUb\0\x16\x99V[\x88\x81\x10\x15b\0\x16\xFEW\x80b\0\x16\xF7b\0\x16\xE4`\x01\x93\x86Qb\0\x11EV[Qb\0\x16\xF0\x83b\0\x13\xF5V[\x90b\0\x14\x12V[\x01b\0\x16\x0BV[b\0\x17\x0B\x81\x84Qb\0\x11EV[Q\x90\x88T\x83\x81\x10\x15b\0\x07]W`\x01\x92b\0\x16\xF0\x82\x85b\0\x17/\x94\x01\x8DUb\0\x13\xF5V[b\0\x16\xF7V[\x80`\x01\x91\x84\x03b\0\x17RW[`\x03\x80\x91\x01\x93\x01\x91\x01\x90\x91b\0\x15\xDEV[\x80T\x84U\x81\x80\x85\x01\x90\x83\x80`\xA0\x1B\x03\x90\x83\x01T\x16\x83\x80`\xA0\x1B\x03\x19\x82T\x16\x17\x90Ub\0\x17\x85`\x02\x80\x83\x01\x90\x86\x01b\0\x13\tV[b\0\x17AV[`\x03\x90\x80`\x03\x02\x90`\x03\x82\x04\x03b\0\n\x1CW\x82`\x03\x02`\x03\x81\x04\x84\x03b\0\n\x1CW`\t_R\x89_ \x91\x82\x01\x91\x01[\x81\x81\x10b\0\x17\xC9WPPb\0\x15\xCCV[\x80_\x84\x92U\x8A`\x01_\x81\x84\x01U`\x02\x83\x01\x90b\0\x17\xE7\x82Tb\0\x11\xF3V[\x90\x81b\0\x17\xFAW[PPPP\x01b\0\x17\xB9V[_\x93`\x1F\x90\x81\x84\x11`\x01\x14b\0\x18\x1AWPPPPU[\x8A_\x80\x80b\0\x17\xEFV[\x91b\0\x189\x91\x85\x93\x87\x95\x96\x86R\x85 \x95\x01`\x05\x1C\x85\x01\x90\x85\x01b\0\x11-V[UUb\0\x18\x10V[\x83\x83Q\x16\x84`\nT\x16\x90\x81\x81\x14b\0\x19\xB7W\x10b\0\x19\xA5W`@Q\x90b\0\x18h\x82b\0\x10\x9DV[`\x07Tb\0\x18v\x81b\0\x10\xDDV[\x90b\0\x18\x86`@Q\x92\x83b\0\x10\xB9V[\x80\x82R\x87\x82\x01`\x07_R\x88_ _\x91[\x83\x83\x10b\0\x18\xC1WPPP\x90\x83RP\x85\x82\x01Rb\0\x18\xB5\x90\x82b\0\x1E\xCEV[b\0\x16*W_b\0\x15\xA2V[\x8A`@Qb\0\x18\xD0\x81b\0\x10\x81V[\x83T\x81R`\x01\x84\x01T`\x01`\x01`\xA0\x1B\x03\x16\x82\x82\x01R`@Q`\x02\x85\x01\x80T_\x91b\0\x18\xFC\x82b\0\x11\xF3V[\x80\x85R\x91`\x01\x81\x16\x90\x81\x15b\0\x19\x85WP`\x01\x14b\0\x19BW[PP\x91\x81b\0\x19.`\x01\x96\x93`\x03\x96\x95\x03\x82b\0\x10\xB9V[`@\x82\x01R\x81R\x01\x92\x01\x92\x01\x91\x90b\0\x18\x96V[_\x90\x81R\x85\x81 \x90\x92P[\x81\x83\x10b\0\x19fWPP\x81\x01\x83\x01\x81b\0\x19.b\0\x19\x16V[\x80`\x01\x91\x96\x92\x93\x94\x95\x96T\x83\x86\x88\x01\x01R\x01\x92\x01\x90\x8F\x94\x93\x92b\0\x19MV[`\xFF\x19\x16\x85\x88\x01RPP\x15\x15`\x05\x1B\x82\x01\x84\x01\x90P\x81b\0\x19.b\0\x19\x16V[`@Qc7F\xBE%`\xE1\x1B\x81R`\x04\x90\xFD[PPPPPPPV[\x90_\x80Q` b\0&\xE8\x839\x81Q\x91R\x80T\x83\x10\x15b\0\x11ZW_R`\x1C` _ \x83`\x03\x1C\x01\x92`\x02\x1B\x16\x90V[\x91\x90`\x01\x80`\xA0\x1B\x03\x92\x83\x81\x16\x80_R` \x90`\x18\x82Ra\xFF\xFF\x90`@\x96\x82\x88_ T\x16\x80\x15\x80\x15b\0\x1D\xC2WPP\x82`\x14T`\x08\x1C\x16\x83`\x17T\x16\x10b\0\x1D\x8CW\x82`\x17T\x16\x91\x82\x15b\0\x1D{W`\x01\x92\x83_R`\x19\x80\x87R\x83\x8B_ T\x16\x91\x89b\0\x1A\\\x84b\0\"\xDCV[\x10b\0\x1B\x96WPPP_R`\x1B\x84R\x82\x88_ T\x16\x91\x82\x15\x80\x15b\0\x1A\xD8WPPPPPPb\0\x1A\xD3\x7F\x19\xFE<\xA6\x03\xE8xT\xA0t|\xC1\n\xBF\x06\xBA\xC6Ma\xBA\xC7=m\x15\xF2\xFD<\xA4H\xF1Rd\x93\x94b\0\x1A\xB3\x83b\0\"'V[Q`\x01`\x01`\xA0\x1B\x03\x90\x92\x16\x82R` \x82\x01\x92\x90\x92R\x90\x81\x90`@\x82\x01\x90V[\x03\x90\xA1V[b\0\x1B\x85Wb\0\x1A\xE8\x86b\0\"\xDCV[\x92[\x81\x85\x82\x16\x11b\0\x1BAW[PP\x96Q`\x01`\x01`\xA0\x1B\x03\x90\x94\x16\x84RPPP` \x81\x01\x91\x90\x91R\x90\x91P\x7F\xDA\x14\x8F\xAC.\x10c\x17K\xE7\xBC\x08\x95Wk\xDA\xBA\x90\xFD\x14\xE5uF?\xA2j\x96|\xB8\x95\xCA\xDD\x90\x80`@\x81\x01b\0\x1A\xD3V[a\x7F\xFF\x81\x83\x1C\x16\x90\x81_R`\x1C\x87R\x84b\0\x1Ba\x85\x8D_ T\x16b\0\"\xDCV[\x10\x15b\0\x1B~W\x90b\0\x1Bv\x83\x92\x82b\0#\xFCV[\x90Pb\0\x1A\xEAV[Pb\0\x1A\xF5V[\x88Qc\xF2u^7`\xE0\x1B\x81R`\x04\x90\xFD[\x90\x93\x92\x94\x91\x97\x98Pb\0\x1B\xB0\x81\x83\x9C\x98\x97\x9C\x11\x15b\0#DV[\x81_R\x83\x86R\x82\x80\x88_ T\x16\x82_R\x81\x89_ T\x16\x92\x81_R`\x18\x89R\x89_ \x93a\xFF\xFF\x19\x94\x82\x86\x82T\x16\x17\x90U\x80_R\x8A_ \x86\x86\x82T\x16\x17\x90U\x81_R\x87\x8AR\x8A_ `\x01\x80`\xA0\x1B\x03\x19\x93\x84\x82T\x16\x17\x90U\x85_R\x8A_ \x90\x83\x82T\x16\x17\x90U\x8Db\0\x1C \x82b\0 \xA4V[\x16\x84`\x17T\x16\x17`\x17U_R\x85\x88R\x88_ \x80T\x91\x82\x16\x90U\x16_R`\x18\x86R\x86_ \x90\x81T\x16\x90U\x80_R\x82\x85Rb\0\x1C_\x82\x87_ T\x16b\0\"\xDCV[\x91\x81\x93\x82`\x02\x8D`\x17T\x16\x91[b\0\x1C\xFAW[PPPPPPP\x95`\x1Bb\0\x1A\xD3\x93\x92\x7F\xFA\xEB\x8D\xE7q\xB8g\xCF5\x7FkE\x9Ap\x02\xB6.\xC4]TJ\x80x\xA3\xEC\xD9\x12\0\xCC\x82mu\x97\x98_RR\x81_ T\x16b\0\x1C\xE9W[b\0\x1C\xBE\x84b\0\x1F\xDDV[b\0\x1C\xC9\x83b\0\"'V[Q`\x01`\x01`\xA0\x1B\x03\x92\x83\x16\x81R\x92\x90\x91\x16` \x83\x01R\x81\x90`@\x82\x01\x90V[b\0\x1C\xF4\x84b\0 \xB8V[b\0\x1C\xB3V[\x8D\x81\x16\x82\x81\x11b\0\x1DtW\x82\x81\x10\x15b\0\x1DVWP\x80b\0\x1D\x1Fb\0\x1D&\x92b\0\x1F\xC9V[\x90b\0$\xE7V[\x96\x90\x96[\x86\x11\x15b\0\x1DPWb\0\x1D>\x90\x87b\0#`V[b\0\x1DI\x86b\0#,V[\x84b\0\x1ClV[b\0\x1CrV[_\x97\x91\x97R\x82\x89Rb\0\x1Dn\x84\x8B_ T\x16b\0\"\xDCV[b\0\x1D*V[Pb\0\x1CrV[\x88Qc@\xD9\xB0\x11`\xE0\x1B\x81R`\x04\x90\xFD[PPPPb\0\x1A\xD3\x7F.\x808\xF5\x81/v<n\xF0\xC5B|\xB5\xA0\\\xE6\xD7\x06J\xF2\xBA}\x1D'\x0B\xC0&`\xB0\x19\xFD\x93\x94b\0\x1A\xB3\x83b\0\x1F\xDDV[\x90\x98\x93\x92P\x96\x95\x94\x93\x96b\0\x1E\xBDWb\0\x1D\xDC\x84b\0\"\xDCV[\x96b\0\x1D\xE8\x89b\0#,V[\x83`\x17T\x16\x90[\x84\x81\x16\x82\x81\x11b\0\x1E\x97W\x82\x81\x10\x15b\0\x1ExWP\x80b\0\x1D\x1Fb\0\x1E\x14\x92b\0\x1F\xC9V[\x9A\x90\x9A[\x8A\x11\x15b\0\x1E=Wb\0\x1E,\x90\x8Bb\0#`V[b\0\x1E7\x8Ab\0#,V[b\0\x1D\xEFV[PP\x92Q`\x01`\x01`\xA0\x1B\x03\x90\x94\x16\x84RPPP` \x81\x01\x91\x90\x91R\x90\x92P_\x80Q` b\0&\xC8\x839\x81Q\x91R\x91P\x80`@\x81\x01b\0\x1A\xD3V[_\x9B\x91\x9BR`\x19\x83Rb\0\x1E\x91\x84\x87_ T\x16b\0\"\xDCV[b\0\x1E\x18V[PPPPPPb\0\x1A\xD3\x91\x92\x93\x95P_\x80Q` b\0&\xC8\x839\x81Q\x91R\x94Pb\0\x1A\xB3V[\x82Qc\xF2u^7`\xE0\x1B\x81R`\x04\x90\xFD[` \x80\x82\x01Q\x83\x82\x01Q\x91\x92\x91`\x01`\x01`@\x1B\x03\x91\x82\x16\x91\x16\x03b\0\x1F\x80Wb\0\x1E\xF9\x81b\0\"\x9BV[b\0\x1F\x04\x84b\0\"\x9BV[\x03b\0\x1F\x80WQ\x80Q\x83QQ\x03b\0\x1F\x80Wb\0\x1Fyb\0\x1Fl\x91`@Q\x90\x81b\0\x1F:\x86\x82\x01\x92\x87\x84R`@\x83\x01\x90b\0\x12\x86V[\x03\x91b\0\x1FP`\x1F\x19\x93\x84\x81\x01\x83R\x82b\0\x10\xB9V[Q\x90 \x94Q`@Q\x93\x84\x91\x86\x83\x01\x96\x87R`@\x83\x01\x90b\0\x12\x86V[\x03\x90\x81\x01\x83R\x82b\0\x10\xB9V[Q\x90 \x14\x90V[PPP_\x90V[\x80;\x15b\0\x1F\x93WPPV[`@\x80Qc\x91\x984\xB9`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x90\x92\x16`\x04\x83\x01R`$\x82\x01R\x90\x81\x90b\0\np\x90`D\x83\x01\x90b\0\x11\xCCV[\x90`\x01a\xFF\xFF\x80\x93\x16\x01\x91\x82\x11b\0\n\x1CWV[\x90a\xFF\xFF\x90b\0\x1F\xF1\x82`\x17T\x16b\0\x1F\xC9V[`\x01`\x01`\xA0\x1B\x03\x84\x81\x16_\x81\x81R`\x18` \x90\x81R`@\x80\x83 \x80T\x89\x88\x16a\xFF\xFF\x19\x91\x82\x16\x81\x17\x90\x92U\x81\x85R`\x19\x80\x85R\x94\x83\x90 \x80T`\x01`\x01`\xA0\x1B\x03\x19\x16\x90\x96\x17\x90\x95U`\x17\x80T\x90\x95\x16\x17\x90\x93U\x96b\0 R\x90b\0\"\xDCV[\x93[`\x01\x80\x88\x83\x16\x11\x15b\0 \x99W\x81a\x7F\xFF\x91\x1C\x16\x90\x81_R\x82\x89R\x85b\0 \x80\x86\x86_ T\x16b\0\"\xDCV[\x11\x15b\0 \x99Wb\0 \x93\x90\x82b\0#`V[b\0 TV[PPPPPP\x91PPV[a\xFF\xFF\x90\x81\x16_\x19\x01\x91\x90\x82\x11b\0\n\x1CWV[`\x01\x80`\xA0\x1B\x03\x80\x91\x16_R` \x90`\x1B\x82R`@\x80_ T\x90a\xFF\xFF\x80\x83\x16\x93\x84\x15b\0\x1E\xBDW\x81`\x1AT\x16\x92b\0 \xF2\x84\x87b\0#\xFCV[\x85\x83b\0 \xFF\x86b\0 \xA4V[\x16\x94a\xFF\xFF\x19\x95\x86`\x1AT\x16\x17`\x1AU\x80_R`\x1C\x95`\x1C\x8AR\x84\x84_ \x80T\x90`\x01\x80`\xA0\x1B\x03\x19\x82\x16\x90U\x16_R`\x1B\x8AR\x83_ \x90\x81T\x16\x90U\x14b\0\x19\xB7W\x85_\x94\x93\x94R`\x1C\x87Rb\0!gb\0!`\x83\x83_ T\x16b\0\"\xDCV[\x87b\0$\x8DV[\x85_R`\x1C\x87Rb\0!~\x82\x82_ T\x16b\0\"\xDCV[\x95\x92a\xFF\xFE\x97\x88\x96`\x01\x97\x88\x91`\x01\x1B\x16\x87`\x1AT\x16\x91[b\0!\xA9W[PPPPPPPPPPPV[\x87\x81\x16\x82\x81\x11b\0\" W\x82\x81\x10\x15b\0\"\x02WP\x80b\0!\xCEb\0!\xD5\x92b\0\x1F\xC9V[\x90b\0%?V[\x97\x90\x97[\x8A\x10\x15b\0!\xFCWb\0!\xEE\x89\x97\x89b\0#\xFCV[\x96\x95\x86\x81\x1B\x8B\x16\x90b\0!\x96V[b\0!\x9CV[_\x98\x91\x98R\x83\x83Rb\0\"\x1A\x86\x86_ T\x16b\0\"\xDCV[b\0!\xD9V[Pb\0!\x9CV[b\0\x12k\x90b\0\"\x94a\xFF\xFF\x91b\0\"C\x83`\x1AT\x16b\0\x1F\xC9V[\x92`\x01\x80`\xA0\x1B\x03\x82\x16\x90\x81_R`\x1B` R`@_ \x90\x85\x16\x91a\xFF\xFF\x19\x91\x83\x83\x82T\x16\x17\x90U\x82_R`\x1C` R`@_ \x90`\x01\x80`\xA0\x1B\x03\x19\x82T\x16\x17\x90U`\x1AT\x16\x17`\x1AUb\0\"\xDCV[\x90b\0$\x8DV[\x80QQ\x90_\x91_\x91[\x81\x83\x10b\0\"\xB2WPPP\x90V[\x90\x91\x92b\0\"\xD2`\x01\x91b\0\"\xC9\x86\x85Qb\0\x11EV[QQ\x90b\0\x12xV[\x93\x01\x91\x90b\0\"\xA4V[`\x01`\xFF`\x14T\x16b\0\"\xEF\x81b\0\x11nV[\x03b\0#\x0FW`\x01`\x01`\xA0\x1B\x03\x16_\x90\x81R`\x16` R`@\x90 T\x90V[`\x01`\x01`\xA0\x1B\x03\x16_\x90\x81R`\x16` R`@\x90 `\x01\x01T\x90V[`\x01\x1B\x90b\x01\xFF\xFEa\xFF\xFE\x83\x16\x92\x16\x82\x03b\0\n\x1CWV[\x15b\0#LWV[cNH{q`\xE0\x1B_R`\x01`\x04R`$_\xFD[b\0#\x89a\xFF\xFF\x80\x80`\x17T\x16\x93\x16\x93b\0#~\x84\x86\x11\x15b\0#DV[\x16\x91\x82\x11\x15b\0#DV[_\x82\x81R`\x19` \x81\x81R`@\x80\x84 \x80T\x86\x86R\x82\x86 \x80T`\x01`\x01`\xA0\x1B\x03\x92\x83\x16\x80\x89R`\x18\x87R\x85\x89 \x80Ta\xFF\xFF\x19\x90\x81\x16\x90\x9B\x17\x90U\x92\x16\x80\x88R\x93\x87 \x80T\x90\x98\x16\x89\x17\x90\x97U\x93\x90\x92R\x84T`\x01`\x01`\xA0\x1B\x03\x19\x90\x81\x16\x90\x93\x17\x90\x94U\x93\x90\x91R\x82T\x16\x17\x90UV[b\0$\x1Aa\xFF\xFF\x80\x80`\x1AT\x16\x93\x16\x93b\0#~\x84\x86\x11\x15b\0#DV[_\x82\x81R`\x1C` \x81\x81R`@\x80\x84 \x80T\x86\x86R\x82\x86 \x80T`\x01`\x01`\xA0\x1B\x03\x92\x83\x16\x80\x89R`\x1B\x87R\x85\x89 \x80Ta\xFF\xFF\x19\x90\x81\x16\x90\x9B\x17\x90U\x92\x16\x80\x88R\x93\x87 \x80T\x90\x98\x16\x89\x17\x90\x97U\x93\x90\x92R\x84T`\x01`\x01`\xA0\x1B\x03\x19\x90\x81\x16\x90\x93\x17\x90\x94U\x93\x90\x91R\x82T\x16\x17\x90UV[\x91\x90\x91[`\x01\x80a\xFF\xFF\x83\x16\x11\x15b\0$\xE1W\x81a\x7F\xFF\x91\x1C\x16\x90\x81_R`\x1C` R\x83b\0$\xC8`\x01\x80`\xA0\x1B\x03`@_ T\x16b\0\"\xDCV[\x10\x15b\0$\xE1Wb\0$\xDB\x90\x82b\0#\xFCV[b\0$\x91V[PP\x90PV[\x91\x90a\xFF\xFF\x80\x84\x16_R`\x19` Rb\0%(`\x01\x80`\xA0\x1B\x03b\0%\x12\x81`@_ T\x16b\0\"\xDCV[\x92\x84\x16_R`\x19` R`@_ T\x16b\0\"\xDCV[\x93\x84\x82\x11\x15b\0%8WPP\x91\x90V[\x93P\x91\x90PV[\x91\x90\x91a\xFF\xFF\x92\x83\x82\x16_R`\x1C` Rb\0%\x82`\x01\x80`\xA0\x1B\x03b\0%l\x81`@_ T\x16b\0\"\xDCV[\x95\x83\x16_R`\x1C` R`@_ T\x16b\0\"\xDCV[\x90\x81\x85\x10b\0%8WPP\x91\x90V\xFE`\x80`@R6\x15a\0\x89W_\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x90\x91 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15a\0qW_\x80\x836\x82\x807\x816\x91Z\xF4=_\x80>\x15a\0mW=_\xF3[=_\xFD[`$\x90`@Q\x90c\n\x82\xDDs`\xE3\x1B\x82R`\x04\x82\x01R\xFD[_\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x90\x91 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15a\0\xECW_\x80\x836\x82\x807\x816\x91Z\xF4=_\x80>\x15a\0mW=_\xF3[c\n\x82\xDDs`\xE3\x1B`\x80R`\x84R`$`\x80\xFD\xFE\xA2dipfsX\"\x12 F\xC7\xB8\x01\xEB\xD7xx\x9Ed;\xBDU\x82\xFF\xB2\xD2\xC4\xBA\x0C\t\x1A\x04\xE7z\xA8\xA3H}/vqdsolcC\0\x08\x17\x003\x14=\xB2{\xC2\x03fS\xDCo\x96/\xF9\xD0\xB8\x03\x16=J\xF5\x0C%l\xA9\xE6\x92{=m\xCD\x01\x97\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2";
    /// The bytecode of the contract.
    pub static GATEWAYDIAMOND_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__BYTECODE);
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R6\x15a\0\x89W_\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x90\x91 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15a\0qW_\x80\x836\x82\x807\x816\x91Z\xF4=_\x80>\x15a\0mW=_\xF3[=_\xFD[`$\x90`@Q\x90c\n\x82\xDDs`\xE3\x1B\x82R`\x04\x82\x01R\xFD[_\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x90\x91 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15a\0\xECW_\x80\x836\x82\x807\x816\x91Z\xF4=_\x80>\x15a\0mW=_\xF3[c\n\x82\xDDs`\xE3\x1B`\x80R`\x84R`$`\x80\xFD\xFE\xA2dipfsX\"\x12 F\xC7\xB8\x01\xEB\xD7xx\x9Ed;\xBDU\x82\xFF\xB2\xD2\xC4\xBA\x0C\t\x1A\x04\xE7z\xA8\xA3H}/vqdsolcC\0\x08\x17\x003";
    /// The deployed bytecode of the contract.
    pub static GATEWAYDIAMOND_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__DEPLOYED_BYTECODE);
    pub struct GatewayDiamond<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for GatewayDiamond<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for GatewayDiamond<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for GatewayDiamond<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for GatewayDiamond<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(GatewayDiamond))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> GatewayDiamond<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                GATEWAYDIAMOND_ABI.clone(),
                client,
            ))
        }
        /// Constructs the general purpose `Deployer` instance based on the provided constructor arguments and sends it.
        /// Returns a new instance of a deployer that returns an instance of this contract after sending the transaction
        ///
        /// Notes:
        /// - If there are no constructor arguments, you should pass `()` as the argument.
        /// - The default poll duration is 7 seconds.
        /// - The default number of confirmations is 1 block.
        ///
        ///
        /// # Example
        ///
        /// Generate contract bindings with `abigen!` and deploy a new contract instance.
        ///
        /// *Note*: this requires a `bytecode` and `abi` object in the `greeter.json` artifact.
        ///
        /// ```ignore
        /// # async fn deploy<M: ethers::providers::Middleware>(client: ::std::sync::Arc<M>) {
        ///     abigen!(Greeter, "../greeter.json");
        ///
        ///    let greeter_contract = Greeter::deploy(client, "Hello world!".to_string()).unwrap().send().await.unwrap();
        ///    let msg = greeter_contract.greet().call().await.unwrap();
        /// # }
        /// ```
        pub fn deploy<T: ::ethers::core::abi::Tokenize>(
            client: ::std::sync::Arc<M>,
            constructor_args: T,
        ) -> ::core::result::Result<
            ::ethers::contract::builders::ContractDeployer<M, Self>,
            ::ethers::contract::ContractError<M>,
        > {
            let factory = ::ethers::contract::ContractFactory::new(
                GATEWAYDIAMOND_ABI.clone(),
                GATEWAYDIAMOND_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Gets the contract's `ActiveValidatorCollateralUpdated` event
        pub fn active_validator_collateral_updated_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ActiveValidatorCollateralUpdatedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ActiveValidatorReplaced` event
        pub fn active_validator_replaced_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ActiveValidatorReplacedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `DiamondCut` event
        pub fn diamond_cut_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, DiamondCutFilter> {
            self.0.event()
        }
        ///Gets the contract's `MembershipUpdated` event
        pub fn membership_updated_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, MembershipUpdatedFilter>
        {
            self.0.event()
        }
        ///Gets the contract's `NewActiveValidator` event
        pub fn new_active_validator_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, NewActiveValidatorFilter>
        {
            self.0.event()
        }
        ///Gets the contract's `NewWaitingValidator` event
        pub fn new_waiting_validator_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, NewWaitingValidatorFilter>
        {
            self.0.event()
        }
        ///Gets the contract's `OwnershipTransferred` event
        pub fn ownership_transferred_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, OwnershipTransferredFilter>
        {
            self.0.event()
        }
        ///Gets the contract's `WaitingValidatorCollateralUpdated` event
        pub fn waiting_validator_collateral_updated_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            WaitingValidatorCollateralUpdatedFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, GatewayDiamondEvents>
        {
            self.0
                .event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
        for GatewayDiamond<M>
    {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `CannotAddFunctionToDiamondThatAlreadyExists` with signature `CannotAddFunctionToDiamondThatAlreadyExists(bytes4)` and selector `0xebbf5d07`
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
        name = "CannotAddFunctionToDiamondThatAlreadyExists",
        abi = "CannotAddFunctionToDiamondThatAlreadyExists(bytes4)"
    )]
    pub struct CannotAddFunctionToDiamondThatAlreadyExists {
        pub selector: [u8; 4],
    }
    ///Custom Error type `CannotAddSelectorsToZeroAddress` with signature `CannotAddSelectorsToZeroAddress(bytes4[])` and selector `0x0ae3681c`
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
        name = "CannotAddSelectorsToZeroAddress",
        abi = "CannotAddSelectorsToZeroAddress(bytes4[])"
    )]
    pub struct CannotAddSelectorsToZeroAddress {
        pub selectors: ::std::vec::Vec<[u8; 4]>,
    }
    ///Custom Error type `CannotRemoveFunctionThatDoesNotExist` with signature `CannotRemoveFunctionThatDoesNotExist(bytes4)` and selector `0x7a08a22d`
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
        name = "CannotRemoveFunctionThatDoesNotExist",
        abi = "CannotRemoveFunctionThatDoesNotExist(bytes4)"
    )]
    pub struct CannotRemoveFunctionThatDoesNotExist {
        pub selector: [u8; 4],
    }
    ///Custom Error type `CannotRemoveImmutableFunction` with signature `CannotRemoveImmutableFunction(bytes4)` and selector `0x6fafeb08`
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
        name = "CannotRemoveImmutableFunction",
        abi = "CannotRemoveImmutableFunction(bytes4)"
    )]
    pub struct CannotRemoveImmutableFunction {
        pub selector: [u8; 4],
    }
    ///Custom Error type `CannotReplaceFunctionThatDoesNotExists` with signature `CannotReplaceFunctionThatDoesNotExists(bytes4)` and selector `0x7479f939`
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
        name = "CannotReplaceFunctionThatDoesNotExists",
        abi = "CannotReplaceFunctionThatDoesNotExists(bytes4)"
    )]
    pub struct CannotReplaceFunctionThatDoesNotExists {
        pub selector: [u8; 4],
    }
    ///Custom Error type `CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet` with signature `CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(bytes4)` and selector `0x358d9d1a`
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
        name = "CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet",
        abi = "CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(bytes4)"
    )]
    pub struct CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet {
        pub selector: [u8; 4],
    }
    ///Custom Error type `CannotReplaceFunctionsFromFacetWithZeroAddress` with signature `CannotReplaceFunctionsFromFacetWithZeroAddress(bytes4[])` and selector `0xcd98a96f`
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
        name = "CannotReplaceFunctionsFromFacetWithZeroAddress",
        abi = "CannotReplaceFunctionsFromFacetWithZeroAddress(bytes4[])"
    )]
    pub struct CannotReplaceFunctionsFromFacetWithZeroAddress {
        pub selectors: ::std::vec::Vec<[u8; 4]>,
    }
    ///Custom Error type `CannotReplaceImmutableFunction` with signature `CannotReplaceImmutableFunction(bytes4)` and selector `0x520300da`
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
        name = "CannotReplaceImmutableFunction",
        abi = "CannotReplaceImmutableFunction(bytes4)"
    )]
    pub struct CannotReplaceImmutableFunction {
        pub selector: [u8; 4],
    }
    ///Custom Error type `FunctionNotFound` with signature `FunctionNotFound(bytes4)` and selector `0x5416eb98`
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
    #[etherror(name = "FunctionNotFound", abi = "FunctionNotFound(bytes4)")]
    pub struct FunctionNotFound {
        pub function_selector: [u8; 4],
    }
    ///Custom Error type `IncorrectFacetCutAction` with signature `IncorrectFacetCutAction(uint8)` and selector `0x7fe9a41e`
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
        name = "IncorrectFacetCutAction",
        abi = "IncorrectFacetCutAction(uint8)"
    )]
    pub struct IncorrectFacetCutAction {
        pub action: u8,
    }
    ///Custom Error type `InitializationFunctionReverted` with signature `InitializationFunctionReverted(address,bytes)` and selector `0x192105d7`
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
        name = "InitializationFunctionReverted",
        abi = "InitializationFunctionReverted(address,bytes)"
    )]
    pub struct InitializationFunctionReverted {
        pub initialization_contract_address: ::ethers::core::types::Address,
        pub calldata: ::ethers::core::types::Bytes,
    }
    ///Custom Error type `InvalidMajorityPercentage` with signature `InvalidMajorityPercentage()` and selector `0x75c3b427`
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
        name = "InvalidMajorityPercentage",
        abi = "InvalidMajorityPercentage()"
    )]
    pub struct InvalidMajorityPercentage;
    ///Custom Error type `InvalidSubmissionPeriod` with signature `InvalidSubmissionPeriod()` and selector `0x312f8e05`
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
    #[etherror(name = "InvalidSubmissionPeriod", abi = "InvalidSubmissionPeriod()")]
    pub struct InvalidSubmissionPeriod;
    ///Custom Error type `NoBytecodeAtAddress` with signature `NoBytecodeAtAddress(address,string)` and selector `0x919834b9`
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
        name = "NoBytecodeAtAddress",
        abi = "NoBytecodeAtAddress(address,string)"
    )]
    pub struct NoBytecodeAtAddress {
        pub contract_address: ::ethers::core::types::Address,
        pub message: ::std::string::String,
    }
    ///Custom Error type `NoSelectorsProvidedForFacetForCut` with signature `NoSelectorsProvidedForFacetForCut(address)` and selector `0xe767f91f`
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
        name = "NoSelectorsProvidedForFacetForCut",
        abi = "NoSelectorsProvidedForFacetForCut(address)"
    )]
    pub struct NoSelectorsProvidedForFacetForCut {
        pub facet_address: ::ethers::core::types::Address,
    }
    ///Custom Error type `OldConfigurationNumber` with signature `OldConfigurationNumber()` and selector `0x6e8d7c4a`
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
        Hash,
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
        Hash,
    )]
    #[etherror(name = "PQEmpty", abi = "PQEmpty()")]
    pub struct PQEmpty;
    ///Custom Error type `RemoveFacetAddressMustBeZeroAddress` with signature `RemoveFacetAddressMustBeZeroAddress(address)` and selector `0xd091bc81`
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
        name = "RemoveFacetAddressMustBeZeroAddress",
        abi = "RemoveFacetAddressMustBeZeroAddress(address)"
    )]
    pub struct RemoveFacetAddressMustBeZeroAddress {
        pub facet_address: ::ethers::core::types::Address,
    }
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum GatewayDiamondErrors {
        CannotAddFunctionToDiamondThatAlreadyExists(CannotAddFunctionToDiamondThatAlreadyExists),
        CannotAddSelectorsToZeroAddress(CannotAddSelectorsToZeroAddress),
        CannotRemoveFunctionThatDoesNotExist(CannotRemoveFunctionThatDoesNotExist),
        CannotRemoveImmutableFunction(CannotRemoveImmutableFunction),
        CannotReplaceFunctionThatDoesNotExists(CannotReplaceFunctionThatDoesNotExists),
        CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(
            CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet,
        ),
        CannotReplaceFunctionsFromFacetWithZeroAddress(
            CannotReplaceFunctionsFromFacetWithZeroAddress,
        ),
        CannotReplaceImmutableFunction(CannotReplaceImmutableFunction),
        FunctionNotFound(FunctionNotFound),
        IncorrectFacetCutAction(IncorrectFacetCutAction),
        InitializationFunctionReverted(InitializationFunctionReverted),
        InvalidMajorityPercentage(InvalidMajorityPercentage),
        InvalidSubmissionPeriod(InvalidSubmissionPeriod),
        NoBytecodeAtAddress(NoBytecodeAtAddress),
        NoSelectorsProvidedForFacetForCut(NoSelectorsProvidedForFacetForCut),
        OldConfigurationNumber(OldConfigurationNumber),
        PQDoesNotContainAddress(PQDoesNotContainAddress),
        PQEmpty(PQEmpty),
        RemoveFacetAddressMustBeZeroAddress(RemoveFacetAddressMustBeZeroAddress),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for GatewayDiamondErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) =
                <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded) = <CannotAddFunctionToDiamondThatAlreadyExists as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CannotAddFunctionToDiamondThatAlreadyExists(decoded));
            }
            if let Ok(decoded) =
                <CannotAddSelectorsToZeroAddress as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::CannotAddSelectorsToZeroAddress(decoded));
            }
            if let Ok(decoded) =
                <CannotRemoveFunctionThatDoesNotExist as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                )
            {
                return Ok(Self::CannotRemoveFunctionThatDoesNotExist(decoded));
            }
            if let Ok(decoded) =
                <CannotRemoveImmutableFunction as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::CannotRemoveImmutableFunction(decoded));
            }
            if let Ok(decoded) =
                <CannotReplaceFunctionThatDoesNotExists as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                )
            {
                return Ok(Self::CannotReplaceFunctionThatDoesNotExists(decoded));
            }
            if let Ok(decoded) = <CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(
                    Self::CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(
                        decoded,
                    ),
                );
            }
            if let Ok(decoded) = <CannotReplaceFunctionsFromFacetWithZeroAddress as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CannotReplaceFunctionsFromFacetWithZeroAddress(decoded));
            }
            if let Ok(decoded) =
                <CannotReplaceImmutableFunction as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::CannotReplaceImmutableFunction(decoded));
            }
            if let Ok(decoded) = <FunctionNotFound as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::FunctionNotFound(decoded));
            }
            if let Ok(decoded) =
                <IncorrectFacetCutAction as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::IncorrectFacetCutAction(decoded));
            }
            if let Ok(decoded) =
                <InitializationFunctionReverted as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::InitializationFunctionReverted(decoded));
            }
            if let Ok(decoded) =
                <InvalidMajorityPercentage as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::InvalidMajorityPercentage(decoded));
            }
            if let Ok(decoded) =
                <InvalidSubmissionPeriod as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::InvalidSubmissionPeriod(decoded));
            }
            if let Ok(decoded) =
                <NoBytecodeAtAddress as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NoBytecodeAtAddress(decoded));
            }
            if let Ok(decoded) =
                <NoSelectorsProvidedForFacetForCut as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NoSelectorsProvidedForFacetForCut(decoded));
            }
            if let Ok(decoded) =
                <OldConfigurationNumber as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::OldConfigurationNumber(decoded));
            }
            if let Ok(decoded) =
                <PQDoesNotContainAddress as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::PQDoesNotContainAddress(decoded));
            }
            if let Ok(decoded) = <PQEmpty as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::PQEmpty(decoded));
            }
            if let Ok(decoded) =
                <RemoveFacetAddressMustBeZeroAddress as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                )
            {
                return Ok(Self::RemoveFacetAddressMustBeZeroAddress(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for GatewayDiamondErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::CannotAddFunctionToDiamondThatAlreadyExists(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotAddSelectorsToZeroAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotRemoveFunctionThatDoesNotExist(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotRemoveImmutableFunction(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotReplaceFunctionThatDoesNotExists(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotReplaceFunctionsFromFacetWithZeroAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotReplaceImmutableFunction(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FunctionNotFound(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::IncorrectFacetCutAction(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitializationFunctionReverted(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidMajorityPercentage(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidSubmissionPeriod(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NoBytecodeAtAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NoSelectorsProvidedForFacetForCut(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OldConfigurationNumber(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PQDoesNotContainAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PQEmpty(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::RemoveFacetAddressMustBeZeroAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for GatewayDiamondErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <CannotAddFunctionToDiamondThatAlreadyExists as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CannotAddSelectorsToZeroAddress as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CannotRemoveFunctionThatDoesNotExist as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CannotRemoveImmutableFunction as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CannotReplaceFunctionThatDoesNotExists as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CannotReplaceFunctionsFromFacetWithZeroAddress as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CannotReplaceImmutableFunction as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <FunctionNotFound as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <IncorrectFacetCutAction as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InitializationFunctionReverted as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidMajorityPercentage as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidSubmissionPeriod as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NoBytecodeAtAddress as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NoSelectorsProvidedForFacetForCut as ::ethers::contract::EthError>::selector() => {
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
                    == <RemoveFacetAddressMustBeZeroAddress as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for GatewayDiamondErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::CannotAddFunctionToDiamondThatAlreadyExists(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotAddSelectorsToZeroAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotRemoveFunctionThatDoesNotExist(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotRemoveImmutableFunction(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotReplaceFunctionThatDoesNotExists(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotReplaceFunctionsFromFacetWithZeroAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotReplaceImmutableFunction(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::FunctionNotFound(element) => ::core::fmt::Display::fmt(element, f),
                Self::IncorrectFacetCutAction(element) => ::core::fmt::Display::fmt(element, f),
                Self::InitializationFunctionReverted(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidMajorityPercentage(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidSubmissionPeriod(element) => ::core::fmt::Display::fmt(element, f),
                Self::NoBytecodeAtAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::NoSelectorsProvidedForFacetForCut(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OldConfigurationNumber(element) => ::core::fmt::Display::fmt(element, f),
                Self::PQDoesNotContainAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::PQEmpty(element) => ::core::fmt::Display::fmt(element, f),
                Self::RemoveFacetAddressMustBeZeroAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for GatewayDiamondErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<CannotAddFunctionToDiamondThatAlreadyExists> for GatewayDiamondErrors {
        fn from(value: CannotAddFunctionToDiamondThatAlreadyExists) -> Self {
            Self::CannotAddFunctionToDiamondThatAlreadyExists(value)
        }
    }
    impl ::core::convert::From<CannotAddSelectorsToZeroAddress> for GatewayDiamondErrors {
        fn from(value: CannotAddSelectorsToZeroAddress) -> Self {
            Self::CannotAddSelectorsToZeroAddress(value)
        }
    }
    impl ::core::convert::From<CannotRemoveFunctionThatDoesNotExist> for GatewayDiamondErrors {
        fn from(value: CannotRemoveFunctionThatDoesNotExist) -> Self {
            Self::CannotRemoveFunctionThatDoesNotExist(value)
        }
    }
    impl ::core::convert::From<CannotRemoveImmutableFunction> for GatewayDiamondErrors {
        fn from(value: CannotRemoveImmutableFunction) -> Self {
            Self::CannotRemoveImmutableFunction(value)
        }
    }
    impl ::core::convert::From<CannotReplaceFunctionThatDoesNotExists> for GatewayDiamondErrors {
        fn from(value: CannotReplaceFunctionThatDoesNotExists) -> Self {
            Self::CannotReplaceFunctionThatDoesNotExists(value)
        }
    }
    impl ::core::convert::From<CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet>
        for GatewayDiamondErrors
    {
        fn from(value: CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet) -> Self {
            Self::CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(value)
        }
    }
    impl ::core::convert::From<CannotReplaceFunctionsFromFacetWithZeroAddress>
        for GatewayDiamondErrors
    {
        fn from(value: CannotReplaceFunctionsFromFacetWithZeroAddress) -> Self {
            Self::CannotReplaceFunctionsFromFacetWithZeroAddress(value)
        }
    }
    impl ::core::convert::From<CannotReplaceImmutableFunction> for GatewayDiamondErrors {
        fn from(value: CannotReplaceImmutableFunction) -> Self {
            Self::CannotReplaceImmutableFunction(value)
        }
    }
    impl ::core::convert::From<FunctionNotFound> for GatewayDiamondErrors {
        fn from(value: FunctionNotFound) -> Self {
            Self::FunctionNotFound(value)
        }
    }
    impl ::core::convert::From<IncorrectFacetCutAction> for GatewayDiamondErrors {
        fn from(value: IncorrectFacetCutAction) -> Self {
            Self::IncorrectFacetCutAction(value)
        }
    }
    impl ::core::convert::From<InitializationFunctionReverted> for GatewayDiamondErrors {
        fn from(value: InitializationFunctionReverted) -> Self {
            Self::InitializationFunctionReverted(value)
        }
    }
    impl ::core::convert::From<InvalidMajorityPercentage> for GatewayDiamondErrors {
        fn from(value: InvalidMajorityPercentage) -> Self {
            Self::InvalidMajorityPercentage(value)
        }
    }
    impl ::core::convert::From<InvalidSubmissionPeriod> for GatewayDiamondErrors {
        fn from(value: InvalidSubmissionPeriod) -> Self {
            Self::InvalidSubmissionPeriod(value)
        }
    }
    impl ::core::convert::From<NoBytecodeAtAddress> for GatewayDiamondErrors {
        fn from(value: NoBytecodeAtAddress) -> Self {
            Self::NoBytecodeAtAddress(value)
        }
    }
    impl ::core::convert::From<NoSelectorsProvidedForFacetForCut> for GatewayDiamondErrors {
        fn from(value: NoSelectorsProvidedForFacetForCut) -> Self {
            Self::NoSelectorsProvidedForFacetForCut(value)
        }
    }
    impl ::core::convert::From<OldConfigurationNumber> for GatewayDiamondErrors {
        fn from(value: OldConfigurationNumber) -> Self {
            Self::OldConfigurationNumber(value)
        }
    }
    impl ::core::convert::From<PQDoesNotContainAddress> for GatewayDiamondErrors {
        fn from(value: PQDoesNotContainAddress) -> Self {
            Self::PQDoesNotContainAddress(value)
        }
    }
    impl ::core::convert::From<PQEmpty> for GatewayDiamondErrors {
        fn from(value: PQEmpty) -> Self {
            Self::PQEmpty(value)
        }
    }
    impl ::core::convert::From<RemoveFacetAddressMustBeZeroAddress> for GatewayDiamondErrors {
        fn from(value: RemoveFacetAddressMustBeZeroAddress) -> Self {
            Self::RemoveFacetAddressMustBeZeroAddress(value)
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
    #[ethevent(
        name = "ActiveValidatorCollateralUpdated",
        abi = "ActiveValidatorCollateralUpdated(address,uint256)"
    )]
    pub struct ActiveValidatorCollateralUpdatedFilter {
        pub validator: ::ethers::core::types::Address,
        pub new_power: ::ethers::core::types::U256,
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
        name = "ActiveValidatorReplaced",
        abi = "ActiveValidatorReplaced(address,address)"
    )]
    pub struct ActiveValidatorReplacedFilter {
        pub old_validator: ::ethers::core::types::Address,
        pub new_validator: ::ethers::core::types::Address,
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
        name = "DiamondCut",
        abi = "DiamondCut((address,uint8,bytes4[])[],address,bytes)"
    )]
    pub struct DiamondCutFilter {
        pub diamond_cut: ::std::vec::Vec<FacetCut>,
        pub init: ::ethers::core::types::Address,
        pub calldata: ::ethers::core::types::Bytes,
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
        name = "MembershipUpdated",
        abi = "MembershipUpdated(((uint256,address,bytes)[],uint64))"
    )]
    pub struct MembershipUpdatedFilter(pub Membership);
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
        name = "NewActiveValidator",
        abi = "NewActiveValidator(address,uint256)"
    )]
    pub struct NewActiveValidatorFilter {
        pub validator: ::ethers::core::types::Address,
        pub power: ::ethers::core::types::U256,
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
        name = "NewWaitingValidator",
        abi = "NewWaitingValidator(address,uint256)"
    )]
    pub struct NewWaitingValidatorFilter {
        pub validator: ::ethers::core::types::Address,
        pub power: ::ethers::core::types::U256,
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
        name = "OwnershipTransferred",
        abi = "OwnershipTransferred(address,address)"
    )]
    pub struct OwnershipTransferredFilter {
        pub old_owner: ::ethers::core::types::Address,
        pub new_owner: ::ethers::core::types::Address,
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
        name = "WaitingValidatorCollateralUpdated",
        abi = "WaitingValidatorCollateralUpdated(address,uint256)"
    )]
    pub struct WaitingValidatorCollateralUpdatedFilter {
        pub validator: ::ethers::core::types::Address,
        pub new_power: ::ethers::core::types::U256,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum GatewayDiamondEvents {
        ActiveValidatorCollateralUpdatedFilter(ActiveValidatorCollateralUpdatedFilter),
        ActiveValidatorReplacedFilter(ActiveValidatorReplacedFilter),
        DiamondCutFilter(DiamondCutFilter),
        MembershipUpdatedFilter(MembershipUpdatedFilter),
        NewActiveValidatorFilter(NewActiveValidatorFilter),
        NewWaitingValidatorFilter(NewWaitingValidatorFilter),
        OwnershipTransferredFilter(OwnershipTransferredFilter),
        WaitingValidatorCollateralUpdatedFilter(WaitingValidatorCollateralUpdatedFilter),
    }
    impl ::ethers::contract::EthLogDecode for GatewayDiamondEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = ActiveValidatorCollateralUpdatedFilter::decode_log(log) {
                return Ok(GatewayDiamondEvents::ActiveValidatorCollateralUpdatedFilter(decoded));
            }
            if let Ok(decoded) = ActiveValidatorReplacedFilter::decode_log(log) {
                return Ok(GatewayDiamondEvents::ActiveValidatorReplacedFilter(decoded));
            }
            if let Ok(decoded) = DiamondCutFilter::decode_log(log) {
                return Ok(GatewayDiamondEvents::DiamondCutFilter(decoded));
            }
            if let Ok(decoded) = MembershipUpdatedFilter::decode_log(log) {
                return Ok(GatewayDiamondEvents::MembershipUpdatedFilter(decoded));
            }
            if let Ok(decoded) = NewActiveValidatorFilter::decode_log(log) {
                return Ok(GatewayDiamondEvents::NewActiveValidatorFilter(decoded));
            }
            if let Ok(decoded) = NewWaitingValidatorFilter::decode_log(log) {
                return Ok(GatewayDiamondEvents::NewWaitingValidatorFilter(decoded));
            }
            if let Ok(decoded) = OwnershipTransferredFilter::decode_log(log) {
                return Ok(GatewayDiamondEvents::OwnershipTransferredFilter(decoded));
            }
            if let Ok(decoded) = WaitingValidatorCollateralUpdatedFilter::decode_log(log) {
                return Ok(GatewayDiamondEvents::WaitingValidatorCollateralUpdatedFilter(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for GatewayDiamondEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::ActiveValidatorCollateralUpdatedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ActiveValidatorReplacedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::DiamondCutFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::MembershipUpdatedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::NewActiveValidatorFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::NewWaitingValidatorFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::OwnershipTransferredFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::WaitingValidatorCollateralUpdatedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<ActiveValidatorCollateralUpdatedFilter> for GatewayDiamondEvents {
        fn from(value: ActiveValidatorCollateralUpdatedFilter) -> Self {
            Self::ActiveValidatorCollateralUpdatedFilter(value)
        }
    }
    impl ::core::convert::From<ActiveValidatorReplacedFilter> for GatewayDiamondEvents {
        fn from(value: ActiveValidatorReplacedFilter) -> Self {
            Self::ActiveValidatorReplacedFilter(value)
        }
    }
    impl ::core::convert::From<DiamondCutFilter> for GatewayDiamondEvents {
        fn from(value: DiamondCutFilter) -> Self {
            Self::DiamondCutFilter(value)
        }
    }
    impl ::core::convert::From<MembershipUpdatedFilter> for GatewayDiamondEvents {
        fn from(value: MembershipUpdatedFilter) -> Self {
            Self::MembershipUpdatedFilter(value)
        }
    }
    impl ::core::convert::From<NewActiveValidatorFilter> for GatewayDiamondEvents {
        fn from(value: NewActiveValidatorFilter) -> Self {
            Self::NewActiveValidatorFilter(value)
        }
    }
    impl ::core::convert::From<NewWaitingValidatorFilter> for GatewayDiamondEvents {
        fn from(value: NewWaitingValidatorFilter) -> Self {
            Self::NewWaitingValidatorFilter(value)
        }
    }
    impl ::core::convert::From<OwnershipTransferredFilter> for GatewayDiamondEvents {
        fn from(value: OwnershipTransferredFilter) -> Self {
            Self::OwnershipTransferredFilter(value)
        }
    }
    impl ::core::convert::From<WaitingValidatorCollateralUpdatedFilter> for GatewayDiamondEvents {
        fn from(value: WaitingValidatorCollateralUpdatedFilter) -> Self {
            Self::WaitingValidatorCollateralUpdatedFilter(value)
        }
    }
    ///`ConstructorParams(uint256,uint16,uint8,(uint64,address[]),(uint256,address,bytes)[],bytes32)`
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
    pub struct ConstructorParams {
        pub bottom_up_check_period: ::ethers::core::types::U256,
        pub active_validators_limit: u16,
        pub majority_percentage: u8,
        pub network_name: SubnetID,
        pub genesis_validators: ::std::vec::Vec<Validator>,
        pub commit_sha: [u8; 32],
    }
    ///`FacetCut(address,uint8,bytes4[])`
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
    pub struct FacetCut {
        pub facet_address: ::ethers::core::types::Address,
        pub action: u8,
        pub function_selectors: ::std::vec::Vec<[u8; 4]>,
    }
    ///`Membership((uint256,address,bytes)[],uint64)`
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
    pub struct Membership {
        pub validators: ::std::vec::Vec<Validator>,
        pub configuration_number: u64,
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
    ///`Validator(uint256,address,bytes)`
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
    pub struct Validator {
        pub weight: ::ethers::core::types::U256,
        pub addr: ::ethers::core::types::Address,
        pub metadata: ::ethers::core::types::Bytes,
    }
}
