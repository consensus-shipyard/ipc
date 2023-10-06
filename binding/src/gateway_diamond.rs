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
            ]),
            receive: true,
            fallback: true,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static GATEWAYDIAMOND_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(__abi);
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = b"`\x80`@R4a\x07\xBCWa\x0Bn\x808\x03\x80a\0\x19\x81a\x08\xF7V[\x92\x839\x81\x01`@\x82\x82\x03\x12a\x07\xBCW\x81Q`\x01`\x01`@\x1B\x03\x81\x11a\x07\xBCW\x82\x01\x81`\x1F\x82\x01\x12\x15a\x07\xBCW\x80Q\x92a\0Ya\0T\x85a\t\x1CV[a\x08\xF7V[\x91` \x83\x86\x81R\x01` \x81\x96`\x05\x1B\x83\x01\x01\x91\x85\x83\x11a\x07\xBCW` \x81\x01\x91[\x83\x83\x10a\x07\xD9WPPPP` \x81\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11a\x07\xBCW\x01\x92`\xC0\x84\x84\x03\x12a\x07\xBCW`@Q\x92`\xC0\x84\x01`\x01`\x01`@\x1B\x03\x81\x11\x85\x82\x10\x17a\x04\x05W`@R\x84Q`\x01`\x01`@\x1B\x03\x81\x11a\x07\xBCW\x85\x01\x90`@\x82\x82\x03\x12a\x07\xBCWa\0\xE6a\x08\xD8V[\x91a\0\xF0\x81a\tGV[\x83R` \x81\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11a\x07\xBCW\x01\x90\x80`\x1F\x83\x01\x12\x15a\x07\xBCW\x81Qa\x01!a\0T\x82a\t\x1CV[\x92` \x80\x85\x84\x81R\x01\x92`\x05\x1B\x82\x01\x01\x92\x83\x11a\x07\xBCW` \x01\x90[\x82\x82\x10a\x07\xC1WPPP` \x82\x01R\x83Ra\x01Z` \x85\x01a\tGV[` \x84\x01Ra\x01k`@\x85\x01a\tGV[\x93`@\x84\x01\x94\x85R`\xA0``\x82\x01Q\x91\x82``\x87\x01R`\x80\x81\x01Q`\x80\x87\x01R\x01Q`\xFF\x81\x16\x81\x03a\x07\xBCW`\xA0\x85\x01R\x15a\x07\xAAW` \x83\x01Q`\x01`\x01`@\x1B\x03\x16\x15a\x07\x98W`\xFF`\xA0\x84\x01Q\x16`3\x81\x10\x90\x81\x15a\x07\x8DW[Pa\x07{W\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD5\x80T`\x01`\x01`\xA0\x1B\x03\x19\x163\x17\x90U`@Q` \x81\x01`\x01`\x01`@\x1B\x03\x81\x11\x82\x82\x10\x17a\x04\x05W`@R`\0\x81R\x82Q`\0[\x81\x81\x10a\x04pWPP`@Q\x92``\x84\x01\x90``\x85RQ\x80\x91R`\x80\x84\x01\x90`\x80\x81`\x05\x1B\x86\x01\x01\x93\x91`\0\x90[\x82\x82\x10a\x04\x1BW\x88\x88\x7F\x8F\xAAp\x87\x86q\xCC\xD2\x12\xD2\x07q\xB7\x95\xC5\n\xF8\xFD?\xF6\xCF'\xF4\xBD\xE5~]M\xE0\xAE\xB6s\x89\x80a\x02\x96\x8B\x8B`\0` \x85\x01R\x83\x82\x03`@\x85\x01Ra\t\xBAV[\x03\x90\xA1\x80Q\x80Q`\x15\x80T`\x01`\x01`@\x1B\x03\x19\x90\x81\x16`\x01`\x01`@\x1B\x03\x93\x84\x16\x17\x90\x91U` \x90\x92\x01Q\x80Q\x90\x94\x92\x93\x92\x91\x81\x11a\x04\x05Wh\x01\0\0\0\0\0\0\0\0\x81\x11a\x04\x05W`\x01\x94`\x16T\x82`\x16U\x80\x83\x10a\x03\xC0W[P` \x01`\x16`\0R` `\0 `\0[\x83\x81\x10a\x03\xA6WPPPP``\x82\x01Q`\x17U` \x82\x01Q`\x1AT\x90h\x01\0\0\0\0\0\0\0\0\x86`\x80\x1B\x03\x90`@\x1B\x16\x90h\x01\0\0\0\0\0\0\0\0\x86`\x80\x1B\x03\x19\x16\x17`\x1AUQ\x90`\xFF`\xA0`\x19T\x92`\x80\x81\x01Q`\x18U\x01Q\x16\x91\x84`\x88\x1B\x85`\xC8\x1B\x03\x90`\x88\x1B\x16\x90x\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\xFF\x19\x16\x17\x17`\x19U`\x0FT\x16\x17`\x0FU`@Qa\x013\x90\x81a\t\xFB\x829\xF3[\x87\x90` \x82\x80`\xA0\x1B\x03\x85Q\x16\x94\x01\x93\x81\x84\x01U\x01a\x03\x03V[`\x16`\0R\x7F\xD83\x14}}\xC3U\xBAE\x9F\xC7\x88\xF6i\xE5\x8C\xFA\xF9\xDC%\xDD\xCD\x07\x02\xE8}i\xC7\xB5\x12B\x89\x90\x81\x01\x90\x83\x01[\x81\x81\x10a\x03\xFAWPa\x02\xF2V[`\0\x81U\x87\x01a\x03\xEDV[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[\x90\x91\x92\x94` \x80a\x04b`\x01\x93`\x7F\x19\x8B\x82\x03\x01\x86R```@\x8BQ\x87\x80`\xA0\x1B\x03\x81Q\x16\x84Ra\x04R\x86\x82\x01Q\x87\x86\x01\x90a\toV[\x01Q\x91\x81`@\x82\x01R\x01\x90a\t|V[\x97\x01\x92\x01\x92\x01\x90\x92\x91a\x02QV[`@a\x04|\x82\x87a\t[V[Q\x01Q`\x01`\x01`\xA0\x1B\x03a\x04\x91\x83\x88a\t[V[QQ\x16\x90\x80Q\x15a\x07bW` a\x04\xA8\x84\x89a\t[V[Q\x01Q`\x03\x81\x10\x15a\x07LW\x80a\x07,WP\x81\x15a\x07\x07Wa\xFF\xFF`\0\x80Q` a\x0B.\x839\x81Q\x91RT\x16\x91`@Q\x80``\x81\x01\x10`\x01\x80`@\x1B\x03``\x83\x01\x11\x17a\x04\x05W``\x81\x01`@R`!\x81R\x7FdiamondCut: Add facet has no cod` \x82\x01R`e`\xF8\x1B`@\x82\x01R\x81;\x15a\x06\xD9WP\x81Q\x91`\0\x93[\x83\x85\x10a\x05MWPPPPP`\x01\x01a\x02#V[a\x05W\x85\x83a\t[V[Q`\x01`\x01`\xE0\x1B\x03\x19\x81\x16`\0\x90\x81R`\0\x80Q` a\x0BN\x839\x81Q\x91R` R`@\x90 T`\x01`\x01`\xA0\x1B\x03\x16a\x06\xB6Wa\x05\x94a\x08\xD8V[\x84\x81Ra\xFF\xFF\x83\x16` \x80\x83\x01\x91\x82R`\x01`\x01`\xE0\x1B\x03\x19\x84\x16`\0\x90\x81R`\0\x80Q` a\x0BN\x839\x81Q\x91R\x90\x91R`@\x90 \x91Q\x82T\x91Q`\x01`\x01`\xB0\x1B\x03\x19\x90\x92\x16`\x01`\x01`\xA0\x1B\x03\x91\x90\x91\x16\x17`\xA0\x91\x90\x91\x1Ba\xFF\xFF`\xA0\x1B\x16\x17\x90U`\0\x80Q` a\x0B.\x839\x81Q\x91RTh\x01\0\0\0\0\0\0\0\0\x81\x10\x15a\x04\x05W`\x01\x81\x01\x80`\0\x80Q` a\x0B.\x839\x81Q\x91RU\x81\x10\x15a\x06\xA0W`\0\x80Q` a\x0B.\x839\x81Q\x91R`\0R` `\0 \x81`\x03\x1C\x01\x91c\xFF\xFF\xFF\xFF\x83T\x91`\xE0\x80\x94`\x05\x1B\x16\x93\x1C\x83\x1B\x92\x1B\x19\x16\x17\x90Ua\xFF\xFF\x80\x82\x16\x14a\x06\x8AW`\x01a\xFF\xFF\x81\x92\x16\x01\x94\x01\x93a\x059V[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[`@Qc\xEB\xBF]\x07`\xE0\x1B\x81R`\x01`\x01`\xE0\x1B\x03\x19\x90\x91\x16`\x04\x82\x01R`$\x90\xFD[\x90a\x07\x03`@Q\x92\x83\x92c\x91\x984\xB9`\xE0\x1B\x84R`\x04\x84\x01R`@`$\x84\x01R`D\x83\x01\x90a\t\xBAV[\x03\x90\xFD[`@Qc\x02\xB8\xDA\x07`\xE2\x1B\x81R` `\x04\x82\x01R\x90\x81\x90a\x07\x03\x90`$\x83\x01\x90a\t|V[`@Qc?\xF4\xD2\x0F`\xE1\x1B\x81R`$\x91a\x07J\x90`\x04\x83\x01\x90a\toV[\xFD[cNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[`@Qc\xE7g\xF9\x1F`\xE0\x1B\x81R`\x04\x81\x01\x83\x90R`$\x90\xFD[`@Qcu\xC3\xB4'`\xE0\x1B\x81R`\x04\x90\xFD[`d\x91P\x118a\x01\xC8V[`@Qc1/\x8E\x05`\xE0\x1B\x81R`\x04\x90\xFD[`@Qch\xF7\xA6u`\xE1\x1B\x81R`\x04\x90\xFD[`\0\x80\xFD[` \x80\x91a\x07\xCE\x84a\t3V[\x81R\x01\x91\x01\x90a\x01=V[\x82Q`\x01`\x01`@\x1B\x03\x81\x11a\x07\xBCW\x82\x01``\x90\x81`\x1F\x19\x82\x8B\x03\x01\x12a\x07\xBCW`@Q\x91\x80\x83\x01`\x01`\x01`@\x1B\x03\x81\x11\x84\x82\x10\x17a\x08\xC3W`@Ra\x08#` \x83\x01a\t3V[\x83R`@\x82\x01Q`\x03\x81\x10\x15a\x07\xBCW` \x84\x01R\x81\x01Q\x90`\x01`\x01`@\x1B\x03\x82\x11a\x07\xBCW\x01\x88`?\x82\x01\x12\x15a\x07\xBCW` \x81\x01Qa\x08ga\0T\x82a\t\x1CV[\x91`@` \x84\x84\x81R\x01\x92`\x05\x1B\x82\x01\x01\x90\x8B\x82\x11a\x07\xBCW`@\x01\x91[\x81\x83\x10a\x08\xA2WPPP`@\x82\x01R\x81R` \x92\x83\x01\x92\x01a\0yV[\x82Q`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x81\x03a\x07\xBCW\x81R` \x92\x83\x01\x92\x01a\x08\x85V[`$`\0cNH{q`\xE0\x1B\x81R`A`\x04R\xFD[`@\x80Q\x91\x90\x82\x01`\x01`\x01`@\x1B\x03\x81\x11\x83\x82\x10\x17a\x04\x05W`@RV[`@Q\x91\x90`\x1F\x01`\x1F\x19\x16\x82\x01`\x01`\x01`@\x1B\x03\x81\x11\x83\x82\x10\x17a\x04\x05W`@RV[`\x01`\x01`@\x1B\x03\x81\x11a\x04\x05W`\x05\x1B` \x01\x90V[Q\x90`\x01`\x01`\xA0\x1B\x03\x82\x16\x82\x03a\x07\xBCWV[Q\x90`\x01`\x01`@\x1B\x03\x82\x16\x82\x03a\x07\xBCWV[\x80Q\x82\x10\x15a\x06\xA0W` \x91`\x05\x1B\x01\x01\x90V[\x90`\x03\x82\x10\x15a\x07LWRV[\x90\x81Q\x80\x82R` \x80\x80\x93\x01\x93\x01\x91`\0[\x82\x81\x10a\t\x9CWPPPP\x90V[\x83Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x93\x81\x01\x93\x92\x81\x01\x92`\x01\x01a\t\x8EV[\x91\x90\x82Q\x92\x83\x82R`\0[\x84\x81\x10a\t\xE6WPP\x82`\0` \x80\x94\x95\x84\x01\x01R`\x1F\x80\x19\x91\x01\x16\x01\x01\x90V[` \x81\x83\x01\x81\x01Q\x84\x83\x01\x82\x01R\x01a\t\xC5V\xFE`\x80`@R6\x15`\x87W`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`oWP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[=\x90\xFD[`$\x90`@Q\x90c\n\x82\xDDs`\xE3\x1B\x82R`\x04\x82\x01R\xFD[`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`\xE9WP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[c\n\x82\xDDs`\xE3\x1B`\x80R`\x84R`$`\x80\xFD\xFE\xA2dipfsX\"\x12 \xED\xD2t\xA8\xE5a\xBE\xB5\xCAGX\xDA\xFE-\x13C\xC5^\xF4A\xDD\xB0\x17\x19\xB8dX(\xFB\x04\xFF\xD8dsolcC\0\x08\x13\x003\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2";
    /// The bytecode of the contract.
    pub static GATEWAYDIAMOND_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__BYTECODE);
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R6\x15`\x87W`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`oWP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[=\x90\xFD[`$\x90`@Q\x90c\n\x82\xDDs`\xE3\x1B\x82R`\x04\x82\x01R\xFD[`\0\x805`\x01`\x01`\xE0\x1B\x03\x19\x16\x80\x82R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@\x82 T`\x01`\x01`\xA0\x1B\x03\x16\x90\x81\x15`\xE9WP\x81\x80\x916\x82\x807\x816\x91Z\xF4=\x82\x80>\x15`kW=\x90\xF3[c\n\x82\xDDs`\xE3\x1B`\x80R`\x84R`$`\x80\xFD\xFE\xA2dipfsX\"\x12 \xED\xD2t\xA8\xE5a\xBE\xB5\xCAGX\xDA\xFE-\x13C\xC5^\xF4A\xDD\xB0\x17\x19\xB8dX(\xFB\x04\xFF\xD8dsolcC\0\x08\x13\x003";
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
}
