pub use subnet_registry_diamond::*;
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
pub mod subnet_registry_diamond {
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
                                ::ethers::core::abi::ethabi::ParamType::Address,
                                ::ethers::core::abi::ethabi::ParamType::Address,
                                ::ethers::core::abi::ethabi::ParamType::Address,
                                ::ethers::core::abi::ethabi::ParamType::Address,
                                ::ethers::core::abi::ethabi::ParamType::Address,
                                ::ethers::core::abi::ethabi::ParamType::Address,
                                ::ethers::core::abi::ethabi::ParamType::Address,
                                ::ethers::core::abi::ethabi::ParamType::Address,
                                ::ethers::core::abi::ethabi::ParamType::Address,
                                ::ethers::core::abi::ethabi::ParamType::Address,
                                ::ethers::core::abi::ethabi::ParamType::Array(
                                    ::std::boxed::Box::new(
                                        ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                    ),
                                ),
                                ::ethers::core::abi::ethabi::ParamType::Array(
                                    ::std::boxed::Box::new(
                                        ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                    ),
                                ),
                                ::ethers::core::abi::ethabi::ParamType::Array(
                                    ::std::boxed::Box::new(
                                        ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                    ),
                                ),
                                ::ethers::core::abi::ethabi::ParamType::Array(
                                    ::std::boxed::Box::new(
                                        ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                    ),
                                ),
                                ::ethers::core::abi::ethabi::ParamType::Array(
                                    ::std::boxed::Box::new(
                                        ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                    ),
                                ),
                                ::ethers::core::abi::ethabi::ParamType::Array(
                                    ::std::boxed::Box::new(
                                        ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                    ),
                                ),
                                ::ethers::core::abi::ethabi::ParamType::Array(
                                    ::std::boxed::Box::new(
                                        ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                    ),
                                ),
                                ::ethers::core::abi::ethabi::ParamType::Array(
                                    ::std::boxed::Box::new(
                                        ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                    ),
                                ),
                                ::ethers::core::abi::ethabi::ParamType::Array(
                                    ::std::boxed::Box::new(
                                        ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                    ),
                                ),
                                ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                            ],
                        ),
                        internal_type: ::core::option::Option::Some(
                            ::std::borrow::ToOwned::to_owned(
                                "struct SubnetRegistryDiamond.ConstructorParams",
                            ),
                        ),
                    },
                ],
            }),
            functions: ::std::collections::BTreeMap::new(),
            events: ::core::convert::From::from([
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
                    ::std::borrow::ToOwned::to_owned("FacetCannotBeZero"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("FacetCannotBeZero"),
                            inputs: ::std::vec![],
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
                    ::std::borrow::ToOwned::to_owned("GatewayCannotBeZero"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "GatewayCannotBeZero",
                            ),
                            inputs: ::std::vec![],
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
    pub static SUBNETREGISTRYDIAMOND_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(__abi);
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = b"`\x80`@R4b\0\x1ClWb\0 \xEE\x808\x03\x90\x81b\0\0\x1E\x81b\0\x1DhV[\x91\x829`@\x81\x83\x81\x01\x03\x12b\0\x1ClW\x80Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x1ClW\x81\x01\x91\x80\x82\x01`\x1F\x84\x01\x12\x15b\0\x1ClW\x82Q\x90b\0\0hb\0\0b\x83b\0\x1D\x8EV[b\0\x1DhV[\x93` \x85\x84\x81R\x01` \x81\x94`\x05\x1B\x83\x01\x01\x91\x83\x86\x01\x83\x11b\0\x1ClW` \x81\x01\x91[\x83\x83\x10b\0\x1CqWPPPP` \x83\x01Q\x91`\x01`\x01`@\x1B\x03\x83\x11b\0\x1ClWa\x02\x80\x94\x85\x84\x86\x01\x84\x87\x01\x03\x12b\0\x1ClW`@Q\x95\x86\x01`\x01`\x01`@\x1B\x03\x81\x11\x87\x82\x10\x17b\0\x0B\xCBW`@Rb\0\0\xE7\x84\x86\x01b\0\x1D\xA6V[\x86Rb\0\0\xF9` \x85\x87\x01\x01b\0\x1D\xA6V[` \x87\x01Rb\0\x01\x0E`@\x85\x87\x01\x01b\0\x1D\xA6V[`@\x87\x01Rb\0\x01#``\x85\x87\x01\x01b\0\x1D\xA6V[``\x87\x01Rb\0\x018`\x80\x85\x87\x01\x01b\0\x1D\xA6V[`\x80\x87\x01Rb\0\x01M`\xA0\x85\x87\x01\x01b\0\x1D\xA6V[`\xA0\x87\x01Rb\0\x01b`\xC0\x85\x87\x01\x01b\0\x1D\xA6V[`\xC0\x87\x01Rb\0\x01w`\xE0\x85\x87\x01\x01b\0\x1D\xA6V[`\xE0\x87\x01Ra\x01\0\x90b\0\x01\x8F\x82\x86\x88\x01\x01b\0\x1D\xA6V[\x82\x88\x01Ra\x01 \x92b\0\x01\xA6\x84\x87\x89\x01\x01b\0\x1D\xA6V[\x88\x85\x01R\x86\x86\x01a\x01@\x01Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x1ClWb\0\x01\xD4\x90\x86\x89\x01\x90\x88\x8A\x01\x01b\0\x1D\xBBV[a\x01@\x89\x01Ra\x01`\x86\x88\x01\x81\x01Q\x90\x95\x90`\x01`\x01`@\x1B\x03\x81\x11b\0\x1ClWb\0\x02\x08\x90\x82\x8A\x01\x90\x89\x8B\x01\x01b\0\x1D\xBBV[\x86\x8A\x01Ra\x01\x80\x87\x89\x01\x81\x01Q\x90\x97\x90`\x01`\x01`@\x1B\x03\x81\x11b\0\x1ClWb\0\x02:\x90\x83\x8B\x01\x90\x83\x8C\x01\x01b\0\x1D\xBBV[\x8A\x89\x01R\x88\x81\x01a\x01\xA0\x01Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x1ClWb\0\x02h\x90\x83\x8B\x01\x90\x83\x8C\x01\x01b\0\x1D\xBBV[a\x01\xA0\x8B\x01Ra\x01\xC0\x89\x82\x01\x81\x01Q\x90\x99\x90`\x01`\x01`@\x1B\x03\x81\x11b\0\x1ClWb\0\x02\x9C\x90\x84\x83\x01\x90\x84\x84\x01\x01b\0\x1D\xBBV[\x8B\x8B\x01R\x80\x82\x01a\x01\xE0\x01Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x1ClWb\0\x02\xCA\x90\x84\x83\x01\x90\x84\x84\x01\x01b\0\x1D\xBBV[a\x01\xE0\x8C\x01R\x80\x82\x01a\x02\0\x01Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x1ClWb\0\x02\xFA\x90\x84\x83\x01\x90\x84\x84\x01\x01b\0\x1D\xBBV[a\x02\0\x8C\x01R\x80\x82\x01a\x02 \x01Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x1ClWb\0\x03*\x90\x84\x83\x01\x90\x84\x84\x01\x01b\0\x1D\xBBV[a\x02 \x8C\x01R\x80\x82\x01a\x02@\x01Q\x92`\x01`\x01`@\x1B\x03\x84\x11b\0\x1ClWa\x02`\x93b\0\x03^\x91\x83\x01\x90\x84\x84\x01\x01b\0\x1D\xBBV[a\x02@\x8C\x01R\x01\x01Q`\x02\x81\x10\x15b\0\x1ClWa\x02`\x89\x01R\x87Q`\x01`\x01`\xA0\x1B\x03\x16\x15b\0\x1CZW` \x88\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x15b\0\x1CHW`@\x88\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x15b\0\x1CHW``\x88\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x15b\0\x1CHW`\x80\x88\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x15b\0\x1CHW`\xA0\x88\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x15b\0\x1CHW`\xC0\x88\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x15b\0\x1CHW`\xE0\x88\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x15b\0\x1CHW\x87\x83\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x15b\0\x1CHW\x87\x84\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x15b\0\x1CHW\x7F\x8B\xE0\x07\x9CS\x16Y\x14\x13D\xCD\x1F\xD0\xA4\xF2\x84\x19I\x7F\x97\"\xA3\xDA\xAF\xE3\xB4\x18okdW\xE0`@`\0\x80Q` b\0 \x8E\x839\x81Q\x91RT3`\x01\x80`\xA0\x1B\x03\x19\x82\x16\x17`\0\x80Q` b\0 \x8E\x839\x81Q\x91RU\x81Q\x90`\x01\x80`\xA0\x1B\x03\x16\x81R3` \x82\x01R\xA1`@Q`\x01`\x01`@\x1B\x03` \x82\x01\x90\x81\x11\x90\x82\x11\x17b\0\x0B\xCBW` \x81\x01`@R`\0\x81R\x82Q`\0[\x81\x81\x10b\0\x15\xD3WPP`@Q\x92``\x84RQ\x80``\x85\x01R`\x80\x84\x01\x90`\x80\x81`\x05\x1B\x86\x01\x01\x93\x91`\0\x90[\x82\x82\x10b\0\x15yWPPPP\x91b\0\x05A\x81\x92\x7F\x8F\xAAp\x87\x86q\xCC\xD2\x12\xD2\x07q\xB7\x95\xC5\n\xF8\xFD?\xF6\xCF'\xF4\xBD\xE5~]M\xE0\xAE\xB6s\x94`\0` \x85\x01R\x83\x82\x03`@\x85\x01Rb\0\x1E\xA4V[\x03\x90\xA1\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD4` \x90\x81R\x7F}\xFDH\xDD\x9D\xEF\0/\xA9\xB4\xA0[\xD6\xB7&\xA6\xC3\x13\xC3b\xD3\xF3\xE8A=zu \xF0\t\r%\x80T`\xFF\x19\x90\x81\x16`\x01\x90\x81\x17\x90\x92U\x7FM\x7FL\x8A/\xB5\xB3\\\xA3\xC2w\xC98\x88\xB4\x7F\x0F\")\xBD\xCC\xCFfPM\x1B\xA4\x8E\x88\xB8\x81d\x80T\x82\x16\x83\x17\x90UcH\xE2\xB0\x93`\xE0\x1B`\0\x90\x81R\x7FY\xBAM\xB4\xA2\x13\xE8\x16\x1D\xE5\x97\xB8\xC1\r\xB0\xE7\xE7\xBAZ\xCE\\&\x8E67\x9E$\x9Am-B\xC9\x80T\x90\x92\x16\x83\x17\x90\x91U\x88Q\x81T`\x01`\x01`\xA0\x1B\x03\x19\x90\x81\x16`\x01`\x01`\xA0\x1B\x03\x92\x83\x16\x17\x90\x92U\x92\x89\x01Q\x82T\x82\x16\x90\x84\x16\x17\x90\x91U`@\x88\x01Q`\x02\x80T\x83\x16\x91\x84\x16\x91\x90\x91\x17\x90U``\x88\x01Q`\x03\x80T\x83\x16\x91\x84\x16\x91\x90\x91\x17\x90U`\x80\x88\x01Q`\x04\x80T\x83\x16\x91\x84\x16\x91\x90\x91\x17\x90U`\xA0\x88\x01Q`\x05\x80T\x83\x16\x91\x84\x16\x91\x90\x91\x17\x90U`\xC0\x88\x01Q`\x06\x80T\x83\x16\x91\x84\x16\x91\x90\x91\x17\x90U`\xE0\x88\x01Q`\x07\x80T\x83\x16\x91\x84\x16\x91\x90\x91\x17\x90U\x91\x87\x01Q`\x08\x80T\x84\x16\x91\x83\x16\x91\x90\x91\x17\x90U\x91\x86\x01Q`\t\x80T\x90\x92\x16\x92\x16\x91\x90\x91\x17\x90Ua\x01@\x84\x01Q\x80Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0B\xCBWh\x01\0\0\0\0\0\0\0\0\x82\x11b\0\x0B\xCBW`\nT\x82`\nU\x80\x83\x10b\0\x14\xF0W[P` \x01\x90`\n`\0R` `\0 \x90`\0[\x81`\x03\x1C\x81\x10b\0\x14\xA0WP`\x07\x19\x81\x16\x80\x82\x03b\0\x14FW[PPPP\x83\x01Q\x80Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0B\xCBWh\x01\0\0\0\0\0\0\0\0\x82\x11b\0\x0B\xCBW`\x0BT\x82`\x0BU\x80\x83\x10b\0\x13\xBDW[P` \x01\x90`\x0B`\0R` `\0 \x90`\0[\x81`\x03\x1C\x81\x10b\0\x13mWP`\x07\x19\x81\x16\x80\x82\x03b\0\x13\x13W[PPPP\x82\x01Q\x80Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0B\xCBWh\x01\0\0\0\0\0\0\0\0\x82\x11b\0\x0B\xCBW`\x0CT\x82`\x0CU\x80\x83\x10b\0\x12\x8AW[P` \x01\x90`\x0C`\0R` `\0 \x90`\0[\x81`\x03\x1C\x81\x10b\0\x12:WP`\x07\x19\x81\x16\x80\x82\x03b\0\x11\xE0W[PPPPa\x01\xA0\x82\x01Q\x80Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0B\xCBWh\x01\0\0\0\0\0\0\0\0\x82\x11b\0\x0B\xCBW`\rT\x82`\rU\x80\x83\x10b\0\x11WW[P` \x01\x90`\r`\0R` `\0 \x90`\0[\x81`\x03\x1C\x81\x10b\0\x11\x07WP`\x07\x19\x81\x16\x80\x82\x03b\0\x10\xADW[PPPP\x81\x01Q\x80Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0B\xCBWh\x01\0\0\0\0\0\0\0\0\x82\x11b\0\x0B\xCBW`\x0ET\x82`\x0EU\x80\x83\x10b\0\x10$W[P` \x01\x90`\x0E`\0R` `\0 \x90`\0[\x81`\x03\x1C\x81\x10b\0\x0F\xD4WP`\x07\x19\x81\x16\x80\x82\x03b\0\x0FzW[PPPPa\x01\xE0\x81\x01Q\x80Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0B\xCBWh\x01\0\0\0\0\0\0\0\0\x82\x11b\0\x0B\xCBW`\x0FT\x82`\x0FU\x80\x83\x10b\0\x0E\xF1W[P` \x01\x90`\x0F`\0R` `\0 \x90`\0[\x81`\x03\x1C\x81\x10b\0\x0E\xA1WP`\x07\x19\x81\x16\x80\x82\x03b\0\x0EGW[PPPPa\x02\0\x81\x01Q\x80Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0B\xCBWh\x01\0\0\0\0\0\0\0\0\x82\x11b\0\x0B\xCBW`\x10T\x82`\x10U\x80\x83\x10b\0\r\xBEW[P` \x01\x90`\x10`\0R` `\0 \x90`\0[\x81`\x03\x1C\x81\x10b\0\rnWP`\x07\x19\x81\x16\x80\x82\x03b\0\r\x14W[PPPPa\x02 \x81\x01Q\x80Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0B\xCBWh\x01\0\0\0\0\0\0\0\0\x82\x11b\0\x0B\xCBW`\x11T\x82`\x11U\x80\x83\x10b\0\x0C\x8BW[P` \x01\x90`\x11`\0R` `\0 \x90`\0[\x81`\x03\x1C\x81\x10b\0\x0C;WP`\x07\x19\x81\x16\x80\x82\x03b\0\x0B\xE1W[PPPPa\x02@\x81\x01Q\x80Q\x91\x90`\x01`\x01`@\x1B\x03\x83\x11b\0\x0B\xCBWh\x01\0\0\0\0\0\0\0\0\x83\x11b\0\x0B\xCBW`\x12T\x83`\x12U\x80\x84\x10b\0\x0BhW[P` \x01\x91`\x12`\0R` `\0 `\0[\x82`\x03\x1C\x81\x10b\0\x0B\x18WP`\x07\x19\x82\x16\x82\x03\x91\x82b\0\n\xBEW[PPPa\x02`\x91P\x01Q`\x02\x81\x10\x15b\0\n\xA8W`\xFF\x80\x19`\x15T\x16\x91\x16\x17`\x15U`@Qa\x013\x90\x81b\0\x1F[\x829\xF3[cNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[`\0\x94\x85\x93[\x80\x87\x10b\0\n\xE1WPPa\x02`\x94P`\x03\x1C\x01U8\x80\x80b\0\nvV[\x90\x93` b\0\x0B\x0C`\x01\x92\x87Q`\xE0\x1C\x90\x8A`\x02\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x95\x01\x96\x01\x95\x90b\0\n\xC4V[`\0\x80[`\x08\x81\x10b\0\x0B3WP\x82\x82\x01U`\x01\x01b\0\n[V[\x95\x90` b\0\x0B^`\x01\x92\x84Q`\xE0\x1C\x90\x8A`\x02\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x96\x01b\0\x0B\x1CV[`\x12`\0R` `\0 `\x07\x80\x86\x01`\x03\x1C\x82\x01\x92\x01`\x03\x1C\x01\x90`\x1C\x85`\x02\x1B\x16\x80b\0\x0B\xAFW[P[\x81\x81\x10b\0\x0B\xA2WPb\0\nIV[`\0\x81U`\x01\x01b\0\x0B\x93V[`\0\x19\x90\x81\x83\x01\x91\x82T\x91` \x03`\x03\x1B\x1C\x16\x90U8b\0\x0B\x91V[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[\x92`\0\x93`\0[\x81\x84\x03\x81\x10b\0\x0C\x04WPPP`\x03\x1C\x01U8\x80\x80\x80b\0\n\x0BV[\x90\x91\x94` b\0\x0C0`\x01\x92\x88Q`\xE0\x1C\x90\x85`\x02\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x96\x01\x92\x91\x01b\0\x0B\xE8V[`\0\x80[`\x08\x81\x10b\0\x0CVWP\x83\x82\x01U`\x01\x01b\0\t\xF1V[\x94\x90` b\0\x0C\x81`\x01\x92\x84Q`\xE0\x1C\x90\x89`\x02\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x95\x01b\0\x0C?V[`\x11`\0R\x7F1\xEC\xC2\x1At^9h\xA0N\x95p\xE4B[\xC1\x8F\xA8\x01\x9Ch\x02\x81\x96\xB5F\xD1f\x9C \x0Ch`\x02\x84\x90\x1B`\x1C\x16\x80b\0\x0C\xF0W[P`\x07\x84\x01`\x03\x1C\x81\x01[`\x07\x83\x01`\x03\x1C\x82\x01\x81\x10b\0\x0C\xE3WPPb\0\t\xDEV[`\0\x81U`\x01\x01b\0\x0C\xCBV[`\0\x19\x90\x81`\x07\x87\x01`\x03\x1C\x84\x01\x01\x91\x82T\x91` \x03`\x03\x1B\x1C\x16\x90U8b\0\x0C\xC0V[\x92`\0\x93`\0[\x81\x84\x03\x81\x10b\0\r7WPPP`\x03\x1C\x01U8\x80\x80\x80b\0\t\xA1V[\x90\x91\x94` b\0\rc`\x01\x92\x88Q`\xE0\x1C\x90\x85`\x02\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x96\x01\x92\x91\x01b\0\r\x1BV[`\0\x80[`\x08\x81\x10b\0\r\x89WP\x83\x82\x01U`\x01\x01b\0\t\x87V[\x94\x90` b\0\r\xB4`\x01\x92\x84Q`\xE0\x1C\x90\x89`\x02\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x95\x01b\0\rrV[`\x10`\0R\x7F\x1BhG\xDCt\x1A\x1B\x0C\xD0\x8D'\x88E\xF9\xD8\x19\xD8{sGY\xAF\xB5_\xE2\xDE\\\xB8*\x9A\xE6r`\x02\x84\x90\x1B`\x1C\x16\x80b\0\x0E#W[P`\x07\x84\x01`\x03\x1C\x81\x01[`\x07\x83\x01`\x03\x1C\x82\x01\x81\x10b\0\x0E\x16WPPb\0\ttV[`\0\x81U`\x01\x01b\0\r\xFEV[`\0\x19\x90\x81`\x07\x87\x01`\x03\x1C\x84\x01\x01\x91\x82T\x91` \x03`\x03\x1B\x1C\x16\x90U8b\0\r\xF3V[\x92`\0\x93`\0[\x81\x84\x03\x81\x10b\0\x0EjWPPP`\x03\x1C\x01U8\x80\x80\x80b\0\t7V[\x90\x91\x94` b\0\x0E\x96`\x01\x92\x88Q`\xE0\x1C\x90\x85`\x02\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x96\x01\x92\x91\x01b\0\x0ENV[`\0\x80[`\x08\x81\x10b\0\x0E\xBCWP\x83\x82\x01U`\x01\x01b\0\t\x1DV[\x94\x90` b\0\x0E\xE7`\x01\x92\x84Q`\xE0\x1C\x90\x89`\x02\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x95\x01b\0\x0E\xA5V[`\x0F`\0R\x7F\x8D\x11\x08\xE1\x0B\xCB|'\xDD\xDF\xC0.\xD9\xD6\x93\xA0t\x03\x9D\x02l\xF4\xEAB@\xB4\x0F}X\x1A\xC8\x02`\x02\x84\x90\x1B`\x1C\x16\x80b\0\x0FVW[P`\x07\x84\x01`\x03\x1C\x81\x01[`\x07\x83\x01`\x03\x1C\x82\x01\x81\x10b\0\x0FIWPPb\0\t\nV[`\0\x81U`\x01\x01b\0\x0F1V[`\0\x19\x90\x81`\x07\x87\x01`\x03\x1C\x84\x01\x01\x91\x82T\x91` \x03`\x03\x1B\x1C\x16\x90U8b\0\x0F&V[\x92`\0\x93`\0[\x81\x84\x03\x81\x10b\0\x0F\x9DWPPP`\x03\x1C\x01U8\x80\x80\x80b\0\x08\xCDV[\x90\x91\x94` b\0\x0F\xC9`\x01\x92\x88Q`\xE0\x1C\x90\x85`\x02\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x96\x01\x92\x91\x01b\0\x0F\x81V[`\0\x80[`\x08\x81\x10b\0\x0F\xEFWP\x83\x82\x01U`\x01\x01b\0\x08\xB3V[\x94\x90` b\0\x10\x1A`\x01\x92\x84Q`\xE0\x1C\x90\x89`\x02\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x95\x01b\0\x0F\xD8V[`\x0E`\0R\x7F\xBB{JEM\xC3I9#H/\x07\x82#)\xED\x19\xE8$N\xFFX,\xC2\x04\xF8UL6 \xC3\xFD`\x02\x84\x90\x1B`\x1C\x16\x80b\0\x10\x89W[P`\x07\x84\x01`\x03\x1C\x81\x01[`\x07\x83\x01`\x03\x1C\x82\x01\x81\x10b\0\x10|WPPb\0\x08\xA0V[`\0\x81U`\x01\x01b\0\x10dV[`\0\x19\x90\x81`\x07\x87\x01`\x03\x1C\x84\x01\x01\x91\x82T\x91` \x03`\x03\x1B\x1C\x16\x90U8b\0\x10YV[\x92`\0\x93`\0[\x81\x84\x03\x81\x10b\0\x10\xD0WPPP`\x03\x1C\x01U8\x80\x80\x80b\0\x08fV[\x90\x91\x94` b\0\x10\xFC`\x01\x92\x88Q`\xE0\x1C\x90\x85`\x02\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x96\x01\x92\x91\x01b\0\x10\xB4V[`\0\x80[`\x08\x81\x10b\0\x11\"WP\x83\x82\x01U`\x01\x01b\0\x08LV[\x94\x90` b\0\x11M`\x01\x92\x84Q`\xE0\x1C\x90\x89`\x02\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x95\x01b\0\x11\x0BV[`\r`\0R\x7F\xD7\xB6\x99\x01\x05q\x91\x01\xDA\xBE\xB7qD\xF2\xA38\\\x803\xAC\xD3\xAF\x97\xE9B:i^\x81\xAD\x1E\xB5`\x02\x84\x90\x1B`\x1C\x16\x80b\0\x11\xBCW[P`\x07\x84\x01`\x03\x1C\x81\x01[`\x07\x83\x01`\x03\x1C\x82\x01\x81\x10b\0\x11\xAFWPPb\0\x089V[`\0\x81U`\x01\x01b\0\x11\x97V[`\0\x19\x90\x81`\x07\x87\x01`\x03\x1C\x84\x01\x01\x91\x82T\x91` \x03`\x03\x1B\x1C\x16\x90U8b\0\x11\x8CV[\x92`\0\x93`\0[\x81\x84\x03\x81\x10b\0\x12\x03WPPP`\x03\x1C\x01U8\x80\x80\x80b\0\x07\xFCV[\x90\x91\x94` b\0\x12/`\x01\x92\x88Q`\xE0\x1C\x90\x85`\x02\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x96\x01\x92\x91\x01b\0\x11\xE7V[`\0\x80[`\x08\x81\x10b\0\x12UWP\x83\x82\x01U`\x01\x01b\0\x07\xE2V[\x94\x90` b\0\x12\x80`\x01\x92\x84Q`\xE0\x1C\x90\x89`\x02\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x95\x01b\0\x12>V[`\x0C`\0R\x7F\xDFif\xC9q\x05\x1C=T\xECY\x16&\x06S\x14\x93\xA5\x14\x04\xA0\x02\x84/V\0\x9D~\\\xF4\xA8\xC7`\x02\x84\x90\x1B`\x1C\x16\x80b\0\x12\xEFW[P`\x07\x84\x01`\x03\x1C\x81\x01[`\x07\x83\x01`\x03\x1C\x82\x01\x81\x10b\0\x12\xE2WPPb\0\x07\xCFV[`\0\x81U`\x01\x01b\0\x12\xCAV[`\0\x19\x90\x81`\x07\x87\x01`\x03\x1C\x84\x01\x01\x91\x82T\x91` \x03`\x03\x1B\x1C\x16\x90U8b\0\x12\xBFV[\x92`\0\x93`\0[\x81\x84\x03\x81\x10b\0\x136WPPP`\x03\x1C\x01U8\x80\x80\x80b\0\x07\x95V[\x90\x91\x94` b\0\x13b`\x01\x92\x88Q`\xE0\x1C\x90\x85`\x02\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x96\x01\x92\x91\x01b\0\x13\x1AV[`\0\x80[`\x08\x81\x10b\0\x13\x88WP\x83\x82\x01U`\x01\x01b\0\x07{V[\x94\x90` b\0\x13\xB3`\x01\x92\x84Q`\xE0\x1C\x90\x89`\x02\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x95\x01b\0\x13qV[`\x0B`\0R\x7F\x01u\xB7\xA68Bw\x03\xF0\xDB\xE7\xBB\x9B\xBF\x98z%Qq{4\xE7\x9F3\xB5\xB1\0\x8D\x1F\xA0\x1D\xB9`\x02\x84\x90\x1B`\x1C\x16\x80b\0\x14\"W[P`\x07\x84\x01`\x03\x1C\x81\x01[`\x07\x83\x01`\x03\x1C\x82\x01\x81\x10b\0\x14\x15WPPb\0\x07hV[`\0\x81U`\x01\x01b\0\x13\xFDV[`\0\x19\x90\x81`\x07\x87\x01`\x03\x1C\x84\x01\x01\x91\x82T\x91` \x03`\x03\x1B\x1C\x16\x90U8b\0\x13\xF2V[\x92`\0\x93`\0[\x81\x84\x03\x81\x10b\0\x14iWPPP`\x03\x1C\x01U8\x80\x80\x80b\0\x07.V[\x90\x91\x94` b\0\x14\x95`\x01\x92\x88Q`\xE0\x1C\x90\x85`\x02\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x96\x01\x92\x91\x01b\0\x14MV[`\0\x80[`\x08\x81\x10b\0\x14\xBBWP\x83\x82\x01U`\x01\x01b\0\x07\x14V[\x94\x90` b\0\x14\xE6`\x01\x92\x84Q`\xE0\x1C\x90\x89`\x02\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x95\x01b\0\x14\xA4V[`\n`\0R\x7F\xC6Z{\xB8\xD65\x1C\x1C\xF7\x0C\x95\xA3\x16\xCCj\x92\x83\x9C\x98f\x82\xD9\x8B\xC3_\x95\x8FH\x83\xF9\xD2\xA8`\x02\x84\x90\x1B`\x1C\x16\x80b\0\x15UW[P`\x07\x84\x01`\x03\x1C\x81\x01[`\x07\x83\x01`\x03\x1C\x82\x01\x81\x10b\0\x15HWPPb\0\x07\x01V[`\0\x81U`\x01\x01b\0\x150V[`\0\x19\x90\x81`\x07\x87\x01`\x03\x1C\x84\x01\x01\x91\x82T\x91` \x03`\x03\x1B\x1C\x16\x90U8b\0\x15%V[\x90\x91\x92\x94` \x80b\0\x15\xC4`\x01\x93`\x7F\x19\x8B\x82\x03\x01\x86R```@\x8BQ\x87\x80`\xA0\x1B\x03\x81Q\x16\x84Rb\0\x15\xB3\x86\x82\x01Q\x87\x86\x01\x90b\0\x1EVV[\x01Q\x91\x81`@\x82\x01R\x01\x90b\0\x1EdV[\x97\x01\x92\x01\x92\x01\x90\x92\x91b\0\x04\xF7V[`@b\0\x15\xE1\x82\x87b\0\x1E+V[Q\x01Q`\x01`\x01`\xA0\x1B\x03b\0\x15\xF8\x83\x88b\0\x1E+V[QQ\x16\x90\x80Q\x15b\0\x1C/W` b\0\x16\x12\x84\x89b\0\x1E+V[Q\x01Q`\x03\x81\x10\x15b\0\n\xA8W\x80b\0\x18NWP\x81\x15b\0\x18#Wa\xFF\xFF`\0\x80Q` b\0 \xAE\x839\x81Q\x91RT\x16b\0\x16\x8Bb\0\x16Pb\0\x1DHV[`!\x81R\x7FdiamondCut: Add facet has no cod` \x82\x01R`e`\xF8\x1B`@\x82\x01R\x84b\0\x1F\x18V[\x81Q\x91`\0\x91[\x83\x83\x10b\0\x16\xAAWPPPPP`\x01\x90[\x01b\0\x04\xCAV[b\0\x16\xB6\x83\x83b\0\x1E+V[Q`\x01`\x01`\xE0\x1B\x03\x19\x81\x16`\0\x90\x81R`\0\x80Q` b\0 \xCE\x839\x81Q\x91R` R`@\x90 T\x90\x91\x90`\x01`\x01`\xA0\x1B\x03\x16b\0\x18\x01Wb\0\x17fb\0\x16\xFEb\0\x1D(V[\x87\x81Ra\xFF\xFF\x83\x16` \x80\x83\x01\x91\x82R`\x01`\x01`\xE0\x1B\x03\x19\x86\x16`\0\x90\x81R`\0\x80Q` b\0 \xCE\x839\x81Q\x91R\x90\x91R`@\x90 \x91Q\x82T\x91Q`\x01`\x01`\xB0\x1B\x03\x19\x90\x92\x16`\x01`\x01`\xA0\x1B\x03\x91\x90\x91\x16\x17`\xA0\x91\x90\x91\x1Ba\xFF\xFF`\xA0\x1B\x16\x17\x90UV[`\0\x80Q` b\0 \xAE\x839\x81Q\x91RT\x91h\x01\0\0\0\0\0\0\0\0\x83\x10\x15b\0\x0B\xCBWb\0\x17\xCEb\0\x17\xAF`\x01\x94\x85\x81\x01`\0\x80Q` b\0 \xAE\x839\x81Q\x91RUb\0\x1E\xE6V[\x90\x92`\xE0\x1C\x90\x83T\x90`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x90Ua\xFF\xFF\x80\x82\x16\x14b\0\x17\xEBW\x92\x81\x01\x92a\xFF\xFF\x16\x01b\0\x16\x92V[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[`@Qc\xEB\xBF]\x07`\xE0\x1B\x81R`\x01`\x01`\xE0\x1B\x03\x19\x83\x16`\x04\x82\x01R`$\x90\xFD[`@Qc\x02\xB8\xDA\x07`\xE2\x1B\x81R` `\x04\x82\x01R\x90\x81\x90b\0\x18J\x90`$\x83\x01\x90b\0\x1EdV[\x03\x90\xFD[`\x01\x81\x03b\0\x19\xC3WP\x81\x15b\0\x19\x9CWb\0\x18\xAFb\0\x18mb\0\x1DHV[`(\x81R\x7FLibDiamondCut: Replace facet has` \x82\x01Rg no code`\xC0\x1B`@\x82\x01R\x83b\0\x1F\x18V[\x80Q\x90`\0[\x82\x81\x10b\0\x18\xCAWPPPP`\x01\x90b\0\x16\xA3V[`\x01`\x01`\xE0\x1B\x03\x19b\0\x18\xDF\x82\x84b\0\x1E+V[Q\x16`\0\x81\x81R`\0\x80Q` b\0 \xCE\x839\x81Q\x91R` R`@\x90 T`\x01`\x01`\xA0\x1B\x03\x160\x81\x14b\0\x19\x83W\x85\x81\x14b\0\x19jW\x15b\0\x19RW`\0\x90\x81R`\0\x80Q` b\0 \xCE\x839\x81Q\x91R` R`@\x90 \x80T`\x01`\x01`\xA0\x1B\x03\x19\x16\x85\x17\x90U`\x01\x01b\0\x18\xB5V[`$\x90`@Q\x90cty\xF99`\xE0\x1B\x82R`\x04\x82\x01R\xFD[`@Qc\x1A\xC6\xCE\x8D`\xE1\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qc)\x01\x80m`\xE1\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qc\xCD\x98\xA9o`\xE0\x1B\x81R` `\x04\x82\x01R\x90\x81\x90b\0\x18J\x90`$\x83\x01\x90b\0\x1EdV[`\x02\x81\x03b\0\x1C\rWP`\0\x80Q` b\0 \xAE\x839\x81Q\x91RT\x91\x80b\0\x1B\xF5WP\x80Q\x90`\0[\x82\x81\x10b\0\x1A\x01WPPPP`\x01\x90b\0\x16\xA3V[`\x01`\x01`\xE0\x1B\x03\x19b\0\x1A\x16\x82\x84b\0\x1E+V[Q\x16\x90\x81`\0R`\0\x80Q` b\0 \xCE\x839\x81Q\x91R` R`@`\0 \x94b\0\x1A@b\0\x1D(V[\x95T`\x01`\x01`\xA0\x1B\x03\x81\x16\x80\x88R`\xA0\x91\x90\x91\x1Ca\xFF\xFF\x16` \x88\x01R\x15b\0\x1B\xDCW\x85Q`\x01`\x01`\xA0\x1B\x03\x160\x14b\0\x1B\xC3W\x80\x15b\0\x17\xEBW`\0\x19\x01\x94\x85a\xFF\xFF` \x83\x01Q\x16\x03b\0\x1B\x1AW[P`\0\x80Q` b\0 \xAE\x839\x81Q\x91RT\x91\x82\x15b\0\x1B\x04W`\x01\x92`\0\x19\x01b\0\x1A\xBF\x81b\0\x1E\xE6V[c\xFF\xFF\xFF\xFF\x82T\x91`\x03\x1B\x1B\x19\x16\x90U`\0\x80Q` b\0 \xAE\x839\x81Q\x91RU`\0R`\0\x80Q` b\0 \xCE\x839\x81Q\x91R` R`\0`@\x81 U\x01b\0\x19\xECV[cNH{q`\xE0\x1B`\0R`1`\x04R`$`\0\xFD[b\0\x1B\xBC\x90a\xFF\xFF` b\0\x1B/\x89b\0\x1E\xE6V[\x90T\x90`\x03\x1B\x1C\x92b\0\x1Bob\0\x1BK\x84\x84\x84\x01Q\x16b\0\x1E\xE6V[c\xFF\xFF\xFF\xFF\x87\x93\x92\x93\x16\x90\x83T\x90`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x90U\x01Q`\xE0\x92\x90\x92\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16`\0\x90\x81R`\0\x80Q` b\0 \xCE\x839\x81Q\x91R` R`@\x90 \x80Ta\xFF\xFF`\xA0\x1B\x19\x16\x91\x90\x92\x16`\xA0\x1Ba\xFF\xFF`\xA0\x1B\x16\x17\x90UV[8b\0\x1A\x93V[`@Qc\r\xF5\xFDa`\xE3\x1B\x81R`\x04\x81\x01\x84\x90R`$\x90\xFD[`@Qcz\x08\xA2-`\xE0\x1B\x81R`\x04\x81\x01\x84\x90R`$\x90\xFD[`$\x90`@Q\x90c\xD0\x91\xBC\x81`\xE0\x1B\x82R`\x04\x82\x01R\xFD[`@Qc?\xF4\xD2\x0F`\xE1\x1B\x81R`$\x91b\0\x1C-\x90`\x04\x83\x01\x90b\0\x1EVV[\xFD[`@Qc\xE7g\xF9\x1F`\xE0\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qc\x07\xA0CQ`\xE5\x1B\x81R`\x04\x90\xFD[`@Qc\x8B=\xDC3`\xE0\x1B\x81R`\x04\x90\xFD[`\0\x80\xFD[\x82Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x1ClW\x82\x01``\x91\x82`\x1F\x19\x83\x89\x8C\x01\x03\x01\x12b\0\x1ClW`@Q\x92\x80\x84\x01`\x01`\x01`@\x1B\x03\x81\x11\x85\x82\x10\x17b\0\x1D\x13W`@Rb\0\x1C\xC2` \x84\x01b\0\x1D\xA6V[\x84R`@\x83\x01Q`\x03\x81\x10\x15b\0\x1ClW` \x85\x01R\x82\x01Q\x92`\x01`\x01`@\x1B\x03\x84\x11b\0\x1ClWb\0\x1D\x02` \x94\x93\x85\x80\x95\x8B\x8E\x01\x92\x01\x01b\0\x1D\xBBV[`@\x82\x01R\x81R\x01\x92\x01\x91b\0\0\x8BV[`$`\0cNH{q`\xE0\x1B\x81R`A`\x04R\xFD[`@\x80Q\x91\x90\x82\x01`\x01`\x01`@\x1B\x03\x81\x11\x83\x82\x10\x17b\0\x0B\xCBW`@RV[`@Q\x90``\x82\x01`\x01`\x01`@\x1B\x03\x81\x11\x83\x82\x10\x17b\0\x0B\xCBW`@RV[`@Q\x91\x90`\x1F\x01`\x1F\x19\x16\x82\x01`\x01`\x01`@\x1B\x03\x81\x11\x83\x82\x10\x17b\0\x0B\xCBW`@RV[`\x01`\x01`@\x1B\x03\x81\x11b\0\x0B\xCBW`\x05\x1B` \x01\x90V[Q\x90`\x01`\x01`\xA0\x1B\x03\x82\x16\x82\x03b\0\x1ClWV[\x90\x80`\x1F\x83\x01\x12\x15b\0\x1ClW\x81Q\x90` \x91b\0\x1D\xDDb\0\0b\x82b\0\x1D\x8EV[\x93` \x80\x86\x84\x81R\x01\x92`\x05\x1B\x82\x01\x01\x92\x83\x11b\0\x1ClW` \x01\x90[\x82\x82\x10b\0\x1E\tWPPPP\x90V[\x81Q`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x81\x03b\0\x1ClW\x81R\x90\x83\x01\x90\x83\x01b\0\x1D\xFAV[\x80Q\x82\x10\x15b\0\x1E@W` \x91`\x05\x1B\x01\x01\x90V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[\x90`\x03\x82\x10\x15b\0\n\xA8WRV[\x90\x81Q\x80\x82R` \x80\x80\x93\x01\x93\x01\x91`\0[\x82\x81\x10b\0\x1E\x85WPPPP\x90V[\x83Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x93\x81\x01\x93\x92\x81\x01\x92`\x01\x01b\0\x1EvV[\x91\x90\x82Q\x92\x83\x82R`\0[\x84\x81\x10b\0\x1E\xD1WPP\x82`\0` \x80\x94\x95\x84\x01\x01R`\x1F\x80\x19\x91\x01\x16\x01\x01\x90V[` \x81\x83\x01\x81\x01Q\x84\x83\x01\x82\x01R\x01b\0\x1E\xAFV[\x90`\0\x80Q` b\0 \xAE\x839\x81Q\x91R\x80T\x83\x10\x15b\0\x1E@W`\0R`\x1C` `\0 \x83`\x03\x1C\x01\x92`\x02\x1B\x16\x90V[\x80;\x15b\0\x1F$WPPV[`@\x80Qc\x91\x984\xB9`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x90\x92\x16`\x04\x83\x01R`$\x82\x01R\x90\x81\x90b\0\x18J\x90`D\x83\x01\x90b\0\x1E\xA4V\xFE`\x80`@R6\x15`\x87W`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`oWP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[=\x90\xFD[`$\x90`@Q\x90c\n\x82\xDDs`\xE3\x1B\x82R`\x04\x82\x01R\xFD[`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`\xE9WP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[c\n\x82\xDDs`\xE3\x1B`\x80R`\x84R`$`\x80\xFD\xFE\xA2dipfsX\"\x12 1\t\xA8\x1C\xB3\t\xDD\x85\x9F\xED(\x97\"\xCA\xA0.\t\x08\xD1s\xD7\x13\xCC\xBEj\xCF\x08K\x86\xF0\xA7\xB7dsolcC\0\x08\x17\x003\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD5\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2";
    /// The bytecode of the contract.
    pub static SUBNETREGISTRYDIAMOND_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__BYTECODE);
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R6\x15`\x87W`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`oWP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[=\x90\xFD[`$\x90`@Q\x90c\n\x82\xDDs`\xE3\x1B\x82R`\x04\x82\x01R\xFD[`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`\xE9WP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[c\n\x82\xDDs`\xE3\x1B`\x80R`\x84R`$`\x80\xFD\xFE\xA2dipfsX\"\x12 1\t\xA8\x1C\xB3\t\xDD\x85\x9F\xED(\x97\"\xCA\xA0.\t\x08\xD1s\xD7\x13\xCC\xBEj\xCF\x08K\x86\xF0\xA7\xB7dsolcC\0\x08\x17\x003";
    /// The deployed bytecode of the contract.
    pub static SUBNETREGISTRYDIAMOND_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__DEPLOYED_BYTECODE);
    pub struct SubnetRegistryDiamond<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for SubnetRegistryDiamond<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for SubnetRegistryDiamond<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for SubnetRegistryDiamond<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for SubnetRegistryDiamond<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(SubnetRegistryDiamond))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> SubnetRegistryDiamond<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                SUBNETREGISTRYDIAMOND_ABI.clone(),
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
                SUBNETREGISTRYDIAMOND_ABI.clone(),
                SUBNETREGISTRYDIAMOND_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Gets the contract's `DiamondCut` event
        pub fn diamond_cut_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, DiamondCutFilter> {
            self.0.event()
        }
        ///Gets the contract's `OwnershipTransferred` event
        pub fn ownership_transferred_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, OwnershipTransferredFilter>
        {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, SubnetRegistryDiamondEvents>
        {
            self.0
                .event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
        for SubnetRegistryDiamond<M>
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
    ///Custom Error type `FacetCannotBeZero` with signature `FacetCannotBeZero()` and selector `0xf4086a20`
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
    #[etherror(name = "FacetCannotBeZero", abi = "FacetCannotBeZero()")]
    pub struct FacetCannotBeZero;
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
    ///Custom Error type `GatewayCannotBeZero` with signature `GatewayCannotBeZero()` and selector `0x8b3ddc33`
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
    #[etherror(name = "GatewayCannotBeZero", abi = "GatewayCannotBeZero()")]
    pub struct GatewayCannotBeZero;
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
    pub enum SubnetRegistryDiamondErrors {
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
        FacetCannotBeZero(FacetCannotBeZero),
        FunctionNotFound(FunctionNotFound),
        GatewayCannotBeZero(GatewayCannotBeZero),
        IncorrectFacetCutAction(IncorrectFacetCutAction),
        InitializationFunctionReverted(InitializationFunctionReverted),
        NoBytecodeAtAddress(NoBytecodeAtAddress),
        NoSelectorsProvidedForFacetForCut(NoSelectorsProvidedForFacetForCut),
        RemoveFacetAddressMustBeZeroAddress(RemoveFacetAddressMustBeZeroAddress),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for SubnetRegistryDiamondErrors {
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
            if let Ok(decoded) = <FacetCannotBeZero as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::FacetCannotBeZero(decoded));
            }
            if let Ok(decoded) = <FunctionNotFound as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::FunctionNotFound(decoded));
            }
            if let Ok(decoded) =
                <GatewayCannotBeZero as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GatewayCannotBeZero(decoded));
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
                <RemoveFacetAddressMustBeZeroAddress as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                )
            {
                return Ok(Self::RemoveFacetAddressMustBeZeroAddress(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for SubnetRegistryDiamondErrors {
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
                Self::FacetCannotBeZero(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::FunctionNotFound(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GatewayCannotBeZero(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::IncorrectFacetCutAction(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitializationFunctionReverted(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NoBytecodeAtAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NoSelectorsProvidedForFacetForCut(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RemoveFacetAddressMustBeZeroAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for SubnetRegistryDiamondErrors {
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
                    == <FacetCannotBeZero as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <FunctionNotFound as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <GatewayCannotBeZero as ::ethers::contract::EthError>::selector() => {
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
                    == <NoBytecodeAtAddress as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NoSelectorsProvidedForFacetForCut as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <RemoveFacetAddressMustBeZeroAddress as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for SubnetRegistryDiamondErrors {
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
                Self::FacetCannotBeZero(element) => ::core::fmt::Display::fmt(element, f),
                Self::FunctionNotFound(element) => ::core::fmt::Display::fmt(element, f),
                Self::GatewayCannotBeZero(element) => ::core::fmt::Display::fmt(element, f),
                Self::IncorrectFacetCutAction(element) => ::core::fmt::Display::fmt(element, f),
                Self::InitializationFunctionReverted(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NoBytecodeAtAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::NoSelectorsProvidedForFacetForCut(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RemoveFacetAddressMustBeZeroAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for SubnetRegistryDiamondErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<CannotAddFunctionToDiamondThatAlreadyExists>
        for SubnetRegistryDiamondErrors
    {
        fn from(value: CannotAddFunctionToDiamondThatAlreadyExists) -> Self {
            Self::CannotAddFunctionToDiamondThatAlreadyExists(value)
        }
    }
    impl ::core::convert::From<CannotAddSelectorsToZeroAddress> for SubnetRegistryDiamondErrors {
        fn from(value: CannotAddSelectorsToZeroAddress) -> Self {
            Self::CannotAddSelectorsToZeroAddress(value)
        }
    }
    impl ::core::convert::From<CannotRemoveFunctionThatDoesNotExist> for SubnetRegistryDiamondErrors {
        fn from(value: CannotRemoveFunctionThatDoesNotExist) -> Self {
            Self::CannotRemoveFunctionThatDoesNotExist(value)
        }
    }
    impl ::core::convert::From<CannotRemoveImmutableFunction> for SubnetRegistryDiamondErrors {
        fn from(value: CannotRemoveImmutableFunction) -> Self {
            Self::CannotRemoveImmutableFunction(value)
        }
    }
    impl ::core::convert::From<CannotReplaceFunctionThatDoesNotExists> for SubnetRegistryDiamondErrors {
        fn from(value: CannotReplaceFunctionThatDoesNotExists) -> Self {
            Self::CannotReplaceFunctionThatDoesNotExists(value)
        }
    }
    impl ::core::convert::From<CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet>
        for SubnetRegistryDiamondErrors
    {
        fn from(value: CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet) -> Self {
            Self::CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(value)
        }
    }
    impl ::core::convert::From<CannotReplaceFunctionsFromFacetWithZeroAddress>
        for SubnetRegistryDiamondErrors
    {
        fn from(value: CannotReplaceFunctionsFromFacetWithZeroAddress) -> Self {
            Self::CannotReplaceFunctionsFromFacetWithZeroAddress(value)
        }
    }
    impl ::core::convert::From<CannotReplaceImmutableFunction> for SubnetRegistryDiamondErrors {
        fn from(value: CannotReplaceImmutableFunction) -> Self {
            Self::CannotReplaceImmutableFunction(value)
        }
    }
    impl ::core::convert::From<FacetCannotBeZero> for SubnetRegistryDiamondErrors {
        fn from(value: FacetCannotBeZero) -> Self {
            Self::FacetCannotBeZero(value)
        }
    }
    impl ::core::convert::From<FunctionNotFound> for SubnetRegistryDiamondErrors {
        fn from(value: FunctionNotFound) -> Self {
            Self::FunctionNotFound(value)
        }
    }
    impl ::core::convert::From<GatewayCannotBeZero> for SubnetRegistryDiamondErrors {
        fn from(value: GatewayCannotBeZero) -> Self {
            Self::GatewayCannotBeZero(value)
        }
    }
    impl ::core::convert::From<IncorrectFacetCutAction> for SubnetRegistryDiamondErrors {
        fn from(value: IncorrectFacetCutAction) -> Self {
            Self::IncorrectFacetCutAction(value)
        }
    }
    impl ::core::convert::From<InitializationFunctionReverted> for SubnetRegistryDiamondErrors {
        fn from(value: InitializationFunctionReverted) -> Self {
            Self::InitializationFunctionReverted(value)
        }
    }
    impl ::core::convert::From<NoBytecodeAtAddress> for SubnetRegistryDiamondErrors {
        fn from(value: NoBytecodeAtAddress) -> Self {
            Self::NoBytecodeAtAddress(value)
        }
    }
    impl ::core::convert::From<NoSelectorsProvidedForFacetForCut> for SubnetRegistryDiamondErrors {
        fn from(value: NoSelectorsProvidedForFacetForCut) -> Self {
            Self::NoSelectorsProvidedForFacetForCut(value)
        }
    }
    impl ::core::convert::From<RemoveFacetAddressMustBeZeroAddress> for SubnetRegistryDiamondErrors {
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
        name = "OwnershipTransferred",
        abi = "OwnershipTransferred(address,address)"
    )]
    pub struct OwnershipTransferredFilter {
        pub old_owner: ::ethers::core::types::Address,
        pub new_owner: ::ethers::core::types::Address,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum SubnetRegistryDiamondEvents {
        DiamondCutFilter(DiamondCutFilter),
        OwnershipTransferredFilter(OwnershipTransferredFilter),
    }
    impl ::ethers::contract::EthLogDecode for SubnetRegistryDiamondEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = DiamondCutFilter::decode_log(log) {
                return Ok(SubnetRegistryDiamondEvents::DiamondCutFilter(decoded));
            }
            if let Ok(decoded) = OwnershipTransferredFilter::decode_log(log) {
                return Ok(SubnetRegistryDiamondEvents::OwnershipTransferredFilter(
                    decoded,
                ));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for SubnetRegistryDiamondEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::DiamondCutFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::OwnershipTransferredFilter(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<DiamondCutFilter> for SubnetRegistryDiamondEvents {
        fn from(value: DiamondCutFilter) -> Self {
            Self::DiamondCutFilter(value)
        }
    }
    impl ::core::convert::From<OwnershipTransferredFilter> for SubnetRegistryDiamondEvents {
        fn from(value: OwnershipTransferredFilter) -> Self {
            Self::OwnershipTransferredFilter(value)
        }
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
    ///`ConstructorParams(address,address,address,address,address,address,address,address,address,address,bytes4[],bytes4[],bytes4[],bytes4[],bytes4[],bytes4[],bytes4[],bytes4[],bytes4[],uint8)`
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
        pub gateway: ::ethers::core::types::Address,
        pub getter_facet: ::ethers::core::types::Address,
        pub manager_facet: ::ethers::core::types::Address,
        pub rewarder_facet: ::ethers::core::types::Address,
        pub checkpointer_facet: ::ethers::core::types::Address,
        pub pauser_facet: ::ethers::core::types::Address,
        pub diamond_cut_facet: ::ethers::core::types::Address,
        pub diamond_loupe_facet: ::ethers::core::types::Address,
        pub ownership_facet: ::ethers::core::types::Address,
        pub activity_facet: ::ethers::core::types::Address,
        pub subnet_actor_getter_selectors: ::std::vec::Vec<[u8; 4]>,
        pub subnet_actor_manager_selectors: ::std::vec::Vec<[u8; 4]>,
        pub subnet_actor_rewarder_selectors: ::std::vec::Vec<[u8; 4]>,
        pub subnet_actor_checkpointer_selectors: ::std::vec::Vec<[u8; 4]>,
        pub subnet_actor_pauser_selectors: ::std::vec::Vec<[u8; 4]>,
        pub subnet_actor_diamond_cut_selectors: ::std::vec::Vec<[u8; 4]>,
        pub subnet_actor_diamond_loupe_selectors: ::std::vec::Vec<[u8; 4]>,
        pub subnet_actor_ownership_selectors: ::std::vec::Vec<[u8; 4]>,
        pub subnet_actor_activity_selectors: ::std::vec::Vec<[u8; 4]>,
        pub creation_privileges: u8,
    }
}
