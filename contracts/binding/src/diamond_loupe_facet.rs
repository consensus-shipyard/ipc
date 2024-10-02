pub use diamond_loupe_facet::*;
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
pub mod diamond_loupe_facet {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("facetAddress"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("facetAddress"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("_functionSelector"),
                            kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("bytes4"),
                            ),
                        },],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("facetAddress_"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Address,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("address"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("facetAddresses"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("facetAddresses"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("facetAddresses_"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                ::std::boxed::Box::new(
                                    ::ethers::core::abi::ethabi::ParamType::Address,
                                ),
                            ),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("address[]"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("facetFunctionSelectors"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("facetFunctionSelectors",),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("_facet"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Address,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("address"),
                            ),
                        },],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("_facetFunctionSelectors",),
                            kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                ::std::boxed::Box::new(
                                    ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                ),
                            ),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("bytes4[]"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("facets"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("facets"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("facets_"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                ::std::boxed::Box::new(
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Address,
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
                                ::std::borrow::ToOwned::to_owned("struct IDiamondLoupe.Facet[]",),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("supportsInterface"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("supportsInterface"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("_interfaceId"),
                            kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("bytes4"),
                            ),
                        },],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("bool"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
            ]),
            events: ::std::collections::BTreeMap::new(),
            errors: ::std::collections::BTreeMap::new(),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static DIAMONDLOUPEFACET_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(__abi);
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = b"`\x80\x80`@R4`\x15Wa\x08s\x90\x81a\0\x1B\x829\xF3[`\0\x80\xFD\xFE`\x80`@R`\x046\x10\x15a\0\x12W`\0\x80\xFD[`\x005`\xE0\x1C\x80c\x01\xFF\xC9\xA7\x14a\0gW\x80cR\xEFk,\x14a\0bW\x80cz\x0E\xD6'\x14a\0]W\x80c\xAD\xFC\xA1^\x14a\0XWc\xCD\xFF\xAC\xC6\x14a\0SW`\0\x80\xFD[a\x06\0V[a\x05?V[a\x02\xEEV[a\x01%V[4a\0\xBAWa\0u6a\0\xBFV[c\xFF\xFF\xFF\xFF`\xE0\x1B\x16`\0R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD4` R` `\xFF`@`\0 T\x16`@Q\x90\x15\x15\x81R\xF3[`\0\x80\xFD[` \x90`\x03\x19\x01\x12a\0\xBAW`\x045`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x81\x03a\0\xBAW\x90V[` `@\x81\x83\x01\x92\x82\x81R\x84Q\x80\x94R\x01\x92\x01\x90`\0[\x81\x81\x10a\x01\x06WPPP\x90V[\x82Q`\x01`\x01`\xA0\x1B\x03\x16\x84R` \x93\x84\x01\x93\x90\x92\x01\x91`\x01\x01a\0\xF9V[4a\0\xBAW`\x006`\x03\x19\x01\x12a\0\xBAW`\0\x80Q` a\x08\x1E\x839\x81Q\x91RTa\x01O\x81a\x06\xB6V[`\0\x80\x92[\x80\x84\x10a\x01pW\x81\x83R`@Q\x80a\x01l\x85\x82a\0\xE2V[\x03\x90\xF3[\x90a\x01\xA2a\x01\x95a\x01\x90a\x01\x83\x87a\x06\xF9V[\x90T\x90`\x03\x1B\x1C`\xE0\x1B\x90V[a\x06%V[T`\x01`\x01`\xA0\x1B\x03\x16\x90V[`\0`\x01`\x01`\xA0\x1B\x03\x82\x16\x81[\x84\x81\x10a\x01\xFAW[PPa\x01\xF0W\x81a\x01\xE1a\x01\xE6\x92a\x01\xD2`\x01\x95\x88a\x07fV[`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90RV[a\x07\x90V[\x93[\x01\x92\x90a\x01TV[P\x92`\x01\x90a\x01\xE8V[a\x02#a\x02\x17a\x02\n\x83\x8Aa\x07fV[Q`\x01`\x01`\xA0\x1B\x03\x16\x90V[`\x01`\x01`\xA0\x1B\x03\x16\x90V[\x82\x14a\x021W`\x01\x01a\x01\xB0V[PPP`\x018\x80a\x01\xB8V[` \x81\x01` \x82R\x82Q\x80\x91R`@\x82\x01\x91` `@\x83`\x05\x1B\x83\x01\x01\x94\x01\x92`\0\x91[\x83\x83\x10a\x02pWPPPPP\x90V[\x90\x91\x92\x93\x94`?\x19\x82\x82\x03\x01\x83R\x85Q` ``\x81`@\x85\x01\x93`\x01\x80`\xA0\x1B\x03\x81Q\x16\x86R\x01Q\x93`@\x83\x82\x01R\x84Q\x80\x94R\x01\x92\x01\x90`\0\x90[\x80\x82\x10a\x02\xCBWPPP` \x80`\x01\x92\x97\x01\x93\x01\x93\x01\x91\x93\x92\x90a\x02aV[\x82Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x84R` \x93\x84\x01\x93\x90\x92\x01\x91`\x01\x90\x91\x01\x90a\x02\xACV[4a\0\xBAW`\x006`\x03\x19\x01\x12a\0\xBAW`\0\x80Q` a\x08\x1E\x839\x81Q\x91RTa\x03\x18\x81a\x07\xA4V[a\x03!\x82a\x06\xB6V[\x91`\0\x90`\0\x90[\x80\x82\x10a\x03\x85WPP`\0[\x81\x81\x10a\x03MW\x81\x83R`@Q\x80a\x01l\x85\x82a\x02=V[\x80a\x03oa\x03ha\x03``\x01\x94\x88a\x07fV[Qa\xFF\xFF\x16\x90V[a\xFF\xFF\x16\x90V[` a\x03{\x83\x87a\x07fV[Q\x01QR\x01a\x035V[\x90\x91a\x03\x93a\x01\x83\x84a\x06\xF9V[a\x03\x9Fa\x01\x95\x82a\x06%V[`\0\x80`\x01`\x01`\xA0\x1B\x03\x83\x16[\x85\x82\x10a\x04IW[PPa\x04>W\x91a\x04\"a\x045\x92a\x03\xE3`\x01\x95a\x03\xD3\x85\x8Ba\x07fV[Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90RV[a\x03\xEC\x86a\x06\xB6V[` a\x03\xF8\x85\x8Ba\x07fV[Q\x01Ra\x04\x12` a\x04\n\x85\x8Ba\x07fV[Q\x01Qa\x07YV[`\x01`\x01`\xE0\x1B\x03\x19\x90\x91\x16\x90RV[a\x01\xE1a\x04/\x82\x89a\x07fV[`\x01\x90RV[\x92[\x01\x90a\x03)V[PP\x91`\x01\x90a\x047V[\x80a\x04sa\x02\x17a\x04e\x85\x8D\x98\x9C\x9D\x9E\x97\x96\x9E\x9B\x99\x9A\x9Ba\x07fV[QQ`\x01`\x01`\xA0\x1B\x03\x16\x90V[\x14a\x04\x8EW`\x01\x80\x9A\x01\x91\x92\x99P\x97\x96\x92\x97\x95\x94\x93\x95a\x03\xADV[PP\x96\x80a\x04\xCA\x85a\x04\x12\x8Ba\x04\xC4a\x03ha\x03`\x87\x8A\x9F\x9E\x9Aa\x04\xBB\x9C\x9E\x9D\x9Ca\x04\xF2\x9B` \x92a\x07fV[Q\x01Q\x94a\x07fV[\x90a\x07fV[a\x04\xE9a\x04\xE2a\x04\xDDa\x03`\x84\x8Da\x07fV[a\x08\nV[\x91\x8Aa\x07fV[\x90a\xFF\xFF\x16\x90RV[`\x018\x80a\x03\xB5V[` `@\x81\x83\x01\x92\x82\x81R\x84Q\x80\x94R\x01\x92\x01\x90`\0[\x81\x81\x10a\x05\x1FWPPP\x90V[\x82Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x84R` \x93\x84\x01\x93\x90\x92\x01\x91`\x01\x01a\x05\x12V[4a\0\xBAW` 6`\x03\x19\x01\x12a\0\xBAW`\x045`\x01`\x01`\xA0\x1B\x03\x81\x16\x90\x81\x90\x03a\0\xBAW`\0\x80Q` a\x08\x1E\x839\x81Q\x91RT`\0a\x05\x80\x82a\x06\xB6V[\x91`\0[\x81\x81\x10a\x05\x9CW\x82\x84R`@Q\x80a\x01l\x86\x82a\x04\xFBV[a\x05\xA5\x81a\x06\xF9V[\x90T`\x03\x91\x90\x91\x1B\x1C`\xE0\x1B`\x01`\x01`\xA0\x1B\x03a\x05\xC2\x82a\x06%V[T\x16\x86\x14a\x05\xD4W[P`\x01\x01a\x05\x84V[\x83a\x05\xF9\x91a\x05\xE6`\x01\x94\x96\x88a\x07fV[`\x01`\x01`\xE0\x1B\x03\x19\x90\x91\x16\x90Ra\x07\x90V[\x92\x90a\x05\xCBV[4a\0\xBAW` `\x01`\x01`\xA0\x1B\x03a\x06\x1Ba\x01\x906a\0\xBFV[T\x16`@Q\x90\x81R\xF3[c\xFF\xFF\xFF\xFF`\xE0\x1B\x16`\0R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@`\0 \x90V[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[`@Q\x91\x90`\x1F\x01`\x1F\x19\x16\x82\x01g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x83\x82\x10\x17a\x06\x99W`@RV[a\x06]V[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x06\x99W`\x05\x1B` \x01\x90V[\x90a\x06\xC8a\x06\xC3\x83a\x06\x9EV[a\x06sV[\x82\x81R\x80\x92a\x06\xD9`\x1F\x19\x91a\x06\x9EV[\x01\x90` 6\x91\x017V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[\x90`\0\x80Q` a\x08\x1E\x839\x81Q\x91RT\x82\x10\x15a\x07TW`\0\x80Q` a\x08\x1E\x839\x81Q\x91R`\0R`\x03\x82\x90\x1C\x7F\xB6[\xEC\xA8\xB6\xFAx\x8B\xCB\x15(\xC2\xAB_M\xC6\xBC\x98\xE5\x89eP\xBA\xA0\x13\xD83\x0F\xAB\x0B\x86\xF4\x01\x91`\x02\x1B`\x1C\x16\x90V[a\x06\xE3V[\x80Q\x15a\x07TW` \x01\x90V[\x80Q\x82\x10\x15a\x07TW` \x91`\x05\x1B\x01\x01\x90V[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[`\0\x19\x81\x14a\x07\x9FW`\x01\x01\x90V[a\x07zV[\x90a\x07\xB1a\x06\xC3\x83a\x06\x9EV[\x82\x81R\x80\x92a\x07\xC2`\x1F\x19\x91a\x06\x9EV[\x01`\0[\x81\x81\x10a\x07\xD2WPPPV[`@Q\x90`@\x82\x01\x91\x80\x83\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x84\x11\x17a\x06\x99W` \x92`@R`\0\x81R``\x83\x82\x01R\x82\x82\x86\x01\x01R\x01a\x07\xC6V[a\xFF\xFF\x16a\xFF\xFF\x81\x14a\x07\x9FW`\x01\x01\x90V\xFE\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3\xA2dipfsX\"\x12 \x1E\xF4\x89q4h\x10\x9C\xB9\xEA\xF0\xD8d|\xF8\xF25\x19\x83\xC6ly\x8A!\xD6\xF4i\xB20G\x97\xF2dsolcC\0\x08\x1A\x003";
    /// The bytecode of the contract.
    pub static DIAMONDLOUPEFACET_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__BYTECODE);
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R`\x046\x10\x15a\0\x12W`\0\x80\xFD[`\x005`\xE0\x1C\x80c\x01\xFF\xC9\xA7\x14a\0gW\x80cR\xEFk,\x14a\0bW\x80cz\x0E\xD6'\x14a\0]W\x80c\xAD\xFC\xA1^\x14a\0XWc\xCD\xFF\xAC\xC6\x14a\0SW`\0\x80\xFD[a\x06\0V[a\x05?V[a\x02\xEEV[a\x01%V[4a\0\xBAWa\0u6a\0\xBFV[c\xFF\xFF\xFF\xFF`\xE0\x1B\x16`\0R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD4` R` `\xFF`@`\0 T\x16`@Q\x90\x15\x15\x81R\xF3[`\0\x80\xFD[` \x90`\x03\x19\x01\x12a\0\xBAW`\x045`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x81\x03a\0\xBAW\x90V[` `@\x81\x83\x01\x92\x82\x81R\x84Q\x80\x94R\x01\x92\x01\x90`\0[\x81\x81\x10a\x01\x06WPPP\x90V[\x82Q`\x01`\x01`\xA0\x1B\x03\x16\x84R` \x93\x84\x01\x93\x90\x92\x01\x91`\x01\x01a\0\xF9V[4a\0\xBAW`\x006`\x03\x19\x01\x12a\0\xBAW`\0\x80Q` a\x08\x1E\x839\x81Q\x91RTa\x01O\x81a\x06\xB6V[`\0\x80\x92[\x80\x84\x10a\x01pW\x81\x83R`@Q\x80a\x01l\x85\x82a\0\xE2V[\x03\x90\xF3[\x90a\x01\xA2a\x01\x95a\x01\x90a\x01\x83\x87a\x06\xF9V[\x90T\x90`\x03\x1B\x1C`\xE0\x1B\x90V[a\x06%V[T`\x01`\x01`\xA0\x1B\x03\x16\x90V[`\0`\x01`\x01`\xA0\x1B\x03\x82\x16\x81[\x84\x81\x10a\x01\xFAW[PPa\x01\xF0W\x81a\x01\xE1a\x01\xE6\x92a\x01\xD2`\x01\x95\x88a\x07fV[`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90RV[a\x07\x90V[\x93[\x01\x92\x90a\x01TV[P\x92`\x01\x90a\x01\xE8V[a\x02#a\x02\x17a\x02\n\x83\x8Aa\x07fV[Q`\x01`\x01`\xA0\x1B\x03\x16\x90V[`\x01`\x01`\xA0\x1B\x03\x16\x90V[\x82\x14a\x021W`\x01\x01a\x01\xB0V[PPP`\x018\x80a\x01\xB8V[` \x81\x01` \x82R\x82Q\x80\x91R`@\x82\x01\x91` `@\x83`\x05\x1B\x83\x01\x01\x94\x01\x92`\0\x91[\x83\x83\x10a\x02pWPPPPP\x90V[\x90\x91\x92\x93\x94`?\x19\x82\x82\x03\x01\x83R\x85Q` ``\x81`@\x85\x01\x93`\x01\x80`\xA0\x1B\x03\x81Q\x16\x86R\x01Q\x93`@\x83\x82\x01R\x84Q\x80\x94R\x01\x92\x01\x90`\0\x90[\x80\x82\x10a\x02\xCBWPPP` \x80`\x01\x92\x97\x01\x93\x01\x93\x01\x91\x93\x92\x90a\x02aV[\x82Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x84R` \x93\x84\x01\x93\x90\x92\x01\x91`\x01\x90\x91\x01\x90a\x02\xACV[4a\0\xBAW`\x006`\x03\x19\x01\x12a\0\xBAW`\0\x80Q` a\x08\x1E\x839\x81Q\x91RTa\x03\x18\x81a\x07\xA4V[a\x03!\x82a\x06\xB6V[\x91`\0\x90`\0\x90[\x80\x82\x10a\x03\x85WPP`\0[\x81\x81\x10a\x03MW\x81\x83R`@Q\x80a\x01l\x85\x82a\x02=V[\x80a\x03oa\x03ha\x03``\x01\x94\x88a\x07fV[Qa\xFF\xFF\x16\x90V[a\xFF\xFF\x16\x90V[` a\x03{\x83\x87a\x07fV[Q\x01QR\x01a\x035V[\x90\x91a\x03\x93a\x01\x83\x84a\x06\xF9V[a\x03\x9Fa\x01\x95\x82a\x06%V[`\0\x80`\x01`\x01`\xA0\x1B\x03\x83\x16[\x85\x82\x10a\x04IW[PPa\x04>W\x91a\x04\"a\x045\x92a\x03\xE3`\x01\x95a\x03\xD3\x85\x8Ba\x07fV[Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90RV[a\x03\xEC\x86a\x06\xB6V[` a\x03\xF8\x85\x8Ba\x07fV[Q\x01Ra\x04\x12` a\x04\n\x85\x8Ba\x07fV[Q\x01Qa\x07YV[`\x01`\x01`\xE0\x1B\x03\x19\x90\x91\x16\x90RV[a\x01\xE1a\x04/\x82\x89a\x07fV[`\x01\x90RV[\x92[\x01\x90a\x03)V[PP\x91`\x01\x90a\x047V[\x80a\x04sa\x02\x17a\x04e\x85\x8D\x98\x9C\x9D\x9E\x97\x96\x9E\x9B\x99\x9A\x9Ba\x07fV[QQ`\x01`\x01`\xA0\x1B\x03\x16\x90V[\x14a\x04\x8EW`\x01\x80\x9A\x01\x91\x92\x99P\x97\x96\x92\x97\x95\x94\x93\x95a\x03\xADV[PP\x96\x80a\x04\xCA\x85a\x04\x12\x8Ba\x04\xC4a\x03ha\x03`\x87\x8A\x9F\x9E\x9Aa\x04\xBB\x9C\x9E\x9D\x9Ca\x04\xF2\x9B` \x92a\x07fV[Q\x01Q\x94a\x07fV[\x90a\x07fV[a\x04\xE9a\x04\xE2a\x04\xDDa\x03`\x84\x8Da\x07fV[a\x08\nV[\x91\x8Aa\x07fV[\x90a\xFF\xFF\x16\x90RV[`\x018\x80a\x03\xB5V[` `@\x81\x83\x01\x92\x82\x81R\x84Q\x80\x94R\x01\x92\x01\x90`\0[\x81\x81\x10a\x05\x1FWPPP\x90V[\x82Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x84R` \x93\x84\x01\x93\x90\x92\x01\x91`\x01\x01a\x05\x12V[4a\0\xBAW` 6`\x03\x19\x01\x12a\0\xBAW`\x045`\x01`\x01`\xA0\x1B\x03\x81\x16\x90\x81\x90\x03a\0\xBAW`\0\x80Q` a\x08\x1E\x839\x81Q\x91RT`\0a\x05\x80\x82a\x06\xB6V[\x91`\0[\x81\x81\x10a\x05\x9CW\x82\x84R`@Q\x80a\x01l\x86\x82a\x04\xFBV[a\x05\xA5\x81a\x06\xF9V[\x90T`\x03\x91\x90\x91\x1B\x1C`\xE0\x1B`\x01`\x01`\xA0\x1B\x03a\x05\xC2\x82a\x06%V[T\x16\x86\x14a\x05\xD4W[P`\x01\x01a\x05\x84V[\x83a\x05\xF9\x91a\x05\xE6`\x01\x94\x96\x88a\x07fV[`\x01`\x01`\xE0\x1B\x03\x19\x90\x91\x16\x90Ra\x07\x90V[\x92\x90a\x05\xCBV[4a\0\xBAW` `\x01`\x01`\xA0\x1B\x03a\x06\x1Ba\x01\x906a\0\xBFV[T\x16`@Q\x90\x81R\xF3[c\xFF\xFF\xFF\xFF`\xE0\x1B\x16`\0R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2` R`@`\0 \x90V[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[`@Q\x91\x90`\x1F\x01`\x1F\x19\x16\x82\x01g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x83\x82\x10\x17a\x06\x99W`@RV[a\x06]V[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x06\x99W`\x05\x1B` \x01\x90V[\x90a\x06\xC8a\x06\xC3\x83a\x06\x9EV[a\x06sV[\x82\x81R\x80\x92a\x06\xD9`\x1F\x19\x91a\x06\x9EV[\x01\x90` 6\x91\x017V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[\x90`\0\x80Q` a\x08\x1E\x839\x81Q\x91RT\x82\x10\x15a\x07TW`\0\x80Q` a\x08\x1E\x839\x81Q\x91R`\0R`\x03\x82\x90\x1C\x7F\xB6[\xEC\xA8\xB6\xFAx\x8B\xCB\x15(\xC2\xAB_M\xC6\xBC\x98\xE5\x89eP\xBA\xA0\x13\xD83\x0F\xAB\x0B\x86\xF4\x01\x91`\x02\x1B`\x1C\x16\x90V[a\x06\xE3V[\x80Q\x15a\x07TW` \x01\x90V[\x80Q\x82\x10\x15a\x07TW` \x91`\x05\x1B\x01\x01\x90V[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[`\0\x19\x81\x14a\x07\x9FW`\x01\x01\x90V[a\x07zV[\x90a\x07\xB1a\x06\xC3\x83a\x06\x9EV[\x82\x81R\x80\x92a\x07\xC2`\x1F\x19\x91a\x06\x9EV[\x01`\0[\x81\x81\x10a\x07\xD2WPPPV[`@Q\x90`@\x82\x01\x91\x80\x83\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x84\x11\x17a\x06\x99W` \x92`@R`\0\x81R``\x83\x82\x01R\x82\x82\x86\x01\x01R\x01a\x07\xC6V[a\xFF\xFF\x16a\xFF\xFF\x81\x14a\x07\x9FW`\x01\x01\x90V\xFE\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3\xA2dipfsX\"\x12 \x1E\xF4\x89q4h\x10\x9C\xB9\xEA\xF0\xD8d|\xF8\xF25\x19\x83\xC6ly\x8A!\xD6\xF4i\xB20G\x97\xF2dsolcC\0\x08\x1A\x003";
    /// The deployed bytecode of the contract.
    pub static DIAMONDLOUPEFACET_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__DEPLOYED_BYTECODE);
    pub struct DiamondLoupeFacet<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for DiamondLoupeFacet<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for DiamondLoupeFacet<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for DiamondLoupeFacet<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for DiamondLoupeFacet<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(DiamondLoupeFacet))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> DiamondLoupeFacet<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                DIAMONDLOUPEFACET_ABI.clone(),
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
                DIAMONDLOUPEFACET_ABI.clone(),
                DIAMONDLOUPEFACET_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Calls the contract's `facetAddress` (0xcdffacc6) function
        pub fn facet_address(
            &self,
            function_selector: [u8; 4],
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::Address> {
            self.0
                .method_hash([205, 255, 172, 198], function_selector)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `facetAddresses` (0x52ef6b2c) function
        pub fn facet_addresses(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers::core::types::Address>,
        > {
            self.0
                .method_hash([82, 239, 107, 44], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `facetFunctionSelectors` (0xadfca15e) function
        pub fn facet_function_selectors(
            &self,
            facet: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::vec::Vec<[u8; 4]>> {
            self.0
                .method_hash([173, 252, 161, 94], facet)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `facets` (0x7a0ed627) function
        pub fn facets(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::vec::Vec<Facet>> {
            self.0
                .method_hash([122, 14, 214, 39], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `supportsInterface` (0x01ffc9a7) function
        pub fn supports_interface(
            &self,
            interface_id: [u8; 4],
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([1, 255, 201, 167], interface_id)
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
        for DiamondLoupeFacet<M>
    {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Container type for all input parameters for the `facetAddress` function with signature `facetAddress(bytes4)` and selector `0xcdffacc6`
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
    #[ethcall(name = "facetAddress", abi = "facetAddress(bytes4)")]
    pub struct FacetAddressCall {
        pub function_selector: [u8; 4],
    }
    ///Container type for all input parameters for the `facetAddresses` function with signature `facetAddresses()` and selector `0x52ef6b2c`
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
    #[ethcall(name = "facetAddresses", abi = "facetAddresses()")]
    pub struct FacetAddressesCall;
    ///Container type for all input parameters for the `facetFunctionSelectors` function with signature `facetFunctionSelectors(address)` and selector `0xadfca15e`
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
        name = "facetFunctionSelectors",
        abi = "facetFunctionSelectors(address)"
    )]
    pub struct FacetFunctionSelectorsCall {
        pub facet: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `facets` function with signature `facets()` and selector `0x7a0ed627`
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
    #[ethcall(name = "facets", abi = "facets()")]
    pub struct FacetsCall;
    ///Container type for all input parameters for the `supportsInterface` function with signature `supportsInterface(bytes4)` and selector `0x01ffc9a7`
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
    #[ethcall(name = "supportsInterface", abi = "supportsInterface(bytes4)")]
    pub struct SupportsInterfaceCall {
        pub interface_id: [u8; 4],
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum DiamondLoupeFacetCalls {
        FacetAddress(FacetAddressCall),
        FacetAddresses(FacetAddressesCall),
        FacetFunctionSelectors(FacetFunctionSelectorsCall),
        Facets(FacetsCall),
        SupportsInterface(SupportsInterfaceCall),
    }
    impl ::ethers::core::abi::AbiDecode for DiamondLoupeFacetCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <FacetAddressCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::FacetAddress(decoded));
            }
            if let Ok(decoded) =
                <FacetAddressesCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::FacetAddresses(decoded));
            }
            if let Ok(decoded) =
                <FacetFunctionSelectorsCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::FacetFunctionSelectors(decoded));
            }
            if let Ok(decoded) = <FacetsCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Facets(decoded));
            }
            if let Ok(decoded) =
                <SupportsInterfaceCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::SupportsInterface(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for DiamondLoupeFacetCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::FacetAddress(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::FacetAddresses(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::FacetFunctionSelectors(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Facets(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::SupportsInterface(element) => ::ethers::core::abi::AbiEncode::encode(element),
            }
        }
    }
    impl ::core::fmt::Display for DiamondLoupeFacetCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::FacetAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::FacetAddresses(element) => ::core::fmt::Display::fmt(element, f),
                Self::FacetFunctionSelectors(element) => ::core::fmt::Display::fmt(element, f),
                Self::Facets(element) => ::core::fmt::Display::fmt(element, f),
                Self::SupportsInterface(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<FacetAddressCall> for DiamondLoupeFacetCalls {
        fn from(value: FacetAddressCall) -> Self {
            Self::FacetAddress(value)
        }
    }
    impl ::core::convert::From<FacetAddressesCall> for DiamondLoupeFacetCalls {
        fn from(value: FacetAddressesCall) -> Self {
            Self::FacetAddresses(value)
        }
    }
    impl ::core::convert::From<FacetFunctionSelectorsCall> for DiamondLoupeFacetCalls {
        fn from(value: FacetFunctionSelectorsCall) -> Self {
            Self::FacetFunctionSelectors(value)
        }
    }
    impl ::core::convert::From<FacetsCall> for DiamondLoupeFacetCalls {
        fn from(value: FacetsCall) -> Self {
            Self::Facets(value)
        }
    }
    impl ::core::convert::From<SupportsInterfaceCall> for DiamondLoupeFacetCalls {
        fn from(value: SupportsInterfaceCall) -> Self {
            Self::SupportsInterface(value)
        }
    }
    ///Container type for all return fields from the `facetAddress` function with signature `facetAddress(bytes4)` and selector `0xcdffacc6`
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
    pub struct FacetAddressReturn {
        pub facet_address: ::ethers::core::types::Address,
    }
    ///Container type for all return fields from the `facetAddresses` function with signature `facetAddresses()` and selector `0x52ef6b2c`
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
    pub struct FacetAddressesReturn {
        pub facet_addresses: ::std::vec::Vec<::ethers::core::types::Address>,
    }
    ///Container type for all return fields from the `facetFunctionSelectors` function with signature `facetFunctionSelectors(address)` and selector `0xadfca15e`
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
    pub struct FacetFunctionSelectorsReturn {
        pub facet_function_selectors: ::std::vec::Vec<[u8; 4]>,
    }
    ///Container type for all return fields from the `facets` function with signature `facets()` and selector `0x7a0ed627`
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
    pub struct FacetsReturn {
        pub facets: ::std::vec::Vec<Facet>,
    }
    ///Container type for all return fields from the `supportsInterface` function with signature `supportsInterface(bytes4)` and selector `0x01ffc9a7`
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
    pub struct SupportsInterfaceReturn(pub bool);
    ///`Facet(address,bytes4[])`
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
    pub struct Facet {
        pub facet_address: ::ethers::core::types::Address,
        pub function_selectors: ::std::vec::Vec<[u8; 4]>,
    }
}
