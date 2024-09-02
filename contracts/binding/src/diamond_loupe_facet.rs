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
    const __BYTECODE: &[u8] = b"`\x80`@R4\x80\x15`\x0FW`\0\x80\xFD[Pa\nD\x80a\0\x1F`\09`\0\xF3\xFE`\x80`@R4\x80\x15a\0\x10W`\0\x80\xFD[P`\x046\x10a\0WW`\x005`\xE0\x1C\x80c\x01\xFF\xC9\xA7\x14a\0\\W\x80cR\xEFk,\x14a\0\xBDW\x80cz\x0E\xD6'\x14a\0\xD2W\x80c\xAD\xFC\xA1^\x14a\0\xE7W\x80c\xCD\xFF\xAC\xC6\x14a\x01\x07W[`\0\x80\xFD[a\0\xA8a\0j6`\x04a\x07\xEEV[`\x01`\x01`\xE0\x1B\x03\x19\x16`\0\x90\x81R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD4` R`@\x90 T`\xFF\x16\x90V[`@Q\x90\x15\x15\x81R` \x01[`@Q\x80\x91\x03\x90\xF3[a\0\xC5a\x01_V[`@Qa\0\xB4\x91\x90a\x08\x1FV[a\0\xDAa\x02\xEAV[`@Qa\0\xB4\x91\x90a\x08\xB1V[a\0\xFAa\0\xF56`\x04a\t6V[a\x06\xB2V[`@Qa\0\xB4\x91\x90a\t_V[a\x01Ga\x01\x156`\x04a\x07\xEEV[`\x01`\x01`\xE0\x1B\x03\x19\x16`\0\x90\x81R`\0\x80Q` a\t\xEF\x839\x81Q\x91R` R`@\x90 T`\x01`\x01`\xA0\x1B\x03\x16\x90V[`@Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x81R` \x01a\0\xB4V[\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3T``\x90`\0\x80Q` a\t\xEF\x839\x81Q\x91R\x90\x80g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x01\xADWa\x01\xADa\trV[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x01\xD6W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x92P`\0\x80[\x82\x81\x10\x15a\x02\xE0W`\0\x84`\x01\x01\x82\x81T\x81\x10a\x01\xFCWa\x01\xFCa\t\x88V[`\0\x91\x82R` \x80\x83 `\x08\x83\x04\x01T`\x07\x90\x92\x16`\x04\x02a\x01\0\n\x90\x91\x04`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x83R\x90\x87\x90R`@\x82 T\x90\x92P`\x01`\x01`\xA0\x1B\x03\x16\x90\x80[\x85\x81\x10\x15a\x02\x8EW\x88\x81\x81Q\x81\x10a\x02]Wa\x02]a\t\x88V[` \x02` \x01\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x83`\x01`\x01`\xA0\x1B\x03\x16\x03a\x02\x86W`\x01\x91Pa\x02\x8EV[`\x01\x01a\x02CV[P\x80\x15a\x02\x9EWPa\x02\xD8\x91PPV[\x81\x88\x86\x81Q\x81\x10a\x02\xB1Wa\x02\xB1a\t\x88V[`\x01`\x01`\xA0\x1B\x03\x90\x92\x16` \x92\x83\x02\x91\x90\x91\x01\x90\x91\x01Ra\x02\xD2\x85a\t\xB4V[\x94PPPP[`\x01\x01a\x01\xDDV[P\x80\x84RPPP\x90V[\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3T``\x90`\0\x80Q` a\t\xEF\x839\x81Q\x91R\x90\x80g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x038Wa\x038a\trV[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x03~W\x81` \x01[`@\x80Q\x80\x82\x01\x90\x91R`\0\x81R``` \x82\x01R\x81R` \x01\x90`\x01\x90\x03\x90\x81a\x03VW\x90P[P\x92P`\0\x81g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x03\x9CWa\x03\x9Ca\trV[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x03\xC5W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x90P`\0\x80[\x83\x81\x10\x15a\x06JW`\0\x85`\x01\x01\x82\x81T\x81\x10a\x03\xEBWa\x03\xEBa\t\x88V[`\0\x91\x82R` \x80\x83 `\x08\x83\x04\x01T`\x07\x90\x92\x16`\x04\x02a\x01\0\n\x90\x91\x04`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x83R\x90\x88\x90R`@\x82 T\x90\x92P`\x01`\x01`\xA0\x1B\x03\x16\x90\x80[\x85\x81\x10\x15a\x05\x1AW\x82`\x01`\x01`\xA0\x1B\x03\x16\x8A\x82\x81Q\x81\x10a\x04VWa\x04Va\t\x88V[` \x02` \x01\x01Q`\0\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x03a\x05\x12W\x83\x8A\x82\x81Q\x81\x10a\x04\x83Wa\x04\x83a\t\x88V[` \x02` \x01\x01Q` \x01Q\x88\x83\x81Q\x81\x10a\x04\xA1Wa\x04\xA1a\t\x88V[` \x02` \x01\x01Qa\xFF\xFF\x16\x81Q\x81\x10a\x04\xBDWa\x04\xBDa\t\x88V[` \x02` \x01\x01\x90`\x01`\x01`\xE0\x1B\x03\x19\x16\x90\x81`\x01`\x01`\xE0\x1B\x03\x19\x16\x81RPP\x86\x81\x81Q\x81\x10a\x04\xF1Wa\x04\xF1a\t\x88V[` \x02` \x01\x01\x80Qa\x05\x03\x90a\t\xCDV[a\xFF\xFF\x16\x90R`\x01\x91Pa\x05\x1AV[`\x01\x01a\x042V[P\x80\x15a\x05*WPa\x06B\x91PPV[\x81\x89\x86\x81Q\x81\x10a\x05=Wa\x05=a\t\x88V[` \x90\x81\x02\x91\x90\x91\x01\x01Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90R\x86g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x05nWa\x05na\trV[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x05\x97W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x89\x86\x81Q\x81\x10a\x05\xAAWa\x05\xAAa\t\x88V[` \x02` \x01\x01Q` \x01\x81\x90RP\x82\x89\x86\x81Q\x81\x10a\x05\xCCWa\x05\xCCa\t\x88V[` \x02` \x01\x01Q` \x01Q`\0\x81Q\x81\x10a\x05\xEAWa\x05\xEAa\t\x88V[` \x02` \x01\x01\x90`\x01`\x01`\xE0\x1B\x03\x19\x16\x90\x81`\x01`\x01`\xE0\x1B\x03\x19\x16\x81RPP`\x01\x86\x86\x81Q\x81\x10a\x06 Wa\x06 a\t\x88V[a\xFF\xFF\x90\x92\x16` \x92\x83\x02\x91\x90\x91\x01\x90\x91\x01Ra\x06<\x85a\t\xB4V[\x94PPPP[`\x01\x01a\x03\xCCV[P`\0[\x81\x81\x10\x15a\x06\xA7W`\0\x83\x82\x81Q\x81\x10a\x06jWa\x06ja\t\x88V[` \x02` \x01\x01Qa\xFF\xFF\x16\x90P`\0\x87\x83\x81Q\x81\x10a\x06\x8CWa\x06\x8Ca\t\x88V[` \x90\x81\x02\x91\x90\x91\x01\x81\x01Q\x01Q\x91\x90\x91RP`\x01\x01a\x06NV[P\x80\x85RPPPP\x90V[\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3T``\x90`\0\x80Q` a\t\xEF\x839\x81Q\x91R\x90`\0\x81g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x07\x02Wa\x07\x02a\trV[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x07+W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x93P`\0[\x82\x81\x10\x15a\x07\xE3W`\0\x84`\x01\x01\x82\x81T\x81\x10a\x07PWa\x07Pa\t\x88V[`\0\x91\x82R` \x80\x83 `\x08\x83\x04\x01T`\x07\x90\x92\x16`\x04\x02a\x01\0\n\x90\x91\x04`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x83R\x90\x87\x90R`@\x90\x91 T\x90\x91P`\x01`\x01`\xA0\x1B\x03\x90\x81\x16\x90\x88\x16\x81\x90\x03a\x07\xD9W\x81\x87\x85\x81Q\x81\x10a\x07\xB4Wa\x07\xB4a\t\x88V[`\x01`\x01`\xE0\x1B\x03\x19\x90\x92\x16` \x92\x83\x02\x91\x90\x91\x01\x90\x91\x01Ra\x07\xD6\x84a\t\xB4V[\x93P[PP`\x01\x01a\x071V[P\x83RP\x90\x92\x91PPV[`\0` \x82\x84\x03\x12\x15a\x08\0W`\0\x80\xFD[\x815`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x81\x14a\x08\x18W`\0\x80\xFD[\x93\x92PPPV[` \x80\x82R\x82Q\x82\x82\x01\x81\x90R`\0\x91\x84\x01\x90`@\x84\x01\x90\x83[\x81\x81\x10\x15a\x08`W\x83Q`\x01`\x01`\xA0\x1B\x03\x16\x83R` \x93\x84\x01\x93\x90\x92\x01\x91`\x01\x01a\x089V[P\x90\x95\x94PPPPPV[`\0\x81Q\x80\x84R` \x84\x01\x93P` \x83\x01`\0[\x82\x81\x10\x15a\x08\xA7W\x81Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x86R` \x95\x86\x01\x95\x90\x91\x01\x90`\x01\x01a\x08\x7FV[P\x93\x94\x93PPPPV[`\0` \x82\x01` \x83R\x80\x84Q\x80\x83R`@\x85\x01\x91P`@\x81`\x05\x1B\x86\x01\x01\x92P` \x86\x01`\0[\x82\x81\x10\x15a\t*W\x86\x85\x03`?\x19\x01\x84R\x81Q\x80Q`\x01`\x01`\xA0\x1B\x03\x16\x86R` \x90\x81\x01Q`@\x91\x87\x01\x82\x90R\x90a\t\x14\x90\x87\x01\x82a\x08kV[\x95PP` \x93\x84\x01\x93\x91\x90\x91\x01\x90`\x01\x01a\x08\xD9V[P\x92\x96\x95PPPPPPV[`\0` \x82\x84\x03\x12\x15a\tHW`\0\x80\xFD[\x815`\x01`\x01`\xA0\x1B\x03\x81\x16\x81\x14a\x08\x18W`\0\x80\xFD[` \x81R`\0a\x08\x18` \x83\x01\x84a\x08kV[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[`\0`\x01\x82\x01a\t\xC6Wa\t\xC6a\t\x9EV[P`\x01\x01\x90V[`\0a\xFF\xFF\x82\x16a\xFF\xFF\x81\x03a\t\xE5Wa\t\xE5a\t\x9EV[`\x01\x01\x92\x91PPV\xFE\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2\xA2dipfsX\"\x12 \x95:\x07DQ[\xDA^\x1BB+\x9AS\xE7\xED\xB0/^}\xDB\xBD\xD5\xB4%n\"\xAE\x83\xCD\xF2\x10\xF6dsolcC\0\x08\x1A\x003";
    /// The bytecode of the contract.
    pub static DIAMONDLOUPEFACET_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__BYTECODE);
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R4\x80\x15a\0\x10W`\0\x80\xFD[P`\x046\x10a\0WW`\x005`\xE0\x1C\x80c\x01\xFF\xC9\xA7\x14a\0\\W\x80cR\xEFk,\x14a\0\xBDW\x80cz\x0E\xD6'\x14a\0\xD2W\x80c\xAD\xFC\xA1^\x14a\0\xE7W\x80c\xCD\xFF\xAC\xC6\x14a\x01\x07W[`\0\x80\xFD[a\0\xA8a\0j6`\x04a\x07\xEEV[`\x01`\x01`\xE0\x1B\x03\x19\x16`\0\x90\x81R\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD4` R`@\x90 T`\xFF\x16\x90V[`@Q\x90\x15\x15\x81R` \x01[`@Q\x80\x91\x03\x90\xF3[a\0\xC5a\x01_V[`@Qa\0\xB4\x91\x90a\x08\x1FV[a\0\xDAa\x02\xEAV[`@Qa\0\xB4\x91\x90a\x08\xB1V[a\0\xFAa\0\xF56`\x04a\t6V[a\x06\xB2V[`@Qa\0\xB4\x91\x90a\t_V[a\x01Ga\x01\x156`\x04a\x07\xEEV[`\x01`\x01`\xE0\x1B\x03\x19\x16`\0\x90\x81R`\0\x80Q` a\t\xEF\x839\x81Q\x91R` R`@\x90 T`\x01`\x01`\xA0\x1B\x03\x16\x90V[`@Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x81R` \x01a\0\xB4V[\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3T``\x90`\0\x80Q` a\t\xEF\x839\x81Q\x91R\x90\x80g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x01\xADWa\x01\xADa\trV[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x01\xD6W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x92P`\0\x80[\x82\x81\x10\x15a\x02\xE0W`\0\x84`\x01\x01\x82\x81T\x81\x10a\x01\xFCWa\x01\xFCa\t\x88V[`\0\x91\x82R` \x80\x83 `\x08\x83\x04\x01T`\x07\x90\x92\x16`\x04\x02a\x01\0\n\x90\x91\x04`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x83R\x90\x87\x90R`@\x82 T\x90\x92P`\x01`\x01`\xA0\x1B\x03\x16\x90\x80[\x85\x81\x10\x15a\x02\x8EW\x88\x81\x81Q\x81\x10a\x02]Wa\x02]a\t\x88V[` \x02` \x01\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x83`\x01`\x01`\xA0\x1B\x03\x16\x03a\x02\x86W`\x01\x91Pa\x02\x8EV[`\x01\x01a\x02CV[P\x80\x15a\x02\x9EWPa\x02\xD8\x91PPV[\x81\x88\x86\x81Q\x81\x10a\x02\xB1Wa\x02\xB1a\t\x88V[`\x01`\x01`\xA0\x1B\x03\x90\x92\x16` \x92\x83\x02\x91\x90\x91\x01\x90\x91\x01Ra\x02\xD2\x85a\t\xB4V[\x94PPPP[`\x01\x01a\x01\xDDV[P\x80\x84RPPP\x90V[\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3T``\x90`\0\x80Q` a\t\xEF\x839\x81Q\x91R\x90\x80g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x038Wa\x038a\trV[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x03~W\x81` \x01[`@\x80Q\x80\x82\x01\x90\x91R`\0\x81R``` \x82\x01R\x81R` \x01\x90`\x01\x90\x03\x90\x81a\x03VW\x90P[P\x92P`\0\x81g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x03\x9CWa\x03\x9Ca\trV[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x03\xC5W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x90P`\0\x80[\x83\x81\x10\x15a\x06JW`\0\x85`\x01\x01\x82\x81T\x81\x10a\x03\xEBWa\x03\xEBa\t\x88V[`\0\x91\x82R` \x80\x83 `\x08\x83\x04\x01T`\x07\x90\x92\x16`\x04\x02a\x01\0\n\x90\x91\x04`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x83R\x90\x88\x90R`@\x82 T\x90\x92P`\x01`\x01`\xA0\x1B\x03\x16\x90\x80[\x85\x81\x10\x15a\x05\x1AW\x82`\x01`\x01`\xA0\x1B\x03\x16\x8A\x82\x81Q\x81\x10a\x04VWa\x04Va\t\x88V[` \x02` \x01\x01Q`\0\x01Q`\x01`\x01`\xA0\x1B\x03\x16\x03a\x05\x12W\x83\x8A\x82\x81Q\x81\x10a\x04\x83Wa\x04\x83a\t\x88V[` \x02` \x01\x01Q` \x01Q\x88\x83\x81Q\x81\x10a\x04\xA1Wa\x04\xA1a\t\x88V[` \x02` \x01\x01Qa\xFF\xFF\x16\x81Q\x81\x10a\x04\xBDWa\x04\xBDa\t\x88V[` \x02` \x01\x01\x90`\x01`\x01`\xE0\x1B\x03\x19\x16\x90\x81`\x01`\x01`\xE0\x1B\x03\x19\x16\x81RPP\x86\x81\x81Q\x81\x10a\x04\xF1Wa\x04\xF1a\t\x88V[` \x02` \x01\x01\x80Qa\x05\x03\x90a\t\xCDV[a\xFF\xFF\x16\x90R`\x01\x91Pa\x05\x1AV[`\x01\x01a\x042V[P\x80\x15a\x05*WPa\x06B\x91PPV[\x81\x89\x86\x81Q\x81\x10a\x05=Wa\x05=a\t\x88V[` \x90\x81\x02\x91\x90\x91\x01\x01Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x90R\x86g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x05nWa\x05na\trV[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x05\x97W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x89\x86\x81Q\x81\x10a\x05\xAAWa\x05\xAAa\t\x88V[` \x02` \x01\x01Q` \x01\x81\x90RP\x82\x89\x86\x81Q\x81\x10a\x05\xCCWa\x05\xCCa\t\x88V[` \x02` \x01\x01Q` \x01Q`\0\x81Q\x81\x10a\x05\xEAWa\x05\xEAa\t\x88V[` \x02` \x01\x01\x90`\x01`\x01`\xE0\x1B\x03\x19\x16\x90\x81`\x01`\x01`\xE0\x1B\x03\x19\x16\x81RPP`\x01\x86\x86\x81Q\x81\x10a\x06 Wa\x06 a\t\x88V[a\xFF\xFF\x90\x92\x16` \x92\x83\x02\x91\x90\x91\x01\x90\x91\x01Ra\x06<\x85a\t\xB4V[\x94PPPP[`\x01\x01a\x03\xCCV[P`\0[\x81\x81\x10\x15a\x06\xA7W`\0\x83\x82\x81Q\x81\x10a\x06jWa\x06ja\t\x88V[` \x02` \x01\x01Qa\xFF\xFF\x16\x90P`\0\x87\x83\x81Q\x81\x10a\x06\x8CWa\x06\x8Ca\t\x88V[` \x90\x81\x02\x91\x90\x91\x01\x81\x01Q\x01Q\x91\x90\x91RP`\x01\x01a\x06NV[P\x80\x85RPPPP\x90V[\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD3T``\x90`\0\x80Q` a\t\xEF\x839\x81Q\x91R\x90`\0\x81g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x07\x02Wa\x07\x02a\trV[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x07+W\x81` \x01` \x82\x02\x806\x837\x01\x90P[P\x93P`\0[\x82\x81\x10\x15a\x07\xE3W`\0\x84`\x01\x01\x82\x81T\x81\x10a\x07PWa\x07Pa\t\x88V[`\0\x91\x82R` \x80\x83 `\x08\x83\x04\x01T`\x07\x90\x92\x16`\x04\x02a\x01\0\n\x90\x91\x04`\xE0\x1B`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x83R\x90\x87\x90R`@\x90\x91 T\x90\x91P`\x01`\x01`\xA0\x1B\x03\x90\x81\x16\x90\x88\x16\x81\x90\x03a\x07\xD9W\x81\x87\x85\x81Q\x81\x10a\x07\xB4Wa\x07\xB4a\t\x88V[`\x01`\x01`\xE0\x1B\x03\x19\x90\x92\x16` \x92\x83\x02\x91\x90\x91\x01\x90\x91\x01Ra\x07\xD6\x84a\t\xB4V[\x93P[PP`\x01\x01a\x071V[P\x83RP\x90\x92\x91PPV[`\0` \x82\x84\x03\x12\x15a\x08\0W`\0\x80\xFD[\x815`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x81\x14a\x08\x18W`\0\x80\xFD[\x93\x92PPPV[` \x80\x82R\x82Q\x82\x82\x01\x81\x90R`\0\x91\x84\x01\x90`@\x84\x01\x90\x83[\x81\x81\x10\x15a\x08`W\x83Q`\x01`\x01`\xA0\x1B\x03\x16\x83R` \x93\x84\x01\x93\x90\x92\x01\x91`\x01\x01a\x089V[P\x90\x95\x94PPPPPV[`\0\x81Q\x80\x84R` \x84\x01\x93P` \x83\x01`\0[\x82\x81\x10\x15a\x08\xA7W\x81Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x86R` \x95\x86\x01\x95\x90\x91\x01\x90`\x01\x01a\x08\x7FV[P\x93\x94\x93PPPPV[`\0` \x82\x01` \x83R\x80\x84Q\x80\x83R`@\x85\x01\x91P`@\x81`\x05\x1B\x86\x01\x01\x92P` \x86\x01`\0[\x82\x81\x10\x15a\t*W\x86\x85\x03`?\x19\x01\x84R\x81Q\x80Q`\x01`\x01`\xA0\x1B\x03\x16\x86R` \x90\x81\x01Q`@\x91\x87\x01\x82\x90R\x90a\t\x14\x90\x87\x01\x82a\x08kV[\x95PP` \x93\x84\x01\x93\x91\x90\x91\x01\x90`\x01\x01a\x08\xD9V[P\x92\x96\x95PPPPPPV[`\0` \x82\x84\x03\x12\x15a\tHW`\0\x80\xFD[\x815`\x01`\x01`\xA0\x1B\x03\x81\x16\x81\x14a\x08\x18W`\0\x80\xFD[` \x81R`\0a\x08\x18` \x83\x01\x84a\x08kV[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[`\0`\x01\x82\x01a\t\xC6Wa\t\xC6a\t\x9EV[P`\x01\x01\x90V[`\0a\xFF\xFF\x82\x16a\xFF\xFF\x81\x03a\t\xE5Wa\t\xE5a\t\x9EV[`\x01\x01\x92\x91PPV\xFE\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD2\xA2dipfsX\"\x12 \x95:\x07DQ[\xDA^\x1BB+\x9AS\xE7\xED\xB0/^}\xDB\xBD\xD5\xB4%n\"\xAE\x83\xCD\xF2\x10\xF6dsolcC\0\x08\x1A\x003";
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
