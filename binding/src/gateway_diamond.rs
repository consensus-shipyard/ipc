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
                            ::std::boxed::Box::new(::ethers::core::abi::ethabi::ParamType::Tuple(
                                ::std::vec![
                                    ::ethers::core::abi::ethabi::ParamType::Address,
                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                                4usize
                                            ),
                                        ),
                                    ),
                                ],
                            ),),
                        ),
                        internal_type: ::core::option::Option::Some(
                            ::std::borrow::ToOwned::to_owned("struct IDiamond.FacetCut[]",),
                        ),
                    },
                    ::ethers::core::abi::ethabi::Param {
                        name: ::std::borrow::ToOwned::to_owned("params"),
                        kind: ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                            ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                ::ethers::core::abi::ethabi::ParamType::Array(
                                    ::std::boxed::Box::new(
                                        ::ethers::core::abi::ethabi::ParamType::Address,
                                    ),
                                ),
                            ],),
                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                            ::ethers::core::abi::ethabi::ParamType::Array(::std::boxed::Box::new(
                                ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                    ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                        ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    ],),
                                ],),
                            ),),
                        ],),
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
                    ::std::borrow::ToOwned::to_owned("CannotAddFunctionToDiamondThatAlreadyExists"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned(
                            "CannotAddFunctionToDiamondThatAlreadyExists",
                        ),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("_selector"),
                            kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("bytes4"),
                            ),
                        },],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CannotAddSelectorsToZeroAddress"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("CannotAddSelectorsToZeroAddress",),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("_selectors"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                ::std::boxed::Box::new(
                                    ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                ),
                            ),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("bytes4[]"),
                            ),
                        },],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("FunctionNotFound"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("FunctionNotFound"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("_functionSelector"),
                            kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("bytes4"),
                            ),
                        },],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("IncorrectFacetCutAction"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("IncorrectFacetCutAction",),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("_action"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("enum IDiamond.FacetCutAction",),
                            ),
                        },],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InitializationFunctionReverted"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("InitializationFunctionReverted",),
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
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidCollateral"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("InvalidCollateral"),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidMajorityPercentage"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("InvalidMajorityPercentage",),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidSubmissionPeriod"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("InvalidSubmissionPeriod",),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NoBytecodeAtAddress"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("NoBytecodeAtAddress",),
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
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NoSelectorsProvidedForFacetForCut"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("NoSelectorsProvidedForFacetForCut",),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("_facetAddress"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Address,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("address"),
                            ),
                        },],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("OldConfigurationNumber"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("OldConfigurationNumber",),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ValidatorWeightIsZero"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("ValidatorWeightIsZero",),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ValidatorsAndWeightsLengthMismatch"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned(
                            "ValidatorsAndWeightsLengthMismatch",
                        ),
                        inputs: ::std::vec![],
                    },],
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
    const __BYTECODE: &[u8] = b"`\x80\x80`@R4b\0\x0B\x8FWb\0\x18\xB5\x808\x03\x80\x91b\0\0 \x82\x85b\0\x0C\xF9V[\x839`@\x82\x82\x81\x01\x03\x12b\0\x0B\x8FW\x81Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0B\x8FW\x82\x01\x90\x80\x83\x01`\x1F\x83\x01\x12\x15b\0\x0B\x8FW\x81Q\x90b\0\0_\x82b\0\r\x1DV[\x92b\0\0o`@Q\x94\x85b\0\x0C\xF9V[\x82\x84R` \x84\x01` \x81\x94`\x05\x1B\x83\x01\x01\x91\x83\x87\x01\x83\x11b\0\x0B\x8FW` \x81\x01\x91[\x83\x83\x10b\0\x0B\xC4WPPPP` \x84\x01Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0B\x8FW\x84\x01\x90`\xE0\x82\x82\x87\x01\x03\x12b\0\x0B\x8FW`@Q\x94`\xE0\x86\x01`\x01`\x01`@\x1B\x03\x81\x11\x87\x82\x10\x17b\0\x06\xA9W`@\x81\x90R\x83Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0B\x8FW\x84\x01`@\x81\x85\x85\x01\x03\x12b\0\x0B\x8FWb\0\x01\x0C\x82b\0\x0C\xDDV[b\0\x01\x17\x81b\0\rJV[\x82R` \x81\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0B\x8FW\x01\x83\x83\x01`\x1F\x82\x01\x12\x15b\0\x0B\x8FW\x80Q\x90b\0\x01K\x82b\0\r\x1DV[\x91b\0\x01[`@Q\x93\x84b\0\x0C\xF9V[\x80\x83R` \x80\x84\x01\x91`\x05\x1B\x83\x01\x01\x91\x86\x86\x01\x83\x11b\0\x0B\x8FW` \x01\x90[\x82\x82\x10b\0\x0B\xA9WPPPa\x01\0\x88\x01R\x86Rb\0\x01\x9B` \x84\x01b\0\rJV[` \x87\x01Rb\0\x01\xAE`@\x84\x01b\0\rJV[`@\x87\x01R``\x83\x01Q``\x87\x01R`\x80\x83\x01Q`\x80\x87\x01Rb\0\x01\xD5`\xA0\x84\x01b\0\r_V[`\xA0\x87\x01R`\xC0\x83\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0B\x8FW\x82\x81\x01`\x1F\x83\x86\x01\x01\x12\x15b\0\x0B\x8FW\x81\x84\x01Q\x91b\0\x02\x0F\x83b\0\r\x1DV[\x94b\0\x02\x1F`@Q\x96\x87b\0\x0C\xF9V[\x83\x86R` \x86\x01\x91\x85\x84\x01` \x86`\x05\x1B\x83\x85\x01\x01\x01\x11b\0\x0B\x8FW` \x81\x83\x01\x01\x92[` \x86`\x05\x1B\x83\x85\x01\x01\x01\x84\x10b\0\nzW\x8A\x8A\x8A\x8A`\xC0\x84\x01R``\x83\x01Q\x15b\0\nhW` \x83\x01Q`\x01`\x01`@\x1B\x03\x16\x15b\0\nVW`\xFF`\xA0\x84\x01Q\x16`3\x81\x10\x90\x81\x15b\0\nJW[Pb\0\n8W\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD5\x80T`\x01`\x01`\xA0\x1B\x03\x19\x163\x17\x90U`@Q`\x01`\x01`@\x1B\x03` \x82\x01\x90\x81\x11\x90\x82\x11\x17b\0\x06\xA9W` \x81\x01`@R`\0\x81R\x82Q`\0[\x81\x81\x10b\0\x07\x19WPP`@Q\x92``\x84\x01\x90``\x85RQ\x80\x91R`\x80\x84\x01\x90`\x80\x81`\x05\x1B\x86\x01\x01\x93\x91`\0\x90[\x82\x82\x10b\0\x06\xBFW\x87\x7F\x8F\xAAp\x87\x86q\xCC\xD2\x12\xD2\x07q\xB7\x95\xC5\n\xF8\xFD?\xF6\xCF'\xF4\xBD\xE5~]M\xE0\xAE\xB6s\x88\x80b\0\x03j\x8A\x8A`\0` \x85\x01R\x83\x82\x03`@\x85\x01Rb\0\x0E\x0FV[\x03\x90\xA1\x80Q\x80Q`\x15\x80T`\x01`\x01`@\x1B\x03\x19\x90\x81\x16`\x01`\x01`@\x1B\x03\x93\x84\x16\x17\x90\x91U` \x90\x92\x01Q\x80Q\x91\x82\x11b\0\x06\xA9Wh\x01\0\0\0\0\0\0\0\0\x82\x11b\0\x06\xA9W` \x90`\x16T\x83`\x16U\x80\x84\x10b\0\x06\x88W[P\x01`\x16`\0R` `\0 `\0[\x83\x81\x10b\0\x06jW``\x86\x01Q`\x17U` \x86\x01Q`\x1A\x80To\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0`@\x93\x84\x1B\x16`\x01`@\x1B`\x01`\x80\x1B\x03\x19\x90\x91\x16\x17\x90U\x86\x01Q`\x19\x80T`\x80\x89\x01Q`\x18U`\xA0\x89\x01Q`\xFF\x16`\x01`\x88\x1B`\x01`\xC8\x1B\x03`\x88\x94\x90\x94\x1B\x93\x90\x93\x16`\xFF`\x01`\x88\x1B\x03`\x01`\xC8\x1B\x03\x19\x90\x91\x16\x17\x91\x90\x91\x17\x90U`\x0F\x80T\x86\x16`\x01\x90\x81\x17\x90\x91U`\xC0\x87\x01QQ\x87\x91b\0\x04{\x82b\0\r\x1DV[\x91b\0\x04\x8B`@Q\x93\x84b\0\x0C\xF9V[\x80\x83R`\x1F\x19\x90\x81b\0\x04\x9E\x82b\0\r\x1DV[\x01`\0[\x81\x81\x10b\0\x06BWPPb\0\x04\xB7\x81b\0\r\x1DV[\x94b\0\x04\xC7`@Q\x96\x87b\0\x0C\xF9V[\x81\x86Rb\0\x04\xD5\x82b\0\r\x1DV[` \x87\x01\x93\x016\x847\x83`\0[\x83\x81\x10b\0\x05\xEBWPPPP`@Q\x91``\x83\x01\x90`\0\x84R``` \x85\x01R\x84Q\x80\x92R`\x80\x84\x01`\x80\x83`\x05\x1B\x86\x01\x01\x92` \x87\x01\x91`\0\x90[\x82\x82\x10b\0\x05\xBCWPPPP\x83\x82\x03`@\x85\x01R` \x86Q\x92\x83\x81R\x01\x92\x91`\0[\x81\x81\x10b\0\x05\xA6W\x87\x87\x7F#qD8\x80\xFA\xCD\xDA\xC9\n+'h\r8\x15'\xA6ps\xAD\xFE\x98Q,\xAC\x1B\x9DhaB\x8C\x88\x88\x03\x89\xA1Q\x90Q\x03b\0\x05\x94Wb\0\x05\x83b\0\x11\xAFV[P`@Qa\x013\x90\x81b\0\x17B\x829\xF3[`@QcF_\n}`\xE0\x1B\x81R`\x04\x90\xFD[\x83Q\x85R` \x94\x85\x01\x94\x90\x93\x01\x92\x82\x01b\0\x05@V[\x90\x91\x92\x93` \x80b\0\x05\xDA\x83\x98`\x7F\x19\x8C\x82\x03\x01\x86R\x88Qb\0\x0E6V[\x97\x96\x01\x94\x93\x91\x90\x91\x01\x91\x01b\0\x05\x1EV[` b\0\x05\xFD\x82`\xC0\x86\x01Qb\0\r\xACV[Q\x01Qb\0\x06\x0C\x82\x89b\0\r\xACV[Rb\0\x06\x19\x81\x88b\0\r\xACV[Pb\0\x06*\x81`\xC0\x85\x01Qb\0\r\xACV[QQb\0\x068\x82\x8Ab\0\r\xACV[R\x01\x84\x90b\0\x04\xE2V[` \x90`@Qb\0\x06S\x81b\0\x0C\xDDV[`\0\x81R``\x83\x82\x01R\x82\x82\x89\x01\x01R\x01b\0\x04\xA2V[\x82Q`\x01`\x01`\xA0\x1B\x03\x16\x81\x83\x01U` \x90\x92\x01\x91`\x01\x01b\0\x03\xD3V[b\0\x06\xA2\x90`\x16`\0R\x84\x84`\0 \x91\x82\x01\x91\x01b\0\r\x93V[\x85b\0\x03\xC4V[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[\x90\x91\x92\x94` \x80b\0\x07\n`\x01\x93`\x7F\x19\x8B\x82\x03\x01\x86R```@\x8BQ\x87\x80`\xA0\x1B\x03\x81Q\x16\x84Rb\0\x06\xF9\x86\x82\x01Q\x87\x86\x01\x90b\0\r\xC1V[\x01Q\x91\x81`@\x82\x01R\x01\x90b\0\r\xCFV[\x97\x01\x92\x01\x92\x01\x90\x92\x91b\0\x03#V[`@b\0\x07'\x82\x87b\0\r\xACV[Q\x01Q`\x01`\x01`\xA0\x1B\x03b\0\x07>\x83\x88b\0\r\xACV[QQ\x16\x90\x80Q\x15b\0\n\x1FW` b\0\x07X\x84\x89b\0\r\xACV[Q\x01Q`\x03\x81\x10\x15b\0\n\tW\x80b\0\t\xE7WP\x81\x15b\0\t\xC0Wa\xFF\xFF`\0\x80Q` b\0\x18u\x839\x81Q\x91RT\x16\x91`@Qb\0\x07\x97\x81b\0\x0C\xC1V[`!\x81R\x7FdiamondCut: Add facet has no cod` \x82\x01R`e`\xF8\x1B`@\x82\x01R\x81;\x15b\0\t\x90WP\x81Q\x91`\0\x93[\x83\x85\x10b\0\x07\xF1WPPPPP`\x01\x01b\0\x02\xF4V[b\0\x07\xFD\x85\x83b\0\r\xACV[Q`\x01`\x01`\xE0\x1B\x03\x19\x81\x16`\0\x90\x81R`\0\x80Q` b\0\x18\x95\x839\x81Q\x91R` R`@\x90 T`\x01`\x01`\xA0\x1B\x03\x16b\0\tmW`@Qb\0\x08B\x81b\0\x0C\xDDV[\x84\x81Ra\xFF\xFF\x83\x16` \x80\x83\x01\x91\x82R`\x01`\x01`\xE0\x1B\x03\x19\x84\x16`\0\x90\x81R`\0\x80Q` b\0\x18\x95\x839\x81Q\x91R\x90\x91R`@\x90 \x91Q\x82T\x91Q`\x01`\x01`\xB0\x1B\x03\x19\x90\x92\x16`\x01`\x01`\xA0\x1B\x03\x91\x90\x91\x16\x17`\xA0\x91\x90\x91\x1Ba\xFF\xFF`\xA0\x1B\x16\x17\x90U`\0\x80Q` b\0\x18u\x839\x81Q\x91RT\x90h\x01\0\0\0\0\0\0\0\0\x82\x10\x15b\0\x06\xA9W`\x01\x82\x01\x80`\0\x80Q` b\0\x18u\x839\x81Q\x91RU\x82\x10\x15b\0\tWW`\0\x80Q` b\0\x18u\x839\x81Q\x91R`\0R` `\0 \x82`\x03\x1C\x01\x91c\xFF\xFF\xFF\xFF`\xE0\x84T\x92`\x05\x1B\x16\x92`\xE0\x1C\x83\x1B\x92\x1B\x19\x16\x17\x90Ua\xFF\xFF\x80\x82\x16\x14b\0\tAW`\x01a\xFF\xFF\x81\x92\x16\x01\x94\x01\x93b\0\x07\xDBV[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[`@Qc\xEB\xBF]\x07`\xE0\x1B\x81R`\x01`\x01`\xE0\x1B\x03\x19\x90\x91\x16`\x04\x82\x01R`$\x90\xFD[\x90b\0\t\xBC`@Q\x92\x83\x92c\x91\x984\xB9`\xE0\x1B\x84R`\x04\x84\x01R`@`$\x84\x01R`D\x83\x01\x90b\0\x0E\x0FV[\x03\x90\xFD[`@Qc\x02\xB8\xDA\x07`\xE2\x1B\x81R` `\x04\x82\x01R\x90\x81\x90b\0\t\xBC\x90`$\x83\x01\x90b\0\r\xCFV[`@Qc?\xF4\xD2\x0F`\xE1\x1B\x81R`$\x91b\0\n\x07\x90`\x04\x83\x01\x90b\0\r\xC1V[\xFD[cNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[`@Qc\xE7g\xF9\x1F`\xE0\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qcu\xC3\xB4'`\xE0\x1B\x81R`\x04\x90\xFD[`d\x91P\x11\x84b\0\x02\x92V[`@Qc1/\x8E\x05`\xE0\x1B\x81R`\x04\x90\xFD[`@Qch\xF7\xA6u`\xE1\x1B\x81R`\x04\x90\xFD[\x83Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0B\x8FW\x82\x84\x01\x01`\x1F\x19\x91`@\x83\x83\x8B\x8A\x01\x03\x01\x12b\0\x0B\x8FW`@Q\x91b\0\n\xB1\x83b\0\x0C\xDDV[` \x81\x01Q\x83R`@\x81\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0B\x8FW\x01\x90`@\x84\x83\x8C\x8B\x01\x03\x01\x12b\0\x0B\x8FW`@Q\x91b\0\n\xED\x83b\0\x0C\xDDV[b\0\n\xFB` \x82\x01b\0\r_V[\x83R`@\x81\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0B\x8FW\x01\x92\x8A\x89\x01`?\x85\x01\x12\x15b\0\x0B\x8FW` \x84\x01Q\x92`\x01`\x01`@\x1B\x03\x84\x11b\0\x0B\x94Wb\0\x0BN` `@Q\x97`\x1F\x87\x01\x16\x01\x87b\0\x0C\xF9V[\x83\x86R\x8B\x8A\x01`@\x85\x87\x01\x01\x11b\0\x0B\x8FW\x85b\0\x0By` \x97\x95\x88\x97`@\x89\x80\x99\x01\x91\x01b\0\rnV[\x84\x82\x01R\x83\x82\x01R\x81R\x01\x94\x01\x93\x90Pb\0\x02CV[`\0\x80\xFD[`$`\0cNH{q`\xE0\x1B\x81R`A`\x04R\xFD[` \x80\x91b\0\x0B\xB8\x84b\0\r5V[\x81R\x01\x91\x01\x90b\0\x01zV[\x82Q`\x01`\x01`@\x1B\x03\x81\x11b\0\x0B\x8FW\x82\x01```\x1F\x19\x82\x88\x8C\x01\x03\x01\x12b\0\x0B\x8FW`@Q\x90b\0\x0B\xF7\x82b\0\x0C\xC1V[b\0\x0C\x05` \x82\x01b\0\r5V[\x82R`@\x81\x01Q`\x03\x81\x10\x15b\0\x0B\x8FW` \x83\x01R``\x81\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x0B\x8FW\x01\x86\x8A\x01`?\x82\x01\x12\x15b\0\x0B\x8FW` \x81\x01Qb\0\x0CO\x81b\0\r\x1DV[\x91b\0\x0C_`@Q\x93\x84b\0\x0C\xF9V[\x81\x83R`@` \x84\x01\x92`\x05\x1B\x82\x01\x01\x90\x89\x8D\x01\x82\x11b\0\x0B\x8FW`@\x01\x91[\x81\x83\x10b\0\x0C\x9EWPPP`@\x82\x01R\x81R` \x92\x83\x01\x92\x01b\0\0\x91V[\x82Q`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x81\x03b\0\x0B\x8FW\x81R` \x92\x83\x01\x92\x01b\0\x0C\x7FV[``\x81\x01\x90\x81\x10`\x01`\x01`@\x1B\x03\x82\x11\x17b\0\x06\xA9W`@RV[`@\x81\x01\x90\x81\x10`\x01`\x01`@\x1B\x03\x82\x11\x17b\0\x06\xA9W`@RV[`\x1F\x90\x91\x01`\x1F\x19\x16\x81\x01\x90`\x01`\x01`@\x1B\x03\x82\x11\x90\x82\x10\x17b\0\x06\xA9W`@RV[`\x01`\x01`@\x1B\x03\x81\x11b\0\x06\xA9W`\x05\x1B` \x01\x90V[Q\x90`\x01`\x01`\xA0\x1B\x03\x82\x16\x82\x03b\0\x0B\x8FWV[Q\x90`\x01`\x01`@\x1B\x03\x82\x16\x82\x03b\0\x0B\x8FWV[Q\x90`\xFF\x82\x16\x82\x03b\0\x0B\x8FWV[`\0[\x83\x81\x10b\0\r\x82WPP`\0\x91\x01RV[\x81\x81\x01Q\x83\x82\x01R` \x01b\0\rqV[\x81\x81\x10b\0\r\x9FWPPV[`\0\x81U`\x01\x01b\0\r\x93V[\x80Q\x82\x10\x15b\0\tWW` \x91`\x05\x1B\x01\x01\x90V[\x90`\x03\x82\x10\x15b\0\n\tWRV[\x90\x81Q\x80\x82R` \x80\x80\x93\x01\x93\x01\x91`\0[\x82\x81\x10b\0\r\xF0WPPPP\x90V[\x83Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x93\x81\x01\x93\x92\x81\x01\x92`\x01\x01b\0\r\xE1V[\x90` \x91b\0\x0E*\x81Q\x80\x92\x81\x85R\x85\x80\x86\x01\x91\x01b\0\rnV[`\x1F\x01`\x1F\x19\x16\x01\x01\x90V[\x90`@` b\0\x0EX\x93`\xFF\x81Q\x16\x84R\x01Q\x91\x81` \x82\x01R\x01\x90b\0\x0E\x0FV[\x90V[\x90`\x01\x82\x81\x1C\x92\x16\x80\x15b\0\x0E\x8DW[` \x83\x10\x14b\0\x0EwWV[cNH{q`\xE0\x1B`\0R`\"`\x04R`$`\0\xFD[\x91`\x7F\x16\x91b\0\x0EkV[`@\x90\x81Qb\0\x0E\xA8\x81b\0\x0C\xC1V[\x80\x92`\x06Tb\0\x0E\xB8\x81b\0\r\x1DV[b\0\x0E\xC6\x83Q\x91\x82b\0\x0C\xF9V[\x81\x81R`\x06`\0\x90\x81R` \x92\x90\x7F\xF6R\"#\x13\xE2\x84YR\x8D\x92\x0Be\x11\\\x16\xC0O>\xFC\x82\xAA\xED\xC9{\xE5\x9F?7|\r?\x81\x85\x85\x01[\x84\x84\x10b\0\x0F\"WPPP\x91\x85RPP`\x07T`\x01`\x01`@\x1B\x03\x16\x90\x83\x01R`\x08T\x91\x01RV[\x86\x88Qb\0\x0F0\x81b\0\x0C\xDDV[\x84T\x81R\x89Qb\0\x0FA\x81b\0\x0C\xDDV[`\x01`\xFF\x81\x88\x01T\x16\x82R`\x02\x87\x01\x8CQ\x91\x87\x91\x80T\x90b\0\x0Fc\x82b\0\x0E[V[\x80\x86R\x91\x83\x81\x16\x90\x81\x15b\0\x0F\xEBWP`\x01\x14b\0\x0F\xADW[PPP\x91\x81b\0\x0F\x96`\x03\x96\x95\x93`\x01\x98\x95\x03\x82b\0\x0C\xF9V[\x84\x82\x01R\x83\x82\x01R\x81R\x01\x93\x01\x93\x01\x92\x91b\0\x0E\xFAV[\x89R\x86\x89 \x89\x93P\x90\x91\x90[\x82\x84\x10b\0\x0F\xD5WPPP\x81\x01\x84\x01\x81b\0\x0F\x96`\x03b\0\x0F|V[\x80T\x85\x85\x01\x89\x01R\x8E\x97\x90\x93\x01\x92\x81\x01b\0\x0F\xB9V[`\xFF\x19\x16\x89\x87\x01RPP\x15\x15`\x05\x1B\x83\x01\x86\x01\x91P\x82\x90Pb\0\x0F\x96`\x03b\0\x0F|V[\x90\x80\x82\x14b\0\x119Wb\0\x10$\x81Tb\0\x0E[V[\x90`\x01`\x01`@\x1B\x03\x82\x11b\0\x06\xA9W\x81\x90b\0\x10B\x84Tb\0\x0E[V[`\x1F\x81\x11b\0\x10\xF7W[P`\0\x90`\x1F\x83\x11`\x01\x14b\0\x10\x87W`\0\x92b\0\x10{W[PP\x81`\x01\x1B\x91`\0\x19\x90`\x03\x1B\x1C\x19\x16\x17\x90UV[\x01T\x90P8\x80b\0\x10eV[\x81R` \x80\x82 \x85\x83R\x81\x83 \x93P\x90`\x1F\x19\x85\x16\x90\x83\x90[\x82\x82\x10b\0\x10\xDDWPP\x90\x84`\x01\x95\x94\x93\x92\x10b\0\x10\xC3W[PPP\x81\x1B\x01\x90UV[\x01T`\0\x19`\xF8\x84`\x03\x1B\x16\x1C\x19\x16\x90U8\x80\x80b\0\x10\xB9V[\x84\x95\x81\x92\x95\x85\x01T\x81U`\x01\x80\x91\x01\x96\x01\x94\x01\x90b\0\x10\xA0V[b\0\x11'\x90\x85`\0R` `\0 `\x1F\x85\x01`\x05\x1C\x81\x01\x91` \x86\x10b\0\x11.W[`\x1F\x01`\x05\x1C\x01\x90b\0\r\x93V[8b\0\x10LV[\x90\x91P\x81\x90b\0\x11\x19V[PPV[\x90\x80\x82Q\x90\x81\x81R` \x80\x91\x01\x92\x81\x80\x84`\x05\x1B\x83\x01\x01\x95\x01\x93`\0\x91[\x84\x83\x10b\0\x11lWPPPPPP\x90V[\x90\x91\x92\x93\x94\x95\x84\x80b\0\x11\x9E`\x01\x93`\x1F\x19\x86\x82\x03\x01\x87R\x82\x8BQ\x80Q\x83R\x01Q\x90`@\x90\x81\x85\x82\x01R\x01\x90b\0\x0E6V[\x98\x01\x93\x01\x93\x01\x91\x94\x93\x92\x90b\0\x11[V[`@Qb\0\x11\xBD\x81b\0\x0C\xC1V[``\x81R`\0\x90\x81`@` \x92\x82\x84\x82\x01R\x01R`@Qb\0\x11\xDF\x81b\0\x0C\xC1V[`\x06T\x92b\0\x11\xEE\x84b\0\r\x1DV[b\0\x11\xFD`@Q\x91\x82b\0\x0C\xF9V[\x84\x81R`\x06\x82R\x83\x82 \x82\x85\x83\x01[\x87\x82\x10b\0\x15\x9DWPPP\x82R`\x01\x80`@\x1B\x03\x93\x84`\x07T\x16\x84\x84\x01R`\x08T`@\x84\x01R`@Qb\0\x12@\x81b\0\x0C\xC1V[`\tT\x93b\0\x12O\x85b\0\r\x1DV[b\0\x12^`@Q\x91\x82b\0\x0C\xF9V[\x85\x81R`\t\x85R\x86\x85 \x85\x88\x83\x01[\x88\x82\x10b\0\x14\xAFWPPP\x82R`\nT\x87\x16\x86\x83\x01R`\x0BT`@\x83\x01Rb\0\x12\x97\x91\x90b\0\x16\x8DV[b\0\x14\xA0Wh\x01\0\0\0\0\0\0\0\0\x83\x11b\0\x14\x8CW\x82`\x06U\x80\x83\x10b\0\x13\xB5W[P`\t\x81R\x82\x81 `\x06\x82R\x83\x82 \x91\x90[\x83\x82\x10b\0\x13UWPPPP\x7F1\x17\x1D\tv\x8D\xF3\xAF2y\x8D\xB5\x1B\x92Y\xE0Yb\xA1\x8C\x15i\\\xBES\xF8\x82\xF2\xED.\x19W\x90\x82`\nT\x16`\x01\x80`@\x1B\x03\x19`\x07T\x16\x17`\x07U`\x0BT`\x08Ub\0\x13\x1Fb\0\x0E\x98V[\x92\x81\x84\x01Q\x16\x83Qb\0\x13I`@\x86\x01Q\x91```@Q\x95\x86\x95\x86R\x85\x01R``\x84\x01\x90b\0\x11=V[\x90`@\x83\x01R\x03\x90\xA1\x90V[\x80`\x01\x91\x84\x03b\0\x13rW[`\x03\x80\x91\x01\x93\x01\x91\x01\x90\x91b\0\x12\xCCV[\x80T\x84U\x81\x84\x01\x82\x82\x01\x80\x82\x03b\0\x13\x8DW[PPb\0\x13aV[`\xFF\x90T\x16`\xFF\x19\x82T\x16\x17\x90Ub\0\x13\xAD`\x02\x80\x83\x01\x90\x86\x01b\0\x10\x0FV[8\x80b\0\x13\x85V[`\x03\x90\x80\x82\x02\x90\x82\x82\x04\x03b\0\x14xW\x83\x82\x02\x82\x81\x04\x85\x03b\0\x14dW`\x06\x84R\x85\x84 \x91\x82\x01\x91\x01[\x81\x81\x10b\0\x13\xEFWPPb\0\x12\xBAV[\x80\x84\x84\x92U\x84`\x01\x81\x81\x84\x01U`\x02\x83\x01b\0\x14\x0C\x81Tb\0\x0E[V[\x80b\0\x14\x1EW[PPPP\x01b\0\x13\xDFV[\x83`\x1F\x93\x84\x83\x11`\x01\x14b\0\x14>WPPP\x90PU[\x848\x80\x80b\0\x14\x13V[\x83\x92\x94\x82\x94b\0\x14\\\x93R\x8D\x85 \x95\x01`\x05\x1C\x85\x01\x90\x85\x01b\0\r\x93V[UUb\0\x144V[cNH{q`\xE0\x1B\x84R`\x11`\x04R`$\x84\xFD[cNH{q`\xE0\x1B\x83R`\x11`\x04R`$\x83\xFD[cNH{q`\xE0\x1B\x82R`A`\x04R`$\x82\xFD[PPPPPb\0\x0EXb\0\x0E\x98V[\x89`@Qb\0\x14\xBE\x81b\0\x0C\xDDV[\x84T\x81R`@Qb\0\x14\xD0\x81b\0\x0C\xDDV[`\x01\x86\x01T`\xFF\x16\x81R`@Q`\x02\x87\x01\x80T\x8D\x91b\0\x14\xF0\x82b\0\x0E[V[\x80\x85R\x91`\x01\x81\x16\x90\x81\x15b\0\x15}WP`\x01\x14b\0\x15:W[PP\x91\x81b\0\x15#`\x03\x96\x95\x93`\x01\x98\x95\x03\x82b\0\x0C\xF9V[\x84\x82\x01R\x83\x82\x01R\x81R\x01\x93\x01\x91\x01\x90\x91b\0\x12mV[\x8ER\x85\x8E \x8E\x92P[\x81\x83\x10b\0\x15\\WPP\x81\x01\x84\x01\x81b\0\x15#b\0\x15\nV[\x80`\x01\x91\x97\x92\x93\x94\x95\x96\x97T\x83\x86\x88\x01\x01R\x01\x92\x01\x90\x8F\x95\x94\x93\x92b\0\x15CV[`\xFF\x19\x16\x88\x86\x01RPP\x15\x15`\x05\x1B\x82\x01\x85\x01\x90P\x81b\0\x15#b\0\x15\nV[\x86`@Qb\0\x15\xAC\x81b\0\x0C\xDDV[\x84T\x81R`@Qb\0\x15\xBE\x81b\0\x0C\xDDV[`\x01`\xFF\x81\x88\x01T\x16\x82R`\x02\x87\x01`@Q\x91\x8A\x91\x80T\x90b\0\x15\xE1\x82b\0\x0E[V[\x80\x86R\x91\x83\x81\x16\x90\x81\x15b\0\x16iWP`\x01\x14b\0\x16+W[PPP\x91\x81b\0\x16\x14`\x03\x96\x95\x93`\x01\x98\x95\x03\x82b\0\x0C\xF9V[\x84\x82\x01R\x83\x82\x01R\x81R\x01\x93\x01\x91\x01\x90\x91b\0\x12\x0CV[\x8CR\x86\x8C \x8C\x93P\x90\x91\x90[\x82\x84\x10b\0\x16SWPPP\x81\x01\x84\x01\x81b\0\x16\x14`\x03b\0\x15\xFAV[\x80T\x85\x85\x01\x89\x01R\x8E\x97\x90\x93\x01\x92\x81\x01b\0\x167V[`\xFF\x19\x16\x89\x87\x01RPP\x15\x15`\x05\x1B\x83\x01\x86\x01\x91P\x82\x90Pb\0\x16\x14`\x03b\0\x15\xFAV[` \x80\x82\x01Q\x83\x82\x01Q\x91\x92\x91`\x01`\x01`@\x1B\x03\x91\x82\x16\x91\x16\x03b\0\x179W`@\x90\x81\x81\x01Q\x82\x85\x01Q\x03b\0\x170WQ\x80Q\x84QQ\x03b\0\x170Wb\0\x17)\x90\x82Q\x90\x81b\0\x16\xE8\x86\x82\x01\x92\x87\x84R\x86\x83\x01\x90b\0\x11=V[\x03\x91b\0\x16\xFE`\x1F\x19\x93\x84\x81\x01\x83R\x82b\0\x0C\xF9V[Q\x90 \x94Q\x92b\0\x17\x1C\x81Q\x94\x85\x92\x87\x84\x01\x97\x88R\x83\x01\x90b\0\x11=V[\x03\x90\x81\x01\x83R\x82b\0\x0C\xF9V[Q\x90 \x14\x90V[PPPP`\0\x90V[PPP`\0\x90V\xFE`\x80`@R6\x15`\x87W`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`oWP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[=\x90\xFD[`$\x90`@Q\x90c\n\x82\xDDs`\xE3\x1B\x82R`\x04\x82\x01R\xFD[`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`\xE9WP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[c\n\x82\xDDs`\xE3\x1B`\x80R`\x84R`$`\x80\xFD\xFE\xA2dipfsX\"\x12 \x02\xCF\xCA<\xF1\x98\"N\xB2\xE7\x01\x14Qqo\x9F\xC5%\x18c\x8Af\xB4\x9A\xEE\x96\x97\xF6\xED\xE5L\xABdsolcC\0\x08\x13\x003\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2";
    /// The bytecode of the contract.
    pub static GATEWAYDIAMOND_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__BYTECODE);
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R6\x15`\x87W`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`oWP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[=\x90\xFD[`$\x90`@Q\x90c\n\x82\xDDs`\xE3\x1B\x82R`\x04\x82\x01R\xFD[`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`\xE9WP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[c\n\x82\xDDs`\xE3\x1B`\x80R`\x84R`$`\x80\xFD\xFE\xA2dipfsX\"\x12 \x02\xCF\xCA<\xF1\x98\"N\xB2\xE7\x01\x14Qqo\x9F\xC5%\x18c\x8Af\xB4\x9A\xEE\x96\x97\xF6\xED\xE5L\xABdsolcC\0\x08\x13\x003";
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
    ///Custom Error type `InvalidCollateral` with signature `InvalidCollateral()` and selector `0xd1ef4cea`
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
    ///Custom Error type `ValidatorWeightIsZero` with signature `ValidatorWeightIsZero()` and selector `0x389b457d`
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
    #[etherror(name = "ValidatorWeightIsZero", abi = "ValidatorWeightIsZero()")]
    pub struct ValidatorWeightIsZero;
    ///Custom Error type `ValidatorsAndWeightsLengthMismatch` with signature `ValidatorsAndWeightsLengthMismatch()` and selector `0x465f0a7d`
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
        name = "ValidatorsAndWeightsLengthMismatch",
        abi = "ValidatorsAndWeightsLengthMismatch()"
    )]
    pub struct ValidatorsAndWeightsLengthMismatch;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum GatewayDiamondErrors {
        CannotAddFunctionToDiamondThatAlreadyExists(CannotAddFunctionToDiamondThatAlreadyExists),
        CannotAddSelectorsToZeroAddress(CannotAddSelectorsToZeroAddress),
        FunctionNotFound(FunctionNotFound),
        IncorrectFacetCutAction(IncorrectFacetCutAction),
        InitializationFunctionReverted(InitializationFunctionReverted),
        InvalidCollateral(InvalidCollateral),
        InvalidMajorityPercentage(InvalidMajorityPercentage),
        InvalidSubmissionPeriod(InvalidSubmissionPeriod),
        NoBytecodeAtAddress(NoBytecodeAtAddress),
        NoSelectorsProvidedForFacetForCut(NoSelectorsProvidedForFacetForCut),
        OldConfigurationNumber(OldConfigurationNumber),
        ValidatorWeightIsZero(ValidatorWeightIsZero),
        ValidatorsAndWeightsLengthMismatch(ValidatorsAndWeightsLengthMismatch),
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
            if let Ok(decoded) = <InvalidCollateral as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::InvalidCollateral(decoded));
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
                <ValidatorWeightIsZero as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ValidatorWeightIsZero(decoded));
            }
            if let Ok(decoded) =
                <ValidatorsAndWeightsLengthMismatch as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ValidatorsAndWeightsLengthMismatch(decoded));
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
                Self::FunctionNotFound(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::IncorrectFacetCutAction(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitializationFunctionReverted(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidCollateral(element) => ::ethers::core::abi::AbiEncode::encode(element),
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
                Self::ValidatorWeightIsZero(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ValidatorsAndWeightsLengthMismatch(element) => {
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
                    == <ValidatorWeightIsZero as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ValidatorsAndWeightsLengthMismatch as ::ethers::contract::EthError>::selector() => {
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
                Self::FunctionNotFound(element) => ::core::fmt::Display::fmt(element, f),
                Self::IncorrectFacetCutAction(element) => ::core::fmt::Display::fmt(element, f),
                Self::InitializationFunctionReverted(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidCollateral(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidMajorityPercentage(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidSubmissionPeriod(element) => ::core::fmt::Display::fmt(element, f),
                Self::NoBytecodeAtAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::NoSelectorsProvidedForFacetForCut(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::OldConfigurationNumber(element) => ::core::fmt::Display::fmt(element, f),
                Self::ValidatorWeightIsZero(element) => ::core::fmt::Display::fmt(element, f),
                Self::ValidatorsAndWeightsLengthMismatch(element) => {
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
    impl ::core::convert::From<ValidatorWeightIsZero> for GatewayDiamondErrors {
        fn from(value: ValidatorWeightIsZero) -> Self {
            Self::ValidatorWeightIsZero(value)
        }
    }
    impl ::core::convert::From<ValidatorsAndWeightsLengthMismatch> for GatewayDiamondErrors {
        fn from(value: ValidatorsAndWeightsLengthMismatch) -> Self {
            Self::ValidatorsAndWeightsLengthMismatch(value)
        }
    }
}
