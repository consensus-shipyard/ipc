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
    const __BYTECODE: &[u8] = b"`\x80`@R4a\x18$Wa\x1C\\\x808\x03\x90\x81a\0\x1A\x81a\x18\xE1V[\x91\x829`@\x81\x83\x81\x01\x03\x12a\x18$W\x80Q`\x01`\x01`@\x1B\x03\x81\x11a\x18$W\x81\x01\x82\x82\x01`\x1F\x82\x01\x12\x15a\x18$W\x80Q\x92a\0\\a\0W\x85a\x19\x06V[a\x18\xE1V[\x91` \x83\x86\x81R\x01` \x81\x96`\x05\x1B\x83\x01\x01\x91\x83\x86\x01\x83\x11a\x18$W` \x81\x01\x91[\x83\x83\x10a\x18)WPPPP` \x83\x01Q`\x01`\x01`@\x1B\x03\x81\x11a\x18$Wa\x02@\x81\x85\x01\x83\x86\x01\x03\x12a\x18$W`@Q\x93a\x02@\x85\x01`\x01`\x01`@\x1B\x03\x81\x11\x86\x82\x10\x17a\n@W`@Ra\0\xD4\x82\x82\x01a\x19\x1DV[\x85Ra\0\xE4` \x83\x83\x01\x01a\x19\x1DV[` \x86\x01Ra\0\xF7`@\x83\x83\x01\x01a\x19\x1DV[`@\x86\x01Ra\x01\n``\x83\x83\x01\x01a\x19\x1DV[``\x86\x01Ra\x01\x1D`\x80\x83\x83\x01\x01a\x19\x1DV[`\x80\x86\x01Ra\x010`\xA0\x83\x83\x01\x01a\x19\x1DV[`\xA0\x86\x01Ra\x01C`\xC0\x83\x83\x01\x01a\x19\x1DV[`\xC0\x86\x01Ra\x01V`\xE0\x83\x83\x01\x01a\x19\x1DV[`\xE0\x86\x01Ra\x01ja\x01\0\x83\x83\x01\x01a\x19\x1DV[a\x01\0\x86\x01R\x80\x82\x01a\x01 \x01Q`\x01`\x01`@\x1B\x03\x81\x11a\x18$Wa\x01\x97\x90\x84\x83\x01\x90\x84\x84\x01\x01a\x191V[a\x01 \x86\x01R\x80\x82\x01a\x01@\x01Q`\x01`\x01`@\x1B\x03\x81\x11a\x18$Wa\x01\xC4\x90\x84\x83\x01\x90\x84\x84\x01\x01a\x191V[a\x01@\x86\x01R\x80\x82\x01a\x01`\x01Q`\x01`\x01`@\x1B\x03\x81\x11a\x18$Wa\x01\xF1\x90\x84\x83\x01\x90\x84\x84\x01\x01a\x191V[a\x01`\x86\x01R\x80\x82\x01a\x01\x80\x01Q`\x01`\x01`@\x1B\x03\x81\x11a\x18$Wa\x02\x1E\x90\x84\x83\x01\x90\x84\x84\x01\x01a\x191V[a\x01\x80\x86\x01R\x80\x82\x01a\x01\xA0\x01Q`\x01`\x01`@\x1B\x03\x81\x11a\x18$Wa\x02K\x90\x84\x83\x01\x90\x84\x84\x01\x01a\x191V[a\x01\xA0\x86\x01R\x80\x82\x01a\x01\xC0\x01Q`\x01`\x01`@\x1B\x03\x81\x11a\x18$Wa\x02x\x90\x84\x83\x01\x90\x84\x84\x01\x01a\x191V[a\x01\xC0\x86\x01R\x80\x82\x01a\x01\xE0\x01Q`\x01`\x01`@\x1B\x03\x81\x11a\x18$Wa\x02\xA5\x90\x84\x83\x01\x90\x84\x84\x01\x01a\x191V[a\x01\xE0\x86\x01R\x80\x82\x01a\x02\0\x01Q\x92`\x01`\x01`@\x1B\x03\x84\x11a\x18$Wa\x02 \x93a\x02\xD6\x91\x83\x01\x90\x84\x84\x01\x01a\x191V[a\x02\0\x86\x01R\x01\x01Q`\x02\x81\x10\x15a\x18$Wa\x02 \x83\x01R\x81Q`\x01`\x01`\xA0\x1B\x03\x16\x15a\x18\x13W` \x82\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x15a\x18\x02W`@\x82\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x15a\x18\x02W``\x82\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x15a\x18\x02W`\x80\x82\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x15a\x18\x02W`\xA0\x82\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x15a\x18\x02W`\xC0\x82\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x15a\x18\x02W`\xE0\x82\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x15a\x18\x02Wa\x01\0\x82\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x15a\x18\x02W\x7F\x8B\xE0\x07\x9CS\x16Y\x14\x13D\xCD\x1F\xD0\xA4\xF2\x84\x19I\x7F\x97\"\xA3\xDA\xAF\xE3\xB4\x18okdW\xE0`@`\x01\x80`\xA0\x1B\x03`\0\x80Q` a\x1B\xFC\x839\x81Q\x91RT\x163`\x01\x80`\xA0\x1B\x03\x19`\0\x80Q` a\x1B\xFC\x839\x81Q\x91RT\x16\x17`\0\x80Q` a\x1B\xFC\x839\x81Q\x91RU\x81Q\x90\x81R3` \x82\x01R\xA1` \x92a\x04\x15\x84a\x18\xE1V[`\0\x81R`\x1F\x19\x85\x016\x86\x83\x017\x82Q`\0[\x81\x81\x10a\x11\xD9WPP`@Q\x92``\x84\x01\x90``\x85RQ\x80\x91R`\x80\x84\x01\x90`\x80\x81`\x05\x1B\x86\x01\x01\x93\x91`\0\x90[\x82\x82\x10a\x11yW\x88\x88\x7F\x8F\xAAp\x87\x86q\xCC\xD2\x12\xD2\x07q\xB7\x95\xC5\n\xF8\xFD?\xF6\xCF'\xF4\xBD\xE5~]M\xE0\xAE\xB6s\x89\x80a\x04\x9A\x8B\x8B`\0\x88\x85\x01R\x83\x82\x03`@\x85\x01Ra\x19\xFEV[\x03\x90\xA1c\x01\xFF\xC9\xA7`\xE0\x1B`\0\x90\x81R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD4\x80\x84R`@\x80\x83 \x80T`\xFF\x19\x90\x81\x16`\x01\x90\x81\x17\x90\x92Uc\x07\xE4\xC7\x07`\xE2\x1B\x85R\x83\x87R\x82\x85 \x80T\x82\x16\x83\x17\x90UcH\xE2\xB0\x93`\xE0\x1B\x85R\x92\x86R\x81\x84 \x80T\x90\x93\x16\x81\x17\x90\x92U\x83Q\x83T`\x01`\x01`\xA0\x1B\x03\x19\x90\x81\x16`\x01`\x01`\xA0\x1B\x03\x92\x83\x16\x17\x90\x94U` \x85\x01Q\x83T\x85\x16\x90\x82\x16\x17\x90\x92U\x83\x01Q`\x02\x80T\x84\x16\x91\x83\x16\x91\x90\x91\x17\x90U``\x83\x01Q`\x03\x80T\x84\x16\x91\x83\x16\x91\x90\x91\x17\x90U`\x80\x83\x01Q`\x04\x80T\x84\x16\x91\x83\x16\x91\x90\x91\x17\x90U`\xA0\x83\x01Q`\x05\x80T\x84\x16\x91\x83\x16\x91\x90\x91\x17\x90U`\xC0\x83\x01Q`\x06\x80T\x84\x16\x91\x83\x16\x91\x90\x91\x17\x90U`\xE0\x83\x01Q`\x07\x80T\x84\x16\x91\x83\x16\x91\x90\x91\x17\x90Ua\x01\0\x83\x01Q`\x08\x80T\x90\x93\x16\x91\x16\x17\x90Ua\x01 \x81\x01Q\x80Q\x90`\x01`\x01`@\x1B\x03\x82\x11a\n@Wh\x01\0\0\0\0\0\0\0\0\x82\x11a\n@W\x83\x90`\tT\x83`\tU\x80\x84\x10a\x11\x11W[P\x01\x90`\t`\0R\x83`\0 \x81`\x03\x1C\x91`\0[\x83\x81\x10a\x10\xC5WP`\x07\x19\x81\x16\x90\x03\x80a\x10tW[PPPPa\x01@\x81\x01Q\x80Q\x90`\x01`\x01`@\x1B\x03\x82\x11a\n@Wh\x01\0\0\0\0\0\0\0\0\x82\x11a\n@W\x83\x90`\nT\x83`\nU\x80\x84\x10a\x10\x0CW[P\x01\x90`\n`\0R\x83`\0 \x81`\x03\x1C\x91`\0[\x83\x81\x10a\x0F\xC0WP`\x07\x19\x81\x16\x90\x03\x80a\x0FoW[PPPPa\x01`\x81\x01Q\x80Q\x90`\x01`\x01`@\x1B\x03\x82\x11a\n@Wh\x01\0\0\0\0\0\0\0\0\x82\x11a\n@W\x83\x90`\x0BT\x83`\x0BU\x80\x84\x10a\x0F\x07W[P\x01\x90`\x0B`\0R\x83`\0 \x81`\x03\x1C\x91`\0[\x83\x81\x10a\x0E\xBBWP`\x07\x19\x81\x16\x90\x03\x80a\x0EjW[PPPPa\x01\x80\x81\x01Q\x80Q\x90`\x01`\x01`@\x1B\x03\x82\x11a\n@Wh\x01\0\0\0\0\0\0\0\0\x82\x11a\n@W\x83\x90`\x0CT\x83`\x0CU\x80\x84\x10a\x0E\x02W[P\x01\x90`\x0C`\0R\x83`\0 \x81`\x03\x1C\x91`\0[\x83\x81\x10a\r\xB6WP`\x07\x19\x81\x16\x90\x03\x80a\reW[PPPPa\x01\xA0\x81\x01Q\x80Q\x90`\x01`\x01`@\x1B\x03\x82\x11a\n@Wh\x01\0\0\0\0\0\0\0\0\x82\x11a\n@W\x83\x90`\rT\x83`\rU\x80\x84\x10a\x0C\xFDW[P\x01\x90`\r`\0R\x83`\0 \x81`\x03\x1C\x91`\0[\x83\x81\x10a\x0C\xB1WP`\x07\x19\x81\x16\x90\x03\x80a\x0C`W[PPPPa\x01\xC0\x81\x01Q\x80Q\x90`\x01`\x01`@\x1B\x03\x82\x11a\n@Wh\x01\0\0\0\0\0\0\0\0\x82\x11a\n@W\x83\x90`\x0ET\x83`\x0EU\x80\x84\x10a\x0B\xF8W[P\x01\x90`\x0E`\0R\x83`\0 \x81`\x03\x1C\x91`\0[\x83\x81\x10a\x0B\xACWP`\x07\x19\x81\x16\x90\x03\x80a\x0B[W[PPPPa\x01\xE0\x81\x01Q\x80Q\x90`\x01`\x01`@\x1B\x03\x82\x11a\n@Wh\x01\0\0\0\0\0\0\0\0\x82\x11a\n@W\x83\x90`\x0FT\x83`\x0FU\x80\x84\x10a\n\xF3W[P\x01\x90`\x0F`\0R\x83`\0 \x81`\x03\x1C\x91`\0[\x83\x81\x10a\n\xA7WP`\x07\x19\x81\x16\x90\x03\x80a\nVW[PPPPa\x02\0\x81\x01Q\x80Q\x90`\x01`\x01`@\x1B\x03\x82\x11a\n@Wh\x01\0\0\0\0\0\0\0\0\x82\x11a\n@W\x83\x90`\x10T\x83`\x10U\x80\x84\x10a\t\xD8W[P\x01\x90`\x10`\0R\x83`\0 \x81`\x03\x1C\x91`\0[\x83\x81\x10a\t\x8CWP`\x07\x19\x81\x16\x90\x03\x80a\t7W[PPPPa\x02 \x91P\x01Q`\x02\x81\x10\x15a\t!W`\xFF\x80\x19`\x13T\x16\x91\x16\x17`\x13U`@Qa\x01\"\x90\x81a\x1A\xDA\x829\xF3[cNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[\x92`\0\x93`\0[\x81\x81\x10a\tXWPPPa\x02 \x94P\x01U\x82\x80\x80\x80a\x08\xF0V[\x90\x91\x94\x87a\t\x82`\x01\x92\x88Q`\xE0\x1C\x90\x85`\x02\x1Bc\xFF\xFF\xFF\xFF\x80\x91`\x03\x1B\x93\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x96\x01\x92\x91\x01a\t>V[`\0\x80[\x88`\x08\x82\x10a\t\xA7WPP\x83\x82\x01U`\x01\x01a\x08\xDBV[a\t\xCF\x88\x93`\x01\x93\x99Q`\xE0\x1C\x90\x8A`\x02\x1Bc\xFF\xFF\xFF\xFF\x80\x91`\x03\x1B\x93\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x96\x01a\t\x90V[\x90\x91P`\x10`\0R\x84`\0 `\x07\x80\x85\x01`\x03\x1C\x82\x01\x92\x01`\x03\x1C\x01\x90`\x1C\x84`\x02\x1B\x16\x80a\n$W[P\x90\x85\x92\x91[\x81\x81\x10a\n\x15WPa\x08\xC7V[`\0\x81U\x86\x93P`\x01\x01a\n\x08V[`\0\x19\x82\x01\x90\x81T\x90`\0\x19\x90\x89\x03`\x03\x1B\x1C\x16\x90U\x86a\n\x02V[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[\x92`\0\x93`\0[\x87\x82\x82\x10a\ntWPPPP\x01U\x82\x80\x80\x80a\x08\x8BV[a\n\x9D\x84\x97`\x01\x93\x94\x95Q`\xE0\x1C\x90\x85`\x02\x1Bc\xFF\xFF\xFF\xFF\x80\x91`\x03\x1B\x93\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x96\x01\x92\x91\x01a\n]V[`\0\x80[\x88`\x08\x82\x10a\n\xC2WPP\x83\x82\x01U`\x01\x01a\x08vV[a\n\xEA\x88\x93`\x01\x93\x99Q`\xE0\x1C\x90\x8A`\x02\x1Bc\xFF\xFF\xFF\xFF\x80\x91`\x03\x1B\x93\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x96\x01a\n\xABV[\x90\x91P`\x0F`\0R\x84`\0 `\x07\x80\x85\x01`\x03\x1C\x82\x01\x92\x01`\x03\x1C\x01\x90`\x1C\x84`\x02\x1B\x16\x80a\x0B?W[P\x90\x85\x92\x91[\x81\x81\x10a\x0B0WPa\x08bV[`\0\x81U\x86\x93P`\x01\x01a\x0B#V[`\0\x19\x82\x01\x90\x81T\x90`\0\x19\x90\x89\x03`\x03\x1B\x1C\x16\x90U\x86a\x0B\x1DV[\x92`\0\x93`\0[\x87\x82\x82\x10a\x0ByWPPPP\x01U\x82\x80\x80\x80a\x08&V[a\x0B\xA2\x84\x97`\x01\x93\x94\x95Q`\xE0\x1C\x90\x85`\x02\x1Bc\xFF\xFF\xFF\xFF\x80\x91`\x03\x1B\x93\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x96\x01\x92\x91\x01a\x0BbV[`\0\x80[\x88`\x08\x82\x10a\x0B\xC7WPP\x83\x82\x01U`\x01\x01a\x08\x11V[a\x0B\xEF\x88\x93`\x01\x93\x99Q`\xE0\x1C\x90\x8A`\x02\x1Bc\xFF\xFF\xFF\xFF\x80\x91`\x03\x1B\x93\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x96\x01a\x0B\xB0V[\x90\x91P`\x0E`\0R\x84`\0 `\x07\x80\x85\x01`\x03\x1C\x82\x01\x92\x01`\x03\x1C\x01\x90`\x1C\x84`\x02\x1B\x16\x80a\x0CDW[P\x90\x85\x92\x91[\x81\x81\x10a\x0C5WPa\x07\xFDV[`\0\x81U\x86\x93P`\x01\x01a\x0C(V[`\0\x19\x82\x01\x90\x81T\x90`\0\x19\x90\x89\x03`\x03\x1B\x1C\x16\x90U\x86a\x0C\"V[\x92`\0\x93`\0[\x87\x82\x82\x10a\x0C~WPPPP\x01U\x82\x80\x80\x80a\x07\xC1V[a\x0C\xA7\x84\x97`\x01\x93\x94\x95Q`\xE0\x1C\x90\x85`\x02\x1Bc\xFF\xFF\xFF\xFF\x80\x91`\x03\x1B\x93\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x96\x01\x92\x91\x01a\x0CgV[`\0\x80[\x88`\x08\x82\x10a\x0C\xCCWPP\x83\x82\x01U`\x01\x01a\x07\xACV[a\x0C\xF4\x88\x93`\x01\x93\x99Q`\xE0\x1C\x90\x8A`\x02\x1Bc\xFF\xFF\xFF\xFF\x80\x91`\x03\x1B\x93\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x96\x01a\x0C\xB5V[\x90\x91P`\r`\0R\x84`\0 `\x07\x80\x85\x01`\x03\x1C\x82\x01\x92\x01`\x03\x1C\x01\x90`\x1C\x84`\x02\x1B\x16\x80a\rIW[P\x90\x85\x92\x91[\x81\x81\x10a\r:WPa\x07\x98V[`\0\x81U\x86\x93P`\x01\x01a\r-V[`\0\x19\x82\x01\x90\x81T\x90`\0\x19\x90\x89\x03`\x03\x1B\x1C\x16\x90U\x86a\r'V[\x92`\0\x93`\0[\x87\x82\x82\x10a\r\x83WPPPP\x01U\x82\x80\x80\x80a\x07\\V[a\r\xAC\x84\x97`\x01\x93\x94\x95Q`\xE0\x1C\x90\x85`\x02\x1Bc\xFF\xFF\xFF\xFF\x80\x91`\x03\x1B\x93\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x96\x01\x92\x91\x01a\rlV[`\0\x80[\x88`\x08\x82\x10a\r\xD1WPP\x83\x82\x01U`\x01\x01a\x07GV[a\r\xF9\x88\x93`\x01\x93\x99Q`\xE0\x1C\x90\x8A`\x02\x1Bc\xFF\xFF\xFF\xFF\x80\x91`\x03\x1B\x93\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x96\x01a\r\xBAV[\x90\x91P`\x0C`\0R\x84`\0 `\x07\x80\x85\x01`\x03\x1C\x82\x01\x92\x01`\x03\x1C\x01\x90`\x1C\x84`\x02\x1B\x16\x80a\x0ENW[P\x90\x85\x92\x91[\x81\x81\x10a\x0E?WPa\x073V[`\0\x81U\x86\x93P`\x01\x01a\x0E2V[`\0\x19\x82\x01\x90\x81T\x90`\0\x19\x90\x89\x03`\x03\x1B\x1C\x16\x90U\x86a\x0E,V[\x92`\0\x93`\0[\x87\x82\x82\x10a\x0E\x88WPPPP\x01U\x82\x80\x80\x80a\x06\xF7V[a\x0E\xB1\x84\x97`\x01\x93\x94\x95Q`\xE0\x1C\x90\x85`\x02\x1Bc\xFF\xFF\xFF\xFF\x80\x91`\x03\x1B\x93\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x96\x01\x92\x91\x01a\x0EqV[`\0\x80[\x88`\x08\x82\x10a\x0E\xD6WPP\x83\x82\x01U`\x01\x01a\x06\xE2V[a\x0E\xFE\x88\x93`\x01\x93\x99Q`\xE0\x1C\x90\x8A`\x02\x1Bc\xFF\xFF\xFF\xFF\x80\x91`\x03\x1B\x93\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x96\x01a\x0E\xBFV[\x90\x91P`\x0B`\0R\x84`\0 `\x07\x80\x85\x01`\x03\x1C\x82\x01\x92\x01`\x03\x1C\x01\x90`\x1C\x84`\x02\x1B\x16\x80a\x0FSW[P\x90\x85\x92\x91[\x81\x81\x10a\x0FDWPa\x06\xCEV[`\0\x81U\x86\x93P`\x01\x01a\x0F7V[`\0\x19\x82\x01\x90\x81T\x90`\0\x19\x90\x89\x03`\x03\x1B\x1C\x16\x90U\x86a\x0F1V[\x92`\0\x93`\0[\x87\x82\x82\x10a\x0F\x8DWPPPP\x01U\x82\x80\x80\x80a\x06\x92V[a\x0F\xB6\x84\x97`\x01\x93\x94\x95Q`\xE0\x1C\x90\x85`\x02\x1Bc\xFF\xFF\xFF\xFF\x80\x91`\x03\x1B\x93\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x96\x01\x92\x91\x01a\x0FvV[`\0\x80[\x88`\x08\x82\x10a\x0F\xDBWPP\x83\x82\x01U`\x01\x01a\x06}V[a\x10\x03\x88\x93`\x01\x93\x99Q`\xE0\x1C\x90\x8A`\x02\x1Bc\xFF\xFF\xFF\xFF\x80\x91`\x03\x1B\x93\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x96\x01a\x0F\xC4V[\x90\x91P`\n`\0R\x84`\0 `\x07\x80\x85\x01`\x03\x1C\x82\x01\x92\x01`\x03\x1C\x01\x90`\x1C\x84`\x02\x1B\x16\x80a\x10XW[P\x90\x85\x92\x91[\x81\x81\x10a\x10IWPa\x06iV[`\0\x81U\x86\x93P`\x01\x01a\x10<V[`\0\x19\x82\x01\x90\x81T\x90`\0\x19\x90\x89\x03`\x03\x1B\x1C\x16\x90U\x86a\x106V[\x92`\0\x93`\0[\x87\x82\x82\x10a\x10\x92WPPPP\x01U\x82\x80\x80\x80a\x06-V[a\x10\xBB\x84\x97`\x01\x93\x94\x95Q`\xE0\x1C\x90\x85`\x02\x1Bc\xFF\xFF\xFF\xFF\x80\x91`\x03\x1B\x93\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x96\x01\x92\x91\x01a\x10{V[`\0\x80[\x88`\x08\x82\x10a\x10\xE0WPP\x83\x82\x01U`\x01\x01a\x06\x18V[a\x11\x08\x88\x93`\x01\x93\x99Q`\xE0\x1C\x90\x8A`\x02\x1Bc\xFF\xFF\xFF\xFF\x80\x91`\x03\x1B\x93\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x96\x01a\x10\xC9V[\x90\x91P`\t`\0R\x84`\0 `\x07\x80\x85\x01`\x03\x1C\x82\x01\x92\x01`\x03\x1C\x01\x90`\x1C\x84`\x02\x1B\x16\x80a\x11]W[P\x90\x85\x92\x91[\x81\x81\x10a\x11NWPa\x06\x04V[`\0\x81U\x86\x93P`\x01\x01a\x11AV[`\0\x19\x82\x01\x90\x81T\x90`\0\x19\x90\x89\x03`\x03\x1B\x1C\x16\x90U\x86a\x11;V[\x86\x86\x03`\x7F\x19\x01\x81R\x83Q\x80Q`\x01`\x01`\xA0\x1B\x03\x16\x87R\x89\x81\x01Q\x94\x96\x93\x94\x92\x93\x91\x92\x91\x90`\x03\x83\x10\x15a\t!Wa\x11\xCB\x82```@\x8E\x95\x94`\x01\x97\x87\x80\x97\x01R\x01Q\x91\x81`@\x82\x01R\x01\x90a\x19\xC0V[\x97\x01\x92\x01\x92\x01\x90\x92\x91a\x04VV[`@a\x11\xE5\x82\x87a\x19\x96V[Q\x01Q`\x01`\x01`\xA0\x1B\x03a\x11\xFA\x83\x88a\x19\x96V[QQ\x16\x90\x80Q\x15a\x17\xEDW\x88a\x12\x10\x84\x89a\x19\x96V[Q\x01Q\x91`\x03\x83\x10\x92\x83\x15a\t!W`\0\x81a\x146WPP\x80\x91\x92P\x15a\x14\x0EWa\xFF\xFF`\0\x80Q` a\x1C\x1C\x839\x81Q\x91RT\x16\x90a\x12\x8Ca\x12S``a\x18\xE1V[`!\x81R\x7FdiamondCut: Add facet has no cod\x8C\x82\x01R`e`\xF8\x1B`@\x82\x01R\x82a\x1A\x9AV[\x82Q\x92`\0\x92[\x84\x84\x10a\x12\xA9WPPPPP`\x01\x90[\x01a\x04(V[a\x12\xB3\x84\x83a\x19\x96V[Q`\x01`\x01`\xE0\x1B\x03\x19\x81\x16`\0\x90\x81R`\0\x80Q` a\x1C<\x839\x81Q\x91R\x8ER`@\x90 T\x90\x91\x90`\x01`\x01`\xA0\x1B\x03\x16a\x13\xF0W\x90a\x13c\x8D\x92a\xFF\xFFa\x12\xFBa\x18\xC2V[\x87\x81R\x91\x81\x16\x85\x83\x01\x81\x81R`\x01`\x01`\xE0\x1B\x03\x19\x86\x16`\0\x90\x81R`\0\x80Q` a\x1C<\x839\x81Q\x91R\x90\x97R`@\x90\x96 \x92Q\x83T\x96Q`\x01`\x01`\xB0\x1B\x03\x19\x90\x97\x16`\x01`\x01`\xA0\x1B\x03\x91\x90\x91\x16\x17\x95\x90\x91\x16`\xA0\x1Ba\xFF\xFF`\xA0\x1B\x16\x94\x90\x94\x17\x90UV[`\0\x80Q` a\x1C\x1C\x839\x81Q\x91RTh\x01\0\0\0\0\0\0\0\0\x81\x10\x15a\n@Wa\x13\xA4\x81`\x01a\x13\xC3\x93\x01`\0\x80Q` a\x1C\x1C\x839\x81Q\x91RUa\x1A?V[\x90\x92`\xE0\x1C\x90\x83T\x90c\xFF\xFF\xFF\xFF\x80\x91`\x03\x1B\x93\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x90Ua\xFF\xFF\x81\x14a\x13\xDAW`\x01\x93\x84\x01\x93\x01a\x12\x93V[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[Pc\xEB\xBF]\x07`\xE0\x1B`\0Rc\xFF\xFF\xFF\xFF`\xE0\x1B\x16`\x04R`$`\0\xFD[P\x87a\x142`@Q\x92\x83\x92c\x02\xB8\xDA\x07`\xE2\x1B\x84R`\x04\x84\x01R`$\x83\x01\x90a\x19\xC0V[\x03\x90\xFD[P`\x01\x81\x03a\x15\x98WP\x80\x91\x92P\x15a\x15tWa\x14\x96a\x14V``a\x18\xE1V[`(\x81R\x7FLibDiamondCut: Replace facet has\x8B\x82\x01Rg no code`\xC0\x1B`@\x82\x01R\x82a\x1A\x9AV[\x81Q\x91`\0[\x83\x81\x10a\x14\xAFWPPPP`\x01\x90a\x12\xA3V[`\x01`\x01`\xE0\x1B\x03\x19a\x14\xC2\x82\x84a\x19\x96V[Q\x16`\0\x81\x81R`\0\x80Q` a\x1C<\x839\x81Q\x91R\x8DR`@\x90 T`\x01`\x01`\xA0\x1B\x03\x160\x81\x14a\x15_W\x84\x81\x14a\x15JW\x15a\x156W`\0\x90\x81R`\0\x80Q` a\x1C<\x839\x81Q\x91R\x8CR`@\x90 \x80T`\x01`\x01`\xA0\x1B\x03\x19\x16`\x01`\x01`\xA0\x1B\x03\x85\x16\x17\x90U`\x01\x01a\x14\x9CV[cty\xF99`\xE0\x1B`\0R`\x04R`$`\0\xFD[Pc\x1A\xC6\xCE\x8D`\xE1\x1B`\0R`\x04R`$`\0\xFD[Pc)\x01\x80m`\xE1\x1B`\0R`\x04R`$`\0\xFD[P\x87a\x142`@Q\x92\x83\x92c\xCD\x98\xA9o`\xE0\x1B\x84R`\x04\x84\x01R`$\x83\x01\x90a\x19\xC0V[`\0\x93`\x02\x82\x03a\x17\xC2WPP`\0\x80Q` a\x1C\x1C\x839\x81Q\x91RT\x90\x80a\x17\xB0WP\x81Q\x91\x83[\x83\x81\x10a\x15\xD5WPPPPP`\x01\x90a\x12\xA3V[`\x01`\x01`\xE0\x1B\x03\x19a\x15\xE8\x82\x84a\x19\x96V[Q\x16\x80\x86R`\0\x80Q` a\x1C<\x839\x81Q\x91R\x8DR`@\x86 \x93\x8Da\x16\x0Ca\x18\xC2V[\x95T\x90a\xFF\xFF`\x01\x80`\xA0\x1B\x03\x83\x16\x92\x83\x89R`\xA0\x1C\x16\x90\x87\x01R\x15a\x17\x9CW\x84Q`\x01`\x01`\xA0\x1B\x03\x160\x14a\x17\x88W\x80\x15a\x17tW\x8D\x90`\0\x19\x01\x94\x85a\xFF\xFF\x83\x83\x01Q\x16\x03a\x16\xD6W[PP`\0\x80Q` a\x1C\x1C\x839\x81Q\x91RT\x80\x15a\x16\xC2W`\x01\x92\x91\x90`\0\x19\x01a\x16\x83\x81a\x1A?V[c\xFF\xFF\xFF\xFF\x82T\x91`\x03\x1B\x1B\x19\x16\x90U`\0\x80Q` a\x1C\x1C\x839\x81Q\x91RU\x86R`\0\x80Q` a\x1C<\x839\x81Q\x91R\x8DR\x85`@\x81 U\x01a\x15\xC1V[cNH{q`\xE0\x1B\x87R`1`\x04R`$\x87\xFD[`\0\x80Q` a\x1C<\x839\x81Q\x91R\x82a\xFF\xFFa\x17m\x94a\x16\xF6\x8Aa\x1A?V[\x90T\x90`\x03\x1B\x1C\x94a\x173a\x17\x0F\x84\x84\x84\x01Q\x16a\x1A?V[c\xFF\xFF\xFF\xFF\x89\x93\x92\x93\x16\x90\x83T\x90c\xFF\xFF\xFF\xFF\x80\x91`\x03\x1B\x93\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x90U\x01Q`\xE0\x94\x90\x94\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16\x8BR\x91\x90R`@\x89 \x80Ta\xFF\xFF`\xA0\x1B\x19\x16\x91\x90\x92\x16`\xA0\x1Ba\xFF\xFF`\xA0\x1B\x16\x17\x90UV[\x8C8a\x16YV[cNH{q`\xE0\x1B\x87R`\x11`\x04R`$\x87\xFD[c\r\xF5\xFDa`\xE3\x1B\x87R`\x04\x82\x90R`$\x87\xFD[cz\x08\xA2-`\xE0\x1B\x87R`\x04\x82\x90R`$\x87\xFD[c\xD0\x91\xBC\x81`\xE0\x1B\x84R`\x04R`$\x83\xFD[c?\xF4\xD2\x0F`\xE1\x1B\x85R`$\x91\x85\x91\x15a\x17\xDBW`\x04R\xFD[PcNH{q`\xE0\x1B\x81R`!`\x04R\xFD[Pc\xE7g\xF9\x1F`\xE0\x1B`\0R`\x04R`$`\0\xFD[c\x07\xA0CQ`\xE5\x1B`\0R`\x04`\0\xFD[c\x8B=\xDC3`\xE0\x1B`\0R`\x04`\0\xFD[`\0\x80\xFD[\x82Q`\x01`\x01`@\x1B\x03\x81\x11a\x18$W\x82\x01```\x1F\x19\x82\x88\x8B\x01\x03\x01\x12a\x18$W`@Q\x91``\x83\x01`\x01`\x01`@\x1B\x03\x81\x11\x84\x82\x10\x17a\n@W`@Ra\x18t` \x83\x01a\x19\x1DV[\x83R`@\x82\x01Q`\x03\x81\x10\x15a\x18$W` \x84\x01R``\x82\x01Q\x92`\x01`\x01`@\x1B\x03\x84\x11a\x18$Wa\x18\xB2` \x94\x93\x85\x80\x95\x8B\x8E\x01\x92\x01\x01a\x191V[`@\x82\x01R\x81R\x01\x92\x01\x91a\0~V[`@\x80Q\x91\x90\x82\x01`\x01`\x01`@\x1B\x03\x81\x11\x83\x82\x10\x17a\n@W`@RV[`@Q\x91\x90`\x1F\x01`\x1F\x19\x16\x82\x01`\x01`\x01`@\x1B\x03\x81\x11\x83\x82\x10\x17a\n@W`@RV[`\x01`\x01`@\x1B\x03\x81\x11a\n@W`\x05\x1B` \x01\x90V[Q\x90`\x01`\x01`\xA0\x1B\x03\x82\x16\x82\x03a\x18$WV[\x90\x80`\x1F\x83\x01\x12\x15a\x18$W\x81Q\x90a\x19La\0W\x83a\x19\x06V[\x92` \x80\x85\x85\x81R\x01\x93`\x05\x1B\x82\x01\x01\x91\x82\x11a\x18$W` \x01\x91[\x81\x83\x10a\x19uWPPP\x90V[\x82Q`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x81\x03a\x18$W\x81R` \x92\x83\x01\x92\x01a\x19hV[\x80Q\x82\x10\x15a\x19\xAAW` \x91`\x05\x1B\x01\x01\x90V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[\x90` \x80\x83Q\x92\x83\x81R\x01\x92\x01\x90`\0[\x81\x81\x10a\x19\xDEWPPP\x90V[\x82Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x84R` \x93\x84\x01\x93\x90\x92\x01\x91`\x01\x01a\x19\xD1V[\x91\x90\x82Q\x92\x83\x82R`\0[\x84\x81\x10a\x1A*WPP\x82`\0` \x80\x94\x95\x84\x01\x01R`\x1F\x80\x19\x91\x01\x16\x01\x01\x90V[\x80` \x80\x92\x84\x01\x01Q\x82\x82\x86\x01\x01R\x01a\x1A\tV[\x90`\0\x80Q` a\x1C\x1C\x839\x81Q\x91RT\x82\x10\x15a\x19\xAAW`\0\x80Q` a\x1C\x1C\x839\x81Q\x91R`\0R`\x03\x82\x90\x1C\x7F\xB6[\xEC\xA8\xB6\xFAx\x8B\xCB\x15(\xC2\xAB_M\xC6\xBC\x98\xE5\x89eP\xBA\xA0\x13\xD83\x0F\xAB\x0B\x86\xF4\x01\x91`\x02\x1B`\x1C\x16\x90V[\x80;\x15a\x1A\xA5WPPV[`@\x80Qc\x91\x984\xB9`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x90\x92\x16`\x04\x83\x01R`$\x82\x01R\x90\x81\x90a\x142\x90`D\x83\x01\x90a\x19\xFEV\xFE`\x80`@R6\x15`\x87W`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x90\x91 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`sW`\0\x80\x836\x82\x807\x816\x91Z\xF4=`\0\x80>\x15`nW=`\0\xF3[=`\0\xFD[c\n\x82\xDDs`\xE3\x1B`\0R`\x04R`$`\0\xFD[`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x90\x91 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`sW`\0\x80\x836\x82\x807\x816\x91Z\xF4=`\0\x80>\x15`nW=`\0\xF3\xFE\xA2dipfsX\"\x12 \xE3I,O*\x8C\xBF\x08\xF6\xEDv\x9C\xC4\x9D\xE7\xB5>\x8A|\xDA\0KW'\xD0\xFDDg\xD2c\xD0\x9AdsolcC\0\x08\x1A\x003\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD5\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2";
    /// The bytecode of the contract.
    pub static SUBNETREGISTRYDIAMOND_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__BYTECODE);
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R6\x15`\x87W`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x90\x91 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`sW`\0\x80\x836\x82\x807\x816\x91Z\xF4=`\0\x80>\x15`nW=`\0\xF3[=`\0\xFD[c\n\x82\xDDs`\xE3\x1B`\0R`\x04R`$`\0\xFD[`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x90\x91 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`sW`\0\x80\x836\x82\x807\x816\x91Z\xF4=`\0\x80>\x15`nW=`\0\xF3\xFE\xA2dipfsX\"\x12 \xE3I,O*\x8C\xBF\x08\xF6\xEDv\x9C\xC4\x9D\xE7\xB5>\x8A|\xDA\0KW'\xD0\xFDDg\xD2c\xD0\x9AdsolcC\0\x08\x1A\x003";
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
    ///`ConstructorParams(address,address,address,address,address,address,address,address,address,bytes4[],bytes4[],bytes4[],bytes4[],bytes4[],bytes4[],bytes4[],bytes4[],uint8)`
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
        pub subnet_actor_getter_selectors: ::std::vec::Vec<[u8; 4]>,
        pub subnet_actor_manager_selectors: ::std::vec::Vec<[u8; 4]>,
        pub subnet_actor_rewarder_selectors: ::std::vec::Vec<[u8; 4]>,
        pub subnet_actor_checkpointer_selectors: ::std::vec::Vec<[u8; 4]>,
        pub subnet_actor_pauser_selectors: ::std::vec::Vec<[u8; 4]>,
        pub subnet_actor_diamond_cut_selectors: ::std::vec::Vec<[u8; 4]>,
        pub subnet_actor_diamond_loupe_selectors: ::std::vec::Vec<[u8; 4]>,
        pub subnet_actor_ownership_selectors: ::std::vec::Vec<[u8; 4]>,
        pub creation_privileges: u8,
    }
}
