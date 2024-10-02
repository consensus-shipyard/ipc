pub use diamond_cut_facet::*;
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
pub mod diamond_cut_facet {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([(
                ::std::borrow::ToOwned::to_owned("diamondCut"),
                ::std::vec![::ethers::core::abi::ethabi::Function {
                    name: ::std::borrow::ToOwned::to_owned("diamondCut"),
                    inputs: ::std::vec![
                        ::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("_diamondCut"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                ::std::boxed::Box::new(
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Address,
                                        ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                        ::ethers::core::abi::ethabi::ParamType::Array(
                                            ::std::boxed::Box::new(
                                                ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                                    4usize
                                                ),
                                            ),
                                        ),
                                    ],),
                                ),
                            ),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("struct IDiamond.FacetCut[]",),
                            ),
                        },
                        ::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("_init"),
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
                    outputs: ::std::vec![],
                    constant: ::core::option::Option::None,
                    state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                },],
            )]),
            events: ::core::convert::From::from([(
                ::std::borrow::ToOwned::to_owned("DiamondCut"),
                ::std::vec![::ethers::core::abi::ethabi::Event {
                    name: ::std::borrow::ToOwned::to_owned("DiamondCut"),
                    inputs: ::std::vec![
                        ::ethers::core::abi::ethabi::EventParam {
                            name: ::std::borrow::ToOwned::to_owned("_diamondCut"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                ::std::boxed::Box::new(
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Address,
                                        ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                        ::ethers::core::abi::ethabi::ParamType::Array(
                                            ::std::boxed::Box::new(
                                                ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                                    4usize
                                                ),
                                            ),
                                        ),
                                    ],),
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
                },],
            )]),
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
                    ::std::borrow::ToOwned::to_owned("CannotRemoveFunctionThatDoesNotExist"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned(
                            "CannotRemoveFunctionThatDoesNotExist",
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
                    ::std::borrow::ToOwned::to_owned("CannotRemoveImmutableFunction"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("CannotRemoveImmutableFunction",),
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
                    ::std::borrow::ToOwned::to_owned("CannotReplaceFunctionThatDoesNotExists"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned(
                            "CannotReplaceFunctionThatDoesNotExists",
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
                    ::std::borrow::ToOwned::to_owned(
                        "CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet",
                    ),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned(
                            "CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet",
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
                    ::std::borrow::ToOwned::to_owned(
                        "CannotReplaceFunctionsFromFacetWithZeroAddress",
                    ),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned(
                            "CannotReplaceFunctionsFromFacetWithZeroAddress",
                        ),
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
                    ::std::borrow::ToOwned::to_owned("CannotReplaceImmutableFunction"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("CannotReplaceImmutableFunction",),
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
                    ::std::borrow::ToOwned::to_owned("NotOwner"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("NotOwner"),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("RemoveFacetAddressMustBeZeroAddress"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned(
                            "RemoveFacetAddressMustBeZeroAddress",
                        ),
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
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static DIAMONDCUTFACET_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(__abi);
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = b"`\x80\x80`@R4`\x15Wa\x0C\xEC\x90\x81a\0\x1B\x829\xF3[`\0\x80\xFD\xFE`\x80`@R`\x046\x10\x15a\0\x12W`\0\x80\xFD[`\x005`\xE0\x1Cc\x1F\x93\x1C\x1C\x14a\0'W`\0\x80\xFD[4a\t\xBDW``6`\x03\x19\x01\x12a\t\xBDW`\x045g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\t\xBDW6`#\x82\x01\x12\x15a\t\xBDW\x80`\x04\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\t\xBDW`$\x81`\x05\x1B\x83\x01\x016\x81\x11a\t\xBDW`$5`\x01`\x01`\xA0\x1B\x03\x81\x16\x91\x82\x82\x03a\t\xBDW`D5\x92g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x84\x11a\t\xBDW6`#\x85\x01\x12\x15a\t\xBDW\x83`\x04\x015\x95g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x87\x11a\t\xBDW6`$\x88\x87\x01\x01\x11a\t\xBDW\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD5T`\x01`\x01`\xA0\x1B\x03\x163\x03a\t\xACW` a\x01\x11a\x01\x0C\x88a\n\x1EV[a\t\xF8V[\x80\x97\x81R\x01\x96\x87\x93`\0\x98\x83`$\x8B\x95\x01\x91[\x83\x83\x10a\x08hWPPPP\x80\x80`$a\x01Aa\x01\x0C` \x95a\n6V[\x98\x82\x8AR\x01\x83\x89\x017\x86\x01\x01R\x84Q\x86[\x81\x81\x10a\x02xWPP`@Q\x94``\x86\x01\x90``\x87RQ\x80\x91R`\x80\x86\x01\x90`\x80\x81`\x05\x1B\x88\x01\x01\x93\x91\x88\x90[\x82\x82\x10a\x01\xD3WPPPP\x93\x80a\x01\xC8\x7F\x8F\xAAp\x87\x86q\xCC\xD2\x12\xD2\x07q\xB7\x95\xC5\n\xF8\xFD?\xF6\xCF'\xF4\xBD\xE5~]M\xE0\xAE\xB6s\x93a\x01\xD0\x97` \x84\x01R\x82\x81\x03`@\x84\x01R\x86a\n|V[\x03\x90\xA1a\x0B\\V[\x80\xF3[\x90\x91\x92\x94`\x7F\x19\x89\x82\x03\x01\x82R\x85Q``\x82\x01\x90`\x01\x80`\xA0\x1B\x03\x81Q\x16\x83R` \x81\x01Q`\x03\x81\x10\x15a\x02dW`@` \x92`\x80\x92\x84\x87\x01R\x01Q\x93```@\x82\x01R\x84Q\x80\x94R\x01\x92\x01\x90\x8B\x90[\x80\x82\x10a\x02AWPPP` \x80`\x01\x92\x97\x01\x92\x01\x92\x01\x90\x92\x91a\x01\x7FV[\x82Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x84R` \x93\x84\x01\x93\x90\x92\x01\x91`\x01\x90\x91\x01\x90a\x02#V[cNH{q`\xE0\x1B\x8DR`!`\x04R`$\x8D\xFD[`@a\x02\x84\x82\x89a\nRV[Q\x01Q`\x01`\x01`\xA0\x1B\x03a\x02\x99\x83\x8Aa\nRV[QQ\x16\x90\x80Q\x15a\x08TW` a\x02\xB0\x84\x8Ba\nRV[Q\x01Q`\x03\x81\x10\x80\x15a\x08@W\x81a\x04\xC2WPP\x81\x15a\x04\xA1Wa\xFF\xFF`\0\x80Q` a\x0Cw\x839\x81Q\x91RT\x16\x91a\x03&a\x02\xEC``a\t\xF8V[`!\x81R\x7FdiamondCut: Add facet has no cod` \x82\x01R`e`\xF8\x1B`@\x82\x01R\x82a\x0C7V[\x81Q\x91\x8B\x80\x94[\x84\x86\x10a\x03DWPPPPPP`\x01\x90[\x01a\x01RV[`\x01`\x01`\xE0\x1B\x03\x19a\x03W\x87\x85a\nRV[Q\x16\x91\x82\x90R`\0\x80Q` a\x0C\x97\x839\x81Q\x91R` R`@\x8E T`\x01`\x01`\xA0\x1B\x03\x16a\x04\x8DW\x90a\x03\xEF\x8E\x92a\xFF\xFFa\x03\x92a\t\xC2V[\x87\x81R\x91\x81\x16` \x80\x84\x01\x82\x81R\x86\x88R`\0\x80Q` a\x0C\x97\x839\x81Q\x91R\x90\x91R`@\x90\x96 \x92Q\x83T\x96Q`\x01`\x01`\xB0\x1B\x03\x19\x90\x97\x16`\x01`\x01`\xA0\x1B\x03\x91\x90\x91\x16\x17\x95\x90\x91\x16`\xA0\x1Ba\xFF\xFF`\xA0\x1B\x16\x94\x90\x94\x17\x90UV[`\0\x80Q` a\x0Cw\x839\x81Q\x91RTh\x01\0\0\0\0\0\0\0\0\x81\x10\x15a\x04yW\x90a\x041\x82`\x01a\x04N\x94\x01`\0\x80Q` a\x0Cw\x839\x81Q\x91RUa\n\xBDV[\x90\x91\x90c\xFF\xFF\xFF\xFF\x83T\x91`\x03\x1B\x92`\xE0\x1C\x83\x1B\x92\x1B\x19\x16\x17\x90UV[a\xFF\xFF\x81\x14a\x04eW`\x01\x94\x85\x01\x94\x8D\x91\x01a\x03-V[cNH{q`\xE0\x1B\x8DR`\x11`\x04R`$\x8D\xFD[cNH{q`\xE0\x1B\x8FR`A`\x04R`$\x8F\xFD[c\xEB\xBF]\x07`\xE0\x1B\x8ER`\x04\x82\x90R`$\x8E\xFD[`@Qc\x02\xB8\xDA\x07`\xE2\x1B\x81R\x90\x81\x90a\x04\xBE\x90`\x04\x83\x01a\x0B\x18V[\x03\x90\xFD[`\x01\x82\x03a\x06\x18WPP\x81\x15a\x05\xFBWa\x05 a\x04\xDF``a\t\xF8V[`(\x81R\x7FLibDiamondCut: Replace facet has` \x82\x01Rg no code`\xC0\x1B`@\x82\x01R\x83a\x0C7V[\x80Q\x90`\x01`\x01`\xA0\x1B\x03\x83\x16\x90\x8B[\x83\x81\x10a\x05DWPPPPP`\x01\x90a\x03>V[\x8C`\x01`\x01`\xE0\x1B\x03\x19a\x05X\x83\x85a\nRV[Q\x16\x90\x81\x90R`\0\x80Q` a\x0C\x97\x839\x81Q\x91R` R`@\x8E T`\x01`\x01`\xA0\x1B\x03\x160\x81\x14a\x05\xE7W\x86\x81\x14a\x05\xD3W\x15a\x05\xC1W\x8DR`\0\x80Q` a\x0C\x97\x839\x81Q\x91R` R`@\x8D \x80T`\x01`\x01`\xA0\x1B\x03\x19\x16\x84\x17\x90U`\x01\x01a\x050V[cty\xF99`\xE0\x1B\x8ER`\x04R`$\x8D\xFD[c\x1A\xC6\xCE\x8D`\xE1\x1B\x8FR`\x04\x82\x90R`$\x8F\xFD[c)\x01\x80m`\xE1\x1B\x8FR`\x04\x82\x90R`$\x8F\xFD[`@Qc\xCD\x98\xA9o`\xE0\x1B\x81R\x90\x81\x90a\x04\xBE\x90`\x04\x83\x01a\x0B\x18V[`\x02\x82\x03a\x08\x15WPP`\0\x80Q` a\x0Cw\x839\x81Q\x91RT\x91\x80a\x08\x03WP\x80Q\x90\x8A[\x82\x81\x10a\x06QWPPPP`\x01\x90a\x03>V[`\x01`\x01`\xE0\x1B\x03\x19a\x06d\x82\x84a\nRV[Q\x16\x80\x8DR`\0\x80Q` a\x0C\x97\x839\x81Q\x91R` R`@\x8D \x94a\x06\x88a\t\xC2V[\x95T\x95`\x01\x80`\xA0\x1B\x03\x87\x16\x80\x82Ra\xFF\xFF` \x83\x01\x98`\xA0\x1C\x16\x88R\x15a\x07\xEFWQ`\x01`\x01`\xA0\x1B\x03\x160\x14a\x07\xDBW\x80\x15a\x07\xC7W\x8D\x90`\0\x19\x01\x95\x86a\xFF\xFF\x82Q\x16\x03a\x07RW[PP`\0\x80Q` a\x0Cw\x839\x81Q\x91RT\x80\x15a\x07>W`\x01\x92\x91\x90`\0\x19\x01a\x06\xFE\x81a\n\xBDV[c\xFF\xFF\xFF\xFF\x82T\x91`\x03\x1B\x1B\x19\x16\x90U`\0\x80Q` a\x0Cw\x839\x81Q\x91RU\x8DR`\0\x80Q` a\x0C\x97\x839\x81Q\x91R` R\x8C`@\x81 U\x01a\x06>V[cNH{q`\xE0\x1B\x8ER`1`\x04R`$\x8E\xFD[`@a\x07\xC0\x92a\xFF\xFFa\x07d\x8Aa\n\xBDV[\x90T\x90`\x03\x1B\x1C`\xE0\x1B\x93a\x07\x7F\x85a\x041\x84\x84Q\x16a\n\xBDV[Q`\x01`\x01`\xE0\x1B\x03\x19\x90\x94\x16\x82R`\0\x80Q` a\x0C\x97\x839\x81Q\x91R` R\x91\x90 \x80Ta\xFF\xFF`\xA0\x1B\x19\x16\x91\x90\x92\x16`\xA0\x1Ba\xFF\xFF`\xA0\x1B\x16\x17\x90UV[\x8C8a\x06\xD4V[cNH{q`\xE0\x1B\x8ER`\x11`\x04R`$\x8E\xFD[c\r\xF5\xFDa`\xE3\x1B\x8ER`\x04\x82\x90R`$\x8E\xFD[cz\x08\xA2-`\xE0\x1B\x8FR`\x04\x83\x90R`$\x8F\xFD[c\xD0\x91\xBC\x81`\xE0\x1B\x8BR`\x04R`$\x8A\xFD[c?\xF4\xD2\x0F`\xE1\x1B\x8CR`$\x91\x8C\x91\x15a\x08.W`\x04R\xFD[PcNH{q`\xE0\x1B\x81R`!`\x04R\xFD[cNH{q`\xE0\x1B\x8CR`!`\x04R`$\x8C\xFD[c\xE7g\xF9\x1F`\xE0\x1B\x8AR`\x04\x82\x90R`$\x8A\xFD[\x90\x91\x80\x93\x94\x95P5g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\t\xA8W\x82\x01```#\x19\x826\x03\x01\x12a\t\xA8W`@Q\x90``\x82\x01\x82\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\t\x94W`@R`$\x81\x015`\x01`\x01`\xA0\x1B\x03\x81\x16\x81\x03a\t\x90W\x82R`D\x81\x015`\x03\x81\x10\x15a\t\x90W` \x83\x01R`d\x81\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\t\x90W`$\x91\x01\x016`\x1F\x82\x01\x12\x15a\t\x8CW\x805a\t\ta\x01\x0C\x82a\n\x1EV[\x91` \x80\x84\x84\x81R\x01\x92`\x05\x1B\x82\x01\x01\x906\x82\x11a\t\x88W\x90\x8F\x99\x98\x97\x96\x95\x94\x93\x92\x91` \x01\x91[\x81\x83\x10a\tNWPPP`@\x82\x01R\x81R` \x92\x83\x01\x92\x01a\x01$V[\x90\x91\x80\x93\x94\x95\x96\x97\x98\x99\x9AP5c\xFF\xFF\xFF\xFF`\xE0\x1B\x81\x16\x81\x03a\t\x83W\x81R\x8F\x99\x98\x97\x96\x95\x94\x93\x92` \x90\x81\x01\x92\x91\x01a\t1V[P\x8F\x80\xFD[\x8F\x80\xFD[\x8C\x80\xFD[\x8D\x80\xFD[cNH{q`\xE0\x1B\x8ER`A`\x04R`$\x8E\xFD[\x8B\x80\xFD[c0\xCDtq`\xE0\x1B`\0R`\x04`\0\xFD[`\0\x80\xFD[`@Q\x90`@\x82\x01\x82\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\t\xE2W`@RV[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[`@Q\x91\x90`\x1F\x01`\x1F\x19\x16\x82\x01g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x83\x82\x10\x17a\t\xE2W`@RV[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\t\xE2W`\x05\x1B` \x01\x90V[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\t\xE2W`\x1F\x01`\x1F\x19\x16` \x01\x90V[\x80Q\x82\x10\x15a\nfW` \x91`\x05\x1B\x01\x01\x90V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[\x91\x90\x82Q\x92\x83\x82R`\0[\x84\x81\x10a\n\xA8WPP\x82`\0` \x80\x94\x95\x84\x01\x01R`\x1F\x80\x19\x91\x01\x16\x01\x01\x90V[\x80` \x80\x92\x84\x01\x01Q\x82\x82\x86\x01\x01R\x01a\n\x87V[\x90`\0\x80Q` a\x0Cw\x839\x81Q\x91RT\x82\x10\x15a\nfW`\0\x80Q` a\x0Cw\x839\x81Q\x91R`\0R`\x03\x82\x90\x1C\x7F\xB6[\xEC\xA8\xB6\xFAx\x8B\xCB\x15(\xC2\xAB_M\xC6\xBC\x98\xE5\x89eP\xBA\xA0\x13\xD83\x0F\xAB\x0B\x86\xF4\x01\x91`\x02\x1B`\x1C\x16\x90V[` `@\x81\x83\x01\x92\x82\x81R\x84Q\x80\x94R\x01\x92\x01\x90`\0[\x81\x81\x10a\x0B<WPPP\x90V[\x82Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x84R` \x93\x84\x01\x93\x90\x92\x01\x91`\x01\x01a\x0B/V[`\x01`\x01`\xA0\x1B\x03\x81\x16\x91\x90\x82\x15a\x0C2W`\0\x80\x91a\x0B\xBDa\x0B\x7F``a\t\xF8V[`%\x81R\x7FdiamondCut: _init address has no` \x82\x01Rd code`\xD8\x1B`@\x82\x01R\x82a\x0C7V[\x83Q\x90` \x85\x01\x90Z\xF4\x91=\x15a\x0C*W=\x92a\x0B\xDCa\x01\x0C\x85a\n6V[\x93\x84R=`\0` \x86\x01>[\x15a\x0B\xF2WPPPV[\x82Q\x15a\x0C\x01W\x82Q` \x84\x01\xFD[a\x04\xBE`@Q\x92\x83\x92c\x19!\x05\xD7`\xE0\x1B\x84R`\x04\x84\x01R`@`$\x84\x01R`D\x83\x01\x90a\n|V[``\x92a\x0B\xE8V[PPPV[\x80;\x15a\x0CBWPPV[`@\x80Qc\x91\x984\xB9`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x90\x92\x16`\x04\x83\x01R`$\x82\x01R\x90\x81\x90a\x04\xBE\x90`D\x83\x01\x90a\n|V\xFE\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2\xA2dipfsX\"\x12 l\xE5\x90}\xDB\x84\x12a\xE1;u\xB6Xfz\x824?/\x0F\xD3\x11\xD0X\xD3\xB8\x12J\xE8)c\xA2dsolcC\0\x08\x1A\x003";
    /// The bytecode of the contract.
    pub static DIAMONDCUTFACET_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__BYTECODE);
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R`\x046\x10\x15a\0\x12W`\0\x80\xFD[`\x005`\xE0\x1Cc\x1F\x93\x1C\x1C\x14a\0'W`\0\x80\xFD[4a\t\xBDW``6`\x03\x19\x01\x12a\t\xBDW`\x045g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\t\xBDW6`#\x82\x01\x12\x15a\t\xBDW\x80`\x04\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\t\xBDW`$\x81`\x05\x1B\x83\x01\x016\x81\x11a\t\xBDW`$5`\x01`\x01`\xA0\x1B\x03\x81\x16\x91\x82\x82\x03a\t\xBDW`D5\x92g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x84\x11a\t\xBDW6`#\x85\x01\x12\x15a\t\xBDW\x83`\x04\x015\x95g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x87\x11a\t\xBDW6`$\x88\x87\x01\x01\x11a\t\xBDW\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD5T`\x01`\x01`\xA0\x1B\x03\x163\x03a\t\xACW` a\x01\x11a\x01\x0C\x88a\n\x1EV[a\t\xF8V[\x80\x97\x81R\x01\x96\x87\x93`\0\x98\x83`$\x8B\x95\x01\x91[\x83\x83\x10a\x08hWPPPP\x80\x80`$a\x01Aa\x01\x0C` \x95a\n6V[\x98\x82\x8AR\x01\x83\x89\x017\x86\x01\x01R\x84Q\x86[\x81\x81\x10a\x02xWPP`@Q\x94``\x86\x01\x90``\x87RQ\x80\x91R`\x80\x86\x01\x90`\x80\x81`\x05\x1B\x88\x01\x01\x93\x91\x88\x90[\x82\x82\x10a\x01\xD3WPPPP\x93\x80a\x01\xC8\x7F\x8F\xAAp\x87\x86q\xCC\xD2\x12\xD2\x07q\xB7\x95\xC5\n\xF8\xFD?\xF6\xCF'\xF4\xBD\xE5~]M\xE0\xAE\xB6s\x93a\x01\xD0\x97` \x84\x01R\x82\x81\x03`@\x84\x01R\x86a\n|V[\x03\x90\xA1a\x0B\\V[\x80\xF3[\x90\x91\x92\x94`\x7F\x19\x89\x82\x03\x01\x82R\x85Q``\x82\x01\x90`\x01\x80`\xA0\x1B\x03\x81Q\x16\x83R` \x81\x01Q`\x03\x81\x10\x15a\x02dW`@` \x92`\x80\x92\x84\x87\x01R\x01Q\x93```@\x82\x01R\x84Q\x80\x94R\x01\x92\x01\x90\x8B\x90[\x80\x82\x10a\x02AWPPP` \x80`\x01\x92\x97\x01\x92\x01\x92\x01\x90\x92\x91a\x01\x7FV[\x82Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x84R` \x93\x84\x01\x93\x90\x92\x01\x91`\x01\x90\x91\x01\x90a\x02#V[cNH{q`\xE0\x1B\x8DR`!`\x04R`$\x8D\xFD[`@a\x02\x84\x82\x89a\nRV[Q\x01Q`\x01`\x01`\xA0\x1B\x03a\x02\x99\x83\x8Aa\nRV[QQ\x16\x90\x80Q\x15a\x08TW` a\x02\xB0\x84\x8Ba\nRV[Q\x01Q`\x03\x81\x10\x80\x15a\x08@W\x81a\x04\xC2WPP\x81\x15a\x04\xA1Wa\xFF\xFF`\0\x80Q` a\x0Cw\x839\x81Q\x91RT\x16\x91a\x03&a\x02\xEC``a\t\xF8V[`!\x81R\x7FdiamondCut: Add facet has no cod` \x82\x01R`e`\xF8\x1B`@\x82\x01R\x82a\x0C7V[\x81Q\x91\x8B\x80\x94[\x84\x86\x10a\x03DWPPPPPP`\x01\x90[\x01a\x01RV[`\x01`\x01`\xE0\x1B\x03\x19a\x03W\x87\x85a\nRV[Q\x16\x91\x82\x90R`\0\x80Q` a\x0C\x97\x839\x81Q\x91R` R`@\x8E T`\x01`\x01`\xA0\x1B\x03\x16a\x04\x8DW\x90a\x03\xEF\x8E\x92a\xFF\xFFa\x03\x92a\t\xC2V[\x87\x81R\x91\x81\x16` \x80\x84\x01\x82\x81R\x86\x88R`\0\x80Q` a\x0C\x97\x839\x81Q\x91R\x90\x91R`@\x90\x96 \x92Q\x83T\x96Q`\x01`\x01`\xB0\x1B\x03\x19\x90\x97\x16`\x01`\x01`\xA0\x1B\x03\x91\x90\x91\x16\x17\x95\x90\x91\x16`\xA0\x1Ba\xFF\xFF`\xA0\x1B\x16\x94\x90\x94\x17\x90UV[`\0\x80Q` a\x0Cw\x839\x81Q\x91RTh\x01\0\0\0\0\0\0\0\0\x81\x10\x15a\x04yW\x90a\x041\x82`\x01a\x04N\x94\x01`\0\x80Q` a\x0Cw\x839\x81Q\x91RUa\n\xBDV[\x90\x91\x90c\xFF\xFF\xFF\xFF\x83T\x91`\x03\x1B\x92`\xE0\x1C\x83\x1B\x92\x1B\x19\x16\x17\x90UV[a\xFF\xFF\x81\x14a\x04eW`\x01\x94\x85\x01\x94\x8D\x91\x01a\x03-V[cNH{q`\xE0\x1B\x8DR`\x11`\x04R`$\x8D\xFD[cNH{q`\xE0\x1B\x8FR`A`\x04R`$\x8F\xFD[c\xEB\xBF]\x07`\xE0\x1B\x8ER`\x04\x82\x90R`$\x8E\xFD[`@Qc\x02\xB8\xDA\x07`\xE2\x1B\x81R\x90\x81\x90a\x04\xBE\x90`\x04\x83\x01a\x0B\x18V[\x03\x90\xFD[`\x01\x82\x03a\x06\x18WPP\x81\x15a\x05\xFBWa\x05 a\x04\xDF``a\t\xF8V[`(\x81R\x7FLibDiamondCut: Replace facet has` \x82\x01Rg no code`\xC0\x1B`@\x82\x01R\x83a\x0C7V[\x80Q\x90`\x01`\x01`\xA0\x1B\x03\x83\x16\x90\x8B[\x83\x81\x10a\x05DWPPPPP`\x01\x90a\x03>V[\x8C`\x01`\x01`\xE0\x1B\x03\x19a\x05X\x83\x85a\nRV[Q\x16\x90\x81\x90R`\0\x80Q` a\x0C\x97\x839\x81Q\x91R` R`@\x8E T`\x01`\x01`\xA0\x1B\x03\x160\x81\x14a\x05\xE7W\x86\x81\x14a\x05\xD3W\x15a\x05\xC1W\x8DR`\0\x80Q` a\x0C\x97\x839\x81Q\x91R` R`@\x8D \x80T`\x01`\x01`\xA0\x1B\x03\x19\x16\x84\x17\x90U`\x01\x01a\x050V[cty\xF99`\xE0\x1B\x8ER`\x04R`$\x8D\xFD[c\x1A\xC6\xCE\x8D`\xE1\x1B\x8FR`\x04\x82\x90R`$\x8F\xFD[c)\x01\x80m`\xE1\x1B\x8FR`\x04\x82\x90R`$\x8F\xFD[`@Qc\xCD\x98\xA9o`\xE0\x1B\x81R\x90\x81\x90a\x04\xBE\x90`\x04\x83\x01a\x0B\x18V[`\x02\x82\x03a\x08\x15WPP`\0\x80Q` a\x0Cw\x839\x81Q\x91RT\x91\x80a\x08\x03WP\x80Q\x90\x8A[\x82\x81\x10a\x06QWPPPP`\x01\x90a\x03>V[`\x01`\x01`\xE0\x1B\x03\x19a\x06d\x82\x84a\nRV[Q\x16\x80\x8DR`\0\x80Q` a\x0C\x97\x839\x81Q\x91R` R`@\x8D \x94a\x06\x88a\t\xC2V[\x95T\x95`\x01\x80`\xA0\x1B\x03\x87\x16\x80\x82Ra\xFF\xFF` \x83\x01\x98`\xA0\x1C\x16\x88R\x15a\x07\xEFWQ`\x01`\x01`\xA0\x1B\x03\x160\x14a\x07\xDBW\x80\x15a\x07\xC7W\x8D\x90`\0\x19\x01\x95\x86a\xFF\xFF\x82Q\x16\x03a\x07RW[PP`\0\x80Q` a\x0Cw\x839\x81Q\x91RT\x80\x15a\x07>W`\x01\x92\x91\x90`\0\x19\x01a\x06\xFE\x81a\n\xBDV[c\xFF\xFF\xFF\xFF\x82T\x91`\x03\x1B\x1B\x19\x16\x90U`\0\x80Q` a\x0Cw\x839\x81Q\x91RU\x8DR`\0\x80Q` a\x0C\x97\x839\x81Q\x91R` R\x8C`@\x81 U\x01a\x06>V[cNH{q`\xE0\x1B\x8ER`1`\x04R`$\x8E\xFD[`@a\x07\xC0\x92a\xFF\xFFa\x07d\x8Aa\n\xBDV[\x90T\x90`\x03\x1B\x1C`\xE0\x1B\x93a\x07\x7F\x85a\x041\x84\x84Q\x16a\n\xBDV[Q`\x01`\x01`\xE0\x1B\x03\x19\x90\x94\x16\x82R`\0\x80Q` a\x0C\x97\x839\x81Q\x91R` R\x91\x90 \x80Ta\xFF\xFF`\xA0\x1B\x19\x16\x91\x90\x92\x16`\xA0\x1Ba\xFF\xFF`\xA0\x1B\x16\x17\x90UV[\x8C8a\x06\xD4V[cNH{q`\xE0\x1B\x8ER`\x11`\x04R`$\x8E\xFD[c\r\xF5\xFDa`\xE3\x1B\x8ER`\x04\x82\x90R`$\x8E\xFD[cz\x08\xA2-`\xE0\x1B\x8FR`\x04\x83\x90R`$\x8F\xFD[c\xD0\x91\xBC\x81`\xE0\x1B\x8BR`\x04R`$\x8A\xFD[c?\xF4\xD2\x0F`\xE1\x1B\x8CR`$\x91\x8C\x91\x15a\x08.W`\x04R\xFD[PcNH{q`\xE0\x1B\x81R`!`\x04R\xFD[cNH{q`\xE0\x1B\x8CR`!`\x04R`$\x8C\xFD[c\xE7g\xF9\x1F`\xE0\x1B\x8AR`\x04\x82\x90R`$\x8A\xFD[\x90\x91\x80\x93\x94\x95P5g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\t\xA8W\x82\x01```#\x19\x826\x03\x01\x12a\t\xA8W`@Q\x90``\x82\x01\x82\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\t\x94W`@R`$\x81\x015`\x01`\x01`\xA0\x1B\x03\x81\x16\x81\x03a\t\x90W\x82R`D\x81\x015`\x03\x81\x10\x15a\t\x90W` \x83\x01R`d\x81\x015g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\t\x90W`$\x91\x01\x016`\x1F\x82\x01\x12\x15a\t\x8CW\x805a\t\ta\x01\x0C\x82a\n\x1EV[\x91` \x80\x84\x84\x81R\x01\x92`\x05\x1B\x82\x01\x01\x906\x82\x11a\t\x88W\x90\x8F\x99\x98\x97\x96\x95\x94\x93\x92\x91` \x01\x91[\x81\x83\x10a\tNWPPP`@\x82\x01R\x81R` \x92\x83\x01\x92\x01a\x01$V[\x90\x91\x80\x93\x94\x95\x96\x97\x98\x99\x9AP5c\xFF\xFF\xFF\xFF`\xE0\x1B\x81\x16\x81\x03a\t\x83W\x81R\x8F\x99\x98\x97\x96\x95\x94\x93\x92` \x90\x81\x01\x92\x91\x01a\t1V[P\x8F\x80\xFD[\x8F\x80\xFD[\x8C\x80\xFD[\x8D\x80\xFD[cNH{q`\xE0\x1B\x8ER`A`\x04R`$\x8E\xFD[\x8B\x80\xFD[c0\xCDtq`\xE0\x1B`\0R`\x04`\0\xFD[`\0\x80\xFD[`@Q\x90`@\x82\x01\x82\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\t\xE2W`@RV[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[`@Q\x91\x90`\x1F\x01`\x1F\x19\x16\x82\x01g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x83\x82\x10\x17a\t\xE2W`@RV[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\t\xE2W`\x05\x1B` \x01\x90V[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\t\xE2W`\x1F\x01`\x1F\x19\x16` \x01\x90V[\x80Q\x82\x10\x15a\nfW` \x91`\x05\x1B\x01\x01\x90V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[\x91\x90\x82Q\x92\x83\x82R`\0[\x84\x81\x10a\n\xA8WPP\x82`\0` \x80\x94\x95\x84\x01\x01R`\x1F\x80\x19\x91\x01\x16\x01\x01\x90V[\x80` \x80\x92\x84\x01\x01Q\x82\x82\x86\x01\x01R\x01a\n\x87V[\x90`\0\x80Q` a\x0Cw\x839\x81Q\x91RT\x82\x10\x15a\nfW`\0\x80Q` a\x0Cw\x839\x81Q\x91R`\0R`\x03\x82\x90\x1C\x7F\xB6[\xEC\xA8\xB6\xFAx\x8B\xCB\x15(\xC2\xAB_M\xC6\xBC\x98\xE5\x89eP\xBA\xA0\x13\xD83\x0F\xAB\x0B\x86\xF4\x01\x91`\x02\x1B`\x1C\x16\x90V[` `@\x81\x83\x01\x92\x82\x81R\x84Q\x80\x94R\x01\x92\x01\x90`\0[\x81\x81\x10a\x0B<WPPP\x90V[\x82Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x84R` \x93\x84\x01\x93\x90\x92\x01\x91`\x01\x01a\x0B/V[`\x01`\x01`\xA0\x1B\x03\x81\x16\x91\x90\x82\x15a\x0C2W`\0\x80\x91a\x0B\xBDa\x0B\x7F``a\t\xF8V[`%\x81R\x7FdiamondCut: _init address has no` \x82\x01Rd code`\xD8\x1B`@\x82\x01R\x82a\x0C7V[\x83Q\x90` \x85\x01\x90Z\xF4\x91=\x15a\x0C*W=\x92a\x0B\xDCa\x01\x0C\x85a\n6V[\x93\x84R=`\0` \x86\x01>[\x15a\x0B\xF2WPPPV[\x82Q\x15a\x0C\x01W\x82Q` \x84\x01\xFD[a\x04\xBE`@Q\x92\x83\x92c\x19!\x05\xD7`\xE0\x1B\x84R`\x04\x84\x01R`@`$\x84\x01R`D\x83\x01\x90a\n|V[``\x92a\x0B\xE8V[PPPV[\x80;\x15a\x0CBWPPV[`@\x80Qc\x91\x984\xB9`\xE0\x1B\x81R`\x01`\x01`\xA0\x1B\x03\x90\x92\x16`\x04\x83\x01R`$\x82\x01R\x90\x81\x90a\x04\xBE\x90`D\x83\x01\x90a\n|V\xFE\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2\xA2dipfsX\"\x12 l\xE5\x90}\xDB\x84\x12a\xE1;u\xB6Xfz\x824?/\x0F\xD3\x11\xD0X\xD3\xB8\x12J\xE8)c\xA2dsolcC\0\x08\x1A\x003";
    /// The deployed bytecode of the contract.
    pub static DIAMONDCUTFACET_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__DEPLOYED_BYTECODE);
    pub struct DiamondCutFacet<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for DiamondCutFacet<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for DiamondCutFacet<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for DiamondCutFacet<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for DiamondCutFacet<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(DiamondCutFacet))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> DiamondCutFacet<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                DIAMONDCUTFACET_ABI.clone(),
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
                DIAMONDCUTFACET_ABI.clone(),
                DIAMONDCUTFACET_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Calls the contract's `diamondCut` (0x1f931c1c) function
        pub fn diamond_cut(
            &self,
            diamond_cut: ::std::vec::Vec<FacetCut>,
            init: ::ethers::core::types::Address,
            calldata: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([31, 147, 28, 28], (diamond_cut, init, calldata))
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `DiamondCut` event
        pub fn diamond_cut_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, DiamondCutFilter> {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, DiamondCutFilter> {
            self.0
                .event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
        for DiamondCutFacet<M>
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
    ///Custom Error type `NotOwner` with signature `NotOwner()` and selector `0x30cd7471`
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
    #[etherror(name = "NotOwner", abi = "NotOwner()")]
    pub struct NotOwner;
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
    pub enum DiamondCutFacetErrors {
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
        IncorrectFacetCutAction(IncorrectFacetCutAction),
        InitializationFunctionReverted(InitializationFunctionReverted),
        NoBytecodeAtAddress(NoBytecodeAtAddress),
        NoSelectorsProvidedForFacetForCut(NoSelectorsProvidedForFacetForCut),
        NotOwner(NotOwner),
        RemoveFacetAddressMustBeZeroAddress(RemoveFacetAddressMustBeZeroAddress),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for DiamondCutFacetErrors {
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
            if let Ok(decoded) = <NotOwner as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NotOwner(decoded));
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
    impl ::ethers::core::abi::AbiEncode for DiamondCutFacetErrors {
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
                Self::NotOwner(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::RemoveFacetAddressMustBeZeroAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for DiamondCutFacetErrors {
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
                    == <NotOwner as ::ethers::contract::EthError>::selector() => true,
                _ if selector
                    == <RemoveFacetAddressMustBeZeroAddress as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for DiamondCutFacetErrors {
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
                Self::IncorrectFacetCutAction(element) => ::core::fmt::Display::fmt(element, f),
                Self::InitializationFunctionReverted(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NoBytecodeAtAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::NoSelectorsProvidedForFacetForCut(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotOwner(element) => ::core::fmt::Display::fmt(element, f),
                Self::RemoveFacetAddressMustBeZeroAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for DiamondCutFacetErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<CannotAddFunctionToDiamondThatAlreadyExists> for DiamondCutFacetErrors {
        fn from(value: CannotAddFunctionToDiamondThatAlreadyExists) -> Self {
            Self::CannotAddFunctionToDiamondThatAlreadyExists(value)
        }
    }
    impl ::core::convert::From<CannotAddSelectorsToZeroAddress> for DiamondCutFacetErrors {
        fn from(value: CannotAddSelectorsToZeroAddress) -> Self {
            Self::CannotAddSelectorsToZeroAddress(value)
        }
    }
    impl ::core::convert::From<CannotRemoveFunctionThatDoesNotExist> for DiamondCutFacetErrors {
        fn from(value: CannotRemoveFunctionThatDoesNotExist) -> Self {
            Self::CannotRemoveFunctionThatDoesNotExist(value)
        }
    }
    impl ::core::convert::From<CannotRemoveImmutableFunction> for DiamondCutFacetErrors {
        fn from(value: CannotRemoveImmutableFunction) -> Self {
            Self::CannotRemoveImmutableFunction(value)
        }
    }
    impl ::core::convert::From<CannotReplaceFunctionThatDoesNotExists> for DiamondCutFacetErrors {
        fn from(value: CannotReplaceFunctionThatDoesNotExists) -> Self {
            Self::CannotReplaceFunctionThatDoesNotExists(value)
        }
    }
    impl ::core::convert::From<CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet>
        for DiamondCutFacetErrors
    {
        fn from(value: CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet) -> Self {
            Self::CannotReplaceFunctionWithTheSameFunctionFromTheSameFacet(value)
        }
    }
    impl ::core::convert::From<CannotReplaceFunctionsFromFacetWithZeroAddress>
        for DiamondCutFacetErrors
    {
        fn from(value: CannotReplaceFunctionsFromFacetWithZeroAddress) -> Self {
            Self::CannotReplaceFunctionsFromFacetWithZeroAddress(value)
        }
    }
    impl ::core::convert::From<CannotReplaceImmutableFunction> for DiamondCutFacetErrors {
        fn from(value: CannotReplaceImmutableFunction) -> Self {
            Self::CannotReplaceImmutableFunction(value)
        }
    }
    impl ::core::convert::From<IncorrectFacetCutAction> for DiamondCutFacetErrors {
        fn from(value: IncorrectFacetCutAction) -> Self {
            Self::IncorrectFacetCutAction(value)
        }
    }
    impl ::core::convert::From<InitializationFunctionReverted> for DiamondCutFacetErrors {
        fn from(value: InitializationFunctionReverted) -> Self {
            Self::InitializationFunctionReverted(value)
        }
    }
    impl ::core::convert::From<NoBytecodeAtAddress> for DiamondCutFacetErrors {
        fn from(value: NoBytecodeAtAddress) -> Self {
            Self::NoBytecodeAtAddress(value)
        }
    }
    impl ::core::convert::From<NoSelectorsProvidedForFacetForCut> for DiamondCutFacetErrors {
        fn from(value: NoSelectorsProvidedForFacetForCut) -> Self {
            Self::NoSelectorsProvidedForFacetForCut(value)
        }
    }
    impl ::core::convert::From<NotOwner> for DiamondCutFacetErrors {
        fn from(value: NotOwner) -> Self {
            Self::NotOwner(value)
        }
    }
    impl ::core::convert::From<RemoveFacetAddressMustBeZeroAddress> for DiamondCutFacetErrors {
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
    ///Container type for all input parameters for the `diamondCut` function with signature `diamondCut((address,uint8,bytes4[])[],address,bytes)` and selector `0x1f931c1c`
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
        name = "diamondCut",
        abi = "diamondCut((address,uint8,bytes4[])[],address,bytes)"
    )]
    pub struct DiamondCutCall {
        pub diamond_cut: ::std::vec::Vec<FacetCut>,
        pub init: ::ethers::core::types::Address,
        pub calldata: ::ethers::core::types::Bytes,
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
}
