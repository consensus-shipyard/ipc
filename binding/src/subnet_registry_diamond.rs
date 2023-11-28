pub use subnet_registry_diamond::*;
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
    pub static SUBNETREGISTRYDIAMOND_ABI: ::ethers::contract::Lazy<
        ::ethers::core::abi::Abi,
    > = ::ethers::contract::Lazy::new(__abi);
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = b"`\x80`@R4a\x0C\xE3Wa\x11\x1E\x808\x03\x80a\0\x19\x81a\r\xCFV[\x92\x839\x81\x01\x90`@\x81\x83\x03\x12a\x0C\xE3W\x80Q`\x01`\x01`@\x1B\x03\x81\x11a\x0C\xE3W\x81\x01\x90\x82`\x1F\x83\x01\x12\x15a\x0C\xE3W\x81Q\x90a\0[a\0V\x83a\r\xF4V[a\r\xCFV[\x92` \x84\x84\x81R\x01` \x81\x94`\x05\x1B\x83\x01\x01\x91\x86\x83\x11a\x0C\xE3W` \x81\x01\x91[\x83\x83\x10a\x0C\xE8WPPPP` \x81\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11a\x0C\xE3W\x01`\xA0\x81\x85\x03\x12a\x0C\xE3W`@Q\x93`\xA0\x85\x01`\x01`\x01`@\x1B\x03\x81\x11\x86\x82\x10\x17a\x05\x1CW`@Ra\0\xCB\x82a\x0E\x0BV[\x85Ra\0\xD9` \x83\x01a\x0E\x0BV[` \x86\x01Ra\0\xEA`@\x83\x01a\x0E\x0BV[`@\x86\x01R``\x82\x01Q`\x01`\x01`@\x1B\x03\x81\x11a\x0C\xE3W\x81a\x01\x0E\x91\x84\x01a\x0E\x1FV[``\x86\x01R`\x80\x82\x01Q`\x01`\x01`@\x1B\x03\x81\x11a\x0C\xE3Wa\x010\x92\x01a\x0E\x1FV[`\x80\x84\x01R\x82Q`\x01`\x01`\xA0\x1B\x03\x16\x15a\x0C\xD1W` \x83\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x15a\x0C\xBFW`@\x83\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x15a\x0C\xBFW\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD5\x80T`\x01`\x01`\xA0\x1B\x03\x19\x163\x17\x90U`@Q` \x81\x01`\x01`\x01`@\x1B\x03\x81\x11\x82\x82\x10\x17a\x05\x1CW`@R`\0\x81R\x82Q`\0[\x81\x81\x10a\x06\x88WPP`@Q\x92``\x84\x01\x90``\x85RQ\x80\x91R`\x80\x84\x01\x90`\x80\x81`\x05\x1B\x86\x01\x01\x93\x91`\0\x90[\x82\x82\x10a\x063W\x87\x7F\x8F\xAAp\x87\x86q\xCC\xD2\x12\xD2\x07q\xB7\x95\xC5\n\xF8\xFD?\xF6\xCF'\xF4\xBD\xE5~]M\xE0\xAE\xB6s\x88\x80a\x023\x8A\x8A`\0` \x85\x01R\x83\x82\x03`@\x85\x01Ra\x0E\xFBV[\x03\x90\xA1\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD4` \x90\x81R\x7F}\xFDH\xDD\x9D\xEF\0/\xA9\xB4\xA0[\xD6\xB7&\xA6\xC3\x13\xC3b\xD3\xF3\xE8A=zu \xF0\t\r%\x80T`\x01`\xFF\x19\x91\x82\x16\x81\x17\x90\x92U\x7FM\x7FL\x8A/\xB5\xB3\\\xA3\xC2w\xC98\x88\xB4\x7F\x0F\")\xBD\xCC\xCFfPM\x1B\xA4\x8E\x88\xB8\x81d\x80T\x82\x16\x83\x17\x90UcH\xE2\xB0\x93`\xE0\x1B`\0\x90\x81R\x7FY\xBAM\xB4\xA2\x13\xE8\x16\x1D\xE5\x97\xB8\xC1\r\xB0\xE7\xE7\xBAZ\xCE\\&\x8E67\x9E$\x9Am-B\xC9\x80T\x90\x92\x16\x83\x17\x90\x91U\x83Q\x81T`\x01`\x01`\xA0\x1B\x03\x91\x82\x16`\x01`\x01`\xA0\x1B\x03\x19\x91\x82\x16\x17\x90\x92U\x92\x84\x01Q\x82T\x90\x84\x16\x90\x82\x16\x17\x82U`@\x84\x01Q`\x02\x80T\x91\x90\x94\x16\x91\x16\x17\x82U``\x83\x01Q\x80Q\x92\x93\x90\x91\x90`\x01`\x01`@\x1B\x03\x84\x11a\x05\x1CWh\x01\0\0\0\0\0\0\0\0\x90\x81\x85\x11a\x05\x1CW`\x03\x93\x84T\x86\x86U\x80\x87\x10a\x05\xD7W[P` \x01\x84`\0R` `\0 \x86\x86\x1C\x90`\0[\x82\x81\x10a\x05\x86WP`\x07\x19\x97\x80\x89\x16\x90\x03\x80a\x052W[PPPP`\x80\x01Q\x80Q\x91`\x01`\x01`@\x1B\x03\x83\x11a\x05\x1CW\x82\x11a\x05\x1CW`\x04T\x82`\x04U\x80\x83\x10a\x04\xA3W[P` \x01`\x04`\0R\x81` `\0 \x94\x1C\x94`\0[\x86\x81\x10a\x04RWP\x82\x16\x80\x83\x03\x92\x03a\x03\xFAW[`@Qa\x013\x90\x81a\x0F\xAB\x829\xF3[\x91\x90`\0\x95`\0\x90[\x82\x82\x10a\x04\x1CWPPPPP\x01U\x80\x80\x80\x80\x80\x80a\x03\xEBV[\x90\x91\x92` a\x04F\x86\x99\x83\x97Q`\xE0\x1C\x90\x86\x86\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x98\x01\x94\x93\x92\x01\x90a\x04\x03V[\x84\x90`\0\x80[\x8A`\x08\x82\x10a\x04mWPP\x81\x88\x01U\x01a\x03\xD7V[a\x04\x96\x90\x87\x94\x95\x93\x83` \x94\x99Q`\xE0\x1C\x92\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x94\x01\x90\x86\x92\x91a\x04XV[`\x04`\0R\x7F\x8A5\xAC\xFB\xC1_\xF8\x1A9\xAE}4O\xD7\t\xF2\x8E\x86\0\xB4\xAA\x8Ce\xC6\xB6K\xFE\x7F\xE3k\xD1\x9B`\x07\x91\x82\x01\x86\x1C\x81\x01\x91\x84\x01\x86\x1C\x01`\x1C\x84\x89\x1B\x16\x80a\x05\x02W[P\x84\x90[\x82\x81\x10a\x04\xF6WPPa\x03\xC2V[`\0\x81U\x01\x84\x90a\x04\xE8V[`\0\x19\x90\x81\x83\x01\x91\x82T\x91` \x03\x89\x1B\x1C\x16\x90U\x88a\x04\xE4V[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[\x92`\0\x93`\0[\x88\x8C\x83\x83\x10a\x05SWPPPPP\x01U`\x80\x87\x80\x80a\x03\x94V[a\x05|\x90\x85\x98\x84` \x94\x95\x96\x97Q`\xE0\x1C\x92\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x96\x01\x92\x91\x01a\x059V[\x86\x90`\0\x80[\x8C`\x08\x82\x10a\x05\xA1WPP\x81\x84\x01U\x01a\x03}V[a\x05\xCA\x90\x88\x94\x95\x93\x83` \x94\x9AQ`\xE0\x1C\x92\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x95\x01\x90\x88\x92\x91a\x05\x8CV[\x85`\0R` `\0 `\x07\x80\x89\x01\x88\x1C\x82\x01\x92\x01\x87\x1C\x01\x90`\x1C\x88\x8A\x1B\x16\x80a\x06\x19W[P\x85\x90[\x82\x81\x10a\x06\rWPPa\x03iV[`\0\x81U\x01\x85\x90a\x05\xFFV[`\0\x19\x90\x81\x83\x01\x91\x82T\x91` \x03\x8A\x1B\x1C\x16\x90U\x89a\x05\xFBV[\x90\x91\x92\x94` \x80a\x06z`\x01\x93`\x7F\x19\x8B\x82\x03\x01\x86R```@\x8BQ\x87\x80`\xA0\x1B\x03\x81Q\x16\x84Ra\x06j\x86\x82\x01Q\x87\x86\x01\x90a\x0E\xB0V[\x01Q\x91\x81`@\x82\x01R\x01\x90a\x0E\xBDV[\x97\x01\x92\x01\x92\x01\x90\x92\x91a\x01\xEFV[`@a\x06\x94\x82\x87a\x0E\x86V[Q\x01Q`\x01`\x01`\xA0\x1B\x03a\x06\xA9\x83\x88a\x0E\x86V[QQ\x16\x90\x80Q\x15a\x0C\xA6W` a\x06\xC0\x84\x89a\x0E\x86V[Q\x01Q`\x03\x81\x10\x15a\x0C\x90W\x80a\x08\xDEWP\x81\x15a\x08\xB5Wa\xFF\xFF`\0\x80Q` a\x10\xDE\x839\x81Q\x91RT\x16a\x071a\x06\xF7a\r\xB0V[`!\x81R\x7FdiamondCut: Add facet has no cod` \x82\x01R`e`\xF8\x1B`@\x82\x01R\x84a\x0FkV[\x81Q\x91`\0\x91[\x83\x83\x10a\x07NWPPPPP`\x01\x90[\x01a\x01\xC1V[a\x07X\x83\x83a\x0E\x86V[Q`\x01`\x01`\xE0\x1B\x03\x19\x81\x16`\0\x90\x81R`\0\x80Q` a\x10\xFE\x839\x81Q\x91R` R`@\x90 T\x90\x91\x90`\x01`\x01`\xA0\x1B\x03\x16a\x08\x93Wa\x08\x06a\x07\x9Ba\r\x91V[\x87\x81Ra\xFF\xFF\x92\x90\x92\x16` \x80\x84\x01\x82\x81R`\x01`\x01`\xE0\x1B\x03\x19\x86\x16`\0\x90\x81R`\0\x80Q` a\x10\xFE\x839\x81Q\x91R\x90\x92R`@\x90\x91 \x93Q\x84T\x91Q`\x01`\x01`\xB0\x1B\x03\x19\x90\x92\x16`\x01`\x01`\xA0\x1B\x03\x91\x90\x91\x16\x17`\xA0\x91\x90\x91\x1Ba\xFF\xFF`\xA0\x1B\x16\x17\x90\x92UV[`\0\x80Q` a\x10\xDE\x839\x81Q\x91RT\x91h\x01\0\0\0\0\0\0\0\0\x83\x10\x15a\x05\x1CWa\x08ha\x08I`\x01\x94\x85\x81\x01`\0\x80Q` a\x10\xDE\x839\x81Q\x91RUa\x0F;V[\x90\x92`\xE0\x1C\x90\x83T\x90`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x90Ua\xFF\xFF\x81\x14a\x08}W\x81\x01\x92\x01\x91a\x078V[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[`@Qc\xEB\xBF]\x07`\xE0\x1B\x81R`\x01`\x01`\xE0\x1B\x03\x19\x83\x16`\x04\x82\x01R`$\x90\xFD[`@Qc\x02\xB8\xDA\x07`\xE2\x1B\x81R` `\x04\x82\x01R\x90\x81\x90a\x08\xDA\x90`$\x83\x01\x90a\x0E\xBDV[\x03\x90\xFD[`\x01\x81\x03a\nAWP\x81\x15a\n\x1CWa\t9a\x08\xF8a\r\xB0V[`(\x81R\x7FLibDiamondCut: Replace facet has` \x82\x01Rg no code`\xC0\x1B`@\x82\x01R\x83a\x0FkV[\x80Q\x90`\0[\x82\x81\x10a\tRWPPPP`\x01\x90a\x07HV[`\x01`\x01`\xE0\x1B\x03\x19a\te\x82\x84a\x0E\x86V[Q\x16`\0\x81\x81R`\0\x80Q` a\x10\xFE\x839\x81Q\x91R` R`@\x90 T`\x01`\x01`\xA0\x1B\x03\x160\x81\x14a\n\x03W\x85\x81\x14a\t\xEAW\x15a\t\xD2W`\0\x90\x81R`\0\x80Q` a\x10\xFE\x839\x81Q\x91R` R`@\x90 \x80T`\x01`\x01`\xA0\x1B\x03\x19\x16\x85\x17\x90U`\x01\x01a\t?V[`$\x90`@Q\x90cty\xF99`\xE0\x1B\x82R`\x04\x82\x01R\xFD[`@Qc\x1A\xC6\xCE\x8D`\xE1\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qc)\x01\x80m`\xE1\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qc\xCD\x98\xA9o`\xE0\x1B\x81R` `\x04\x82\x01R\x90\x81\x90a\x08\xDA\x90`$\x83\x01\x90a\x0E\xBDV[`\x02\x81\x03a\x0CpWP`\0\x80Q` a\x10\xDE\x839\x81Q\x91RT\x91\x80a\x0CXWP\x80Q\x90`\0[\x82\x81\x10a\nzWPPPP`\x01\x90a\x07HV[`\x01`\x01`\xE0\x1B\x03\x19a\n\x8D\x82\x84a\x0E\x86V[Q\x16\x90\x81`\0R`\0\x80Q` a\x10\xFE\x839\x81Q\x91R` R`@`\0 \x94a\n\xB4a\r\x91V[\x95T`\x01`\x01`\xA0\x1B\x03\x81\x16\x80\x88R`\xA0\x91\x90\x91\x1Ca\xFF\xFF\x16` \x88\x01R\x15a\x0C?W\x85Q`\x01`\x01`\xA0\x1B\x03\x160\x14a\x0C&W\x80\x15a\x08}W`\0\x19\x01\x80\x95\x81a\xFF\xFF` \x83\x01Q\x16\x03a\x0B\x85W[PP`\0\x80Q` a\x10\xDE\x839\x81Q\x91RT\x91\x82\x15a\x0BoW`\x01\x92`\0\x19\x01a\x0B-\x81a\x0F;V[c\xFF\xFF\xFF\xFF\x82T\x91`\x03\x1B\x1B\x19\x16\x90U`\0\x80Q` a\x10\xDE\x839\x81Q\x91RU`\0R`\0\x80Q` a\x10\xFE\x839\x81Q\x91R` R`\0`@\x81 U\x01a\ngV[cNH{q`\xE0\x1B`\0R`1`\x04R`$`\0\xFD[a\xFF\xFF` a\x0B\x96a\x0C\x1F\x94a\x0F;V[\x90T\x90`\x03\x1B\x1C\x92a\x0B\xD3a\x0B\xAF\x84\x84\x84\x01Q\x16a\x0F;V[c\xFF\xFF\xFF\xFF\x87\x93\x92\x93\x16\x90\x83T\x90`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x90U\x01Q`\xE0\x92\x90\x92\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16`\0\x90\x81R`\0\x80Q` a\x10\xFE\x839\x81Q\x91R` R`@\x90 \x80Ta\xFF\xFF`\xA0\x1B\x19\x16\x91\x90\x92\x16`\xA0\x1Ba\xFF\xFF`\xA0\x1B\x16\x17\x90UV[8\x80a\x0B\x04V[`@Qc\r\xF5\xFDa`\xE3\x1B\x81R`\x04\x81\x01\x84\x90R`$\x90\xFD[`@Qcz\x08\xA2-`\xE0\x1B\x81R`\x04\x81\x01\x84\x90R`$\x90\xFD[`$\x90`@Q\x90c\xD0\x91\xBC\x81`\xE0\x1B\x82R`\x04\x82\x01R\xFD[`@Qc?\xF4\xD2\x0F`\xE1\x1B\x81R`$\x91a\x0C\x8E\x90`\x04\x83\x01\x90a\x0E\xB0V[\xFD[cNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[`@Qc\xE7g\xF9\x1F`\xE0\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qc\x07\xA0CQ`\xE5\x1B\x81R`\x04\x90\xFD[`@Qc\x8B=\xDC3`\xE0\x1B\x81R`\x04\x90\xFD[`\0\x80\xFD[\x82Q`\x01`\x01`@\x1B\x03\x81\x11a\x0C\xE3W\x82\x01\x90``\x90\x81`\x1F\x19\x84\x8C\x03\x01\x12a\x0C\xE3W`@Q\x91\x80\x83\x01`\x01`\x01`@\x1B\x03\x81\x11\x84\x82\x10\x17a\r|W`@Ra\r3` \x85\x01a\x0E\x0BV[\x83R`@\x84\x01Q`\x03\x81\x10\x15a\x0C\xE3W` \x84\x01R\x83\x01Q\x91`\x01`\x01`@\x1B\x03\x83\x11a\x0C\xE3Wa\rl\x8B` \x80\x96\x95\x81\x96\x01\x01a\x0E\x1FV[`@\x82\x01R\x81R\x01\x92\x01\x91a\0{V[`$`\0cNH{q`\xE0\x1B\x81R`A`\x04R\xFD[`@\x80Q\x91\x90\x82\x01`\x01`\x01`@\x1B\x03\x81\x11\x83\x82\x10\x17a\x05\x1CW`@RV[`@Q\x90``\x82\x01`\x01`\x01`@\x1B\x03\x81\x11\x83\x82\x10\x17a\x05\x1CW`@RV[`@Q\x91\x90`\x1F\x01`\x1F\x19\x16\x82\x01`\x01`\x01`@\x1B\x03\x81\x11\x83\x82\x10\x17a\x05\x1CW`@RV[`\x01`\x01`@\x1B\x03\x81\x11a\x05\x1CW`\x05\x1B` \x01\x90V[Q\x90`\x01`\x01`\xA0\x1B\x03\x82\x16\x82\x03a\x0C\xE3WV[\x90\x80`\x1F\x83\x01\x12\x15a\x0C\xE3W\x81Q\x90a\x0E:a\0V\x83a\r\xF4V[\x91\x82\x93\x81\x84R` \x80\x80\x95\x01\x92`\x05\x1B\x82\x01\x01\x92\x83\x11a\x0C\xE3W\x83\x01\x90[\x82\x82\x10a\x0EfWPPPP\x90V[\x81Q`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x81\x03a\x0C\xE3W\x81R\x90\x83\x01\x90\x83\x01a\x0EXV[\x80Q\x82\x10\x15a\x0E\x9AW` \x91`\x05\x1B\x01\x01\x90V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[\x90`\x03\x82\x10\x15a\x0C\x90WRV[\x90\x81Q\x80\x82R` \x80\x80\x93\x01\x93\x01\x91`\0[\x82\x81\x10a\x0E\xDDWPPPP\x90V[\x83Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x93\x81\x01\x93\x92\x81\x01\x92`\x01\x01a\x0E\xCFV[\x91\x90\x82Q\x92\x83\x82R`\0[\x84\x81\x10a\x0F'WPP\x82`\0` \x80\x94\x95\x84\x01\x01R`\x1F\x80\x19\x91\x01\x16\x01\x01\x90V[` \x81\x83\x01\x81\x01Q\x84\x83\x01\x82\x01R\x01a\x0F\x06V[\x90`\0\x80Q` a\x10\xDE\x839\x81Q\x91R\x80T\x83\x10\x15a\x0E\x9AW`\0R`\x1C` `\0 \x83`\x03\x1C\x01\x92`\x02\x1B\x16\x90V[\x80;\x15a\x0FvWPPV[`@\x80Qc\x91\x984\xB9`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x90\x92\x16`\x04\x83\x01R`$\x82\x01R\x90\x81\x90a\x08\xDA\x90`D\x83\x01\x90a\x0E\xFBV\xFE`\x80`@R6\x15`\x87W`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`oWP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[=\x90\xFD[`$\x90`@Q\x90c\n\x82\xDDs`\xE3\x1B\x82R`\x04\x82\x01R\xFD[`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`\xE9WP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[c\n\x82\xDDs`\xE3\x1B`\x80R`\x84R`$`\x80\xFD\xFE\xA2dipfsX\"\x12 \x92E]O\xC2\x8E\xBB\x9F0\\\xADA\0\xA5\x07Q\t\xA3\x9A\xEB\xEF\xFF'\"\xECQ\x16r\xE6S\"\x05dsolcC\0\x08\x13\x003\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2";
    /// The bytecode of the contract.
    pub static SUBNETREGISTRYDIAMOND_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __BYTECODE,
    );
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R6\x15`\x87W`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`oWP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[=\x90\xFD[`$\x90`@Q\x90c\n\x82\xDDs`\xE3\x1B\x82R`\x04\x82\x01R\xFD[`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`\xE9WP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[c\n\x82\xDDs`\xE3\x1B`\x80R`\x84R`$`\x80\xFD\xFE\xA2dipfsX\"\x12 \x92E]O\xC2\x8E\xBB\x9F0\\\xADA\0\xA5\x07Q\t\xA3\x9A\xEB\xEF\xFF'\"\xECQ\x16r\xE6S\"\x05dsolcC\0\x08\x13\x003";
    /// The deployed bytecode of the contract.
    pub static SUBNETREGISTRYDIAMOND_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __DEPLOYED_BYTECODE,
    );
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
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    SUBNETREGISTRYDIAMOND_ABI.clone(),
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
                SUBNETREGISTRYDIAMOND_ABI.clone(),
                SUBNETREGISTRYDIAMOND_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for SubnetRegistryDiamond<M> {
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
    ///Custom Error type `FacetCannotBeZero` with signature `FacetCannotBeZero()` and selector `0xf4086a20`
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
        Hash
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
        Hash
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
    pub enum SubnetRegistryDiamondErrors {
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
            if let Ok(decoded) = <FacetCannotBeZero as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::FacetCannotBeZero(decoded));
            }
            if let Ok(decoded) = <FunctionNotFound as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::FunctionNotFound(decoded));
            }
            if let Ok(decoded) = <GatewayCannotBeZero as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GatewayCannotBeZero(decoded));
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
            if let Ok(decoded) = <RemoveFacetAddressMustBeZeroAddress as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
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
                Self::CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(
                    element,
                ) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::CannotReplaceFunctionsFromFacetWithZeroAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotReplaceImmutableFunction(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FacetCannotBeZero(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FunctionNotFound(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
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
                Self::CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(
                    element,
                ) => ::core::fmt::Display::fmt(element, f),
                Self::CannotReplaceFunctionsFromFacetWithZeroAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotReplaceImmutableFunction(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::FacetCannotBeZero(element) => ::core::fmt::Display::fmt(element, f),
                Self::FunctionNotFound(element) => ::core::fmt::Display::fmt(element, f),
                Self::GatewayCannotBeZero(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::IncorrectFacetCutAction(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitializationFunctionReverted(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NoBytecodeAtAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
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
    for SubnetRegistryDiamondErrors {
        fn from(value: CannotAddFunctionToDiamondThatAlreadyExists) -> Self {
            Self::CannotAddFunctionToDiamondThatAlreadyExists(value)
        }
    }
    impl ::core::convert::From<CannotAddSelectorsToZeroAddress>
    for SubnetRegistryDiamondErrors {
        fn from(value: CannotAddSelectorsToZeroAddress) -> Self {
            Self::CannotAddSelectorsToZeroAddress(value)
        }
    }
    impl ::core::convert::From<CannotRemoveFunctionThatDoesNotExist>
    for SubnetRegistryDiamondErrors {
        fn from(value: CannotRemoveFunctionThatDoesNotExist) -> Self {
            Self::CannotRemoveFunctionThatDoesNotExist(value)
        }
    }
    impl ::core::convert::From<CannotRemoveImmutableFunction>
    for SubnetRegistryDiamondErrors {
        fn from(value: CannotRemoveImmutableFunction) -> Self {
            Self::CannotRemoveImmutableFunction(value)
        }
    }
    impl ::core::convert::From<CannotReplaceFunctionThatDoesNotExists>
    for SubnetRegistryDiamondErrors {
        fn from(value: CannotReplaceFunctionThatDoesNotExists) -> Self {
            Self::CannotReplaceFunctionThatDoesNotExists(value)
        }
    }
    impl ::core::convert::From<CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet>
    for SubnetRegistryDiamondErrors {
        fn from(
            value: CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet,
        ) -> Self {
            Self::CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(value)
        }
    }
    impl ::core::convert::From<CannotReplaceFunctionsFromFacetWithZeroAddress>
    for SubnetRegistryDiamondErrors {
        fn from(value: CannotReplaceFunctionsFromFacetWithZeroAddress) -> Self {
            Self::CannotReplaceFunctionsFromFacetWithZeroAddress(value)
        }
    }
    impl ::core::convert::From<CannotReplaceImmutableFunction>
    for SubnetRegistryDiamondErrors {
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
    impl ::core::convert::From<InitializationFunctionReverted>
    for SubnetRegistryDiamondErrors {
        fn from(value: InitializationFunctionReverted) -> Self {
            Self::InitializationFunctionReverted(value)
        }
    }
    impl ::core::convert::From<NoBytecodeAtAddress> for SubnetRegistryDiamondErrors {
        fn from(value: NoBytecodeAtAddress) -> Self {
            Self::NoBytecodeAtAddress(value)
        }
    }
    impl ::core::convert::From<NoSelectorsProvidedForFacetForCut>
    for SubnetRegistryDiamondErrors {
        fn from(value: NoSelectorsProvidedForFacetForCut) -> Self {
            Self::NoSelectorsProvidedForFacetForCut(value)
        }
    }
    impl ::core::convert::From<RemoveFacetAddressMustBeZeroAddress>
    for SubnetRegistryDiamondErrors {
        fn from(value: RemoveFacetAddressMustBeZeroAddress) -> Self {
            Self::RemoveFacetAddressMustBeZeroAddress(value)
        }
    }
}
