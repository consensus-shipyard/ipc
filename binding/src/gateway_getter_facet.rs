pub use gateway_getter_facet::*;
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
pub mod gateway_getter_facet {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("appliedTopDownNonce"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "appliedTopDownNonce",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                    ::std::borrow::ToOwned::to_owned("bottomUpCheckPeriod"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "bottomUpCheckPeriod",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                    ::std::borrow::ToOwned::to_owned("bottomUpCheckpoint"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("bottomUpCheckpoint"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("e"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct BottomUpCheckpoint",
                                        ),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("bottomUpMessages"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("bottomUpMessages"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("e"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
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
                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                        ::std::vec![
                                                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                        ],
                                                                    ),
                                                                ],
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
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
                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                        ::std::vec![
                                                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                        ],
                                                                    ),
                                                                ],
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                        ],
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::Bool,
                                                ],
                                            ),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct CrossMsg[]"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("bottomUpNonce"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("bottomUpNonce"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                    ::std::borrow::ToOwned::to_owned("crossMsgFee"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("crossMsgFee"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("executableQueue"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("executableQueue"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                    ::std::borrow::ToOwned::to_owned("getAppliedTopDownNonce"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getAppliedTopDownNonce",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("subnetId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Address,
                                                ),
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct SubnetID"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                    ::std::borrow::ToOwned::to_owned("getBottomUpRetentionHeight"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getBottomUpRetentionHeight",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                    ::std::borrow::ToOwned::to_owned("getCheckpointAndSignatures"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getCheckpointAndSignatures",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("h"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("ch"),
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
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct BottomUpCheckpoint",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("info"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers::core::abi::ethabi::ParamType::Bool,
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct CheckpointInfo"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("signatures"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes[]"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getCheckpointCurrentWeight"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getCheckpointCurrentWeight",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("h"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getCheckpointInfo"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getCheckpointInfo"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("h"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers::core::abi::ethabi::ParamType::Bool,
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct CheckpointInfo"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getCurrentConfigurationNumber"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getCurrentConfigurationNumber",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                    ::std::borrow::ToOwned::to_owned("getCurrentMembership"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getCurrentMembership",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                ::std::vec![
                                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                ],
                                                            ),
                                                        ],
                                                    ),
                                                ),
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct Membership"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getCurrentTotalWeight"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getCurrentTotalWeight",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getGenesisEpoch"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getGenesisEpoch"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                    ::std::borrow::ToOwned::to_owned("getIncompleteCheckpointHeights"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getIncompleteCheckpointHeights",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256[]"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getIncompleteCheckpoints"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getIncompleteCheckpoints",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
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
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                                ],
                                            ),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct BottomUpCheckpoint[]",
                                        ),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getLastConfigurationNumber"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getLastConfigurationNumber",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                    ::std::borrow::ToOwned::to_owned("getLastMembership"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getLastMembership"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                ::std::vec![
                                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                ],
                                                            ),
                                                        ],
                                                    ),
                                                ),
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct Membership"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getLastTotalWeight"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getLastTotalWeight"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getLatestParentFinality"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getLatestParentFinality",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct ParentFinality"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getNetworkName"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getNetworkName"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Address,
                                                ),
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct SubnetID"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getParentFinality"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getParentFinality"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("blockNumber"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct ParentFinality"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getQuorumThreshold"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getQuorumThreshold"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("totalWeight"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getSubnet"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getSubnet"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("subnetId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Address,
                                                ),
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct SubnetID"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
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
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
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
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Array(
                                                        ::std::boxed::Box::new(
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                ::std::vec![
                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                        ::std::vec![
                                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
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
                                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                                        ::std::vec![
                                                                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                                        ],
                                                                                    ),
                                                                                ],
                                                                            ),
                                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
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
                                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                                        ::std::vec![
                                                                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                                        ],
                                                                                    ),
                                                                                ],
                                                                            ),
                                                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                        ],
                                                                    ),
                                                                    ::ethers::core::abi::ethabi::ParamType::Bool,
                                                                ],
                                                            ),
                                                        ),
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::Array(
                                                        ::std::boxed::Box::new(
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
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
                                                                    ::ethers::core::abi::ethabi::ParamType::Array(
                                                                        ::std::boxed::Box::new(
                                                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                                                        ),
                                                                    ),
                                                                ],
                                                            ),
                                                        ),
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                ],
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct Subnet"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getSubnetTopDownMsgsLength"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "getSubnetTopDownMsgsLength",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("subnetId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Address,
                                                ),
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct SubnetID"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getTopDownMsgs"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("getTopDownMsgs"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("subnetId"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Address,
                                                ),
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct SubnetID"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("fromBlock"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("toBlock"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
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
                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                        ::std::vec![
                                                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                        ],
                                                                    ),
                                                                ],
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
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
                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                        ::std::vec![
                                                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                        ],
                                                                    ),
                                                                ],
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                        ],
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::Bool,
                                                ],
                                            ),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct CrossMsg[]"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("hasValidatorVotedForSubmission"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "hasValidatorVotedForSubmission",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("epoch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint64"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("submitter"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("address"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("initialized"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("initialized"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("lastVotingExecutedEpoch"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "lastVotingExecutedEpoch",
                            ),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                    ::std::borrow::ToOwned::to_owned("listSubnets"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("listSubnets"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
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
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
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
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                                ::std::boxed::Box::new(
                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                        ::std::vec![
                                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                                ::std::vec![
                                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
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
                                                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                                                ::std::vec![
                                                                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                                                ],
                                                                                            ),
                                                                                        ],
                                                                                    ),
                                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
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
                                                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                                                ::std::vec![
                                                                                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                                                ],
                                                                                            ),
                                                                                        ],
                                                                                    ),
                                                                                    ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                                                    ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                                ],
                                                                            ),
                                                                            ::ethers::core::abi::ethabi::ParamType::Bool,
                                                                        ],
                                                                    ),
                                                                ),
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                                ::std::boxed::Box::new(
                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
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
                                                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                                                ::std::boxed::Box::new(
                                                                                    ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                                                                ),
                                                                            ),
                                                                        ],
                                                                    ),
                                                                ),
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                        ],
                                                    ),
                                                ],
                                            ),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct Subnet[]"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("majorityPercentage"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("majorityPercentage"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                    ::std::borrow::ToOwned::to_owned("minStake"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("minStake"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("postbox"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("postbox"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("id"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("storableMsg"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
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
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                        ],
                                                    ),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
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
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                        ],
                                                    ),
                                                ],
                                            ),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct StorableMsg"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("wrapped"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("subnets"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("subnets"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("h"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("subnet"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
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
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
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
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Array(
                                                        ::std::boxed::Box::new(
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                ::std::vec![
                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                        ::std::vec![
                                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
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
                                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                                        ::std::vec![
                                                                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                                        ],
                                                                                    ),
                                                                                ],
                                                                            ),
                                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
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
                                                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                                                        ::std::vec![
                                                                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                                        ],
                                                                                    ),
                                                                                ],
                                                                            ),
                                                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(4usize),
                                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                                        ],
                                                                    ),
                                                                    ::ethers::core::abi::ethabi::ParamType::Bool,
                                                                ],
                                                            ),
                                                        ),
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::Array(
                                                        ::std::boxed::Box::new(
                                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
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
                                                                    ::ethers::core::abi::ethabi::ParamType::Array(
                                                                        ::std::boxed::Box::new(
                                                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                                                        ),
                                                                    ),
                                                                ],
                                                            ),
                                                        ),
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                ],
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct Subnet"),
                                    ),
                                },
                            ],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("topDownCheckPeriod"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("topDownCheckPeriod"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                    ::std::borrow::ToOwned::to_owned("totalSubnets"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("totalSubnets"),
                            inputs: ::std::vec![],
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
            ]),
            events: ::std::collections::BTreeMap::new(),
            errors: ::std::collections::BTreeMap::new(),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static GATEWAYGETTERFACET_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(__abi);
    pub struct GatewayGetterFacet<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for GatewayGetterFacet<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for GatewayGetterFacet<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for GatewayGetterFacet<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for GatewayGetterFacet<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(GatewayGetterFacet))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> GatewayGetterFacet<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                GATEWAYGETTERFACET_ABI.clone(),
                client,
            ))
        }
        ///Calls the contract's `appliedTopDownNonce` (0x8789f83b) function
        pub fn applied_top_down_nonce(&self) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([135, 137, 248, 59], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `bottomUpCheckPeriod` (0x06c46853) function
        pub fn bottom_up_check_period(&self) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([6, 196, 104, 83], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `bottomUpCheckpoint` (0xb45ff172) function
        pub fn bottom_up_checkpoint(
            &self,
            e: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, BottomUpCheckpoint> {
            self.0
                .method_hash([180, 95, 241, 114], e)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `bottomUpMessages` (0xb932488f) function
        pub fn bottom_up_messages(
            &self,
            e: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::vec::Vec<CrossMsg>> {
            self.0
                .method_hash([185, 50, 72, 143], e)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `bottomUpNonce` (0x41b6a2e8) function
        pub fn bottom_up_nonce(&self) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([65, 182, 162, 232], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `crossMsgFee` (0x24729425) function
        pub fn cross_msg_fee(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([36, 114, 148, 37], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `executableQueue` (0x10d500e1) function
        pub fn executable_queue(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, (u64, u64, u64)> {
            self.0
                .method_hash([16, 213, 0, 225], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getAppliedTopDownNonce` (0x9e530b57) function
        pub fn get_applied_top_down_nonce(
            &self,
            subnet_id: SubnetID,
        ) -> ::ethers::contract::builders::ContractCall<M, (bool, u64)> {
            self.0
                .method_hash([158, 83, 11, 87], (subnet_id,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getBottomUpRetentionHeight` (0x3e37cfa2) function
        pub fn get_bottom_up_retention_height(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([62, 55, 207, 162], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getCheckpointAndSignatures` (0x380a62b3) function
        pub fn get_checkpoint_and_signatures(
            &self,
            h: u64,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (
                BottomUpCheckpoint,
                CheckpointInfo,
                ::std::vec::Vec<::ethers::core::types::Bytes>,
            ),
        > {
            self.0
                .method_hash([56, 10, 98, 179], h)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getCheckpointCurrentWeight` (0xf7ceaf76) function
        pub fn get_checkpoint_current_weight(
            &self,
            h: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([247, 206, 175, 118], h)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getCheckpointInfo` (0x4512ff63) function
        pub fn get_checkpoint_info(
            &self,
            h: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, CheckpointInfo> {
            self.0
                .method_hash([69, 18, 255, 99], h)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getCurrentConfigurationNumber` (0x544dddff) function
        pub fn get_current_configuration_number(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([84, 77, 221, 255], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getCurrentMembership` (0x6ad21bb0) function
        pub fn get_current_membership(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, Membership> {
            self.0
                .method_hash([106, 210, 27, 176], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getCurrentTotalWeight` (0x0dc25ea1) function
        pub fn get_current_total_weight(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([13, 194, 94, 161], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getGenesisEpoch` (0x51392fc0) function
        pub fn get_genesis_epoch(&self) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([81, 57, 47, 192], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getIncompleteCheckpointHeights` (0xa517218f) function
        pub fn get_incomplete_checkpoint_heights(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers::core::types::U256>,
        > {
            self.0
                .method_hash([165, 23, 33, 143], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getIncompleteCheckpoints` (0x97042766) function
        pub fn get_incomplete_checkpoints(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::vec::Vec<BottomUpCheckpoint>>
        {
            self.0
                .method_hash([151, 4, 39, 102], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getLastConfigurationNumber` (0xb1ba49b0) function
        pub fn get_last_configuration_number(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([177, 186, 73, 176], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getLastMembership` (0xf3229131) function
        pub fn get_last_membership(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, Membership> {
            self.0
                .method_hash([243, 34, 145, 49], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getLastTotalWeight` (0x7e84946f) function
        pub fn get_last_total_weight(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([126, 132, 148, 111], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getLatestParentFinality` (0x0338150f) function
        pub fn get_latest_parent_finality(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ParentFinality> {
            self.0
                .method_hash([3, 56, 21, 15], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getNetworkName` (0x94074b03) function
        pub fn get_network_name(&self) -> ::ethers::contract::builders::ContractCall<M, SubnetID> {
            self.0
                .method_hash([148, 7, 75, 3], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getParentFinality` (0x7edeac92) function
        pub fn get_parent_finality(
            &self,
            block_number: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ParentFinality> {
            self.0
                .method_hash([126, 222, 172, 146], block_number)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getQuorumThreshold` (0x06572c1a) function
        pub fn get_quorum_threshold(
            &self,
            total_weight: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([6, 87, 44, 26], total_weight)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getSubnet` (0xc66c66a1) function
        pub fn get_subnet(
            &self,
            subnet_id: SubnetID,
        ) -> ::ethers::contract::builders::ContractCall<M, (bool, Subnet)> {
            self.0
                .method_hash([198, 108, 102, 161], (subnet_id,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getSubnetTopDownMsgsLength` (0x9d3070b5) function
        pub fn get_subnet_top_down_msgs_length(
            &self,
            subnet_id: SubnetID,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([157, 48, 112, 181], (subnet_id,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getTopDownMsgs` (0x77296bb1) function
        pub fn get_top_down_msgs(
            &self,
            subnet_id: SubnetID,
            from_block: ::ethers::core::types::U256,
            to_block: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::vec::Vec<CrossMsg>> {
            self.0
                .method_hash([119, 41, 107, 177], (subnet_id, from_block, to_block))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `hasValidatorVotedForSubmission` (0x66d7bbbc) function
        pub fn has_validator_voted_for_submission(
            &self,
            epoch: u64,
            submitter: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([102, 215, 187, 188], (epoch, submitter))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `initialized` (0x158ef93e) function
        pub fn initialized(&self) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([21, 142, 249, 62], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `lastVotingExecutedEpoch` (0xad81e244) function
        pub fn last_voting_executed_epoch(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([173, 129, 226, 68], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `listSubnets` (0x5d029685) function
        pub fn list_subnets(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::vec::Vec<Subnet>> {
            self.0
                .method_hash([93, 2, 150, 133], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `majorityPercentage` (0x599c7bd1) function
        pub fn majority_percentage(&self) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([89, 156, 123, 209], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `minStake` (0x375b3c0a) function
        pub fn min_stake(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([55, 91, 60, 10], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `postbox` (0x8cfd78e7) function
        pub fn postbox(
            &self,
            id: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, (StorableMsg, bool)> {
            self.0
                .method_hash([140, 253, 120, 231], id)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `subnets` (0x02e30f9a) function
        pub fn subnets(
            &self,
            h: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, Subnet> {
            self.0
                .method_hash([2, 227, 15, 154], h)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `topDownCheckPeriod` (0x7d9740f4) function
        pub fn top_down_check_period(&self) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([125, 151, 64, 244], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `totalSubnets` (0xa2b67158) function
        pub fn total_subnets(&self) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([162, 182, 113, 88], ())
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
        for GatewayGetterFacet<M>
    {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Container type for all input parameters for the `appliedTopDownNonce` function with signature `appliedTopDownNonce()` and selector `0x8789f83b`
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
    #[ethcall(name = "appliedTopDownNonce", abi = "appliedTopDownNonce()")]
    pub struct AppliedTopDownNonceCall;
    ///Container type for all input parameters for the `bottomUpCheckPeriod` function with signature `bottomUpCheckPeriod()` and selector `0x06c46853`
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
    #[ethcall(name = "bottomUpCheckPeriod", abi = "bottomUpCheckPeriod()")]
    pub struct BottomUpCheckPeriodCall;
    ///Container type for all input parameters for the `bottomUpCheckpoint` function with signature `bottomUpCheckpoint(uint64)` and selector `0xb45ff172`
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
    #[ethcall(name = "bottomUpCheckpoint", abi = "bottomUpCheckpoint(uint64)")]
    pub struct BottomUpCheckpointCall {
        pub e: u64,
    }
    ///Container type for all input parameters for the `bottomUpMessages` function with signature `bottomUpMessages(uint64)` and selector `0xb932488f`
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
    #[ethcall(name = "bottomUpMessages", abi = "bottomUpMessages(uint64)")]
    pub struct BottomUpMessagesCall {
        pub e: u64,
    }
    ///Container type for all input parameters for the `bottomUpNonce` function with signature `bottomUpNonce()` and selector `0x41b6a2e8`
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
    #[ethcall(name = "bottomUpNonce", abi = "bottomUpNonce()")]
    pub struct BottomUpNonceCall;
    ///Container type for all input parameters for the `crossMsgFee` function with signature `crossMsgFee()` and selector `0x24729425`
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
    #[ethcall(name = "crossMsgFee", abi = "crossMsgFee()")]
    pub struct CrossMsgFeeCall;
    ///Container type for all input parameters for the `executableQueue` function with signature `executableQueue()` and selector `0x10d500e1`
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
    #[ethcall(name = "executableQueue", abi = "executableQueue()")]
    pub struct ExecutableQueueCall;
    ///Container type for all input parameters for the `getAppliedTopDownNonce` function with signature `getAppliedTopDownNonce((uint64,address[]))` and selector `0x9e530b57`
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
        name = "getAppliedTopDownNonce",
        abi = "getAppliedTopDownNonce((uint64,address[]))"
    )]
    pub struct GetAppliedTopDownNonceCall {
        pub subnet_id: SubnetID,
    }
    ///Container type for all input parameters for the `getBottomUpRetentionHeight` function with signature `getBottomUpRetentionHeight()` and selector `0x3e37cfa2`
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
        name = "getBottomUpRetentionHeight",
        abi = "getBottomUpRetentionHeight()"
    )]
    pub struct GetBottomUpRetentionHeightCall;
    ///Container type for all input parameters for the `getCheckpointAndSignatures` function with signature `getCheckpointAndSignatures(uint64)` and selector `0x380a62b3`
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
        name = "getCheckpointAndSignatures",
        abi = "getCheckpointAndSignatures(uint64)"
    )]
    pub struct GetCheckpointAndSignaturesCall {
        pub h: u64,
    }
    ///Container type for all input parameters for the `getCheckpointCurrentWeight` function with signature `getCheckpointCurrentWeight(uint64)` and selector `0xf7ceaf76`
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
        name = "getCheckpointCurrentWeight",
        abi = "getCheckpointCurrentWeight(uint64)"
    )]
    pub struct GetCheckpointCurrentWeightCall {
        pub h: u64,
    }
    ///Container type for all input parameters for the `getCheckpointInfo` function with signature `getCheckpointInfo(uint64)` and selector `0x4512ff63`
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
    #[ethcall(name = "getCheckpointInfo", abi = "getCheckpointInfo(uint64)")]
    pub struct GetCheckpointInfoCall {
        pub h: u64,
    }
    ///Container type for all input parameters for the `getCurrentConfigurationNumber` function with signature `getCurrentConfigurationNumber()` and selector `0x544dddff`
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
        name = "getCurrentConfigurationNumber",
        abi = "getCurrentConfigurationNumber()"
    )]
    pub struct GetCurrentConfigurationNumberCall;
    ///Container type for all input parameters for the `getCurrentMembership` function with signature `getCurrentMembership()` and selector `0x6ad21bb0`
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
    #[ethcall(name = "getCurrentMembership", abi = "getCurrentMembership()")]
    pub struct GetCurrentMembershipCall;
    ///Container type for all input parameters for the `getCurrentTotalWeight` function with signature `getCurrentTotalWeight()` and selector `0x0dc25ea1`
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
    #[ethcall(name = "getCurrentTotalWeight", abi = "getCurrentTotalWeight()")]
    pub struct GetCurrentTotalWeightCall;
    ///Container type for all input parameters for the `getGenesisEpoch` function with signature `getGenesisEpoch()` and selector `0x51392fc0`
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
    #[ethcall(name = "getGenesisEpoch", abi = "getGenesisEpoch()")]
    pub struct GetGenesisEpochCall;
    ///Container type for all input parameters for the `getIncompleteCheckpointHeights` function with signature `getIncompleteCheckpointHeights()` and selector `0xa517218f`
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
        name = "getIncompleteCheckpointHeights",
        abi = "getIncompleteCheckpointHeights()"
    )]
    pub struct GetIncompleteCheckpointHeightsCall;
    ///Container type for all input parameters for the `getIncompleteCheckpoints` function with signature `getIncompleteCheckpoints()` and selector `0x97042766`
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
    #[ethcall(name = "getIncompleteCheckpoints", abi = "getIncompleteCheckpoints()")]
    pub struct GetIncompleteCheckpointsCall;
    ///Container type for all input parameters for the `getLastConfigurationNumber` function with signature `getLastConfigurationNumber()` and selector `0xb1ba49b0`
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
        name = "getLastConfigurationNumber",
        abi = "getLastConfigurationNumber()"
    )]
    pub struct GetLastConfigurationNumberCall;
    ///Container type for all input parameters for the `getLastMembership` function with signature `getLastMembership()` and selector `0xf3229131`
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
    #[ethcall(name = "getLastMembership", abi = "getLastMembership()")]
    pub struct GetLastMembershipCall;
    ///Container type for all input parameters for the `getLastTotalWeight` function with signature `getLastTotalWeight()` and selector `0x7e84946f`
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
    #[ethcall(name = "getLastTotalWeight", abi = "getLastTotalWeight()")]
    pub struct GetLastTotalWeightCall;
    ///Container type for all input parameters for the `getLatestParentFinality` function with signature `getLatestParentFinality()` and selector `0x0338150f`
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
    #[ethcall(name = "getLatestParentFinality", abi = "getLatestParentFinality()")]
    pub struct GetLatestParentFinalityCall;
    ///Container type for all input parameters for the `getNetworkName` function with signature `getNetworkName()` and selector `0x94074b03`
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
    #[ethcall(name = "getNetworkName", abi = "getNetworkName()")]
    pub struct GetNetworkNameCall;
    ///Container type for all input parameters for the `getParentFinality` function with signature `getParentFinality(uint256)` and selector `0x7edeac92`
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
    #[ethcall(name = "getParentFinality", abi = "getParentFinality(uint256)")]
    pub struct GetParentFinalityCall {
        pub block_number: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `getQuorumThreshold` function with signature `getQuorumThreshold(uint256)` and selector `0x06572c1a`
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
    #[ethcall(name = "getQuorumThreshold", abi = "getQuorumThreshold(uint256)")]
    pub struct GetQuorumThresholdCall {
        pub total_weight: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `getSubnet` function with signature `getSubnet((uint64,address[]))` and selector `0xc66c66a1`
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
    #[ethcall(name = "getSubnet", abi = "getSubnet((uint64,address[]))")]
    pub struct GetSubnetCall {
        pub subnet_id: SubnetID,
    }
    ///Container type for all input parameters for the `getSubnetTopDownMsgsLength` function with signature `getSubnetTopDownMsgsLength((uint64,address[]))` and selector `0x9d3070b5`
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
        name = "getSubnetTopDownMsgsLength",
        abi = "getSubnetTopDownMsgsLength((uint64,address[]))"
    )]
    pub struct GetSubnetTopDownMsgsLengthCall {
        pub subnet_id: SubnetID,
    }
    ///Container type for all input parameters for the `getTopDownMsgs` function with signature `getTopDownMsgs((uint64,address[]),uint256,uint256)` and selector `0x77296bb1`
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
        name = "getTopDownMsgs",
        abi = "getTopDownMsgs((uint64,address[]),uint256,uint256)"
    )]
    pub struct GetTopDownMsgsCall {
        pub subnet_id: SubnetID,
        pub from_block: ::ethers::core::types::U256,
        pub to_block: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `hasValidatorVotedForSubmission` function with signature `hasValidatorVotedForSubmission(uint64,address)` and selector `0x66d7bbbc`
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
        name = "hasValidatorVotedForSubmission",
        abi = "hasValidatorVotedForSubmission(uint64,address)"
    )]
    pub struct HasValidatorVotedForSubmissionCall {
        pub epoch: u64,
        pub submitter: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `initialized` function with signature `initialized()` and selector `0x158ef93e`
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
    #[ethcall(name = "initialized", abi = "initialized()")]
    pub struct InitializedCall;
    ///Container type for all input parameters for the `lastVotingExecutedEpoch` function with signature `lastVotingExecutedEpoch()` and selector `0xad81e244`
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
    #[ethcall(name = "lastVotingExecutedEpoch", abi = "lastVotingExecutedEpoch()")]
    pub struct LastVotingExecutedEpochCall;
    ///Container type for all input parameters for the `listSubnets` function with signature `listSubnets()` and selector `0x5d029685`
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
    #[ethcall(name = "listSubnets", abi = "listSubnets()")]
    pub struct ListSubnetsCall;
    ///Container type for all input parameters for the `majorityPercentage` function with signature `majorityPercentage()` and selector `0x599c7bd1`
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
    #[ethcall(name = "majorityPercentage", abi = "majorityPercentage()")]
    pub struct MajorityPercentageCall;
    ///Container type for all input parameters for the `minStake` function with signature `minStake()` and selector `0x375b3c0a`
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
    #[ethcall(name = "minStake", abi = "minStake()")]
    pub struct MinStakeCall;
    ///Container type for all input parameters for the `postbox` function with signature `postbox(bytes32)` and selector `0x8cfd78e7`
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
    #[ethcall(name = "postbox", abi = "postbox(bytes32)")]
    pub struct PostboxCall {
        pub id: [u8; 32],
    }
    ///Container type for all input parameters for the `subnets` function with signature `subnets(bytes32)` and selector `0x02e30f9a`
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
    #[ethcall(name = "subnets", abi = "subnets(bytes32)")]
    pub struct SubnetsCall {
        pub h: [u8; 32],
    }
    ///Container type for all input parameters for the `topDownCheckPeriod` function with signature `topDownCheckPeriod()` and selector `0x7d9740f4`
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
    #[ethcall(name = "topDownCheckPeriod", abi = "topDownCheckPeriod()")]
    pub struct TopDownCheckPeriodCall;
    ///Container type for all input parameters for the `totalSubnets` function with signature `totalSubnets()` and selector `0xa2b67158`
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
    #[ethcall(name = "totalSubnets", abi = "totalSubnets()")]
    pub struct TotalSubnetsCall;
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum GatewayGetterFacetCalls {
        AppliedTopDownNonce(AppliedTopDownNonceCall),
        BottomUpCheckPeriod(BottomUpCheckPeriodCall),
        BottomUpCheckpoint(BottomUpCheckpointCall),
        BottomUpMessages(BottomUpMessagesCall),
        BottomUpNonce(BottomUpNonceCall),
        CrossMsgFee(CrossMsgFeeCall),
        ExecutableQueue(ExecutableQueueCall),
        GetAppliedTopDownNonce(GetAppliedTopDownNonceCall),
        GetBottomUpRetentionHeight(GetBottomUpRetentionHeightCall),
        GetCheckpointAndSignatures(GetCheckpointAndSignaturesCall),
        GetCheckpointCurrentWeight(GetCheckpointCurrentWeightCall),
        GetCheckpointInfo(GetCheckpointInfoCall),
        GetCurrentConfigurationNumber(GetCurrentConfigurationNumberCall),
        GetCurrentMembership(GetCurrentMembershipCall),
        GetCurrentTotalWeight(GetCurrentTotalWeightCall),
        GetGenesisEpoch(GetGenesisEpochCall),
        GetIncompleteCheckpointHeights(GetIncompleteCheckpointHeightsCall),
        GetIncompleteCheckpoints(GetIncompleteCheckpointsCall),
        GetLastConfigurationNumber(GetLastConfigurationNumberCall),
        GetLastMembership(GetLastMembershipCall),
        GetLastTotalWeight(GetLastTotalWeightCall),
        GetLatestParentFinality(GetLatestParentFinalityCall),
        GetNetworkName(GetNetworkNameCall),
        GetParentFinality(GetParentFinalityCall),
        GetQuorumThreshold(GetQuorumThresholdCall),
        GetSubnet(GetSubnetCall),
        GetSubnetTopDownMsgsLength(GetSubnetTopDownMsgsLengthCall),
        GetTopDownMsgs(GetTopDownMsgsCall),
        HasValidatorVotedForSubmission(HasValidatorVotedForSubmissionCall),
        Initialized(InitializedCall),
        LastVotingExecutedEpoch(LastVotingExecutedEpochCall),
        ListSubnets(ListSubnetsCall),
        MajorityPercentage(MajorityPercentageCall),
        MinStake(MinStakeCall),
        Postbox(PostboxCall),
        Subnets(SubnetsCall),
        TopDownCheckPeriod(TopDownCheckPeriodCall),
        TotalSubnets(TotalSubnetsCall),
    }
    impl ::ethers::core::abi::AbiDecode for GatewayGetterFacetCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) =
                <AppliedTopDownNonceCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::AppliedTopDownNonce(decoded));
            }
            if let Ok(decoded) =
                <BottomUpCheckPeriodCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::BottomUpCheckPeriod(decoded));
            }
            if let Ok(decoded) =
                <BottomUpCheckpointCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::BottomUpCheckpoint(decoded));
            }
            if let Ok(decoded) =
                <BottomUpMessagesCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::BottomUpMessages(decoded));
            }
            if let Ok(decoded) = <BottomUpNonceCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::BottomUpNonce(decoded));
            }
            if let Ok(decoded) = <CrossMsgFeeCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::CrossMsgFee(decoded));
            }
            if let Ok(decoded) =
                <ExecutableQueueCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ExecutableQueue(decoded));
            }
            if let Ok(decoded) =
                <GetAppliedTopDownNonceCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetAppliedTopDownNonce(decoded));
            }
            if let Ok(decoded) =
                <GetBottomUpRetentionHeightCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetBottomUpRetentionHeight(decoded));
            }
            if let Ok(decoded) =
                <GetCheckpointAndSignaturesCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetCheckpointAndSignatures(decoded));
            }
            if let Ok(decoded) =
                <GetCheckpointCurrentWeightCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetCheckpointCurrentWeight(decoded));
            }
            if let Ok(decoded) =
                <GetCheckpointInfoCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetCheckpointInfo(decoded));
            }
            if let Ok(decoded) =
                <GetCurrentConfigurationNumberCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetCurrentConfigurationNumber(decoded));
            }
            if let Ok(decoded) =
                <GetCurrentMembershipCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetCurrentMembership(decoded));
            }
            if let Ok(decoded) =
                <GetCurrentTotalWeightCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetCurrentTotalWeight(decoded));
            }
            if let Ok(decoded) =
                <GetGenesisEpochCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetGenesisEpoch(decoded));
            }
            if let Ok(decoded) =
                <GetIncompleteCheckpointHeightsCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetIncompleteCheckpointHeights(decoded));
            }
            if let Ok(decoded) =
                <GetIncompleteCheckpointsCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetIncompleteCheckpoints(decoded));
            }
            if let Ok(decoded) =
                <GetLastConfigurationNumberCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetLastConfigurationNumber(decoded));
            }
            if let Ok(decoded) =
                <GetLastMembershipCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetLastMembership(decoded));
            }
            if let Ok(decoded) =
                <GetLastTotalWeightCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetLastTotalWeight(decoded));
            }
            if let Ok(decoded) =
                <GetLatestParentFinalityCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetLatestParentFinality(decoded));
            }
            if let Ok(decoded) =
                <GetNetworkNameCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetNetworkName(decoded));
            }
            if let Ok(decoded) =
                <GetParentFinalityCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetParentFinality(decoded));
            }
            if let Ok(decoded) =
                <GetQuorumThresholdCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetQuorumThreshold(decoded));
            }
            if let Ok(decoded) = <GetSubnetCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetSubnet(decoded));
            }
            if let Ok(decoded) =
                <GetSubnetTopDownMsgsLengthCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetSubnetTopDownMsgsLength(decoded));
            }
            if let Ok(decoded) =
                <GetTopDownMsgsCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetTopDownMsgs(decoded));
            }
            if let Ok(decoded) =
                <HasValidatorVotedForSubmissionCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::HasValidatorVotedForSubmission(decoded));
            }
            if let Ok(decoded) = <InitializedCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Initialized(decoded));
            }
            if let Ok(decoded) =
                <LastVotingExecutedEpochCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::LastVotingExecutedEpoch(decoded));
            }
            if let Ok(decoded) = <ListSubnetsCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ListSubnets(decoded));
            }
            if let Ok(decoded) =
                <MajorityPercentageCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::MajorityPercentage(decoded));
            }
            if let Ok(decoded) = <MinStakeCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::MinStake(decoded));
            }
            if let Ok(decoded) = <PostboxCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Postbox(decoded));
            }
            if let Ok(decoded) = <SubnetsCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Subnets(decoded));
            }
            if let Ok(decoded) =
                <TopDownCheckPeriodCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::TopDownCheckPeriod(decoded));
            }
            if let Ok(decoded) = <TotalSubnetsCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::TotalSubnets(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for GatewayGetterFacetCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::AppliedTopDownNonce(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BottomUpCheckPeriod(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BottomUpCheckpoint(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BottomUpMessages(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::BottomUpNonce(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::CrossMsgFee(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ExecutableQueue(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetAppliedTopDownNonce(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetBottomUpRetentionHeight(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetCheckpointAndSignatures(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetCheckpointCurrentWeight(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetCheckpointInfo(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetCurrentConfigurationNumber(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetCurrentMembership(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetCurrentTotalWeight(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetGenesisEpoch(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetIncompleteCheckpointHeights(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetIncompleteCheckpoints(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetLastConfigurationNumber(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetLastMembership(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetLastTotalWeight(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetLatestParentFinality(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetNetworkName(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetParentFinality(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetQuorumThreshold(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetSubnet(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetSubnetTopDownMsgsLength(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetTopDownMsgs(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::HasValidatorVotedForSubmission(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Initialized(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::LastVotingExecutedEpoch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ListSubnets(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::MajorityPercentage(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MinStake(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Postbox(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Subnets(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::TopDownCheckPeriod(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TotalSubnets(element) => ::ethers::core::abi::AbiEncode::encode(element),
            }
        }
    }
    impl ::core::fmt::Display for GatewayGetterFacetCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AppliedTopDownNonce(element) => ::core::fmt::Display::fmt(element, f),
                Self::BottomUpCheckPeriod(element) => ::core::fmt::Display::fmt(element, f),
                Self::BottomUpCheckpoint(element) => ::core::fmt::Display::fmt(element, f),
                Self::BottomUpMessages(element) => ::core::fmt::Display::fmt(element, f),
                Self::BottomUpNonce(element) => ::core::fmt::Display::fmt(element, f),
                Self::CrossMsgFee(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExecutableQueue(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetAppliedTopDownNonce(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetBottomUpRetentionHeight(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetCheckpointAndSignatures(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetCheckpointCurrentWeight(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetCheckpointInfo(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetCurrentConfigurationNumber(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetCurrentMembership(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetCurrentTotalWeight(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetGenesisEpoch(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetIncompleteCheckpointHeights(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetIncompleteCheckpoints(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetLastConfigurationNumber(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetLastMembership(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetLastTotalWeight(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetLatestParentFinality(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetNetworkName(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetParentFinality(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetQuorumThreshold(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetSubnet(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetSubnetTopDownMsgsLength(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetTopDownMsgs(element) => ::core::fmt::Display::fmt(element, f),
                Self::HasValidatorVotedForSubmission(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::Initialized(element) => ::core::fmt::Display::fmt(element, f),
                Self::LastVotingExecutedEpoch(element) => ::core::fmt::Display::fmt(element, f),
                Self::ListSubnets(element) => ::core::fmt::Display::fmt(element, f),
                Self::MajorityPercentage(element) => ::core::fmt::Display::fmt(element, f),
                Self::MinStake(element) => ::core::fmt::Display::fmt(element, f),
                Self::Postbox(element) => ::core::fmt::Display::fmt(element, f),
                Self::Subnets(element) => ::core::fmt::Display::fmt(element, f),
                Self::TopDownCheckPeriod(element) => ::core::fmt::Display::fmt(element, f),
                Self::TotalSubnets(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<AppliedTopDownNonceCall> for GatewayGetterFacetCalls {
        fn from(value: AppliedTopDownNonceCall) -> Self {
            Self::AppliedTopDownNonce(value)
        }
    }
    impl ::core::convert::From<BottomUpCheckPeriodCall> for GatewayGetterFacetCalls {
        fn from(value: BottomUpCheckPeriodCall) -> Self {
            Self::BottomUpCheckPeriod(value)
        }
    }
    impl ::core::convert::From<BottomUpCheckpointCall> for GatewayGetterFacetCalls {
        fn from(value: BottomUpCheckpointCall) -> Self {
            Self::BottomUpCheckpoint(value)
        }
    }
    impl ::core::convert::From<BottomUpMessagesCall> for GatewayGetterFacetCalls {
        fn from(value: BottomUpMessagesCall) -> Self {
            Self::BottomUpMessages(value)
        }
    }
    impl ::core::convert::From<BottomUpNonceCall> for GatewayGetterFacetCalls {
        fn from(value: BottomUpNonceCall) -> Self {
            Self::BottomUpNonce(value)
        }
    }
    impl ::core::convert::From<CrossMsgFeeCall> for GatewayGetterFacetCalls {
        fn from(value: CrossMsgFeeCall) -> Self {
            Self::CrossMsgFee(value)
        }
    }
    impl ::core::convert::From<ExecutableQueueCall> for GatewayGetterFacetCalls {
        fn from(value: ExecutableQueueCall) -> Self {
            Self::ExecutableQueue(value)
        }
    }
    impl ::core::convert::From<GetAppliedTopDownNonceCall> for GatewayGetterFacetCalls {
        fn from(value: GetAppliedTopDownNonceCall) -> Self {
            Self::GetAppliedTopDownNonce(value)
        }
    }
    impl ::core::convert::From<GetBottomUpRetentionHeightCall> for GatewayGetterFacetCalls {
        fn from(value: GetBottomUpRetentionHeightCall) -> Self {
            Self::GetBottomUpRetentionHeight(value)
        }
    }
    impl ::core::convert::From<GetCheckpointAndSignaturesCall> for GatewayGetterFacetCalls {
        fn from(value: GetCheckpointAndSignaturesCall) -> Self {
            Self::GetCheckpointAndSignatures(value)
        }
    }
    impl ::core::convert::From<GetCheckpointCurrentWeightCall> for GatewayGetterFacetCalls {
        fn from(value: GetCheckpointCurrentWeightCall) -> Self {
            Self::GetCheckpointCurrentWeight(value)
        }
    }
    impl ::core::convert::From<GetCheckpointInfoCall> for GatewayGetterFacetCalls {
        fn from(value: GetCheckpointInfoCall) -> Self {
            Self::GetCheckpointInfo(value)
        }
    }
    impl ::core::convert::From<GetCurrentConfigurationNumberCall> for GatewayGetterFacetCalls {
        fn from(value: GetCurrentConfigurationNumberCall) -> Self {
            Self::GetCurrentConfigurationNumber(value)
        }
    }
    impl ::core::convert::From<GetCurrentMembershipCall> for GatewayGetterFacetCalls {
        fn from(value: GetCurrentMembershipCall) -> Self {
            Self::GetCurrentMembership(value)
        }
    }
    impl ::core::convert::From<GetCurrentTotalWeightCall> for GatewayGetterFacetCalls {
        fn from(value: GetCurrentTotalWeightCall) -> Self {
            Self::GetCurrentTotalWeight(value)
        }
    }
    impl ::core::convert::From<GetGenesisEpochCall> for GatewayGetterFacetCalls {
        fn from(value: GetGenesisEpochCall) -> Self {
            Self::GetGenesisEpoch(value)
        }
    }
    impl ::core::convert::From<GetIncompleteCheckpointHeightsCall> for GatewayGetterFacetCalls {
        fn from(value: GetIncompleteCheckpointHeightsCall) -> Self {
            Self::GetIncompleteCheckpointHeights(value)
        }
    }
    impl ::core::convert::From<GetIncompleteCheckpointsCall> for GatewayGetterFacetCalls {
        fn from(value: GetIncompleteCheckpointsCall) -> Self {
            Self::GetIncompleteCheckpoints(value)
        }
    }
    impl ::core::convert::From<GetLastConfigurationNumberCall> for GatewayGetterFacetCalls {
        fn from(value: GetLastConfigurationNumberCall) -> Self {
            Self::GetLastConfigurationNumber(value)
        }
    }
    impl ::core::convert::From<GetLastMembershipCall> for GatewayGetterFacetCalls {
        fn from(value: GetLastMembershipCall) -> Self {
            Self::GetLastMembership(value)
        }
    }
    impl ::core::convert::From<GetLastTotalWeightCall> for GatewayGetterFacetCalls {
        fn from(value: GetLastTotalWeightCall) -> Self {
            Self::GetLastTotalWeight(value)
        }
    }
    impl ::core::convert::From<GetLatestParentFinalityCall> for GatewayGetterFacetCalls {
        fn from(value: GetLatestParentFinalityCall) -> Self {
            Self::GetLatestParentFinality(value)
        }
    }
    impl ::core::convert::From<GetNetworkNameCall> for GatewayGetterFacetCalls {
        fn from(value: GetNetworkNameCall) -> Self {
            Self::GetNetworkName(value)
        }
    }
    impl ::core::convert::From<GetParentFinalityCall> for GatewayGetterFacetCalls {
        fn from(value: GetParentFinalityCall) -> Self {
            Self::GetParentFinality(value)
        }
    }
    impl ::core::convert::From<GetQuorumThresholdCall> for GatewayGetterFacetCalls {
        fn from(value: GetQuorumThresholdCall) -> Self {
            Self::GetQuorumThreshold(value)
        }
    }
    impl ::core::convert::From<GetSubnetCall> for GatewayGetterFacetCalls {
        fn from(value: GetSubnetCall) -> Self {
            Self::GetSubnet(value)
        }
    }
    impl ::core::convert::From<GetSubnetTopDownMsgsLengthCall> for GatewayGetterFacetCalls {
        fn from(value: GetSubnetTopDownMsgsLengthCall) -> Self {
            Self::GetSubnetTopDownMsgsLength(value)
        }
    }
    impl ::core::convert::From<GetTopDownMsgsCall> for GatewayGetterFacetCalls {
        fn from(value: GetTopDownMsgsCall) -> Self {
            Self::GetTopDownMsgs(value)
        }
    }
    impl ::core::convert::From<HasValidatorVotedForSubmissionCall> for GatewayGetterFacetCalls {
        fn from(value: HasValidatorVotedForSubmissionCall) -> Self {
            Self::HasValidatorVotedForSubmission(value)
        }
    }
    impl ::core::convert::From<InitializedCall> for GatewayGetterFacetCalls {
        fn from(value: InitializedCall) -> Self {
            Self::Initialized(value)
        }
    }
    impl ::core::convert::From<LastVotingExecutedEpochCall> for GatewayGetterFacetCalls {
        fn from(value: LastVotingExecutedEpochCall) -> Self {
            Self::LastVotingExecutedEpoch(value)
        }
    }
    impl ::core::convert::From<ListSubnetsCall> for GatewayGetterFacetCalls {
        fn from(value: ListSubnetsCall) -> Self {
            Self::ListSubnets(value)
        }
    }
    impl ::core::convert::From<MajorityPercentageCall> for GatewayGetterFacetCalls {
        fn from(value: MajorityPercentageCall) -> Self {
            Self::MajorityPercentage(value)
        }
    }
    impl ::core::convert::From<MinStakeCall> for GatewayGetterFacetCalls {
        fn from(value: MinStakeCall) -> Self {
            Self::MinStake(value)
        }
    }
    impl ::core::convert::From<PostboxCall> for GatewayGetterFacetCalls {
        fn from(value: PostboxCall) -> Self {
            Self::Postbox(value)
        }
    }
    impl ::core::convert::From<SubnetsCall> for GatewayGetterFacetCalls {
        fn from(value: SubnetsCall) -> Self {
            Self::Subnets(value)
        }
    }
    impl ::core::convert::From<TopDownCheckPeriodCall> for GatewayGetterFacetCalls {
        fn from(value: TopDownCheckPeriodCall) -> Self {
            Self::TopDownCheckPeriod(value)
        }
    }
    impl ::core::convert::From<TotalSubnetsCall> for GatewayGetterFacetCalls {
        fn from(value: TotalSubnetsCall) -> Self {
            Self::TotalSubnets(value)
        }
    }
    ///Container type for all return fields from the `appliedTopDownNonce` function with signature `appliedTopDownNonce()` and selector `0x8789f83b`
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
    pub struct AppliedTopDownNonceReturn(pub u64);
    ///Container type for all return fields from the `bottomUpCheckPeriod` function with signature `bottomUpCheckPeriod()` and selector `0x06c46853`
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
    pub struct BottomUpCheckPeriodReturn(pub u64);
    ///Container type for all return fields from the `bottomUpCheckpoint` function with signature `bottomUpCheckpoint(uint64)` and selector `0xb45ff172`
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
    pub struct BottomUpCheckpointReturn(pub BottomUpCheckpoint);
    ///Container type for all return fields from the `bottomUpMessages` function with signature `bottomUpMessages(uint64)` and selector `0xb932488f`
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
    pub struct BottomUpMessagesReturn(pub ::std::vec::Vec<CrossMsg>);
    ///Container type for all return fields from the `bottomUpNonce` function with signature `bottomUpNonce()` and selector `0x41b6a2e8`
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
    pub struct BottomUpNonceReturn(pub u64);
    ///Container type for all return fields from the `crossMsgFee` function with signature `crossMsgFee()` and selector `0x24729425`
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
    pub struct CrossMsgFeeReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `executableQueue` function with signature `executableQueue()` and selector `0x10d500e1`
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
    pub struct ExecutableQueueReturn(pub u64, pub u64, pub u64);
    ///Container type for all return fields from the `getAppliedTopDownNonce` function with signature `getAppliedTopDownNonce((uint64,address[]))` and selector `0x9e530b57`
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
    pub struct GetAppliedTopDownNonceReturn(pub bool, pub u64);
    ///Container type for all return fields from the `getBottomUpRetentionHeight` function with signature `getBottomUpRetentionHeight()` and selector `0x3e37cfa2`
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
    pub struct GetBottomUpRetentionHeightReturn(pub u64);
    ///Container type for all return fields from the `getCheckpointAndSignatures` function with signature `getCheckpointAndSignatures(uint64)` and selector `0x380a62b3`
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
    pub struct GetCheckpointAndSignaturesReturn {
        pub ch: BottomUpCheckpoint,
        pub info: CheckpointInfo,
        pub signatures: ::std::vec::Vec<::ethers::core::types::Bytes>,
    }
    ///Container type for all return fields from the `getCheckpointCurrentWeight` function with signature `getCheckpointCurrentWeight(uint64)` and selector `0xf7ceaf76`
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
    pub struct GetCheckpointCurrentWeightReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `getCheckpointInfo` function with signature `getCheckpointInfo(uint64)` and selector `0x4512ff63`
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
    pub struct GetCheckpointInfoReturn(pub CheckpointInfo);
    ///Container type for all return fields from the `getCurrentConfigurationNumber` function with signature `getCurrentConfigurationNumber()` and selector `0x544dddff`
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
    pub struct GetCurrentConfigurationNumberReturn(pub u64);
    ///Container type for all return fields from the `getCurrentMembership` function with signature `getCurrentMembership()` and selector `0x6ad21bb0`
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
    pub struct GetCurrentMembershipReturn(pub Membership);
    ///Container type for all return fields from the `getCurrentTotalWeight` function with signature `getCurrentTotalWeight()` and selector `0x0dc25ea1`
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
    pub struct GetCurrentTotalWeightReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `getGenesisEpoch` function with signature `getGenesisEpoch()` and selector `0x51392fc0`
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
    pub struct GetGenesisEpochReturn(pub u64);
    ///Container type for all return fields from the `getIncompleteCheckpointHeights` function with signature `getIncompleteCheckpointHeights()` and selector `0xa517218f`
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
    pub struct GetIncompleteCheckpointHeightsReturn(
        pub ::std::vec::Vec<::ethers::core::types::U256>,
    );
    ///Container type for all return fields from the `getIncompleteCheckpoints` function with signature `getIncompleteCheckpoints()` and selector `0x97042766`
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
    pub struct GetIncompleteCheckpointsReturn(pub ::std::vec::Vec<BottomUpCheckpoint>);
    ///Container type for all return fields from the `getLastConfigurationNumber` function with signature `getLastConfigurationNumber()` and selector `0xb1ba49b0`
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
    pub struct GetLastConfigurationNumberReturn(pub u64);
    ///Container type for all return fields from the `getLastMembership` function with signature `getLastMembership()` and selector `0xf3229131`
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
    pub struct GetLastMembershipReturn(pub Membership);
    ///Container type for all return fields from the `getLastTotalWeight` function with signature `getLastTotalWeight()` and selector `0x7e84946f`
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
    pub struct GetLastTotalWeightReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `getLatestParentFinality` function with signature `getLatestParentFinality()` and selector `0x0338150f`
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
    pub struct GetLatestParentFinalityReturn(pub ParentFinality);
    ///Container type for all return fields from the `getNetworkName` function with signature `getNetworkName()` and selector `0x94074b03`
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
    pub struct GetNetworkNameReturn(pub SubnetID);
    ///Container type for all return fields from the `getParentFinality` function with signature `getParentFinality(uint256)` and selector `0x7edeac92`
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
    pub struct GetParentFinalityReturn(pub ParentFinality);
    ///Container type for all return fields from the `getQuorumThreshold` function with signature `getQuorumThreshold(uint256)` and selector `0x06572c1a`
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
    pub struct GetQuorumThresholdReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `getSubnet` function with signature `getSubnet((uint64,address[]))` and selector `0xc66c66a1`
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
    pub struct GetSubnetReturn(pub bool, pub Subnet);
    ///Container type for all return fields from the `getSubnetTopDownMsgsLength` function with signature `getSubnetTopDownMsgsLength((uint64,address[]))` and selector `0x9d3070b5`
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
    pub struct GetSubnetTopDownMsgsLengthReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `getTopDownMsgs` function with signature `getTopDownMsgs((uint64,address[]),uint256,uint256)` and selector `0x77296bb1`
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
    pub struct GetTopDownMsgsReturn(pub ::std::vec::Vec<CrossMsg>);
    ///Container type for all return fields from the `hasValidatorVotedForSubmission` function with signature `hasValidatorVotedForSubmission(uint64,address)` and selector `0x66d7bbbc`
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
    pub struct HasValidatorVotedForSubmissionReturn(pub bool);
    ///Container type for all return fields from the `initialized` function with signature `initialized()` and selector `0x158ef93e`
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
    pub struct InitializedReturn(pub bool);
    ///Container type for all return fields from the `lastVotingExecutedEpoch` function with signature `lastVotingExecutedEpoch()` and selector `0xad81e244`
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
    pub struct LastVotingExecutedEpochReturn(pub u64);
    ///Container type for all return fields from the `listSubnets` function with signature `listSubnets()` and selector `0x5d029685`
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
    pub struct ListSubnetsReturn(pub ::std::vec::Vec<Subnet>);
    ///Container type for all return fields from the `majorityPercentage` function with signature `majorityPercentage()` and selector `0x599c7bd1`
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
    pub struct MajorityPercentageReturn(pub u64);
    ///Container type for all return fields from the `minStake` function with signature `minStake()` and selector `0x375b3c0a`
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
    pub struct MinStakeReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `postbox` function with signature `postbox(bytes32)` and selector `0x8cfd78e7`
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
    pub struct PostboxReturn {
        pub storable_msg: StorableMsg,
        pub wrapped: bool,
    }
    ///Container type for all return fields from the `subnets` function with signature `subnets(bytes32)` and selector `0x02e30f9a`
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
    pub struct SubnetsReturn {
        pub subnet: Subnet,
    }
    ///Container type for all return fields from the `topDownCheckPeriod` function with signature `topDownCheckPeriod()` and selector `0x7d9740f4`
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
    pub struct TopDownCheckPeriodReturn(pub u64);
    ///Container type for all return fields from the `totalSubnets` function with signature `totalSubnets()` and selector `0xa2b67158`
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
    pub struct TotalSubnetsReturn(pub u64);
    ///`BottomUpCheckpoint((uint64,address[]),uint64,bytes32,uint64,bytes32)`
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
    pub struct BottomUpCheckpoint {
        pub subnet_id: SubnetID,
        pub block_height: u64,
        pub block_hash: [u8; 32],
        pub next_configuration_number: u64,
        pub cross_messages_hash: [u8; 32],
    }
    ///`BottomUpCheckpointLegacy((uint64,address[]),uint64,uint256,((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes),bool)[],((uint64,address[]),bytes32[])[],bytes32,bytes)`
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
    pub struct BottomUpCheckpointLegacy {
        pub source: SubnetID,
        pub epoch: u64,
        pub fee: ::ethers::core::types::U256,
        pub cross_msgs: ::std::vec::Vec<CrossMsg>,
        pub children: ::std::vec::Vec<ChildCheck>,
        pub prev_hash: [u8; 32],
        pub proof: ::ethers::core::types::Bytes,
    }
    ///`CheckpointInfo(bytes32,bytes32,uint256,uint256,bool)`
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
    pub struct CheckpointInfo {
        pub hash: [u8; 32],
        pub root_hash: [u8; 32],
        pub threshold: ::ethers::core::types::U256,
        pub current_weight: ::ethers::core::types::U256,
        pub reached: bool,
    }
    ///`ChildCheck((uint64,address[]),bytes32[])`
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
    pub struct ChildCheck {
        pub source: SubnetID,
        pub checks: ::std::vec::Vec<[u8; 32]>,
    }
    ///`CrossMsg((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes),bool)`
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
    pub struct CrossMsg {
        pub message: StorableMsg,
        pub wrapped: bool,
    }
    ///`FvmAddress(uint8,bytes)`
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
    pub struct FvmAddress {
        pub addr_type: u8,
        pub payload: ::ethers::core::types::Bytes,
    }
    ///`Ipcaddress((uint64,address[]),(uint8,bytes))`
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
    pub struct Ipcaddress {
        pub subnet_id: SubnetID,
        pub raw_address: FvmAddress,
    }
    ///`Membership((uint256,(uint8,bytes))[],uint64,uint256)`
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
    pub struct Membership {
        pub validators: ::std::vec::Vec<Validator>,
        pub configuration_number: u64,
        pub total_weight: ::ethers::core::types::U256,
    }
    ///`ParentFinality(uint256,bytes32)`
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
    pub struct ParentFinality {
        pub height: ::ethers::core::types::U256,
        pub block_hash: [u8; 32],
    }
    ///`StorableMsg(((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes)`
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
    pub struct StorableMsg {
        pub from: Ipcaddress,
        pub to: Ipcaddress,
        pub value: ::ethers::core::types::U256,
        pub nonce: u64,
        pub method: [u8; 4],
        pub params: ::ethers::core::types::Bytes,
    }
    ///`Subnet(uint256,uint256,uint256,uint64,uint64,uint8,(uint64,address[]),((uint64,address[]),uint64,uint256,((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes),bool)[],((uint64,address[]),bytes32[])[],bytes32,bytes))`
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
    pub struct Subnet {
        pub stake: ::ethers::core::types::U256,
        pub genesis_epoch: ::ethers::core::types::U256,
        pub circ_supply: ::ethers::core::types::U256,
        pub top_down_nonce: u64,
        pub applied_bottom_up_nonce: u64,
        pub status: u8,
        pub id: SubnetID,
        pub prev_checkpoint: BottomUpCheckpointLegacy,
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
        Hash,
    )]
    pub struct SubnetID {
        pub root: u64,
        pub route: ::std::vec::Vec<::ethers::core::types::Address>,
    }
    ///`Validator(uint256,(uint8,bytes))`
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
    pub struct Validator {
        pub weight: ::ethers::core::types::U256,
        pub addr: FvmAddress,
    }
}
