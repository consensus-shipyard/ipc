pub use subnet_getter_facet::*;
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
pub mod subnet_getter_facet {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("getGateway"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getGateway"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getSubnetActorGetterFacet"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getSubnetActorGetterFacet",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getSubnetActorGetterSelectors"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getSubnetActorGetterSelectors",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getSubnetActorManagerFacet"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getSubnetActorManagerFacet",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getSubnetActorManagerSelectors"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getSubnetActorManagerSelectors",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getSubnetDeployedByNonce"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getSubnetDeployedByNonce",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("owner"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("nonce"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("subnet"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getUserLastNonce"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getUserLastNonce"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("user"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("nonce"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("latestSubnetDeployed"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "latestSubnetDeployed",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("owner"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("subnet"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("updateReferenceSubnetContract"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "updateReferenceSubnetContract",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newGetterFacet"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("newManagerFacet"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newSubnetGetterSelectors",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes4[]"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newSubnetManagerSelectors",
                                    ),
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
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
            ]),
            events: ::std::collections::BTreeMap::new(),
            errors: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("CannotFindSubnet"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("CannotFindSubnet"),
                            inputs: ::std::vec![],
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
                    ::std::borrow::ToOwned::to_owned("NotOwner"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("NotOwner"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
            ]),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static SUBNETGETTERFACET_ABI: ::ethers::contract::Lazy<
        ::ethers::core::abi::Abi,
    > = ::ethers::contract::Lazy::new(__abi);
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = b"`\x80\x80`@R4a\0\x16Wa\n\xDB\x90\x81a\0\x1C\x829\xF3[`\0\x80\xFD\xFE`\x80`@\x90\x80\x82R`\x04\x806\x10\x15a\0\x16W`\0\x80\xFD[`\0\x92`\xE0\x92\x845\x84\x1C\x90\x81c\x03\x0F`Q\x14a\t\x7FWP\x80c\x0B\xE0a\x11\x14a\tVW\x80c\x0FXI\xD1\x14a\x089W\x80c\x11c\xDC\xA5\x14a\x07\xD4W\x80c\x1B\x07f\xC3\x14a\x05\xB1W\x80cB\xBF<\xC1\x14a\x05\x89W\x80c\x986\xB7_\x14a\x05\x16W\x80c\xA3r\xBF0\x14a\x04\xE9Wc\xA4m\x04M\x14a\0\x89W`\0\x80\xFD[4a\x04\xE5W`\x806`\x03\x19\x01\x12a\x04\xE5Wa\0\xA2a\t\xBFV[\x91`\x01`\x01`\xA0\x1B\x03\x90`$5\x82\x81\x16\x90\x81\x90\x03a\x04\xE1Wg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`D5\x81\x81\x11a\x04\xDDWa\0\xDA\x906\x90\x85\x01a\n\x1FV[\x96\x90\x91`d5\x81\x81\x11a\x04\xD9Wa\0\xF4\x906\x90\x87\x01a\n\x1FV[\x96\x90\x92\x81\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD5T\x163\x03a\x04\xC9W\x16\x96\x87\x15a\x04\xBAW\x84\x15a\x04\xBAWPk\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\xA0\x1B\x93`\x01\x97\x85\x89T\x16\x17\x88U`\x02\x94\x85T\x16\x17\x84U\x80\x88\x11a\x03SWh\x01\0\0\0\0\0\0\0\0\x90\x81\x89\x11a\x04\xA7W`\x03\x93\x84T\x8A\x86U\x80\x8B\x10a\x04#W[P\x8B\x90\x99\x85\x82R\x80\x86\x1C\x90\x82[\x82\x81\x10a\x03\xD4WP`\x07\x19\x9B\x81\x8D\x16\x90\x91\x03\x90\x81a\x03fW[PPPP\x86\x11a\x03SW\x85\x11a\x03@W\x90\x84\x91\x84T\x83\x86U\x80\x84\x10a\x02\xB4W[P\x93\x89R\x1C\x94\x87[\x86\x81\x10a\x02RWP\x83\x16\x80\x84\x03\x93\x03a\x01\xE2W\x86\x80\xF3[\x94\x86\x93\x92\x91\x93\x95\x87\x91[\x83\x83\x10a\x02\x14WPPPPPP`\0\x80Q` a\n\x86\x839\x81Q\x91R\x01U8\x80\x80\x80\x80\x80\x86\x80\xF3[\x90\x91\x92\x93` a\x02E\x87\x99a\x02)\x84\x99a\nPV[\x85\x1C\x90\x87\x87\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x98\x01\x95\x94\x93\x01\x91\x90a\x01\xECV[\x85\x90\x89\x8A[`\x08\x81\x10a\x02xWP\x81`\0\x80Q` a\n\x86\x839\x81Q\x91R\x01U\x01a\x01\xCBV[\x95\x91\x92\x90a\x02\xA7` \x91a\x02\x8B\x85a\nPV[\x8D\x1C\x90\x89\x89\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x95\x01\x90\x87\x92\x91a\x02WV[\x90\x91\x92`\x07\x01\x83\x1C`\x07`\0\x80Q` a\n\x86\x839\x81Q\x91R\x92\x01\x84\x1C\x82\x01\x91`\x1C\x88\x87\x1B\x16\x80a\x03\x06W[P\x87\x94\x93\x92\x91\x89\x91\x01[\x82\x81\x10a\x02\xF8WPPa\x01\xC3V[\x8C\x81U\x88\x95P\x89\x91\x01a\x02\xEAV[\x7F\x8A5\xAC\xFB\xC1_\xF8\x1A9\xAE}4O\xD7\t\xF2\x8E\x86\0\xB4\xAA\x8Ce\xC6\xB6K\xFE\x7F\xE3k\xD1\x9A\x83\x01\x90\x81T\x90`\0\x19\x90` \x03\x88\x1B\x1C\x16\x90U8a\x02\xE0V[cNH{q`\xE0\x1B\x89R`A\x84R`$\x89\xFD[cNH{q`\xE0\x1B\x8AR`A\x85R`$\x8A\xFD[\x90\x8D\x8C\x8A\x86\x92[\x84\x84\x10a\x03\x94WPPPPPP`\0\x80Q` a\nf\x839\x81Q\x91R\x01U\x8A8\x80\x80a\x01\xA3V[\x90\x85\x97\x84a\x03\xC4\x93a\x03\xAA` \x96\x97\x98\x99a\nPV[\x90\x1C\x92\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x95\x01\x91\x01\x8C\x8F\x8B\x90\x94\x93\x94a\x03mV[\x90\x91\x92\x8D\x81\x90[`\x08\x82\x10a\x04\x02WPP`\0\x80Q` a\nf\x839\x81Q\x91R\x82\x01U\x8D\x92\x91\x90\x8B\x01a\x01\x8AV[a\x04\x19\x82\x9F\x93\x8F\x93\x90\x8D\x91` \x94a\x03\xAA\x88a\nPV[\x92\x01\x9D\x01\x8Ea\x03\xDBV[`\x07\x8B\x01\x86\x1C`\x07`\0\x80Q` a\nf\x839\x81Q\x91R\x92\x01\x87\x1C\x82\x01\x91`\x1C\x8D\x8A\x1B\x16\x80a\x04mW[P\x8B\x91\x01\x8E[\x83\x82\x10a\x04bWPPPa\x01}V[\x81U\x01\x8A\x90\x8Ea\x04SV[\x7F\xC2WZ\x0E\x9EY<\0\xF9Y\xF8\xC9/\x12\xDB(i\xC39Z;\x05\x02\xD0^%\x16Doq\xF8Z\x83\x01\x90\x81T\x90`\0\x19\x90` \x03\x8B\x1B\x1C\x16\x90U8a\x04MV[cNH{q`\xE0\x1B\x8BR`A\x86R`$\x8B\xFD[Qc\x07\xA0CQ`\xE5\x1B\x81R\x85\x90\xFD[\x88Qc0\xCDtq`\xE0\x1B\x81R\x87\x90\xFD[\x8A\x80\xFD[\x88\x80\xFD[\x86\x80\xFD[\x83\x80\xFD[\x84\x824a\x05\x12W\x81`\x03\x196\x01\x12a\x05\x12W`\x02T\x90Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x81R` \x90\xF3[P\x80\xFD[P\x834a\x05\x86W\x81`\x03\x196\x01\x12a\x05\x86Wa\x050a\t\xBFV[\x90`$5g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x16\x80\x91\x03a\x05\x12W`\x01`\x01`\xA0\x1B\x03\x92\x83\x16\x82R`\x05` \x90\x81R\x84\x83 \x91\x83RR\x82\x90 T\x16\x90\x81\x15a\x05wW` \x92PQ\x90\x81R\xF3[Qc'nt\xA7`\xE1\x1B\x81R\x90P\xFD[\x80\xFD[\x84\x824a\x05\x12W\x81`\x03\x196\x01\x12a\x05\x12W\x90T\x90Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x81R` \x90\xF3[P\x82\x84\x914a\x07\xD0W\x82`\x03\x196\x01\x12a\x07\xD0W\x80Q\x80\x92`\x03T\x90\x81\x83R` \x80\x93\x01\x91`\x03\x87R`\0\x80Q` a\nf\x839\x81Q\x91R\x84\x88\x91[\x83`\x07\x84\x01\x10a\x07cWT\x93\x83\x83\x10a\x07FW[P\x82\x82\x10a\x07(W[\x82\x82\x10a\x07\nW[\x82\x82\x10a\x06\xECW[\x82\x82\x10a\x06\xCEW[\x82\x82\x10a\x06\xB2W[\x82\x82\x10a\x06\x96W[P\x10a\x06\x82W[P\x83\x90\x03`\x1F\x01`\x1F\x19\x16\x83\x01\x93\x90Pg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x84\x11\x83\x85\x10\x17a\x06oWP\x82\x91\x82a\x06k\x92R\x82a\t\xDAV[\x03\x90\xF3[cNH{q`\xE0\x1B\x81R`A\x85R`$\x90\xFD[`\x01`\x01`\xE0\x1B\x03\x19\x16\x81R\x01\x80\x86a\x069V[\x83\x81\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x90\x93\x01\x92`\x01\x01\x84a\x062V[\x83\x87\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x90\x93\x01\x92`\x01\x01\x84a\x06*V[``\x84\x90\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x90\x93\x01\x92`\x01\x01\x84a\x06\"V[`\x80\x84\x90\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x90\x93\x01\x92`\x01\x01\x84a\x06\x1AV[`\xA0\x84\x90\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x90\x93\x01\x92`\x01\x01\x84a\x06\x12V[`\xC0\x84\x90\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x90\x93\x01\x92`\x01\x01\x84a\x06\nV[\x84\x90\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x90\x93\x01\x92`\x01\x01\x84\x8Aa\x06\x01V[\x94`\x08\x91Pa\x01\0`\x01\x91\x87Tc\xFF\xFF\xFF\xFF`\xE0\x1B\x90\x81\x81\x8A\x1B\x16\x83R`\xC0\x82\x82\x82\x1B\x16\x8C\x85\x01R\x8C\x83\x83`\xA0\x92\x82\x82\x85\x1B\x16\x81\x89\x01R`\x80\x83\x83``\x82\x82\x85\x1B\x16\x81\x8D\x01R\x1B\x16\x90\x89\x01R\x1B\x16\x90\x85\x01R\x82\x82\x8D\x1B\x16\x90\x84\x01R\x16\x87\x82\x01R\x01\x95\x01\x91\x01\x90\x85\x90a\x05\xEDV[\x82\x80\xFD[P\x834a\x05\x86W` 6`\x03\x19\x01\x12a\x05\x86W`\x01`\x01`\xA0\x1B\x03\x90\x82\x90\x82a\x07\xFBa\t\xBFV[\x16\x81R`\x06` Rg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\0\x19\x81\x84\x84 T\x16\x01\x16`\x05` R\x82\x82 \x90\x82R` R T\x16\x90\x81\x15a\x05wW` \x92PQ\x90\x81R\xF3[P\x82\x84\x914a\x07\xD0W\x82`\x03\x196\x01\x12a\x07\xD0W\x80Q\x80\x92\x85T\x90\x81\x83R` \x80\x93\x01\x91\x87\x87R`\0\x80Q` a\n\x86\x839\x81Q\x91R\x84\x88\x91[\x83`\x07\x84\x01\x10a\x08\xE9WT\x93\x83\x83\x10a\x07FWP\x82\x82\x10a\x07(W\x82\x82\x10a\x07\nW\x82\x82\x10a\x06\xECW\x82\x82\x10a\x06\xCEW\x82\x82\x10a\x06\xB2W\x82\x82\x10a\x06\x96WP\x10a\x06\x82WP\x83\x90\x03`\x1F\x01`\x1F\x19\x16\x83\x01\x93\x90Pg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x84\x11\x83\x85\x10\x17a\x06oWP\x82\x91\x82a\x06k\x92R\x82a\t\xDAV[\x94`\x08\x91Pa\x01\0`\x01\x91\x87Tc\xFF\xFF\xFF\xFF`\xE0\x1B\x90\x81\x81\x8A\x1B\x16\x83R`\xC0\x82\x82\x82\x1B\x16\x8C\x85\x01R\x8C\x83\x83`\xA0\x92\x82\x82\x85\x1B\x16\x81\x89\x01R`\x80\x83\x83``\x82\x82\x85\x1B\x16\x81\x8D\x01R\x1B\x16\x90\x89\x01R\x1B\x16\x90\x85\x01R\x82\x82\x8D\x1B\x16\x90\x84\x01R\x16\x87\x82\x01R\x01\x95\x01\x91\x01\x90\x85\x90a\x08sV[\x84\x824a\x05\x12W\x81`\x03\x196\x01\x12a\x05\x12W`\x01T\x90Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x81R` \x90\xF3[\x85\x90\x834a\x07\xD0W` 6`\x03\x19\x01\x12a\x07\xD0W` \x92g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x91\x90`\x01`\x01`\xA0\x1B\x03a\t\xB1a\t\xBFV[\x16\x81R`\x06\x85R T\x16\x81R\xF3[`\x045\x90`\x01`\x01`\xA0\x1B\x03\x82\x16\x82\x03a\t\xD5WV[`\0\x80\xFD[` \x90\x81`@\x81\x83\x01\x92\x82\x81R\x85Q\x80\x94R\x01\x93\x01\x91`\0[\x82\x81\x10a\n\x01WPPPP\x90V[\x83Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x93\x81\x01\x93\x92\x81\x01\x92`\x01\x01a\t\xF3V[\x91\x81`\x1F\x84\x01\x12\x15a\t\xD5W\x825\x91g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x83\x11a\t\xD5W` \x80\x85\x01\x94\x84`\x05\x1B\x01\x01\x11a\t\xD5WV[5`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x81\x03a\t\xD5W\x90V\xFE\xC2WZ\x0E\x9EY<\0\xF9Y\xF8\xC9/\x12\xDB(i\xC39Z;\x05\x02\xD0^%\x16Doq\xF8[\x8A5\xAC\xFB\xC1_\xF8\x1A9\xAE}4O\xD7\t\xF2\x8E\x86\0\xB4\xAA\x8Ce\xC6\xB6K\xFE\x7F\xE3k\xD1\x9B\xA2dipfsX\"\x12 \x0E\x0F\xB4J\xDD;1\xF0\xDC\xD8A\xAB\xA5\xFDY\xAD\xFFP&\xC6\xA8\x07n\xD7\x80\x88\xFFMr\xAAt;dsolcC\0\x08\x13\x003";
    /// The bytecode of the contract.
    pub static SUBNETGETTERFACET_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __BYTECODE,
    );
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@\x90\x80\x82R`\x04\x806\x10\x15a\0\x16W`\0\x80\xFD[`\0\x92`\xE0\x92\x845\x84\x1C\x90\x81c\x03\x0F`Q\x14a\t\x7FWP\x80c\x0B\xE0a\x11\x14a\tVW\x80c\x0FXI\xD1\x14a\x089W\x80c\x11c\xDC\xA5\x14a\x07\xD4W\x80c\x1B\x07f\xC3\x14a\x05\xB1W\x80cB\xBF<\xC1\x14a\x05\x89W\x80c\x986\xB7_\x14a\x05\x16W\x80c\xA3r\xBF0\x14a\x04\xE9Wc\xA4m\x04M\x14a\0\x89W`\0\x80\xFD[4a\x04\xE5W`\x806`\x03\x19\x01\x12a\x04\xE5Wa\0\xA2a\t\xBFV[\x91`\x01`\x01`\xA0\x1B\x03\x90`$5\x82\x81\x16\x90\x81\x90\x03a\x04\xE1Wg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`D5\x81\x81\x11a\x04\xDDWa\0\xDA\x906\x90\x85\x01a\n\x1FV[\x96\x90\x91`d5\x81\x81\x11a\x04\xD9Wa\0\xF4\x906\x90\x87\x01a\n\x1FV[\x96\x90\x92\x81\x7F\x80n\x0C\xBB\x9F\xCE)k\xBC3jH\xF4+\xF1\xDB\xC6\x97\"\xD1\x8D\x90\xD6\xFEp[u\x82\xC2\xBBK\xD5T\x163\x03a\x04\xC9W\x16\x96\x87\x15a\x04\xBAW\x84\x15a\x04\xBAWPk\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\xA0\x1B\x93`\x01\x97\x85\x89T\x16\x17\x88U`\x02\x94\x85T\x16\x17\x84U\x80\x88\x11a\x03SWh\x01\0\0\0\0\0\0\0\0\x90\x81\x89\x11a\x04\xA7W`\x03\x93\x84T\x8A\x86U\x80\x8B\x10a\x04#W[P\x8B\x90\x99\x85\x82R\x80\x86\x1C\x90\x82[\x82\x81\x10a\x03\xD4WP`\x07\x19\x9B\x81\x8D\x16\x90\x91\x03\x90\x81a\x03fW[PPPP\x86\x11a\x03SW\x85\x11a\x03@W\x90\x84\x91\x84T\x83\x86U\x80\x84\x10a\x02\xB4W[P\x93\x89R\x1C\x94\x87[\x86\x81\x10a\x02RWP\x83\x16\x80\x84\x03\x93\x03a\x01\xE2W\x86\x80\xF3[\x94\x86\x93\x92\x91\x93\x95\x87\x91[\x83\x83\x10a\x02\x14WPPPPPP`\0\x80Q` a\n\x86\x839\x81Q\x91R\x01U8\x80\x80\x80\x80\x80\x86\x80\xF3[\x90\x91\x92\x93` a\x02E\x87\x99a\x02)\x84\x99a\nPV[\x85\x1C\x90\x87\x87\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x98\x01\x95\x94\x93\x01\x91\x90a\x01\xECV[\x85\x90\x89\x8A[`\x08\x81\x10a\x02xWP\x81`\0\x80Q` a\n\x86\x839\x81Q\x91R\x01U\x01a\x01\xCBV[\x95\x91\x92\x90a\x02\xA7` \x91a\x02\x8B\x85a\nPV[\x8D\x1C\x90\x89\x89\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x92\x01\x95\x01\x90\x87\x92\x91a\x02WV[\x90\x91\x92`\x07\x01\x83\x1C`\x07`\0\x80Q` a\n\x86\x839\x81Q\x91R\x92\x01\x84\x1C\x82\x01\x91`\x1C\x88\x87\x1B\x16\x80a\x03\x06W[P\x87\x94\x93\x92\x91\x89\x91\x01[\x82\x81\x10a\x02\xF8WPPa\x01\xC3V[\x8C\x81U\x88\x95P\x89\x91\x01a\x02\xEAV[\x7F\x8A5\xAC\xFB\xC1_\xF8\x1A9\xAE}4O\xD7\t\xF2\x8E\x86\0\xB4\xAA\x8Ce\xC6\xB6K\xFE\x7F\xE3k\xD1\x9A\x83\x01\x90\x81T\x90`\0\x19\x90` \x03\x88\x1B\x1C\x16\x90U8a\x02\xE0V[cNH{q`\xE0\x1B\x89R`A\x84R`$\x89\xFD[cNH{q`\xE0\x1B\x8AR`A\x85R`$\x8A\xFD[\x90\x8D\x8C\x8A\x86\x92[\x84\x84\x10a\x03\x94WPPPPPP`\0\x80Q` a\nf\x839\x81Q\x91R\x01U\x8A8\x80\x80a\x01\xA3V[\x90\x85\x97\x84a\x03\xC4\x93a\x03\xAA` \x96\x97\x98\x99a\nPV[\x90\x1C\x92\x1B`\x03\x1B\x91c\xFF\xFF\xFF\xFF\x80\x91\x16\x83\x1B\x92\x1B\x19\x16\x17\x90V[\x95\x01\x91\x01\x8C\x8F\x8B\x90\x94\x93\x94a\x03mV[\x90\x91\x92\x8D\x81\x90[`\x08\x82\x10a\x04\x02WPP`\0\x80Q` a\nf\x839\x81Q\x91R\x82\x01U\x8D\x92\x91\x90\x8B\x01a\x01\x8AV[a\x04\x19\x82\x9F\x93\x8F\x93\x90\x8D\x91` \x94a\x03\xAA\x88a\nPV[\x92\x01\x9D\x01\x8Ea\x03\xDBV[`\x07\x8B\x01\x86\x1C`\x07`\0\x80Q` a\nf\x839\x81Q\x91R\x92\x01\x87\x1C\x82\x01\x91`\x1C\x8D\x8A\x1B\x16\x80a\x04mW[P\x8B\x91\x01\x8E[\x83\x82\x10a\x04bWPPPa\x01}V[\x81U\x01\x8A\x90\x8Ea\x04SV[\x7F\xC2WZ\x0E\x9EY<\0\xF9Y\xF8\xC9/\x12\xDB(i\xC39Z;\x05\x02\xD0^%\x16Doq\xF8Z\x83\x01\x90\x81T\x90`\0\x19\x90` \x03\x8B\x1B\x1C\x16\x90U8a\x04MV[cNH{q`\xE0\x1B\x8BR`A\x86R`$\x8B\xFD[Qc\x07\xA0CQ`\xE5\x1B\x81R\x85\x90\xFD[\x88Qc0\xCDtq`\xE0\x1B\x81R\x87\x90\xFD[\x8A\x80\xFD[\x88\x80\xFD[\x86\x80\xFD[\x83\x80\xFD[\x84\x824a\x05\x12W\x81`\x03\x196\x01\x12a\x05\x12W`\x02T\x90Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x81R` \x90\xF3[P\x80\xFD[P\x834a\x05\x86W\x81`\x03\x196\x01\x12a\x05\x86Wa\x050a\t\xBFV[\x90`$5g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x16\x80\x91\x03a\x05\x12W`\x01`\x01`\xA0\x1B\x03\x92\x83\x16\x82R`\x05` \x90\x81R\x84\x83 \x91\x83RR\x82\x90 T\x16\x90\x81\x15a\x05wW` \x92PQ\x90\x81R\xF3[Qc'nt\xA7`\xE1\x1B\x81R\x90P\xFD[\x80\xFD[\x84\x824a\x05\x12W\x81`\x03\x196\x01\x12a\x05\x12W\x90T\x90Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x81R` \x90\xF3[P\x82\x84\x914a\x07\xD0W\x82`\x03\x196\x01\x12a\x07\xD0W\x80Q\x80\x92`\x03T\x90\x81\x83R` \x80\x93\x01\x91`\x03\x87R`\0\x80Q` a\nf\x839\x81Q\x91R\x84\x88\x91[\x83`\x07\x84\x01\x10a\x07cWT\x93\x83\x83\x10a\x07FW[P\x82\x82\x10a\x07(W[\x82\x82\x10a\x07\nW[\x82\x82\x10a\x06\xECW[\x82\x82\x10a\x06\xCEW[\x82\x82\x10a\x06\xB2W[\x82\x82\x10a\x06\x96W[P\x10a\x06\x82W[P\x83\x90\x03`\x1F\x01`\x1F\x19\x16\x83\x01\x93\x90Pg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x84\x11\x83\x85\x10\x17a\x06oWP\x82\x91\x82a\x06k\x92R\x82a\t\xDAV[\x03\x90\xF3[cNH{q`\xE0\x1B\x81R`A\x85R`$\x90\xFD[`\x01`\x01`\xE0\x1B\x03\x19\x16\x81R\x01\x80\x86a\x069V[\x83\x81\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x90\x93\x01\x92`\x01\x01\x84a\x062V[\x83\x87\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x90\x93\x01\x92`\x01\x01\x84a\x06*V[``\x84\x90\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x90\x93\x01\x92`\x01\x01\x84a\x06\"V[`\x80\x84\x90\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x90\x93\x01\x92`\x01\x01\x84a\x06\x1AV[`\xA0\x84\x90\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x90\x93\x01\x92`\x01\x01\x84a\x06\x12V[`\xC0\x84\x90\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x90\x93\x01\x92`\x01\x01\x84a\x06\nV[\x84\x90\x1B`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x90\x93\x01\x92`\x01\x01\x84\x8Aa\x06\x01V[\x94`\x08\x91Pa\x01\0`\x01\x91\x87Tc\xFF\xFF\xFF\xFF`\xE0\x1B\x90\x81\x81\x8A\x1B\x16\x83R`\xC0\x82\x82\x82\x1B\x16\x8C\x85\x01R\x8C\x83\x83`\xA0\x92\x82\x82\x85\x1B\x16\x81\x89\x01R`\x80\x83\x83``\x82\x82\x85\x1B\x16\x81\x8D\x01R\x1B\x16\x90\x89\x01R\x1B\x16\x90\x85\x01R\x82\x82\x8D\x1B\x16\x90\x84\x01R\x16\x87\x82\x01R\x01\x95\x01\x91\x01\x90\x85\x90a\x05\xEDV[\x82\x80\xFD[P\x834a\x05\x86W` 6`\x03\x19\x01\x12a\x05\x86W`\x01`\x01`\xA0\x1B\x03\x90\x82\x90\x82a\x07\xFBa\t\xBFV[\x16\x81R`\x06` Rg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF`\0\x19\x81\x84\x84 T\x16\x01\x16`\x05` R\x82\x82 \x90\x82R` R T\x16\x90\x81\x15a\x05wW` \x92PQ\x90\x81R\xF3[P\x82\x84\x914a\x07\xD0W\x82`\x03\x196\x01\x12a\x07\xD0W\x80Q\x80\x92\x85T\x90\x81\x83R` \x80\x93\x01\x91\x87\x87R`\0\x80Q` a\n\x86\x839\x81Q\x91R\x84\x88\x91[\x83`\x07\x84\x01\x10a\x08\xE9WT\x93\x83\x83\x10a\x07FWP\x82\x82\x10a\x07(W\x82\x82\x10a\x07\nW\x82\x82\x10a\x06\xECW\x82\x82\x10a\x06\xCEW\x82\x82\x10a\x06\xB2W\x82\x82\x10a\x06\x96WP\x10a\x06\x82WP\x83\x90\x03`\x1F\x01`\x1F\x19\x16\x83\x01\x93\x90Pg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x84\x11\x83\x85\x10\x17a\x06oWP\x82\x91\x82a\x06k\x92R\x82a\t\xDAV[\x94`\x08\x91Pa\x01\0`\x01\x91\x87Tc\xFF\xFF\xFF\xFF`\xE0\x1B\x90\x81\x81\x8A\x1B\x16\x83R`\xC0\x82\x82\x82\x1B\x16\x8C\x85\x01R\x8C\x83\x83`\xA0\x92\x82\x82\x85\x1B\x16\x81\x89\x01R`\x80\x83\x83``\x82\x82\x85\x1B\x16\x81\x8D\x01R\x1B\x16\x90\x89\x01R\x1B\x16\x90\x85\x01R\x82\x82\x8D\x1B\x16\x90\x84\x01R\x16\x87\x82\x01R\x01\x95\x01\x91\x01\x90\x85\x90a\x08sV[\x84\x824a\x05\x12W\x81`\x03\x196\x01\x12a\x05\x12W`\x01T\x90Q`\x01`\x01`\xA0\x1B\x03\x90\x91\x16\x81R` \x90\xF3[\x85\x90\x834a\x07\xD0W` 6`\x03\x19\x01\x12a\x07\xD0W` \x92g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x91\x90`\x01`\x01`\xA0\x1B\x03a\t\xB1a\t\xBFV[\x16\x81R`\x06\x85R T\x16\x81R\xF3[`\x045\x90`\x01`\x01`\xA0\x1B\x03\x82\x16\x82\x03a\t\xD5WV[`\0\x80\xFD[` \x90\x81`@\x81\x83\x01\x92\x82\x81R\x85Q\x80\x94R\x01\x93\x01\x91`\0[\x82\x81\x10a\n\x01WPPPP\x90V[\x83Q`\x01`\x01`\xE0\x1B\x03\x19\x16\x85R\x93\x81\x01\x93\x92\x81\x01\x92`\x01\x01a\t\xF3V[\x91\x81`\x1F\x84\x01\x12\x15a\t\xD5W\x825\x91g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x83\x11a\t\xD5W` \x80\x85\x01\x94\x84`\x05\x1B\x01\x01\x11a\t\xD5WV[5`\x01`\x01`\xE0\x1B\x03\x19\x81\x16\x81\x03a\t\xD5W\x90V\xFE\xC2WZ\x0E\x9EY<\0\xF9Y\xF8\xC9/\x12\xDB(i\xC39Z;\x05\x02\xD0^%\x16Doq\xF8[\x8A5\xAC\xFB\xC1_\xF8\x1A9\xAE}4O\xD7\t\xF2\x8E\x86\0\xB4\xAA\x8Ce\xC6\xB6K\xFE\x7F\xE3k\xD1\x9B\xA2dipfsX\"\x12 \x0E\x0F\xB4J\xDD;1\xF0\xDC\xD8A\xAB\xA5\xFDY\xAD\xFFP&\xC6\xA8\x07n\xD7\x80\x88\xFFMr\xAAt;dsolcC\0\x08\x13\x003";
    /// The deployed bytecode of the contract.
    pub static SUBNETGETTERFACET_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __DEPLOYED_BYTECODE,
    );
    pub struct SubnetGetterFacet<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for SubnetGetterFacet<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for SubnetGetterFacet<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for SubnetGetterFacet<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for SubnetGetterFacet<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(SubnetGetterFacet))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> SubnetGetterFacet<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    SUBNETGETTERFACET_ABI.clone(),
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
                SUBNETGETTERFACET_ABI.clone(),
                SUBNETGETTERFACET_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Calls the contract's `getGateway` (0x42bf3cc1) function
        pub fn get_gateway(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([66, 191, 60, 193], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getSubnetActorGetterFacet` (0x0be06111) function
        pub fn get_subnet_actor_getter_facet(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([11, 224, 97, 17], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getSubnetActorGetterSelectors` (0x1b0766c3) function
        pub fn get_subnet_actor_getter_selectors(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::vec::Vec<[u8; 4]>> {
            self.0
                .method_hash([27, 7, 102, 195], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getSubnetActorManagerFacet` (0xa372bf30) function
        pub fn get_subnet_actor_manager_facet(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([163, 114, 191, 48], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getSubnetActorManagerSelectors` (0x0f5849d1) function
        pub fn get_subnet_actor_manager_selectors(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::vec::Vec<[u8; 4]>> {
            self.0
                .method_hash([15, 88, 73, 209], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getSubnetDeployedByNonce` (0x9836b75f) function
        pub fn get_subnet_deployed_by_nonce(
            &self,
            owner: ::ethers::core::types::Address,
            nonce: u64,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([152, 54, 183, 95], (owner, nonce))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getUserLastNonce` (0x030f6051) function
        pub fn get_user_last_nonce(
            &self,
            user: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([3, 15, 96, 81], user)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `latestSubnetDeployed` (0x1163dca5) function
        pub fn latest_subnet_deployed(
            &self,
            owner: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([17, 99, 220, 165], owner)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `updateReferenceSubnetContract` (0xa46d044d) function
        pub fn update_reference_subnet_contract(
            &self,
            new_getter_facet: ::ethers::core::types::Address,
            new_manager_facet: ::ethers::core::types::Address,
            new_subnet_getter_selectors: ::std::vec::Vec<[u8; 4]>,
            new_subnet_manager_selectors: ::std::vec::Vec<[u8; 4]>,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [164, 109, 4, 77],
                    (
                        new_getter_facet,
                        new_manager_facet,
                        new_subnet_getter_selectors,
                        new_subnet_manager_selectors,
                    ),
                )
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for SubnetGetterFacet<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `CannotFindSubnet` with signature `CannotFindSubnet()` and selector `0x4edce94e`
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
    #[etherror(name = "CannotFindSubnet", abi = "CannotFindSubnet()")]
    pub struct CannotFindSubnet;
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
    ///Custom Error type `NotOwner` with signature `NotOwner()` and selector `0x30cd7471`
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
    #[etherror(name = "NotOwner", abi = "NotOwner()")]
    pub struct NotOwner;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum SubnetGetterFacetErrors {
        CannotFindSubnet(CannotFindSubnet),
        FacetCannotBeZero(FacetCannotBeZero),
        NotOwner(NotOwner),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for SubnetGetterFacetErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded) = <CannotFindSubnet as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CannotFindSubnet(decoded));
            }
            if let Ok(decoded) = <FacetCannotBeZero as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::FacetCannotBeZero(decoded));
            }
            if let Ok(decoded) = <NotOwner as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NotOwner(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for SubnetGetterFacetErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::CannotFindSubnet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FacetCannotBeZero(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotOwner(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for SubnetGetterFacetErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <CannotFindSubnet as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <FacetCannotBeZero as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotOwner as ::ethers::contract::EthError>::selector() => true,
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for SubnetGetterFacetErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::CannotFindSubnet(element) => ::core::fmt::Display::fmt(element, f),
                Self::FacetCannotBeZero(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotOwner(element) => ::core::fmt::Display::fmt(element, f),
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for SubnetGetterFacetErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<CannotFindSubnet> for SubnetGetterFacetErrors {
        fn from(value: CannotFindSubnet) -> Self {
            Self::CannotFindSubnet(value)
        }
    }
    impl ::core::convert::From<FacetCannotBeZero> for SubnetGetterFacetErrors {
        fn from(value: FacetCannotBeZero) -> Self {
            Self::FacetCannotBeZero(value)
        }
    }
    impl ::core::convert::From<NotOwner> for SubnetGetterFacetErrors {
        fn from(value: NotOwner) -> Self {
            Self::NotOwner(value)
        }
    }
    ///Container type for all input parameters for the `getGateway` function with signature `getGateway()` and selector `0x42bf3cc1`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getGateway", abi = "getGateway()")]
    pub struct GetGatewayCall;
    ///Container type for all input parameters for the `getSubnetActorGetterFacet` function with signature `getSubnetActorGetterFacet()` and selector `0x0be06111`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getSubnetActorGetterFacet", abi = "getSubnetActorGetterFacet()")]
    pub struct GetSubnetActorGetterFacetCall;
    ///Container type for all input parameters for the `getSubnetActorGetterSelectors` function with signature `getSubnetActorGetterSelectors()` and selector `0x1b0766c3`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "getSubnetActorGetterSelectors",
        abi = "getSubnetActorGetterSelectors()"
    )]
    pub struct GetSubnetActorGetterSelectorsCall;
    ///Container type for all input parameters for the `getSubnetActorManagerFacet` function with signature `getSubnetActorManagerFacet()` and selector `0xa372bf30`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getSubnetActorManagerFacet", abi = "getSubnetActorManagerFacet()")]
    pub struct GetSubnetActorManagerFacetCall;
    ///Container type for all input parameters for the `getSubnetActorManagerSelectors` function with signature `getSubnetActorManagerSelectors()` and selector `0x0f5849d1`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "getSubnetActorManagerSelectors",
        abi = "getSubnetActorManagerSelectors()"
    )]
    pub struct GetSubnetActorManagerSelectorsCall;
    ///Container type for all input parameters for the `getSubnetDeployedByNonce` function with signature `getSubnetDeployedByNonce(address,uint64)` and selector `0x9836b75f`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "getSubnetDeployedByNonce",
        abi = "getSubnetDeployedByNonce(address,uint64)"
    )]
    pub struct GetSubnetDeployedByNonceCall {
        pub owner: ::ethers::core::types::Address,
        pub nonce: u64,
    }
    ///Container type for all input parameters for the `getUserLastNonce` function with signature `getUserLastNonce(address)` and selector `0x030f6051`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "getUserLastNonce", abi = "getUserLastNonce(address)")]
    pub struct GetUserLastNonceCall {
        pub user: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `latestSubnetDeployed` function with signature `latestSubnetDeployed(address)` and selector `0x1163dca5`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(name = "latestSubnetDeployed", abi = "latestSubnetDeployed(address)")]
    pub struct LatestSubnetDeployedCall {
        pub owner: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `updateReferenceSubnetContract` function with signature `updateReferenceSubnetContract(address,address,bytes4[],bytes4[])` and selector `0xa46d044d`
    #[derive(
        Clone,
        ::ethers::contract::EthCall,
        ::ethers::contract::EthDisplay,
        Default,
        Debug,
        PartialEq,
        Eq,
        Hash
    )]
    #[ethcall(
        name = "updateReferenceSubnetContract",
        abi = "updateReferenceSubnetContract(address,address,bytes4[],bytes4[])"
    )]
    pub struct UpdateReferenceSubnetContractCall {
        pub new_getter_facet: ::ethers::core::types::Address,
        pub new_manager_facet: ::ethers::core::types::Address,
        pub new_subnet_getter_selectors: ::std::vec::Vec<[u8; 4]>,
        pub new_subnet_manager_selectors: ::std::vec::Vec<[u8; 4]>,
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum SubnetGetterFacetCalls {
        GetGateway(GetGatewayCall),
        GetSubnetActorGetterFacet(GetSubnetActorGetterFacetCall),
        GetSubnetActorGetterSelectors(GetSubnetActorGetterSelectorsCall),
        GetSubnetActorManagerFacet(GetSubnetActorManagerFacetCall),
        GetSubnetActorManagerSelectors(GetSubnetActorManagerSelectorsCall),
        GetSubnetDeployedByNonce(GetSubnetDeployedByNonceCall),
        GetUserLastNonce(GetUserLastNonceCall),
        LatestSubnetDeployed(LatestSubnetDeployedCall),
        UpdateReferenceSubnetContract(UpdateReferenceSubnetContractCall),
    }
    impl ::ethers::core::abi::AbiDecode for SubnetGetterFacetCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <GetGatewayCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetGateway(decoded));
            }
            if let Ok(decoded) = <GetSubnetActorGetterFacetCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetSubnetActorGetterFacet(decoded));
            }
            if let Ok(decoded) = <GetSubnetActorGetterSelectorsCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetSubnetActorGetterSelectors(decoded));
            }
            if let Ok(decoded) = <GetSubnetActorManagerFacetCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetSubnetActorManagerFacet(decoded));
            }
            if let Ok(decoded) = <GetSubnetActorManagerSelectorsCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetSubnetActorManagerSelectors(decoded));
            }
            if let Ok(decoded) = <GetSubnetDeployedByNonceCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetSubnetDeployedByNonce(decoded));
            }
            if let Ok(decoded) = <GetUserLastNonceCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::GetUserLastNonce(decoded));
            }
            if let Ok(decoded) = <LatestSubnetDeployedCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::LatestSubnetDeployed(decoded));
            }
            if let Ok(decoded) = <UpdateReferenceSubnetContractCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::UpdateReferenceSubnetContract(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for SubnetGetterFacetCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::GetGateway(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetSubnetActorGetterFacet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetSubnetActorGetterSelectors(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetSubnetActorManagerFacet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetSubnetActorManagerSelectors(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetSubnetDeployedByNonce(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetUserLastNonce(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::LatestSubnetDeployed(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::UpdateReferenceSubnetContract(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for SubnetGetterFacetCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::GetGateway(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetSubnetActorGetterFacet(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetSubnetActorGetterSelectors(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetSubnetActorManagerFacet(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetSubnetActorManagerSelectors(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetSubnetDeployedByNonce(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetUserLastNonce(element) => ::core::fmt::Display::fmt(element, f),
                Self::LatestSubnetDeployed(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::UpdateReferenceSubnetContract(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<GetGatewayCall> for SubnetGetterFacetCalls {
        fn from(value: GetGatewayCall) -> Self {
            Self::GetGateway(value)
        }
    }
    impl ::core::convert::From<GetSubnetActorGetterFacetCall>
    for SubnetGetterFacetCalls {
        fn from(value: GetSubnetActorGetterFacetCall) -> Self {
            Self::GetSubnetActorGetterFacet(value)
        }
    }
    impl ::core::convert::From<GetSubnetActorGetterSelectorsCall>
    for SubnetGetterFacetCalls {
        fn from(value: GetSubnetActorGetterSelectorsCall) -> Self {
            Self::GetSubnetActorGetterSelectors(value)
        }
    }
    impl ::core::convert::From<GetSubnetActorManagerFacetCall>
    for SubnetGetterFacetCalls {
        fn from(value: GetSubnetActorManagerFacetCall) -> Self {
            Self::GetSubnetActorManagerFacet(value)
        }
    }
    impl ::core::convert::From<GetSubnetActorManagerSelectorsCall>
    for SubnetGetterFacetCalls {
        fn from(value: GetSubnetActorManagerSelectorsCall) -> Self {
            Self::GetSubnetActorManagerSelectors(value)
        }
    }
    impl ::core::convert::From<GetSubnetDeployedByNonceCall> for SubnetGetterFacetCalls {
        fn from(value: GetSubnetDeployedByNonceCall) -> Self {
            Self::GetSubnetDeployedByNonce(value)
        }
    }
    impl ::core::convert::From<GetUserLastNonceCall> for SubnetGetterFacetCalls {
        fn from(value: GetUserLastNonceCall) -> Self {
            Self::GetUserLastNonce(value)
        }
    }
    impl ::core::convert::From<LatestSubnetDeployedCall> for SubnetGetterFacetCalls {
        fn from(value: LatestSubnetDeployedCall) -> Self {
            Self::LatestSubnetDeployed(value)
        }
    }
    impl ::core::convert::From<UpdateReferenceSubnetContractCall>
    for SubnetGetterFacetCalls {
        fn from(value: UpdateReferenceSubnetContractCall) -> Self {
            Self::UpdateReferenceSubnetContract(value)
        }
    }
    ///Container type for all return fields from the `getGateway` function with signature `getGateway()` and selector `0x42bf3cc1`
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
    pub struct GetGatewayReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `getSubnetActorGetterFacet` function with signature `getSubnetActorGetterFacet()` and selector `0x0be06111`
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
    pub struct GetSubnetActorGetterFacetReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `getSubnetActorGetterSelectors` function with signature `getSubnetActorGetterSelectors()` and selector `0x1b0766c3`
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
    pub struct GetSubnetActorGetterSelectorsReturn(pub ::std::vec::Vec<[u8; 4]>);
    ///Container type for all return fields from the `getSubnetActorManagerFacet` function with signature `getSubnetActorManagerFacet()` and selector `0xa372bf30`
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
    pub struct GetSubnetActorManagerFacetReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `getSubnetActorManagerSelectors` function with signature `getSubnetActorManagerSelectors()` and selector `0x0f5849d1`
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
    pub struct GetSubnetActorManagerSelectorsReturn(pub ::std::vec::Vec<[u8; 4]>);
    ///Container type for all return fields from the `getSubnetDeployedByNonce` function with signature `getSubnetDeployedByNonce(address,uint64)` and selector `0x9836b75f`
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
    pub struct GetSubnetDeployedByNonceReturn {
        pub subnet: ::ethers::core::types::Address,
    }
    ///Container type for all return fields from the `getUserLastNonce` function with signature `getUserLastNonce(address)` and selector `0x030f6051`
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
    pub struct GetUserLastNonceReturn {
        pub nonce: u64,
    }
    ///Container type for all return fields from the `latestSubnetDeployed` function with signature `latestSubnetDeployed(address)` and selector `0x1163dca5`
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
    pub struct LatestSubnetDeployedReturn {
        pub subnet: ::ethers::core::types::Address,
    }
}
