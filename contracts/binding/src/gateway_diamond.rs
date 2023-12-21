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
    const __BYTECODE: &[u8] = b"`\x80`@R4b\0\x0C\x82Wb\0\x1BM\x808\x03\x80\x91b\0\0 \x82`\x80b\0\x0E\xDFV[`\x809`@\x81\x12b\0\x0C\x82W`\x80Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0C\x82W`\x80\x82\x01`\x9F\x82\x01\x12\x15b\0\x0C\x82W\x80`\x80\x01Q\x90b\0\0^\x82b\0\x0F\x03V[\x91b\0\0n`@Q\x93\x84b\0\x0E\xDFV[\x80\x83R` \x83\x01\x80\x92\x85`\x80\x01` \x84`\x05\x1B\x83`\x80\x01\x01\x01\x11b\0\x0C\x82W`\xA0\x81\x01\x91[`\xA0`\x05\x85\x90\x1B\x83\x01\x01\x83\x10b\0\r\x9DWPP`\xA0Q\x91PP`\x01`\x01`@\x1B\x03\x81\x11b\0\x0C\x82W`\xC0\x81\x85\x03\x12b\0\x0C\x82W`@Q\x93`\xC0\x85\x01`\x01`\x01`@\x1B\x03\x81\x11\x86\x82\x10\x17b\0\x05\x87W`@\x81\x90R`\x80\x83\x01Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0C\x82W\x83`\x80\x01\x01`@\x81\x84`\x80\x01\x03\x12b\0\x0C\x82Wb\0\x01\x18\x82b\0\x0E\xC3V[\x80Q`\x01`\x01`@\x1B\x03\x81\x16\x81\x03b\0\x0C\x82W\x82R` \x81\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0C\x82W\x01\x82`\x80\x01`\x1F\x82\x01\x12\x15b\0\x0C\x82W\x80Q\x90b\0\x01`\x82b\0\x0F\x03V[\x91b\0\x01p`@Q\x93\x84b\0\x0E\xDFV[\x80\x83R` \x80\x84\x01\x91`\x05\x1B\x83\x01\x01\x91\x85`\x80\x01\x83\x11b\0\x0C\x82W` \x01\x90[\x82\x82\x10b\0\r\x82WPPP`\xE0\x87\x81\x01\x91\x90\x91R\x90\x86R`\xA0\x83\x01Q` \x87\x01R`\xC0\x83\x01Q`@\x87\x01R\x82\x01Q`\xFF\x81\x16\x81\x03b\0\x0C\x82W``\x86\x01Ra\x01\0\x82\x01Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0C\x82W`\x80\x82\x01`\x9F\x84\x83\x01\x01\x12\x15b\0\x0C\x82W\x80\x83`\x80\x01\x01Qb\0\x02\x06\x81b\0\x0F\x03V[\x92b\0\x02\x16`@Q\x94\x85b\0\x0E\xDFV[\x81\x84R` \x84\x01\x92`\x80\x82\x01`\xA0\x87\x83\x01`\x05\x86\x90\x1B\x01\x01\x11b\0\x0C\x82W`\xA0\x86\x82\x01\x01\x93[`\xA0\x87\x83\x01`\x05\x86\x90\x1B\x01\x01\x85\x10b\0\x0C\x87W\x89\x89\x89`\xA0\x8A\x8A`\x80\x86\x01R`\x80\x01\x01Qa\xFF\xFF\x81\x16\x81\x03b\0\x0C\x82W`\xA0\x84\x01R` \x83\x01Q\x15b\0\x0CpW`\xFF``\x84\x01Q\x16`3\x81\x10\x90\x81\x15b\0\x0CdW[Pb\0\x0CRW\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD5\x80T`\x01`\x01`\xA0\x1B\x03\x19\x163\x17\x90U`@Q`\x01`\x01`@\x1B\x03` \x82\x01\x90\x81\x11\x90\x82\x11\x17b\0\x05\x87W` \x81\x01`@R`\0\x81R\x82Q`\0[\x81\x81\x10b\0\x05\xF7WPP`@Q\x92``\x84\x01\x90``\x85RQ\x80\x91R`\x80\x84\x01\x90`\x80\x81`\x05\x1B\x86\x01\x01\x93\x91`\0\x90[\x82\x82\x10b\0\x05\x9DW\x87\x7F\x8F\xAAp\x87\x86q\xCC\xD2\x12\xD2\x07q\xB7\x95\xC5\n\xF8\xFD?\xF6\xCF'\xF4\xBD\xE5~]M\xE0\xAE\xB6s\x88\x80b\0\x03i\x8A\x8A`\0` \x85\x01R\x83\x82\x03`@\x85\x01Rb\0\x0F\xE7V[\x03\x90\xA1\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD4` \x90\x81R\x7F}\xFDH\xDD\x9D\xEF\0/\xA9\xB4\xA0[\xD6\xB7&\xA6\xC3\x13\xC3b\xD3\xF3\xE8A=zu \xF0\t\r%\x80T`\xFF\x19\x90\x81\x16`\x01\x90\x81\x17\x90\x92U\x7FM\x7FL\x8A/\xB5\xB3\\\xA3\xC2w\xC98\x88\xB4\x7F\x0F\")\xBD\xCC\xCFfPM\x1B\xA4\x8E\x88\xB8\x81d\x80T\x82\x16\x83\x17\x90UcH\xE2\xB0\x93`\xE0\x1B`\0R\x7FY\xBAM\xB4\xA2\x13\xE8\x16\x1D\xE5\x97\xB8\xC1\r\xB0\xE7\xE7\xBAZ\xCE\\&\x8E67\x9E$\x9Am-B\xC9\x80T\x82\x16\x83\x17\x90U`+\x80Td\xFF\xFF\xFF\xFF\xFF\x19\x16`\x02\x17\x90U\x83Q\x80Q`\x19\x80T`\x01`\x01`@\x1B\x03\x19\x90\x81\x16`\x01`\x01`@\x1B\x03\x93\x84\x16\x17\x90\x91U\x91\x90\x94\x01Q\x80Q\x93\x95\x94\x91\x93\x91\x82\x11b\0\x05\x87Wh\x01\0\0\0\0\0\0\0\0\x82\x11b\0\x05\x87W` \x90`\x1AT\x83`\x1AU\x80\x84\x10b\0\x05fW[P\x01`\x1A`\0R` `\0 `\0[\x83\x81\x10b\0\x05IWb\0\x059`\x80\x88`\n\x89\x8C\x8A` \x85\x01Q`\x1DU`@\x85\x01Q`\x1BU`\xFF``\x86\x01Q\x16\x90`\x1CT\x16\x17`\x1CU\x80`\x0BU`\x12U`d`\x1EU`\x1FT\x16\x17`\x1FU`\xA0\x81\x01Qb\xFF\xFF\0` T\x91`\x08\x1B\x16\x90b\xFF\xFF\0\x19\x16\x17` Uh\x01\0\0\0\0\0\0\0\x01`\x01\x80\x84\x1B\x03\x19`)T\x16\x17`)U\x01Q`@Q\x90b\0\x05*\x82b\0\x0E\xC3V[\x81R`\0` \x82\x01Rb\0\x13\xBFV[`@Qa\x013\x90\x81b\0\x19\xDA\x829\xF3[\x82Q`\x01`\x01`\xA0\x1B\x03\x16\x81\x83\x01U` \x90\x92\x01\x91\x87\x01b\0\x04\x9BV[b\0\x05\x80\x90`\x1A`\0R\x84\x84`\0 \x91\x82\x01\x91\x01b\0\x0FUV[\x87b\0\x04\x8CV[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[\x90\x91\x92\x94` \x80b\0\x05\xE8`\x01\x93`\x7F\x19\x8B\x82\x03\x01\x86R```@\x8BQ\x87\x80`\xA0\x1B\x03\x81Q\x16\x84Rb\0\x05\xD7\x86\x82\x01Q\x87\x86\x01\x90b\0\x0F\x99V[\x01Q\x91\x81`@\x82\x01R\x01\x90b\0\x0F\xA7V[\x97\x01\x92\x01\x92\x01\x90\x92\x91b\0\x03\"V[`@b\0\x06\x05\x82\x87b\0\x0FnV[Q\x01Q`\x01`\x01`\xA0\x1B\x03b\0\x06\x1C\x83\x88b\0\x0FnV[QQ\x16\x90\x80Q\x15b\0\x0C9W` b\0\x066\x84\x89b\0\x0FnV[Q\x01Q`\x03\x81\x10\x15b\0\x0C#W\x80b\0\x08`WP\x81\x15b\0\x085Wa\xFF\xFF`\0\x80Q` b\0\x1B\r\x839\x81Q\x91RT\x16b\0\x06\xB3`@Qb\0\x06x\x81b\0\x0E\xA7V[`!\x81R\x7FdiamondCut: Add facet has no cod` \x82\x01R`e`\xF8\x1B`@\x82\x01R\x84b\0\x10@V[\x81Q\x91`\0\x91[\x83\x83\x10b\0\x06\xD2WPPPPP`\x01\x90[\x01b\0\x02\xF3V[`\x01`\x01`\xE0\x1B\x03\x19b\0\x06\xE7\x84\x84b\0\x0FnV[Q\x16`\0\x81\x81R`\0\x80Q` b\0\x1B-\x839\x81Q\x91R` R`@\x90 T\x90\x91\x90`\x01`\x01`\xA0\x1B\x03\x16b\0\x08\x1CWb\0\x07\x8A`@Qb\0\x07)\x81b\0\x0E\xC3V[\x87\x81Ra\xFF\xFF\x92\x90\x92\x16` \x80\x84\x01\x82\x81R`\0\x86\x81R`\0\x80Q` b\0\x1B-\x839\x81Q\x91R\x90\x92R`@\x90\x91 \x93Q\x84T\x91Q`\x01`\x01`\xB0\x1B\x03\x19\x90\x92\x16`\x01`\x01`\xA0\x1B\x03\x91\x90\x91\x16\x17`\xA0\x91\x90\x91\x1Ba\xFF\xFF`\xA0\x1B\x16\x17\x90\x92UV[`\0\x80Q` b\0\x1B\r\x839\x81Q\x91RT\x91h\x01\0\0\0\0\0\0\0\0\x83\x10\x15b\0\x05\x87Wb\0\x07\xF1\x90b\0\x07\xD4`\x01\x94\x85\x81\x01`\0\x80Q` b\0\x1B\r\x839\x81Q\x91RUb\0\x10\x0EV[\x90\x91\x90c\xFF\xFF\xFF\xFF\x83T\x91`\x03\x1B\x92`\xE0\x1C\x83\x1B\x92\x1B\x19\x16\x17\x90UV[a\xFF\xFF\x81\x14b\0\x08\x06W\x81\x01\x92\x01\x91b\0\x06\xBAV[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[`@Qc\xEB\xBF]\x07`\xE0\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qc\x02\xB8\xDA\x07`\xE2\x1B\x81R` `\x04\x82\x01R\x90\x81\x90b\0\x08\\\x90`$\x83\x01\x90b\0\x0F\xA7V[\x03\x90\xFD[`\x01\x81\x03b\0\t\xD9WP\x81\x15b\0\t\xB2Wb\0\x08\xC5`@Qb\0\x08\x83\x81b\0\x0E\xA7V[`(\x81R\x7FLibDiamondCut: Replace facet has` \x82\x01Rg no code`\xC0\x1B`@\x82\x01R\x83b\0\x10@V[\x80Q\x90`\0[\x82\x81\x10b\0\x08\xE0WPPPP`\x01\x90b\0\x06\xCBV[`\x01`\x01`\xE0\x1B\x03\x19b\0\x08\xF5\x82\x84b\0\x0FnV[Q\x16`\0\x81\x81R`\0\x80Q` b\0\x1B-\x839\x81Q\x91R` R`@\x90 T`\x01`\x01`\xA0\x1B\x03\x160\x81\x14b\0\t\x99W\x85\x81\x14b\0\t\x80W\x15b\0\thW`\0\x90\x81R`\0\x80Q` b\0\x1B-\x839\x81Q\x91R` R`@\x90 \x80T`\x01`\x01`\xA0\x1B\x03\x19\x16\x85\x17\x90U`\x01\x01b\0\x08\xCBV[`$\x90`@Q\x90cty\xF99`\xE0\x1B\x82R`\x04\x82\x01R\xFD[`@Qc\x1A\xC6\xCE\x8D`\xE1\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qc)\x01\x80m`\xE1\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qc\xCD\x98\xA9o`\xE0\x1B\x81R` `\x04\x82\x01R\x90\x81\x90b\0\x08\\\x90`$\x83\x01\x90b\0\x0F\xA7V[`\x02\x81\x03b\0\x0C\x01WP`\0\x80Q` b\0\x1B\r\x839\x81Q\x91RT\x91\x80b\0\x0B\xE9WP\x80Q\x90`\0[\x82\x81\x10b\0\n\x17WPPPP`\x01\x90b\0\x06\xCBV[`\x01`\x01`\xE0\x1B\x03\x19b\0\n,\x82\x84b\0\x0FnV[Q\x16\x90\x81`\0R`\0\x80Q` b\0\x1B-\x839\x81Q\x91R` R`@`\0 \x94`@Q\x95b\0\n[\x87b\0\x0E\xC3V[T`\x01`\x01`\xA0\x1B\x03\x81\x16\x80\x88R`\xA0\x91\x90\x91\x1Ca\xFF\xFF\x16` \x88\x01R\x15b\0\x0B\xD0W\x85Q`\x01`\x01`\xA0\x1B\x03\x160\x14b\0\x0B\xB7W\x80\x15b\0\x08\x06W`\0\x19\x01\x94\x85a\xFF\xFF` \x83\x01Q\x16\x03b\0\x0B4W[P`\0\x80Q` b\0\x1B\r\x839\x81Q\x91RT\x91\x82\x15b\0\x0B\x1EW`\x01\x92`\0\x19\x01b\0\n\xD9\x81b\0\x10\x0EV[c\xFF\xFF\xFF\xFF\x82T\x91`\x03\x1B\x1B\x19\x16\x90U`\0\x80Q` b\0\x1B\r\x839\x81Q\x91RU`\0R`\0\x80Q` b\0\x1B-\x839\x81Q\x91R` R`\0`@\x81 U\x01b\0\n\x02V[cNH{q`\xE0\x1B`\0R`1`\x04R`$`\0\xFD[b\0\x0B\xB0\x90a\xFF\xFF` b\0\x0BI\x89b\0\x10\x0EV[\x90T\x90`\x03\x1B\x1C`\xE0\x1B\x92b\0\x0Bi\x84b\0\x07\xD4\x85\x85\x85\x01Q\x16b\0\x10\x0EV[\x01Q`\x01`\x01`\xE0\x1B\x03\x19\x90\x92\x16`\0\x90\x81R`\0\x80Q` b\0\x1B-\x839\x81Q\x91R` R`@\x90 \x80Ta\xFF\xFF`\xA0\x1B\x19\x16\x91\x90\x92\x16`\xA0\x1Ba\xFF\xFF`\xA0\x1B\x16\x17\x90UV[\x8Bb\0\n\xADV[`@Qc\r\xF5\xFDa`\xE3\x1B\x81R`\x04\x81\x01\x84\x90R`$\x90\xFD[`@Qcz\x08\xA2-`\xE0\x1B\x81R`\x04\x81\x01\x84\x90R`$\x90\xFD[`$\x90`@Q\x90c\xD0\x91\xBC\x81`\xE0\x1B\x82R`\x04\x82\x01R\xFD[`@Qc?\xF4\xD2\x0F`\xE1\x1B\x81R`$\x91b\0\x0C!\x90`\x04\x83\x01\x90b\0\x0F\x99V[\xFD[cNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[`@Qc\xE7g\xF9\x1F`\xE0\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qcu\xC3\xB4'`\xE0\x1B\x81R`\x04\x90\xFD[`d\x91P\x11\x84b\0\x02\x91V[`@Qc1/\x8E\x05`\xE0\x1B\x81R`\x04\x90\xFD[`\0\x80\xFD[\x84Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0C\x82W`\x1F\x19\x91\x90\x88\x84\x01\x01`\x80\x81\x01\x90``\x90\x86\x03\x84\x01\x12b\0\x0C\x82W`@Q\x90b\0\x0C\xC2\x82b\0\x0E\xA7V[` \x81\x01Q\x82Rb\0\x0C\xD7`@\x82\x01b\0\x0F\x1BV[` \x83\x01R``\x81\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0C\x82W\x01\x91\x85`\x80\x01`?\x84\x01\x12\x15b\0\x0C\x82W` \x83\x01Q\x91`\x01`\x01`@\x1B\x03\x83\x11b\0\rmWb\0\r.` `@Q\x96`\x1F\x86\x01\x16\x01\x86b\0\x0E\xDFV[\x82\x85R\x86`\x80\x01`@\x84\x86\x01\x01\x11b\0\x0C\x82W\x84b\0\rZ` \x96\x94\x87\x96`@\x88\x80\x98\x01\x91\x01b\0\x0F0V[`@\x82\x01R\x81R\x01\x95\x01\x94\x90Pb\0\x02<V[`$`\0cNH{q`\xE0\x1B\x81R`A`\x04R\xFD[` \x80\x91b\0\r\x91\x84b\0\x0F\x1BV[\x81R\x01\x91\x01\x90b\0\x01\x90V[\x82Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0C\x82W``\x90\x83\x01`\x80\x81\x01\x90\x89\x03`\x1F\x19\x01\x82\x13b\0\x0C\x82W`@Q\x91b\0\r\xD4\x83b\0\x0E\xA7V[b\0\r\xE2` \x83\x01b\0\x0F\x1BV[\x83R`@\x82\x01Q`\x03\x81\x10\x15b\0\x0C\x82W` \x84\x01R\x81\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0C\x82W\x01\x88`\x80\x01`?\x82\x01\x12\x15b\0\x0C\x82W` \x81\x01Q\x90b\0\x0E,\x82b\0\x0F\x03V[\x91b\0\x0E<`@Q\x93\x84b\0\x0E\xDFV[\x80\x83R` \x83\x01\x8B`\x80\x01`@\x83`\x05\x1B\x85\x01\x01\x11b\0\x0C\x82W`@\x83\x01\x90[`@\x83`\x05\x1B\x85\x01\x01\x82\x10b\0\x0E\x84WPPPP`@\x82\x01R\x81R` \x92\x83\x01\x92\x01b\0\0\x93V[\x81Q`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x81\x03b\0\x0C\x82W\x81R` \x91\x82\x01\x91\x01b\0\x0E\\V[``\x81\x01\x90\x81\x10`\x01`\x01`@\x1B\x03\x82\x11\x17b\0\x05\x87W`@RV[`@\x81\x01\x90\x81\x10`\x01`\x01`@\x1B\x03\x82\x11\x17b\0\x05\x87W`@RV[`\x1F\x90\x91\x01`\x1F\x19\x16\x81\x01\x90`\x01`\x01`@\x1B\x03\x82\x11\x90\x82\x10\x17b\0\x05\x87W`@RV[`\x01`\x01`@\x1B\x03\x81\x11b\0\x05\x87W`\x05\x1B` \x01\x90V[Q\x90`\x01`\x01`\xA0\x1B\x03\x82\x16\x82\x03b\0\x0C\x82WV[`\0[\x83\x81\x10b\0\x0FDWPP`\0\x91\x01RV[\x81\x81\x01Q\x83\x82\x01R` \x01b\0\x0F3V[\x81\x81\x10b\0\x0FaWPPV[`\0\x81U`\x01\x01b\0\x0FUV[\x80Q\x82\x10\x15b\0\x0F\x83W` \x91`\x05\x1B\x01\x01\x90V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[\x90`\x03\x82\x10\x15b\0\x0C#WRV[\x90\x81Q\x80\x82R` \x80\x80\x93\x01\x93\x01\x91`\0[\x82\x81\x10b\0\x0F\xC8WPPPP\x90V[\x83Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x93\x81\x01\x93\x92\x81\x01\x92`\x01\x01b\0\x0F\xB9V[\x90` \x91b\0\x10\x02\x81Q\x80\x92\x81\x85R\x85\x80\x86\x01\x91\x01b\0\x0F0V[`\x1F\x01`\x1F\x19\x16\x01\x01\x90V[\x90`\0\x80Q` b\0\x1B\r\x839\x81Q\x91R\x80T\x83\x10\x15b\0\x0F\x83W`\0R`\x1C` `\0 \x83`\x03\x1C\x01\x92`\x02\x1B\x16\x90V[\x80;\x15b\0\x10LWPPV[`@\x80Qc\x91\x984\xB9`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x90\x92\x16`\x04\x83\x01R`$\x82\x01R\x90\x81\x90b\0\x08\\\x90`D\x83\x01\x90b\0\x0F\xE7V[\x90\x80\x82Q\x90\x81\x81R` \x80\x91\x01\x92\x81\x80\x84`\x05\x1B\x83\x01\x01\x95\x01\x93`\0\x91[\x84\x83\x10b\0\x10\xB1WPPPPPP\x90V[\x90\x91\x92\x93\x94\x95\x84\x80b\0\x10\xF4`\x01\x93`\x1F\x19\x86\x82\x03\x01\x87R\x8AQ\x80Q\x82R\x85\x80`\xA0\x1B\x03\x84\x82\x01Q\x16\x84\x83\x01R`@\x80\x91\x01Q\x91``\x80\x92\x82\x01R\x01\x90b\0\x0F\xE7V[\x98\x01\x93\x01\x93\x01\x91\x94\x93\x92\x90b\0\x10\xA0V[\x90`\x01\x82\x81\x1C\x92\x16\x80\x15b\0\x117W[` \x83\x10\x14b\0\x11!WV[cNH{q`\xE0\x1B`\0R`\"`\x04R`$`\0\xFD[\x91`\x7F\x16\x91b\0\x11\x15V[\x91\x90`\x1F\x81\x11b\0\x11RWPPPV[b\0\x11\x81\x92`\0R` `\0 \x90` `\x1F\x84\x01`\x05\x1C\x83\x01\x93\x10b\0\x11\x83W[`\x1F\x01`\x05\x1C\x01\x90b\0\x0FUV[V[\x90\x91P\x81\x90b\0\x11sV[\x90\x80\x82\x14b\0\x12wWb\0\x11\xA3\x81Tb\0\x11\x05V[\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x05\x87W\x81\x90b\0\x11\xCD\x82b\0\x11\xC6\x86Tb\0\x11\x05V[\x86b\0\x11BV[`\0\x90`\x1F\x83\x11`\x01\x14b\0\x12\x07W`\0\x92b\0\x11\xFBW[PP\x81`\x01\x1B\x91`\0\x19\x90`\x03\x1B\x1C\x19\x16\x17\x90UV[\x01T\x90P8\x80b\0\x11\xE5V[\x81R` \x80\x82 \x85\x83R\x81\x83 \x93P\x90`\x1F\x19\x85\x16\x90\x83\x90[\x82\x82\x10b\0\x12]WPP\x90\x84`\x01\x95\x94\x93\x92\x10b\0\x12CW[PPP\x81\x1B\x01\x90UV[\x01T`\0\x19`\xF8\x84`\x03\x1B\x16\x1C\x19\x16\x90U8\x80\x80b\0\x129V[\x84\x95\x81\x92\x95\x85\x01T\x81U`\x01\x80\x91\x01\x96\x01\x94\x01\x90b\0\x12 V[PPV[`\x04T\x81\x10\x15b\0\x0F\x83W`\x04`\0R`\x03` `\0 \x91\x02\x01\x90`\0\x90V[\x90b\0\x13\xA9W\x81Q\x81U` \x80\x83\x01Q`\x01\x80\x84\x01\x80T`\x01`\x01`\xA0\x1B\x03\x19\x16`\x01`\x01`\xA0\x1B\x03\x93\x90\x93\x16\x92\x90\x92\x17\x90\x91U`@\x90\x93\x01Q\x80Q`\x02\x90\x93\x01\x93\x92\x91\x90`\x01`\x01`@\x1B\x03\x83\x11b\0\x05\x87Wb\0\x13\x07\x83b\0\x13\0\x87Tb\0\x11\x05V[\x87b\0\x11BV[\x81`\x1F\x84\x11`\x01\x14b\0\x13CWP\x92\x82\x93\x91\x83\x92`\0\x94b\0\x137W[PP\x1B\x91`\0\x19\x90`\x03\x1B\x1C\x19\x16\x17\x90UV[\x01Q\x92P8\x80b\0\x13$V[\x91\x90\x83`\x1F\x19\x81\x16\x87`\0R\x84`\0 \x94`\0\x90[\x88\x83\x83\x10b\0\x13\x8EWPPP\x10b\0\x13tWPPP\x81\x1B\x01\x90UV[\x01Q`\0\x19`\xF8\x84`\x03\x1B\x16\x1C\x19\x16\x90U8\x80\x80b\0\x129V[\x85\x87\x01Q\x88U\x90\x96\x01\x95\x94\x85\x01\x94\x87\x93P\x90\x81\x01\x90b\0\x13XV[cNH{q`\xE0\x1B`\0R`\0`\x04R`$`\0\xFD[`@\x90\x81Q\x90` \x90\x81\x83Rb\0\x13\xE1\x81Q\x85\x84\x86\x01R``\x85\x01\x90b\0\x10\x82V[\x92\x7F~\xCD\xACH#4\xC3o\xCC\xBE7C\x18\xCF\xE7N\xA0\xC8\x18\x13\x94\x89\r\xDE\xC8\x94\xA1\x0F\x0F\xCCt\x81\x83\x83\x01\x91\x80`\x01\x80`@\x1B\x03\x96\x87\x85Q\x16\x89\x83\x01R\x03\x90\xA1`\x05\x94\x84\x86T\x16\x80b\0\x17NW[PP`\x04\x91\x82T\x91h\x01\0\0\0\0\0\0\0\0\x95\x86\x84\x11b\0\x179W`\x06T\x84`\x06U\x80\x85\x10b\0\x16OW[P`\0\x93\x85\x85R\x86\x85 `\x06\x86R\x87\x86 \x90\x86\x90[\x83\x82\x10b\0\x15\xF9WPPPP\x80\x88T\x16\x91`\x01\x80`@\x1B\x03\x19\x92\x83`\x07T\x16\x17`\x07U\x83QQ\x93\x86T\x98\x86[\x86\x81\x10b\0\x15wWPPPQ\x16\x90\x87T\x16\x17\x86U\x80\x85\x11b\0\x14\xBAW[PPPPPPV[\x84\x81\x10\x15b\0\x14\xB2W\x82T\x80\x15b\0\x15dW`\0\x19\x01\x90b\0\x14\xDC\x82b\0\x12{V[\x92\x90\x92b\0\x15RW\x83\x83U\x83`\x02`\x01\x94\x82\x86\x82\x01U\x01b\0\x14\xFF\x81Tb\0\x11\x05V[\x80b\0\x15\x12W[PPP\x84U\x01b\0\x14\xBAV[\x82`\x1F\x80\x83\x11`\x01\x14b\0\x15.WPPPU[\x838\x80b\0\x15\x06V[\x83\x82R\x8A\x82 \x93\x91\x92b\0\x15J\x91\x01\x8D\x1C\x84\x01\x88\x85\x01b\0\x0FUV[UUb\0\x15%V[cNH{q`\xE0\x1B\x84R\x83\x85R`$\x84\xFD[cNH{q`\xE0\x1B\x83R`1\x84R`$\x83\xFD[\x8A\x81\x10\x15b\0\x15\xAEW\x80b\0\x15\xA7b\0\x15\x94`\x01\x93\x86Qb\0\x0FnV[Qb\0\x15\xA0\x83b\0\x12{V[\x90b\0\x12\x9BV[\x01b\0\x14\x95V[b\0\x15\xBB\x81\x84Qb\0\x0FnV[Q\x89T\x83\x81\x10\x15b\0\x15\xE6W`\x01\x92\x91b\0\x15\xA0\x82\x8D\x86b\0\x15\xE0\x95\x01\x90Ub\0\x12{V[b\0\x15\xA7V[cNH{q`\xE0\x1B\x8AR`A\x8BR`$\x8A\xFD[\x80`\x01\x91\x84\x03b\0\x16\x16W[`\x03\x80\x91\x01\x93\x01\x91\x01\x90\x91b\0\x14iV[\x80T\x84U\x81\x80\x85\x01\x90\x83\x80`\xA0\x1B\x03\x90\x83\x01T\x16\x83\x80`\xA0\x1B\x03\x19\x82T\x16\x17\x90Ub\0\x16I`\x02\x80\x83\x01\x90\x86\x01b\0\x11\x8EV[b\0\x16\x05V[`\x03\x90\x80\x82\x02\x90\x82\x82\x04\x03b\0\x17$W\x85\x82\x02\x82\x81\x04\x87\x03b\0\x17\x0FW\x88`\0\x91`\x06\x83R\x81\x83 \x93\x84\x01\x93\x01[\x8C\x84\x82\x10b\0\x16\x91WPPPPPb\0\x14TV[\x85\x92\x84\x91\x82\x84U`\x01\x83\x81\x86\x01U`\x02\x85\x01\x91b\0\x16\xB0\x83Tb\0\x11\x05V[\x91\x82b\0\x16\xC7W[PPPPPP\x01\x89\x90b\0\x16}V[\x85\x91`\x1F\x80\x85\x11`\x01\x14b\0\x16\xEBWPPPPP\x90PU[\x82\x8B8\x8F\x80\x82b\0\x16\xB8V[\x85\x94\x96\x84\x96b\0\x17\x07\x95R\x86 \x96\x01\x90\x1C\x85\x01\x90\x85\x01b\0\x0FUV[UUb\0\x16\xDFV[`\x11\x88cNH{q`\xE0\x1B`\0RR`$`\0\xFD[`\x11\x87cNH{q`\xE0\x1B`\0RR`$`\0\xFD[`A\x85cNH{q`\xE0\x1B`\0RR`$`\0\xFD[\x85\x83Q\x16\x86`\x07T\x16\x90\x81\x81\x14b\0\x18\xD4W\x10b\0\x18\xC3W\x81Q\x91b\0\x17t\x83b\0\x0E\xC3V[`\x04Tb\0\x17\x82\x81b\0\x0F\x03V[\x91b\0\x17\x91\x81Q\x93\x84b\0\x0E\xDFV[\x81\x83R\x87\x83\x01\x90`\x04`\0R\x88`\0 \x90`\0\x92[\x84\x84\x10b\0\x17\xDAWPPP\x91\x84RPP\x84\x82\x01Rb\0\x17\xC6\x90\x83b\0\x18\xDFV[b\0\x17\xD3W8\x80b\0\x14)V[PPPPPV[\x8A\x8D\x83Q\x90b\0\x17\xEA\x82b\0\x0E\xA7V[\x85T\x82R`\x01\x80`\xA0\x1B\x03`\x01\x87\x01T\x16\x83\x83\x01R`\x02\x86\x01\x85Q\x91`\0\x91\x80Tb\0\x18\x16\x81b\0\x11\x05V[\x80\x86R\x92`\x01\x82\x16\x91\x82\x15b\0\x18\xA0WPP`\x01\x14b\0\x18\\W[PP\x91\x81b\0\x18I`\x01\x96\x93`\x03\x96\x95\x03\x82b\0\x0E\xDFV[\x86\x82\x01R\x81R\x01\x93\x01\x93\x01\x92\x91b\0\x17\xA6V[`\0\x90\x81R\x85\x81 \x90\x92P[\x81\x83\x10b\0\x18\x81WPP\x81\x01\x83\x01\x81b\0\x18Ib\0\x181V[\x80`\x01\x91\x96\x92\x93\x94\x95\x96T\x83\x86\x88\x01\x01R\x01\x92\x01\x90\x8F\x94\x93\x92b\0\x18hV[`\xFF\x19\x16\x86\x89\x01R\x92\x15\x15\x90\x92\x1B\x84\x01\x86\x01\x92P\x83\x91Pb\0\x18I\x90Pb\0\x181V[\x81Qc7F\xBE%`\xE1\x1B\x81R`\x04\x90\xFD[PPPPPPPPPV[` \x80\x82\x01Q\x83\x82\x01Q\x91\x92\x91`\x01`\x01`@\x1B\x03\x91\x82\x16\x91\x16\x03b\0\x19\x91Wb\0\x19\n\x81b\0\x19\x99V[b\0\x19\x15\x84b\0\x19\x99V[\x03b\0\x19\x91WQ\x80Q\x83QQ\x03b\0\x19\x91Wb\0\x19\x8Ab\0\x19}\x91`@Q\x90\x81b\0\x19K\x86\x82\x01\x92\x87\x84R`@\x83\x01\x90b\0\x10\x82V[\x03\x91b\0\x19a`\x1F\x19\x93\x84\x81\x01\x83R\x82b\0\x0E\xDFV[Q\x90 \x94Q`@Q\x93\x84\x91\x86\x83\x01\x96\x87R`@\x83\x01\x90b\0\x10\x82V[\x03\x90\x81\x01\x83R\x82b\0\x0E\xDFV[Q\x90 \x14\x90V[PPP`\0\x90V[\x80QQ\x90`\0\x91\x82\x91[\x81\x83\x10b\0\x19\xB1WPPP\x90V[\x90\x91\x92b\0\x19\xC1\x84\x83Qb\0\x0FnV[QQ\x81\x01\x80\x91\x11b\0\x08\x06W\x92`\x01\x01\x91\x90b\0\x19\xA3V\xFE`\x80`@R6\x15`\x87W`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`oWP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[=\x90\xFD[`$\x90`@Q\x90c\n\x82\xDDs`\xE3\x1B\x82R`\x04\x82\x01R\xFD[`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`\xE9WP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[c\n\x82\xDDs`\xE3\x1B`\x80R`\x84R`$`\x80\xFD\xFE\xA2dipfsX\"\x12 oI<\x837\xDE\xB2\xF8,\xFB\xC0\xD0\xFA\xC6\x7F\xA2W\xB8@\xCEz\xFD-\0\xA5^k6x\x8D\xD4-dsolcC\0\x08\x13\x003\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2";
    /// The bytecode of the contract.
    pub static GATEWAYDIAMOND_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __BYTECODE,
    );
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R6\x15`\x87W`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`oWP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[=\x90\xFD[`$\x90`@Q\x90c\n\x82\xDDs`\xE3\x1B\x82R`\x04\x82\x01R\xFD[`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`\xE9WP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[c\n\x82\xDDs`\xE3\x1B`\x80R`\x84R`$`\x80\xFD\xFE\xA2dipfsX\"\x12 oI<\x837\xDE\xB2\xF8,\xFB\xC0\xD0\xFA\xC6\x7F\xA2W\xB8@\xCEz\xFD-\0\xA5^k6x\x8D\xD4-dsolcC\0\x08\x13\x003";
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
    ///`ConstructorParams((uint64,address[]),uint256,uint256,uint8,(uint256,address,bytes)[],uint16)`
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
    pub struct ConstructorParams {
        pub network_name: SubnetID,
        pub bottom_up_check_period: ::ethers::core::types::U256,
        pub msg_fee: ::ethers::core::types::U256,
        pub majority_percentage: u8,
        pub genesis_validators: ::std::vec::Vec<Validator>,
        pub active_validators_limit: u16,
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
        Hash
    )]
    pub struct FacetCut {
        pub facet_address: ::ethers::core::types::Address,
        pub action: u8,
        pub function_selectors: ::std::vec::Vec<[u8; 4]>,
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
        Hash
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
        Hash
    )]
    pub struct Validator {
        pub weight: ::ethers::core::types::U256,
        pub addr: ::ethers::core::types::Address,
        pub metadata: ::ethers::core::types::Bytes,
    }
}
