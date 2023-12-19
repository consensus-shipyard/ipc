pub use gateway_diamond::*;
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
                                ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
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
                                ::ethers::core::abi::ethabi::ParamType::Uint(16usize),
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
            events: ::std::collections::BTreeMap::new(),
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
                    ::std::borrow::ToOwned::to_owned("InvalidCollateral"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("InvalidCollateral"),
                            inputs: ::std::vec![],
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
    pub static GATEWAYDIAMOND_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(
        __abi,
    );
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = b"`\x80`@R4b\0\x0C\xB5Wb\0\x1B\x8C\x808\x03\x80\x91b\0\0 \x82`\x80b\0\x0F\x1EV[`\x809`@\x81\x12b\0\x0C\xB5W`\x80Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0C\xB5W`\x80\x82\x01`\x9F\x82\x01\x12\x15b\0\x0C\xB5W\x80`\x80\x01Q\x90b\0\0^\x82b\0\x0FBV[\x91b\0\0n`@Q\x93\x84b\0\x0F\x1EV[\x80\x83R` \x83\x01\x80\x92\x85`\x80\x01` \x84`\x05\x1B\x83`\x80\x01\x01\x01\x11b\0\x0C\xB5W`\xA0\x81\x01\x91[`\xA0`\x05\x85\x90\x1B\x83\x01\x01\x83\x10b\0\r\xDCWPP`\xA0Q\x91PP`\x01`\x01`@\x1B\x03\x81\x11b\0\x0C\xB5W`\xE0\x81\x85\x03\x12b\0\x0C\xB5W`@Q\x93`\xE0\x85\x01`\x01`\x01`@\x1B\x03\x81\x11\x86\x82\x10\x17b\0\x05\xA8W`@\x81\x90R`\x80\x83\x01Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0C\xB5W\x83`\x80\x01\x01`@\x81\x84`\x80\x01\x03\x12b\0\x0C\xB5Wb\0\x01\x18\x82b\0\x0F\x02V[\x80Q`\x01`\x01`@\x1B\x03\x81\x16\x81\x03b\0\x0C\xB5W\x82R` \x81\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0C\xB5W\x01\x82`\x80\x01`\x1F\x82\x01\x12\x15b\0\x0C\xB5W\x80Q\x90b\0\x01`\x82b\0\x0FBV[\x91b\0\x01p`@Q\x93\x84b\0\x0F\x1EV[\x80\x83R` \x80\x84\x01\x91`\x05\x1B\x83\x01\x01\x91\x85`\x80\x01\x83\x11b\0\x0C\xB5W` \x01\x90[\x82\x82\x10b\0\r\xC1WPPPa\x01\0\x87\x81\x01\x91\x90\x91R\x90\x86R`\xA0\x83\x01Q` \x87\x01R`\xC0\x83\x01Q`@\x87\x01R`\xE0\x83\x01Q``\x87\x01R\x82\x01Q`\xFF\x81\x16\x81\x03b\0\x0C\xB5W`\x80\x86\x01Ra\x01 \x82\x01Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0C\xB5W`\x80\x82\x01`\x9F\x84\x83\x01\x01\x12\x15b\0\x0C\xB5W\x80\x83`\x80\x01\x01Qb\0\x02\x11\x81b\0\x0FBV[\x92b\0\x02!`@Q\x94\x85b\0\x0F\x1EV[\x81\x84R` \x84\x01`\x80\x82\x01`\xA0\x87\x86\x01`\x05\x86\x90\x1B\x01\x01\x11b\0\x0C\xB5W`\xA0\x86\x85\x01\x01\x90[`\xA0\x87\x86\x01`\x05\x86\x90\x1B\x01\x01\x82\x10b\0\x0C\xBAW\x89\x89\x89`\xC0\x8A\x8A`\xA0\x86\x01R`\x80\x01\x01Qa\xFF\xFF\x81\x16\x81\x03b\0\x0C\xB5W`\xC0\x84\x01R`@\x83\x01Q\x15b\0\x0C\xA3W` \x83\x01Q\x15b\0\x0C\x91W`\xFF`\x80\x84\x01Q\x16`3\x81\x10\x90\x81\x15b\0\x0C\x85W[Pb\0\x0CsW\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD5\x80T`\x01`\x01`\xA0\x1B\x03\x19\x163\x17\x90U`@Q`\x01`\x01`@\x1B\x03` \x82\x01\x90\x81\x11\x90\x82\x11\x17b\0\x05\xA8W` \x81\x01`@R`\0\x81R\x82Q`\0[\x81\x81\x10b\0\x06\x18WPP`@Q\x92``\x84\x01\x90``\x85RQ\x80\x91R`\x80\x84\x01\x90`\x80\x81`\x05\x1B\x86\x01\x01\x93\x91`\0\x90[\x82\x82\x10b\0\x05\xBEW\x87\x7F\x8F\xAAp\x87\x86q\xCC\xD2\x12\xD2\x07q\xB7\x95\xC5\n\xF8\xFD?\xF6\xCF'\xF4\xBD\xE5~]M\xE0\xAE\xB6s\x88\x80b\0\x03~\x8A\x8A`\0` \x85\x01R\x83\x82\x03`@\x85\x01Rb\0\x10&V[\x03\x90\xA1\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD4` \x90\x81R\x7F}\xFDH\xDD\x9D\xEF\0/\xA9\xB4\xA0[\xD6\xB7&\xA6\xC3\x13\xC3b\xD3\xF3\xE8A=zu \xF0\t\r%\x80T`\xFF\x19\x90\x81\x16`\x01\x90\x81\x17\x90\x92U\x7FM\x7FL\x8A/\xB5\xB3\\\xA3\xC2w\xC98\x88\xB4\x7F\x0F\")\xBD\xCC\xCFfPM\x1B\xA4\x8E\x88\xB8\x81d\x80T\x82\x16\x83\x17\x90UcH\xE2\xB0\x93`\xE0\x1B`\0R\x7FY\xBAM\xB4\xA2\x13\xE8\x16\x1D\xE5\x97\xB8\xC1\r\xB0\xE7\xE7\xBAZ\xCE\\&\x8E67\x9E$\x9Am-B\xC9\x80T\x90\x91\x16\x90\x91\x17\x90U`,\x80Td\xFF\xFF\xFF\xFF\xFF\x19\x16`\x02\x17\x90U\x81Q\x80Q`\x19\x80T`\x01`\x01`@\x1B\x03\x19\x16`\x01`\x01`@\x1B\x03\x92\x83\x16\x17\x90U\x91\x01Q\x80Q\x91\x82\x11b\0\x05\xA8Wh\x01\0\0\0\0\0\0\0\0\x82\x11b\0\x05\xA8W` \x90`\x1AT\x83`\x1AU\x80\x84\x10b\0\x05\x87W[P\x01`\x1A`\0R` `\0 `\0[\x83\x81\x10b\0\x05iWb\0\x05Y`\xA0\x86`@\x81\x01Q`\x1BU` \x81\x01Q`\x1EU``\x81\x01Q`\x1CU`\xFF`\x80\x82\x01Q\x16`\xFF\x19`\x1DT\x16\x17`\x1DU`\x01`\x0BU`\x01`\x12U`d`\x1FU`\n`\x01\x80`@\x1B\x03\x19` T\x16\x17` U`\xC0\x81\x01Qb\xFF\xFF\0`!T\x91`\x08\x1B\x16\x90b\xFF\xFF\0\x19\x16\x17`!Uh\x01\0\0\0\0\0\0\0\x01`\x01\x80`\x80\x1B\x03\x19`*T\x16\x17`*U\x01Q`@Q\x90b\0\x05J\x82b\0\x0F\x02V[\x81R`\0` \x82\x01Rb\0\x13\xFEV[`@Qa\x013\x90\x81b\0\x1A\x19\x829\xF3[\x82Q`\x01`\x01`\xA0\x1B\x03\x16\x81\x83\x01U` \x90\x92\x01\x91`\x01\x01b\0\x04\xA8V[b\0\x05\xA1\x90`\x1A`\0R\x84\x84`\0 \x91\x82\x01\x91\x01b\0\x0F\x94V[\x84b\0\x04\x99V[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[\x90\x91\x92\x94` \x80b\0\x06\t`\x01\x93`\x7F\x19\x8B\x82\x03\x01\x86R```@\x8BQ\x87\x80`\xA0\x1B\x03\x81Q\x16\x84Rb\0\x05\xF8\x86\x82\x01Q\x87\x86\x01\x90b\0\x0F\xD8V[\x01Q\x91\x81`@\x82\x01R\x01\x90b\0\x0F\xE6V[\x97\x01\x92\x01\x92\x01\x90\x92\x91b\0\x037V[`@b\0\x06&\x82\x87b\0\x0F\xADV[Q\x01Q`\x01`\x01`\xA0\x1B\x03b\0\x06=\x83\x88b\0\x0F\xADV[QQ\x16\x90\x80Q\x15b\0\x0CZW` b\0\x06W\x84\x89b\0\x0F\xADV[Q\x01Q`\x03\x81\x10\x15b\0\x0CDW\x80b\0\x08\x81WP\x81\x15b\0\x08VWa\xFF\xFF`\0\x80Q` b\0\x1BL\x839\x81Q\x91RT\x16b\0\x06\xD4`@Qb\0\x06\x99\x81b\0\x0E\xE6V[`!\x81R\x7FdiamondCut: Add facet has no cod` \x82\x01R`e`\xF8\x1B`@\x82\x01R\x84b\0\x10\x7FV[\x81Q\x91`\0\x91[\x83\x83\x10b\0\x06\xF3WPPPPP`\x01\x90[\x01b\0\x03\x08V[`\x01`\x01`\xE0\x1B\x03\x19b\0\x07\x08\x84\x84b\0\x0F\xADV[Q\x16`\0\x81\x81R`\0\x80Q` b\0\x1Bl\x839\x81Q\x91R` R`@\x90 T\x90\x91\x90`\x01`\x01`\xA0\x1B\x03\x16b\0\x08=Wb\0\x07\xAB`@Qb\0\x07J\x81b\0\x0F\x02V[\x87\x81Ra\xFF\xFF\x92\x90\x92\x16` \x80\x84\x01\x82\x81R`\0\x86\x81R`\0\x80Q` b\0\x1Bl\x839\x81Q\x91R\x90\x92R`@\x90\x91 \x93Q\x84T\x91Q`\x01`\x01`\xB0\x1B\x03\x19\x90\x92\x16`\x01`\x01`\xA0\x1B\x03\x91\x90\x91\x16\x17`\xA0\x91\x90\x91\x1Ba\xFF\xFF`\xA0\x1B\x16\x17\x90\x92UV[`\0\x80Q` b\0\x1BL\x839\x81Q\x91RT\x91h\x01\0\0\0\0\0\0\0\0\x83\x10\x15b\0\x05\xA8Wb\0\x08\x12\x90b\0\x07\xF5`\x01\x94\x85\x81\x01`\0\x80Q` b\0\x1BL\x839\x81Q\x91RUb\0\x10MV[\x90\x91\x90c\xFF\xFF\xFF\xFF\x83T\x91`\x03\x1B\x92`\xE0\x1C\x83\x1B\x92\x1B\x19\x16\x17\x90UV[a\xFF\xFF\x81\x14b\0\x08'W\x81\x01\x92\x01\x91b\0\x06\xDBV[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[`@Qc\xEB\xBF]\x07`\xE0\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qc\x02\xB8\xDA\x07`\xE2\x1B\x81R` `\x04\x82\x01R\x90\x81\x90b\0\x08}\x90`$\x83\x01\x90b\0\x0F\xE6V[\x03\x90\xFD[`\x01\x81\x03b\0\t\xFAWP\x81\x15b\0\t\xD3Wb\0\x08\xE6`@Qb\0\x08\xA4\x81b\0\x0E\xE6V[`(\x81R\x7FLibDiamondCut: Replace facet has` \x82\x01Rg no code`\xC0\x1B`@\x82\x01R\x83b\0\x10\x7FV[\x80Q\x90`\0[\x82\x81\x10b\0\t\x01WPPPP`\x01\x90b\0\x06\xECV[`\x01`\x01`\xE0\x1B\x03\x19b\0\t\x16\x82\x84b\0\x0F\xADV[Q\x16`\0\x81\x81R`\0\x80Q` b\0\x1Bl\x839\x81Q\x91R` R`@\x90 T`\x01`\x01`\xA0\x1B\x03\x160\x81\x14b\0\t\xBAW\x85\x81\x14b\0\t\xA1W\x15b\0\t\x89W`\0\x90\x81R`\0\x80Q` b\0\x1Bl\x839\x81Q\x91R` R`@\x90 \x80T`\x01`\x01`\xA0\x1B\x03\x19\x16\x85\x17\x90U`\x01\x01b\0\x08\xECV[`$\x90`@Q\x90cty\xF99`\xE0\x1B\x82R`\x04\x82\x01R\xFD[`@Qc\x1A\xC6\xCE\x8D`\xE1\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qc)\x01\x80m`\xE1\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qc\xCD\x98\xA9o`\xE0\x1B\x81R` `\x04\x82\x01R\x90\x81\x90b\0\x08}\x90`$\x83\x01\x90b\0\x0F\xE6V[`\x02\x81\x03b\0\x0C\"WP`\0\x80Q` b\0\x1BL\x839\x81Q\x91RT\x91\x80b\0\x0C\nWP\x80Q\x90`\0[\x82\x81\x10b\0\n8WPPPP`\x01\x90b\0\x06\xECV[`\x01`\x01`\xE0\x1B\x03\x19b\0\nM\x82\x84b\0\x0F\xADV[Q\x16\x90\x81`\0R`\0\x80Q` b\0\x1Bl\x839\x81Q\x91R` R`@`\0 \x94`@Q\x95b\0\n|\x87b\0\x0F\x02V[T`\x01`\x01`\xA0\x1B\x03\x81\x16\x80\x88R`\xA0\x91\x90\x91\x1Ca\xFF\xFF\x16` \x88\x01R\x15b\0\x0B\xF1W\x85Q`\x01`\x01`\xA0\x1B\x03\x160\x14b\0\x0B\xD8W\x80\x15b\0\x08'W`\0\x19\x01\x94\x85a\xFF\xFF` \x83\x01Q\x16\x03b\0\x0BUW[P`\0\x80Q` b\0\x1BL\x839\x81Q\x91RT\x91\x82\x15b\0\x0B?W`\x01\x92`\0\x19\x01b\0\n\xFA\x81b\0\x10MV[c\xFF\xFF\xFF\xFF\x82T\x91`\x03\x1B\x1B\x19\x16\x90U`\0\x80Q` b\0\x1BL\x839\x81Q\x91RU`\0R`\0\x80Q` b\0\x1Bl\x839\x81Q\x91R` R`\0`@\x81 U\x01b\0\n#V[cNH{q`\xE0\x1B`\0R`1`\x04R`$`\0\xFD[b\0\x0B\xD1\x90a\xFF\xFF` b\0\x0Bj\x89b\0\x10MV[\x90T\x90`\x03\x1B\x1C`\xE0\x1B\x92b\0\x0B\x8A\x84b\0\x07\xF5\x85\x85\x85\x01Q\x16b\0\x10MV[\x01Q`\x01`\x01`\xE0\x1B\x03\x19\x90\x92\x16`\0\x90\x81R`\0\x80Q` b\0\x1Bl\x839\x81Q\x91R` R`@\x90 \x80Ta\xFF\xFF`\xA0\x1B\x19\x16\x91\x90\x92\x16`\xA0\x1Ba\xFF\xFF`\xA0\x1B\x16\x17\x90UV[\x8Bb\0\n\xCEV[`@Qc\r\xF5\xFDa`\xE3\x1B\x81R`\x04\x81\x01\x84\x90R`$\x90\xFD[`@Qcz\x08\xA2-`\xE0\x1B\x81R`\x04\x81\x01\x84\x90R`$\x90\xFD[`$\x90`@Q\x90c\xD0\x91\xBC\x81`\xE0\x1B\x82R`\x04\x82\x01R\xFD[`@Qc?\xF4\xD2\x0F`\xE1\x1B\x81R`$\x91b\0\x0CB\x90`\x04\x83\x01\x90b\0\x0F\xD8V[\xFD[cNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[`@Qc\xE7g\xF9\x1F`\xE0\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qcu\xC3\xB4'`\xE0\x1B\x81R`\x04\x90\xFD[`d\x91P\x11\x84b\0\x02\xA6V[`@Qc1/\x8E\x05`\xE0\x1B\x81R`\x04\x90\xFD[`@Qch\xF7\xA6u`\xE1\x1B\x81R`\x04\x90\xFD[`\0\x80\xFD[\x81Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0C\xB5W``\x88\x87\x01\x83\x01\x85\x03`\x1F\x19\x01\x12b\0\x0C\xB5W`@Q\x91b\0\x0C\xEE\x83b\0\x0E\xE6V[\x86\x89\x01\x81\x01`\xA0\x81\x01Q\x84Rb\0\r\x08\x90`\xC0\x01b\0\x0FZV[` \x84\x01R`\xE0\x89\x88\x01\x82\x01\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0C\xB5W\x87\x8A`\x80\x01\x01\x01\x01\x90\x84`\x80\x01`?\x83\x01\x12\x15b\0\x0C\xB5W` \x82\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\r\xACW`@Q\x93b\0\rm`\x1F\x84\x01`\x1F\x19\x16` \x01\x86b\0\x0F\x1EV[\x82\x85R\x86`\x80\x01`@\x84\x86\x01\x01\x11b\0\x0C\xB5W\x84b\0\r\x99` \x96\x94\x87\x96`@\x88\x80\x98\x01\x91\x01b\0\x0FoV[`@\x82\x01R\x81R\x01\x92\x01\x91\x90Pb\0\x02FV[`$`\0cNH{q`\xE0\x1B\x81R`A`\x04R\xFD[` \x80\x91b\0\r\xD0\x84b\0\x0FZV[\x81R\x01\x91\x01\x90b\0\x01\x90V[\x82Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0C\xB5W``\x90\x83\x01`\x80\x81\x01\x90\x89\x03`\x1F\x19\x01\x82\x13b\0\x0C\xB5W`@Q\x91b\0\x0E\x13\x83b\0\x0E\xE6V[b\0\x0E!` \x83\x01b\0\x0FZV[\x83R`@\x82\x01Q`\x03\x81\x10\x15b\0\x0C\xB5W` \x84\x01R\x81\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0C\xB5W\x01\x88`\x80\x01`?\x82\x01\x12\x15b\0\x0C\xB5W` \x81\x01Q\x90b\0\x0Ek\x82b\0\x0FBV[\x91b\0\x0E{`@Q\x93\x84b\0\x0F\x1EV[\x80\x83R` \x83\x01\x8B`\x80\x01`@\x83`\x05\x1B\x85\x01\x01\x11b\0\x0C\xB5W`@\x83\x01\x90[`@\x83`\x05\x1B\x85\x01\x01\x82\x10b\0\x0E\xC3WPPPP`@\x82\x01R\x81R` \x92\x83\x01\x92\x01b\0\0\x93V[\x81Q`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x81\x03b\0\x0C\xB5W\x81R` \x91\x82\x01\x91\x01b\0\x0E\x9BV[``\x81\x01\x90\x81\x10`\x01`\x01`@\x1B\x03\x82\x11\x17b\0\x05\xA8W`@RV[`@\x81\x01\x90\x81\x10`\x01`\x01`@\x1B\x03\x82\x11\x17b\0\x05\xA8W`@RV[`\x1F\x90\x91\x01`\x1F\x19\x16\x81\x01\x90`\x01`\x01`@\x1B\x03\x82\x11\x90\x82\x10\x17b\0\x05\xA8W`@RV[`\x01`\x01`@\x1B\x03\x81\x11b\0\x05\xA8W`\x05\x1B` \x01\x90V[Q\x90`\x01`\x01`\xA0\x1B\x03\x82\x16\x82\x03b\0\x0C\xB5WV[`\0[\x83\x81\x10b\0\x0F\x83WPP`\0\x91\x01RV[\x81\x81\x01Q\x83\x82\x01R` \x01b\0\x0FrV[\x81\x81\x10b\0\x0F\xA0WPPV[`\0\x81U`\x01\x01b\0\x0F\x94V[\x80Q\x82\x10\x15b\0\x0F\xC2W` \x91`\x05\x1B\x01\x01\x90V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[\x90`\x03\x82\x10\x15b\0\x0CDWRV[\x90\x81Q\x80\x82R` \x80\x80\x93\x01\x93\x01\x91`\0[\x82\x81\x10b\0\x10\x07WPPPP\x90V[\x83Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x93\x81\x01\x93\x92\x81\x01\x92`\x01\x01b\0\x0F\xF8V[\x90` \x91b\0\x10A\x81Q\x80\x92\x81\x85R\x85\x80\x86\x01\x91\x01b\0\x0FoV[`\x1F\x01`\x1F\x19\x16\x01\x01\x90V[\x90`\0\x80Q` b\0\x1BL\x839\x81Q\x91R\x80T\x83\x10\x15b\0\x0F\xC2W`\0R`\x1C` `\0 \x83`\x03\x1C\x01\x92`\x02\x1B\x16\x90V[\x80;\x15b\0\x10\x8BWPPV[`@\x80Qc\x91\x984\xB9`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x90\x92\x16`\x04\x83\x01R`$\x82\x01R\x90\x81\x90b\0\x08}\x90`D\x83\x01\x90b\0\x10&V[\x90\x80\x82Q\x90\x81\x81R` \x80\x91\x01\x92\x81\x80\x84`\x05\x1B\x83\x01\x01\x95\x01\x93`\0\x91[\x84\x83\x10b\0\x10\xF0WPPPPPP\x90V[\x90\x91\x92\x93\x94\x95\x84\x80b\0\x113`\x01\x93`\x1F\x19\x86\x82\x03\x01\x87R\x8AQ\x80Q\x82R\x85\x80`\xA0\x1B\x03\x84\x82\x01Q\x16\x84\x83\x01R`@\x80\x91\x01Q\x91``\x80\x92\x82\x01R\x01\x90b\0\x10&V[\x98\x01\x93\x01\x93\x01\x91\x94\x93\x92\x90b\0\x10\xDFV[\x90`\x01\x82\x81\x1C\x92\x16\x80\x15b\0\x11vW[` \x83\x10\x14b\0\x11`WV[cNH{q`\xE0\x1B`\0R`\"`\x04R`$`\0\xFD[\x91`\x7F\x16\x91b\0\x11TV[\x91\x90`\x1F\x81\x11b\0\x11\x91WPPPV[b\0\x11\xC0\x92`\0R` `\0 \x90` `\x1F\x84\x01`\x05\x1C\x83\x01\x93\x10b\0\x11\xC2W[`\x1F\x01`\x05\x1C\x01\x90b\0\x0F\x94V[V[\x90\x91P\x81\x90b\0\x11\xB2V[\x90\x80\x82\x14b\0\x12\xB6Wb\0\x11\xE2\x81Tb\0\x11DV[\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x05\xA8W\x81\x90b\0\x12\x0C\x82b\0\x12\x05\x86Tb\0\x11DV[\x86b\0\x11\x81V[`\0\x90`\x1F\x83\x11`\x01\x14b\0\x12FW`\0\x92b\0\x12:W[PP\x81`\x01\x1B\x91`\0\x19\x90`\x03\x1B\x1C\x19\x16\x17\x90UV[\x01T\x90P8\x80b\0\x12$V[\x81R` \x80\x82 \x85\x83R\x81\x83 \x93P\x90`\x1F\x19\x85\x16\x90\x83\x90[\x82\x82\x10b\0\x12\x9CWPP\x90\x84`\x01\x95\x94\x93\x92\x10b\0\x12\x82W[PPP\x81\x1B\x01\x90UV[\x01T`\0\x19`\xF8\x84`\x03\x1B\x16\x1C\x19\x16\x90U8\x80\x80b\0\x12xV[\x84\x95\x81\x92\x95\x85\x01T\x81U`\x01\x80\x91\x01\x96\x01\x94\x01\x90b\0\x12_V[PPV[`\x04T\x81\x10\x15b\0\x0F\xC2W`\x04`\0R`\x03` `\0 \x91\x02\x01\x90`\0\x90V[\x90b\0\x13\xE8W\x81Q\x81U` \x80\x83\x01Q`\x01\x80\x84\x01\x80T`\x01`\x01`\xA0\x1B\x03\x19\x16`\x01`\x01`\xA0\x1B\x03\x93\x90\x93\x16\x92\x90\x92\x17\x90\x91U`@\x90\x93\x01Q\x80Q`\x02\x90\x93\x01\x93\x92\x91\x90`\x01`\x01`@\x1B\x03\x83\x11b\0\x05\xA8Wb\0\x13F\x83b\0\x13?\x87Tb\0\x11DV[\x87b\0\x11\x81V[\x81`\x1F\x84\x11`\x01\x14b\0\x13\x82WP\x92\x82\x93\x91\x83\x92`\0\x94b\0\x13vW[PP\x1B\x91`\0\x19\x90`\x03\x1B\x1C\x19\x16\x17\x90UV[\x01Q\x92P8\x80b\0\x13cV[\x91\x90\x83`\x1F\x19\x81\x16\x87`\0R\x84`\0 \x94`\0\x90[\x88\x83\x83\x10b\0\x13\xCDWPPP\x10b\0\x13\xB3WPPP\x81\x1B\x01\x90UV[\x01Q`\0\x19`\xF8\x84`\x03\x1B\x16\x1C\x19\x16\x90U8\x80\x80b\0\x12xV[\x85\x87\x01Q\x88U\x90\x96\x01\x95\x94\x85\x01\x94\x87\x93P\x90\x81\x01\x90b\0\x13\x97V[cNH{q`\xE0\x1B`\0R`\0`\x04R`$`\0\xFD[`@\x90\x81Q\x90` \x90\x81\x83Rb\0\x14 \x81Q\x85\x84\x86\x01R``\x85\x01\x90b\0\x10\xC1V[\x92\x7F~\xCD\xACH#4\xC3o\xCC\xBE7C\x18\xCF\xE7N\xA0\xC8\x18\x13\x94\x89\r\xDE\xC8\x94\xA1\x0F\x0F\xCCt\x81\x83\x83\x01\x91\x80`\x01\x80`@\x1B\x03\x96\x87\x85Q\x16\x89\x83\x01R\x03\x90\xA1`\x05\x94\x84\x86T\x16\x80b\0\x17\x8DW[PP`\x04\x91\x82T\x91h\x01\0\0\0\0\0\0\0\0\x95\x86\x84\x11b\0\x17xW`\x06T\x84`\x06U\x80\x85\x10b\0\x16\x8EW[P`\0\x93\x85\x85R\x86\x85 `\x06\x86R\x87\x86 \x90\x86\x90[\x83\x82\x10b\0\x168WPPPP\x80\x88T\x16\x91`\x01\x80`@\x1B\x03\x19\x92\x83`\x07T\x16\x17`\x07U\x83QQ\x93\x86T\x98\x86[\x86\x81\x10b\0\x15\xB6WPPPQ\x16\x90\x87T\x16\x17\x86U\x80\x85\x11b\0\x14\xF9W[PPPPPPV[\x84\x81\x10\x15b\0\x14\xF1W\x82T\x80\x15b\0\x15\xA3W`\0\x19\x01\x90b\0\x15\x1B\x82b\0\x12\xBAV[\x92\x90\x92b\0\x15\x91W\x83\x83U\x83`\x02`\x01\x94\x82\x86\x82\x01U\x01b\0\x15>\x81Tb\0\x11DV[\x80b\0\x15QW[PPP\x84U\x01b\0\x14\xF9V[\x82`\x1F\x80\x83\x11`\x01\x14b\0\x15mWPPPU[\x838\x80b\0\x15EV[\x83\x82R\x8A\x82 \x93\x91\x92b\0\x15\x89\x91\x01\x8D\x1C\x84\x01\x88\x85\x01b\0\x0F\x94V[UUb\0\x15dV[cNH{q`\xE0\x1B\x84R\x83\x85R`$\x84\xFD[cNH{q`\xE0\x1B\x83R`1\x84R`$\x83\xFD[\x8A\x81\x10\x15b\0\x15\xEDW\x80b\0\x15\xE6b\0\x15\xD3`\x01\x93\x86Qb\0\x0F\xADV[Qb\0\x15\xDF\x83b\0\x12\xBAV[\x90b\0\x12\xDAV[\x01b\0\x14\xD4V[b\0\x15\xFA\x81\x84Qb\0\x0F\xADV[Q\x89T\x83\x81\x10\x15b\0\x16%W`\x01\x92\x91b\0\x15\xDF\x82\x8D\x86b\0\x16\x1F\x95\x01\x90Ub\0\x12\xBAV[b\0\x15\xE6V[cNH{q`\xE0\x1B\x8AR`A\x8BR`$\x8A\xFD[\x80`\x01\x91\x84\x03b\0\x16UW[`\x03\x80\x91\x01\x93\x01\x91\x01\x90\x91b\0\x14\xA8V[\x80T\x84U\x81\x80\x85\x01\x90\x83\x80`\xA0\x1B\x03\x90\x83\x01T\x16\x83\x80`\xA0\x1B\x03\x19\x82T\x16\x17\x90Ub\0\x16\x88`\x02\x80\x83\x01\x90\x86\x01b\0\x11\xCDV[b\0\x16DV[`\x03\x90\x80\x82\x02\x90\x82\x82\x04\x03b\0\x17cW\x85\x82\x02\x82\x81\x04\x87\x03b\0\x17NW\x88`\0\x91`\x06\x83R\x81\x83 \x93\x84\x01\x93\x01[\x8C\x84\x82\x10b\0\x16\xD0WPPPPPb\0\x14\x93V[\x85\x92\x84\x91\x82\x84U`\x01\x83\x81\x86\x01U`\x02\x85\x01\x91b\0\x16\xEF\x83Tb\0\x11DV[\x91\x82b\0\x17\x06W[PPPPPP\x01\x89\x90b\0\x16\xBCV[\x85\x91`\x1F\x80\x85\x11`\x01\x14b\0\x17*WPPPPP\x90PU[\x82\x8B8\x8F\x80\x82b\0\x16\xF7V[\x85\x94\x96\x84\x96b\0\x17F\x95R\x86 \x96\x01\x90\x1C\x85\x01\x90\x85\x01b\0\x0F\x94V[UUb\0\x17\x1EV[`\x11\x88cNH{q`\xE0\x1B`\0RR`$`\0\xFD[`\x11\x87cNH{q`\xE0\x1B`\0RR`$`\0\xFD[`A\x85cNH{q`\xE0\x1B`\0RR`$`\0\xFD[\x85\x83Q\x16\x86`\x07T\x16\x90\x81\x81\x14b\0\x19\x13W\x10b\0\x19\x02W\x81Q\x91b\0\x17\xB3\x83b\0\x0F\x02V[`\x04Tb\0\x17\xC1\x81b\0\x0FBV[\x91b\0\x17\xD0\x81Q\x93\x84b\0\x0F\x1EV[\x81\x83R\x87\x83\x01\x90`\x04`\0R\x88`\0 \x90`\0\x92[\x84\x84\x10b\0\x18\x19WPPP\x91\x84RPP\x84\x82\x01Rb\0\x18\x05\x90\x83b\0\x19\x1EV[b\0\x18\x12W8\x80b\0\x14hV[PPPPPV[\x8A\x8D\x83Q\x90b\0\x18)\x82b\0\x0E\xE6V[\x85T\x82R`\x01\x80`\xA0\x1B\x03`\x01\x87\x01T\x16\x83\x83\x01R`\x02\x86\x01\x85Q\x91`\0\x91\x80Tb\0\x18U\x81b\0\x11DV[\x80\x86R\x92`\x01\x82\x16\x91\x82\x15b\0\x18\xDFWPP`\x01\x14b\0\x18\x9BW[PP\x91\x81b\0\x18\x88`\x01\x96\x93`\x03\x96\x95\x03\x82b\0\x0F\x1EV[\x86\x82\x01R\x81R\x01\x93\x01\x93\x01\x92\x91b\0\x17\xE5V[`\0\x90\x81R\x85\x81 \x90\x92P[\x81\x83\x10b\0\x18\xC0WPP\x81\x01\x83\x01\x81b\0\x18\x88b\0\x18pV[\x80`\x01\x91\x96\x92\x93\x94\x95\x96T\x83\x86\x88\x01\x01R\x01\x92\x01\x90\x8F\x94\x93\x92b\0\x18\xA7V[`\xFF\x19\x16\x86\x89\x01R\x92\x15\x15\x90\x92\x1B\x84\x01\x86\x01\x92P\x83\x91Pb\0\x18\x88\x90Pb\0\x18pV[\x81Qc7F\xBE%`\xE1\x1B\x81R`\x04\x90\xFD[PPPPPPPPPV[` \x80\x82\x01Q\x83\x82\x01Q\x91\x92\x91`\x01`\x01`@\x1B\x03\x91\x82\x16\x91\x16\x03b\0\x19\xD0Wb\0\x19I\x81b\0\x19\xD8V[b\0\x19T\x84b\0\x19\xD8V[\x03b\0\x19\xD0WQ\x80Q\x83QQ\x03b\0\x19\xD0Wb\0\x19\xC9b\0\x19\xBC\x91`@Q\x90\x81b\0\x19\x8A\x86\x82\x01\x92\x87\x84R`@\x83\x01\x90b\0\x10\xC1V[\x03\x91b\0\x19\xA0`\x1F\x19\x93\x84\x81\x01\x83R\x82b\0\x0F\x1EV[Q\x90 \x94Q`@Q\x93\x84\x91\x86\x83\x01\x96\x87R`@\x83\x01\x90b\0\x10\xC1V[\x03\x90\x81\x01\x83R\x82b\0\x0F\x1EV[Q\x90 \x14\x90V[PPP`\0\x90V[\x80QQ\x90`\0\x91\x82\x91[\x81\x83\x10b\0\x19\xF0WPPP\x90V[\x90\x91\x92b\0\x1A\0\x84\x83Qb\0\x0F\xADV[QQ\x81\x01\x80\x91\x11b\0\x08'W\x92`\x01\x01\x91\x90b\0\x19\xE2V\xFE`\x80`@R6\x15`\x87W`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`oWP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[=\x90\xFD[`$\x90`@Q\x90c\n\x82\xDDs`\xE3\x1B\x82R`\x04\x82\x01R\xFD[`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`\xE9WP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[c\n\x82\xDDs`\xE3\x1B`\x80R`\x84R`$`\x80\xFD\xFE\xA2dipfsX\"\x12 \xE5\x9C&0\x96>\x94\x9B\xE3\x85w}\xE1a\x1B\x924\x86\x90\x1F\xDB\xF3\x97\n\xC61g\xB4\x9C\xDB\xAE*dsolcC\0\x08\x13\x003\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2";
    /// The bytecode of the contract.
    pub static GATEWAYDIAMOND_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __BYTECODE,
    );
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R6\x15`\x87W`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`oWP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[=\x90\xFD[`$\x90`@Q\x90c\n\x82\xDDs`\xE3\x1B\x82R`\x04\x82\x01R\xFD[`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`\xE9WP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[c\n\x82\xDDs`\xE3\x1B`\x80R`\x84R`$`\x80\xFD\xFE\xA2dipfsX\"\x12 \xE5\x9C&0\x96>\x94\x9B\xE3\x85w}\xE1a\x1B\x924\x86\x90\x1F\xDB\xF3\x97\n\xC61g\xB4\x9C\xDB\xAE*dsolcC\0\x08\x13\x003";
    /// The deployed bytecode of the contract.
    pub static GATEWAYDIAMOND_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __DEPLOYED_BYTECODE,
    );
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
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    GATEWAYDIAMOND_ABI.clone(),
                    client,
                ),
            )
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
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for GatewayDiamond<M> {
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
        Hash
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
        Hash
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
        Hash
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
        Hash
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
        Hash
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
        Hash
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
        Hash
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
        Hash
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
        Hash
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
        Hash
    )]
    #[etherror(name = "IncorrectFacetCutAction", abi = "IncorrectFacetCutAction(uint8)")]
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
        Hash
    )]
    #[etherror(
        name = "InitializationFunctionReverted",
        abi = "InitializationFunctionReverted(address,bytes)"
    )]
    pub struct InitializationFunctionReverted {
        pub initialization_contract_address: ::ethers::core::types::Address,
        pub calldata: ::ethers::core::types::Bytes,
    }
    ///Custom Error type `InvalidCollateral` with signature `InvalidCollateral()` and selector `0xd1ef4cea`
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
    #[etherror(name = "InvalidCollateral", abi = "InvalidCollateral()")]
    pub struct InvalidCollateral;
    ///Custom Error type `InvalidMajorityPercentage` with signature `InvalidMajorityPercentage()` and selector `0x75c3b427`
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
    #[etherror(name = "InvalidMajorityPercentage", abi = "InvalidMajorityPercentage()")]
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
        Hash
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
        Hash
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
        Hash
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
        Hash
    )]
    #[etherror(name = "OldConfigurationNumber", abi = "OldConfigurationNumber()")]
    pub struct OldConfigurationNumber;
    ///Custom Error type `RemoveFacetAddressMustBeZeroAddress` with signature `RemoveFacetAddressMustBeZeroAddress(address)` and selector `0xd091bc81`
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
        name = "RemoveFacetAddressMustBeZeroAddress",
        abi = "RemoveFacetAddressMustBeZeroAddress(address)"
    )]
    pub struct RemoveFacetAddressMustBeZeroAddress {
        pub facet_address: ::ethers::core::types::Address,
    }
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum GatewayDiamondErrors {
        CannotAddFunctionToDiamondThatAlreadyExists(
            CannotAddFunctionToDiamondThatAlreadyExists,
        ),
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
        InvalidCollateral(InvalidCollateral),
        InvalidMajorityPercentage(InvalidMajorityPercentage),
        InvalidSubmissionPeriod(InvalidSubmissionPeriod),
        NoBytecodeAtAddress(NoBytecodeAtAddress),
        NoSelectorsProvidedForFacetForCut(NoSelectorsProvidedForFacetForCut),
        OldConfigurationNumber(OldConfigurationNumber),
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
            if let Ok(decoded) = <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded) = <CannotAddFunctionToDiamondThatAlreadyExists as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CannotAddFunctionToDiamondThatAlreadyExists(decoded));
            }
            if let Ok(decoded) = <CannotAddSelectorsToZeroAddress as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CannotAddSelectorsToZeroAddress(decoded));
            }
            if let Ok(decoded) = <CannotRemoveFunctionThatDoesNotExist as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CannotRemoveFunctionThatDoesNotExist(decoded));
            }
            if let Ok(decoded) = <CannotRemoveImmutableFunction as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CannotRemoveImmutableFunction(decoded));
            }
            if let Ok(decoded) = <CannotReplaceFunctionThatDoesNotExists as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
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
            if let Ok(decoded) = <CannotReplaceImmutableFunction as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CannotReplaceImmutableFunction(decoded));
            }
            if let Ok(decoded) = <FunctionNotFound as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::FunctionNotFound(decoded));
            }
            if let Ok(decoded) = <IncorrectFacetCutAction as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::IncorrectFacetCutAction(decoded));
            }
            if let Ok(decoded) = <InitializationFunctionReverted as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InitializationFunctionReverted(decoded));
            }
            if let Ok(decoded) = <InvalidCollateral as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidCollateral(decoded));
            }
            if let Ok(decoded) = <InvalidMajorityPercentage as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidMajorityPercentage(decoded));
            }
            if let Ok(decoded) = <InvalidSubmissionPeriod as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidSubmissionPeriod(decoded));
            }
            if let Ok(decoded) = <NoBytecodeAtAddress as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NoBytecodeAtAddress(decoded));
            }
            if let Ok(decoded) = <NoSelectorsProvidedForFacetForCut as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NoSelectorsProvidedForFacetForCut(decoded));
            }
            if let Ok(decoded) = <OldConfigurationNumber as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OldConfigurationNumber(decoded));
            }
            if let Ok(decoded) = <RemoveFacetAddressMustBeZeroAddress as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
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
                Self::CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(
                    element,
                ) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::CannotReplaceFunctionsFromFacetWithZeroAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotReplaceImmutableFunction(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FunctionNotFound(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::IncorrectFacetCutAction(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitializationFunctionReverted(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidCollateral(element) => {
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
                    == <InvalidCollateral as ::ethers::contract::EthError>::selector() => {
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
                Self::CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(
                    element,
                ) => ::core::fmt::Display::fmt(element, f),
                Self::CannotReplaceFunctionsFromFacetWithZeroAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotReplaceImmutableFunction(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::FunctionNotFound(element) => ::core::fmt::Display::fmt(element, f),
                Self::IncorrectFacetCutAction(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitializationFunctionReverted(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidCollateral(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidMajorityPercentage(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidSubmissionPeriod(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NoBytecodeAtAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NoSelectorsProvidedForFacetForCut(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OldConfigurationNumber(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
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
    impl ::core::convert::From<CannotAddFunctionToDiamondThatAlreadyExists>
    for GatewayDiamondErrors {
        fn from(value: CannotAddFunctionToDiamondThatAlreadyExists) -> Self {
            Self::CannotAddFunctionToDiamondThatAlreadyExists(value)
        }
    }
    impl ::core::convert::From<CannotAddSelectorsToZeroAddress>
    for GatewayDiamondErrors {
        fn from(value: CannotAddSelectorsToZeroAddress) -> Self {
            Self::CannotAddSelectorsToZeroAddress(value)
        }
    }
    impl ::core::convert::From<CannotRemoveFunctionThatDoesNotExist>
    for GatewayDiamondErrors {
        fn from(value: CannotRemoveFunctionThatDoesNotExist) -> Self {
            Self::CannotRemoveFunctionThatDoesNotExist(value)
        }
    }
    impl ::core::convert::From<CannotRemoveImmutableFunction> for GatewayDiamondErrors {
        fn from(value: CannotRemoveImmutableFunction) -> Self {
            Self::CannotRemoveImmutableFunction(value)
        }
    }
    impl ::core::convert::From<CannotReplaceFunctionThatDoesNotExists>
    for GatewayDiamondErrors {
        fn from(value: CannotReplaceFunctionThatDoesNotExists) -> Self {
            Self::CannotReplaceFunctionThatDoesNotExists(value)
        }
    }
    impl ::core::convert::From<CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet>
    for GatewayDiamondErrors {
        fn from(
            value: CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet,
        ) -> Self {
            Self::CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(value)
        }
    }
    impl ::core::convert::From<CannotReplaceFunctionsFromFacetWithZeroAddress>
    for GatewayDiamondErrors {
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
    impl ::core::convert::From<InvalidCollateral> for GatewayDiamondErrors {
        fn from(value: InvalidCollateral) -> Self {
            Self::InvalidCollateral(value)
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
    impl ::core::convert::From<NoSelectorsProvidedForFacetForCut>
    for GatewayDiamondErrors {
        fn from(value: NoSelectorsProvidedForFacetForCut) -> Self {
            Self::NoSelectorsProvidedForFacetForCut(value)
        }
    }
    impl ::core::convert::From<OldConfigurationNumber> for GatewayDiamondErrors {
        fn from(value: OldConfigurationNumber) -> Self {
            Self::OldConfigurationNumber(value)
        }
    }
    impl ::core::convert::From<RemoveFacetAddressMustBeZeroAddress>
    for GatewayDiamondErrors {
        fn from(value: RemoveFacetAddressMustBeZeroAddress) -> Self {
            Self::RemoveFacetAddressMustBeZeroAddress(value)
        }
    }
}
