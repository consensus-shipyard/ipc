pub use gateway_router_facet::*;
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
pub mod gateway_router_facet {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("addBottomUpMsgBatchSignature"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "addBottomUpMsgBatchSignature",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("height"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("membershipProof"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32[]"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("weight"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("signature"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("addCheckpointSignature"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "addCheckpointSignature",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("height"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("membershipProof"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32[]"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("weight"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("signature"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("applyCrossMessages"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("applyCrossMessages"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("crossMsgs"),
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
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
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
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("applyFinalityChanges"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "applyFinalityChanges",
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
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("commitCheckpoint"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned("commitCheckpoint"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("checkpoint"),
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
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct BottomUpCheckpoint",
                                        ),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("commitParentFinality"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "commitParentFinality",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("finality"),
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
                            outputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "hasCommittedBefore",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bool"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("previousFinality"),
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
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("createBottomUpCheckpoint"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "createBottomUpCheckpoint",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("checkpoint"),
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
                                            ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct BottomUpCheckpoint",
                                        ),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "membershipRootHash",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("membershipWeight"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("createBottomUpMsgBatch"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "createBottomUpMsgBatch",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("batch"),
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
                                                                    ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                                ],
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::Bool,
                                                        ],
                                                    ),
                                                ),
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct BottomUpMsgBatch"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "membershipRootHash",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("bytes32"),
                                    ),
                                },
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("membershipWeight"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("execBottomUpMsgBatch"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "execBottomUpMsgBatch",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("batch"),
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
                                                                    ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                                ],
                                                            ),
                                                            ::ethers::core::abi::ethabi::ParamType::Bool,
                                                        ],
                                                    ),
                                                ),
                                            ),
                                        ],
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("struct BottomUpMsgBatch"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("pruneBottomUpCheckpoints"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "pruneBottomUpCheckpoints",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newRetentionHeight",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("pruneBottomUpMsgBatches"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "pruneBottomUpMsgBatches",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned(
                                        "newRetentionHeight",
                                    ),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned("uint256"),
                                    ),
                                },
                            ],
                            outputs: ::std::vec![],
                            constant: ::core::option::Option::None,
                            state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("storeValidatorChanges"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Function {
                            name: ::std::borrow::ToOwned::to_owned(
                                "storeValidatorChanges",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("changeRequests"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                        ::std::boxed::Box::new(
                                            ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                ::std::vec![
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                            ::ethers::core::abi::ethabi::ParamType::Address,
                                                        ],
                                                    ),
                                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                ],
                                            ),
                                        ),
                                    ),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "struct StakingChangeRequest[]",
                                        ),
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
                    ::std::borrow::ToOwned::to_owned("AddressEmptyCode"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("AddressEmptyCode"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("target"),
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
                    ::std::borrow::ToOwned::to_owned("AddressInsufficientBalance"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "AddressInsufficientBalance",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("account"),
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
                    ::std::borrow::ToOwned::to_owned("AddressShouldBeValidator"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "AddressShouldBeValidator",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("BatchAlreadyExists"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("BatchAlreadyExists"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("BatchNotCreated"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("BatchNotCreated"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("BatchWithNoMessages"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "BatchWithNoMessages",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CannotConfirmFutureChanges"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotConfirmFutureChanges",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CheckpointAlreadyExists"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CheckpointAlreadyExists",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CheckpointNotCreated"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CheckpointNotCreated",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("FailedAddIncompleteQuorum"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "FailedAddIncompleteQuorum",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("FailedAddSignatory"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("FailedAddSignatory"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("FailedInnerCall"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("FailedInnerCall"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("FailedRemoveIncompleteQuorum"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "FailedRemoveIncompleteQuorum",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidActorAddress"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidActorAddress",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidBatchEpoch"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("InvalidBatchEpoch"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidBatchSource"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("InvalidBatchSource"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidCheckpointEpoch"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidCheckpointEpoch",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidCheckpointSource"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidCheckpointSource",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidConfigurationNumber"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidConfigurationNumber",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidCrossMsgDstSubnet"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidCrossMsgDstSubnet",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidCrossMsgNonce"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidCrossMsgNonce",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidRetentionHeight"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "InvalidRetentionHeight",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidSignature"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("InvalidSignature"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidSubnet"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("InvalidSubnet"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("MaxMsgsPerBatchExceeded"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "MaxMsgsPerBatchExceeded",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NotAuthorized"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("NotAuthorized"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::string::String::new(),
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
                    ::std::borrow::ToOwned::to_owned("NotEnoughBalance"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("NotEnoughBalance"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NotEnoughSubnetCircSupply"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "NotEnoughSubnetCircSupply",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NotRegisteredSubnet"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "NotRegisteredSubnet",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NotSystemActor"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("NotSystemActor"),
                            inputs: ::std::vec![],
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
                    ::std::borrow::ToOwned::to_owned("PQDoesNotContainAddress"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "PQDoesNotContainAddress",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("PQEmpty"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("PQEmpty"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ParentFinalityAlreadyCommitted"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ParentFinalityAlreadyCommitted",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("QuorumAlreadyProcessed"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "QuorumAlreadyProcessed",
                            ),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("SignatureReplay"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("SignatureReplay"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("SubnetNotActive"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("SubnetNotActive"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("SubnetNotFound"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("SubnetNotFound"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ZeroMembershipWeight"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ZeroMembershipWeight",
                            ),
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
    pub static GATEWAYROUTERFACET_ABI: ::ethers::contract::Lazy<
        ::ethers::core::abi::Abi,
    > = ::ethers::contract::Lazy::new(__abi);
    pub struct GatewayRouterFacet<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for GatewayRouterFacet<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for GatewayRouterFacet<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for GatewayRouterFacet<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for GatewayRouterFacet<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(GatewayRouterFacet))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> GatewayRouterFacet<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    GATEWAYROUTERFACET_ABI.clone(),
                    client,
                ),
            )
        }
        ///Calls the contract's `addBottomUpMsgBatchSignature` (0x0db0f77c) function
        pub fn add_bottom_up_msg_batch_signature(
            &self,
            height: ::ethers::core::types::U256,
            membership_proof: ::std::vec::Vec<[u8; 32]>,
            weight: ::ethers::core::types::U256,
            signature: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [13, 176, 247, 124],
                    (height, membership_proof, weight, signature),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `addCheckpointSignature` (0x53b4e7bf) function
        pub fn add_checkpoint_signature(
            &self,
            height: ::ethers::core::types::U256,
            membership_proof: ::std::vec::Vec<[u8; 32]>,
            weight: ::ethers::core::types::U256,
            signature: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [83, 180, 231, 191],
                    (height, membership_proof, weight, signature),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `applyCrossMessages` (0xc62eb4d5) function
        pub fn apply_cross_messages(
            &self,
            cross_msgs: ::std::vec::Vec<CrossMsg>,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([198, 46, 180, 213], cross_msgs)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `applyFinalityChanges` (0x0df14461) function
        pub fn apply_finality_changes(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([13, 241, 68, 97], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `commitCheckpoint` (0x47dc9b4f) function
        pub fn commit_checkpoint(
            &self,
            checkpoint: BottomUpCheckpoint,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([71, 220, 155, 79], (checkpoint,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `commitParentFinality` (0x11196974) function
        pub fn commit_parent_finality(
            &self,
            finality: ParentFinality,
        ) -> ::ethers::contract::builders::ContractCall<M, (bool, ParentFinality)> {
            self.0
                .method_hash([17, 25, 105, 116], (finality,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `createBottomUpCheckpoint` (0x74303771) function
        pub fn create_bottom_up_checkpoint(
            &self,
            checkpoint: BottomUpCheckpoint,
            membership_root_hash: [u8; 32],
            membership_weight: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [116, 48, 55, 113],
                    (checkpoint, membership_root_hash, membership_weight),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `createBottomUpMsgBatch` (0x32e7661f) function
        pub fn create_bottom_up_msg_batch(
            &self,
            batch: BottomUpMsgBatch,
            membership_root_hash: [u8; 32],
            membership_weight: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [50, 231, 102, 31],
                    (batch, membership_root_hash, membership_weight),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `execBottomUpMsgBatch` (0x0bed7615) function
        pub fn exec_bottom_up_msg_batch(
            &self,
            batch: BottomUpMsgBatch,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([11, 237, 118, 21], (batch,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `pruneBottomUpCheckpoints` (0xac818379) function
        pub fn prune_bottom_up_checkpoints(
            &self,
            new_retention_height: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([172, 129, 131, 121], new_retention_height)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `pruneBottomUpMsgBatches` (0xbacc656d) function
        pub fn prune_bottom_up_msg_batches(
            &self,
            new_retention_height: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([186, 204, 101, 109], new_retention_height)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `storeValidatorChanges` (0xe49a547d) function
        pub fn store_validator_changes(
            &self,
            change_requests: ::std::vec::Vec<StakingChangeRequest>,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([228, 154, 84, 125], change_requests)
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for GatewayRouterFacet<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `AddressEmptyCode` with signature `AddressEmptyCode(address)` and selector `0x9996b315`
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
    #[etherror(name = "AddressEmptyCode", abi = "AddressEmptyCode(address)")]
    pub struct AddressEmptyCode {
        pub target: ::ethers::core::types::Address,
    }
    ///Custom Error type `AddressInsufficientBalance` with signature `AddressInsufficientBalance(address)` and selector `0xcd786059`
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
        name = "AddressInsufficientBalance",
        abi = "AddressInsufficientBalance(address)"
    )]
    pub struct AddressInsufficientBalance {
        pub account: ::ethers::core::types::Address,
    }
    ///Custom Error type `AddressShouldBeValidator` with signature `AddressShouldBeValidator()` and selector `0x2a55ca53`
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
    #[etherror(name = "AddressShouldBeValidator", abi = "AddressShouldBeValidator()")]
    pub struct AddressShouldBeValidator;
    ///Custom Error type `BatchAlreadyExists` with signature `BatchAlreadyExists()` and selector `0xd15f973b`
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
    #[etherror(name = "BatchAlreadyExists", abi = "BatchAlreadyExists()")]
    pub struct BatchAlreadyExists;
    ///Custom Error type `BatchNotCreated` with signature `BatchNotCreated()` and selector `0xa88f96f1`
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
    #[etherror(name = "BatchNotCreated", abi = "BatchNotCreated()")]
    pub struct BatchNotCreated;
    ///Custom Error type `BatchWithNoMessages` with signature `BatchWithNoMessages()` and selector `0x38d2307f`
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
    #[etherror(name = "BatchWithNoMessages", abi = "BatchWithNoMessages()")]
    pub struct BatchWithNoMessages;
    ///Custom Error type `CannotConfirmFutureChanges` with signature `CannotConfirmFutureChanges()` and selector `0x0815540a`
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
        name = "CannotConfirmFutureChanges",
        abi = "CannotConfirmFutureChanges()"
    )]
    pub struct CannotConfirmFutureChanges;
    ///Custom Error type `CheckpointAlreadyExists` with signature `CheckpointAlreadyExists()` and selector `0xb8a1eae1`
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
    #[etherror(name = "CheckpointAlreadyExists", abi = "CheckpointAlreadyExists()")]
    pub struct CheckpointAlreadyExists;
    ///Custom Error type `CheckpointNotCreated` with signature `CheckpointNotCreated()` and selector `0x58cdd2e7`
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
    #[etherror(name = "CheckpointNotCreated", abi = "CheckpointNotCreated()")]
    pub struct CheckpointNotCreated;
    ///Custom Error type `FailedAddIncompleteQuorum` with signature `FailedAddIncompleteQuorum()` and selector `0x197a39a6`
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
    #[etherror(name = "FailedAddIncompleteQuorum", abi = "FailedAddIncompleteQuorum()")]
    pub struct FailedAddIncompleteQuorum;
    ///Custom Error type `FailedAddSignatory` with signature `FailedAddSignatory()` and selector `0x3363140f`
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
    #[etherror(name = "FailedAddSignatory", abi = "FailedAddSignatory()")]
    pub struct FailedAddSignatory;
    ///Custom Error type `FailedInnerCall` with signature `FailedInnerCall()` and selector `0x1425ea42`
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
    #[etherror(name = "FailedInnerCall", abi = "FailedInnerCall()")]
    pub struct FailedInnerCall;
    ///Custom Error type `FailedRemoveIncompleteQuorum` with signature `FailedRemoveIncompleteQuorum()` and selector `0x894f690e`
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
        name = "FailedRemoveIncompleteQuorum",
        abi = "FailedRemoveIncompleteQuorum()"
    )]
    pub struct FailedRemoveIncompleteQuorum;
    ///Custom Error type `InvalidActorAddress` with signature `InvalidActorAddress()` and selector `0x70e45109`
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
    #[etherror(name = "InvalidActorAddress", abi = "InvalidActorAddress()")]
    pub struct InvalidActorAddress;
    ///Custom Error type `InvalidBatchEpoch` with signature `InvalidBatchEpoch()` and selector `0x80314bdc`
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
    #[etherror(name = "InvalidBatchEpoch", abi = "InvalidBatchEpoch()")]
    pub struct InvalidBatchEpoch;
    ///Custom Error type `InvalidBatchSource` with signature `InvalidBatchSource()` and selector `0xff949b40`
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
    #[etherror(name = "InvalidBatchSource", abi = "InvalidBatchSource()")]
    pub struct InvalidBatchSource;
    ///Custom Error type `InvalidCheckpointEpoch` with signature `InvalidCheckpointEpoch()` and selector `0xfae4eadb`
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
    #[etherror(name = "InvalidCheckpointEpoch", abi = "InvalidCheckpointEpoch()")]
    pub struct InvalidCheckpointEpoch;
    ///Custom Error type `InvalidCheckpointSource` with signature `InvalidCheckpointSource()` and selector `0xfe72264e`
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
    #[etherror(name = "InvalidCheckpointSource", abi = "InvalidCheckpointSource()")]
    pub struct InvalidCheckpointSource;
    ///Custom Error type `InvalidConfigurationNumber` with signature `InvalidConfigurationNumber()` and selector `0x6ae94ca4`
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
        name = "InvalidConfigurationNumber",
        abi = "InvalidConfigurationNumber()"
    )]
    pub struct InvalidConfigurationNumber;
    ///Custom Error type `InvalidCrossMsgDstSubnet` with signature `InvalidCrossMsgDstSubnet()` and selector `0xc5f563eb`
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
    #[etherror(name = "InvalidCrossMsgDstSubnet", abi = "InvalidCrossMsgDstSubnet()")]
    pub struct InvalidCrossMsgDstSubnet;
    ///Custom Error type `InvalidCrossMsgNonce` with signature `InvalidCrossMsgNonce()` and selector `0xa57cadff`
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
    #[etherror(name = "InvalidCrossMsgNonce", abi = "InvalidCrossMsgNonce()")]
    pub struct InvalidCrossMsgNonce;
    ///Custom Error type `InvalidRetentionHeight` with signature `InvalidRetentionHeight()` and selector `0x6819a3a9`
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
    #[etherror(name = "InvalidRetentionHeight", abi = "InvalidRetentionHeight()")]
    pub struct InvalidRetentionHeight;
    ///Custom Error type `InvalidSignature` with signature `InvalidSignature()` and selector `0x8baa579f`
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
    #[etherror(name = "InvalidSignature", abi = "InvalidSignature()")]
    pub struct InvalidSignature;
    ///Custom Error type `InvalidSubnet` with signature `InvalidSubnet()` and selector `0x076bb706`
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
    #[etherror(name = "InvalidSubnet", abi = "InvalidSubnet()")]
    pub struct InvalidSubnet;
    ///Custom Error type `MaxMsgsPerBatchExceeded` with signature `MaxMsgsPerBatchExceeded()` and selector `0x351c7007`
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
    #[etherror(name = "MaxMsgsPerBatchExceeded", abi = "MaxMsgsPerBatchExceeded()")]
    pub struct MaxMsgsPerBatchExceeded;
    ///Custom Error type `NotAuthorized` with signature `NotAuthorized(address)` and selector `0x4a0bfec1`
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
    #[etherror(name = "NotAuthorized", abi = "NotAuthorized(address)")]
    pub struct NotAuthorized(pub ::ethers::core::types::Address);
    ///Custom Error type `NotEnoughBalance` with signature `NotEnoughBalance()` and selector `0xad3a8b9e`
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
    #[etherror(name = "NotEnoughBalance", abi = "NotEnoughBalance()")]
    pub struct NotEnoughBalance;
    ///Custom Error type `NotEnoughSubnetCircSupply` with signature `NotEnoughSubnetCircSupply()` and selector `0x74db2854`
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
    #[etherror(name = "NotEnoughSubnetCircSupply", abi = "NotEnoughSubnetCircSupply()")]
    pub struct NotEnoughSubnetCircSupply;
    ///Custom Error type `NotRegisteredSubnet` with signature `NotRegisteredSubnet()` and selector `0xe991abd0`
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
    #[etherror(name = "NotRegisteredSubnet", abi = "NotRegisteredSubnet()")]
    pub struct NotRegisteredSubnet;
    ///Custom Error type `NotSystemActor` with signature `NotSystemActor()` and selector `0xf0d97f3b`
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
    #[etherror(name = "NotSystemActor", abi = "NotSystemActor()")]
    pub struct NotSystemActor;
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
    ///Custom Error type `PQDoesNotContainAddress` with signature `PQDoesNotContainAddress()` and selector `0xf2755e37`
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
    #[etherror(name = "PQDoesNotContainAddress", abi = "PQDoesNotContainAddress()")]
    pub struct PQDoesNotContainAddress;
    ///Custom Error type `PQEmpty` with signature `PQEmpty()` and selector `0x40d9b011`
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
    #[etherror(name = "PQEmpty", abi = "PQEmpty()")]
    pub struct PQEmpty;
    ///Custom Error type `ParentFinalityAlreadyCommitted` with signature `ParentFinalityAlreadyCommitted()` and selector `0x2a75b082`
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
        name = "ParentFinalityAlreadyCommitted",
        abi = "ParentFinalityAlreadyCommitted()"
    )]
    pub struct ParentFinalityAlreadyCommitted;
    ///Custom Error type `QuorumAlreadyProcessed` with signature `QuorumAlreadyProcessed()` and selector `0x042384dc`
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
    #[etherror(name = "QuorumAlreadyProcessed", abi = "QuorumAlreadyProcessed()")]
    pub struct QuorumAlreadyProcessed;
    ///Custom Error type `SignatureReplay` with signature `SignatureReplay()` and selector `0xb47fa1b2`
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
    #[etherror(name = "SignatureReplay", abi = "SignatureReplay()")]
    pub struct SignatureReplay;
    ///Custom Error type `SubnetNotActive` with signature `SubnetNotActive()` and selector `0xc18316bf`
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
    #[etherror(name = "SubnetNotActive", abi = "SubnetNotActive()")]
    pub struct SubnetNotActive;
    ///Custom Error type `SubnetNotFound` with signature `SubnetNotFound()` and selector `0x00476ad8`
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
    #[etherror(name = "SubnetNotFound", abi = "SubnetNotFound()")]
    pub struct SubnetNotFound;
    ///Custom Error type `ZeroMembershipWeight` with signature `ZeroMembershipWeight()` and selector `0x4e8ac6e5`
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
    #[etherror(name = "ZeroMembershipWeight", abi = "ZeroMembershipWeight()")]
    pub struct ZeroMembershipWeight;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum GatewayRouterFacetErrors {
        AddressEmptyCode(AddressEmptyCode),
        AddressInsufficientBalance(AddressInsufficientBalance),
        AddressShouldBeValidator(AddressShouldBeValidator),
        BatchAlreadyExists(BatchAlreadyExists),
        BatchNotCreated(BatchNotCreated),
        BatchWithNoMessages(BatchWithNoMessages),
        CannotConfirmFutureChanges(CannotConfirmFutureChanges),
        CheckpointAlreadyExists(CheckpointAlreadyExists),
        CheckpointNotCreated(CheckpointNotCreated),
        FailedAddIncompleteQuorum(FailedAddIncompleteQuorum),
        FailedAddSignatory(FailedAddSignatory),
        FailedInnerCall(FailedInnerCall),
        FailedRemoveIncompleteQuorum(FailedRemoveIncompleteQuorum),
        InvalidActorAddress(InvalidActorAddress),
        InvalidBatchEpoch(InvalidBatchEpoch),
        InvalidBatchSource(InvalidBatchSource),
        InvalidCheckpointEpoch(InvalidCheckpointEpoch),
        InvalidCheckpointSource(InvalidCheckpointSource),
        InvalidConfigurationNumber(InvalidConfigurationNumber),
        InvalidCrossMsgDstSubnet(InvalidCrossMsgDstSubnet),
        InvalidCrossMsgNonce(InvalidCrossMsgNonce),
        InvalidRetentionHeight(InvalidRetentionHeight),
        InvalidSignature(InvalidSignature),
        InvalidSubnet(InvalidSubnet),
        MaxMsgsPerBatchExceeded(MaxMsgsPerBatchExceeded),
        NotAuthorized(NotAuthorized),
        NotEnoughBalance(NotEnoughBalance),
        NotEnoughSubnetCircSupply(NotEnoughSubnetCircSupply),
        NotRegisteredSubnet(NotRegisteredSubnet),
        NotSystemActor(NotSystemActor),
        OldConfigurationNumber(OldConfigurationNumber),
        PQDoesNotContainAddress(PQDoesNotContainAddress),
        PQEmpty(PQEmpty),
        ParentFinalityAlreadyCommitted(ParentFinalityAlreadyCommitted),
        QuorumAlreadyProcessed(QuorumAlreadyProcessed),
        SignatureReplay(SignatureReplay),
        SubnetNotActive(SubnetNotActive),
        SubnetNotFound(SubnetNotFound),
        ZeroMembershipWeight(ZeroMembershipWeight),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for GatewayRouterFacetErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded) = <AddressEmptyCode as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AddressEmptyCode(decoded));
            }
            if let Ok(decoded) = <AddressInsufficientBalance as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AddressInsufficientBalance(decoded));
            }
            if let Ok(decoded) = <AddressShouldBeValidator as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AddressShouldBeValidator(decoded));
            }
            if let Ok(decoded) = <BatchAlreadyExists as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::BatchAlreadyExists(decoded));
            }
            if let Ok(decoded) = <BatchNotCreated as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::BatchNotCreated(decoded));
            }
            if let Ok(decoded) = <BatchWithNoMessages as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::BatchWithNoMessages(decoded));
            }
            if let Ok(decoded) = <CannotConfirmFutureChanges as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CannotConfirmFutureChanges(decoded));
            }
            if let Ok(decoded) = <CheckpointAlreadyExists as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CheckpointAlreadyExists(decoded));
            }
            if let Ok(decoded) = <CheckpointNotCreated as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CheckpointNotCreated(decoded));
            }
            if let Ok(decoded) = <FailedAddIncompleteQuorum as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::FailedAddIncompleteQuorum(decoded));
            }
            if let Ok(decoded) = <FailedAddSignatory as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::FailedAddSignatory(decoded));
            }
            if let Ok(decoded) = <FailedInnerCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::FailedInnerCall(decoded));
            }
            if let Ok(decoded) = <FailedRemoveIncompleteQuorum as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::FailedRemoveIncompleteQuorum(decoded));
            }
            if let Ok(decoded) = <InvalidActorAddress as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidActorAddress(decoded));
            }
            if let Ok(decoded) = <InvalidBatchEpoch as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidBatchEpoch(decoded));
            }
            if let Ok(decoded) = <InvalidBatchSource as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidBatchSource(decoded));
            }
            if let Ok(decoded) = <InvalidCheckpointEpoch as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidCheckpointEpoch(decoded));
            }
            if let Ok(decoded) = <InvalidCheckpointSource as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidCheckpointSource(decoded));
            }
            if let Ok(decoded) = <InvalidConfigurationNumber as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidConfigurationNumber(decoded));
            }
            if let Ok(decoded) = <InvalidCrossMsgDstSubnet as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidCrossMsgDstSubnet(decoded));
            }
            if let Ok(decoded) = <InvalidCrossMsgNonce as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidCrossMsgNonce(decoded));
            }
            if let Ok(decoded) = <InvalidRetentionHeight as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidRetentionHeight(decoded));
            }
            if let Ok(decoded) = <InvalidSignature as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidSignature(decoded));
            }
            if let Ok(decoded) = <InvalidSubnet as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::InvalidSubnet(decoded));
            }
            if let Ok(decoded) = <MaxMsgsPerBatchExceeded as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::MaxMsgsPerBatchExceeded(decoded));
            }
            if let Ok(decoded) = <NotAuthorized as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NotAuthorized(decoded));
            }
            if let Ok(decoded) = <NotEnoughBalance as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NotEnoughBalance(decoded));
            }
            if let Ok(decoded) = <NotEnoughSubnetCircSupply as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NotEnoughSubnetCircSupply(decoded));
            }
            if let Ok(decoded) = <NotRegisteredSubnet as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NotRegisteredSubnet(decoded));
            }
            if let Ok(decoded) = <NotSystemActor as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NotSystemActor(decoded));
            }
            if let Ok(decoded) = <OldConfigurationNumber as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::OldConfigurationNumber(decoded));
            }
            if let Ok(decoded) = <PQDoesNotContainAddress as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PQDoesNotContainAddress(decoded));
            }
            if let Ok(decoded) = <PQEmpty as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PQEmpty(decoded));
            }
            if let Ok(decoded) = <ParentFinalityAlreadyCommitted as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ParentFinalityAlreadyCommitted(decoded));
            }
            if let Ok(decoded) = <QuorumAlreadyProcessed as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::QuorumAlreadyProcessed(decoded));
            }
            if let Ok(decoded) = <SignatureReplay as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SignatureReplay(decoded));
            }
            if let Ok(decoded) = <SubnetNotActive as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SubnetNotActive(decoded));
            }
            if let Ok(decoded) = <SubnetNotFound as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::SubnetNotFound(decoded));
            }
            if let Ok(decoded) = <ZeroMembershipWeight as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ZeroMembershipWeight(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for GatewayRouterFacetErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::AddressEmptyCode(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AddressInsufficientBalance(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AddressShouldBeValidator(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BatchAlreadyExists(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BatchNotCreated(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BatchWithNoMessages(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotConfirmFutureChanges(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CheckpointAlreadyExists(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CheckpointNotCreated(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FailedAddIncompleteQuorum(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FailedAddSignatory(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FailedInnerCall(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FailedRemoveIncompleteQuorum(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidActorAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidBatchEpoch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidBatchSource(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidCheckpointEpoch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidCheckpointSource(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidConfigurationNumber(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidCrossMsgDstSubnet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidCrossMsgNonce(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidRetentionHeight(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidSignature(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidSubnet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MaxMsgsPerBatchExceeded(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotAuthorized(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotEnoughBalance(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotEnoughSubnetCircSupply(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotRegisteredSubnet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotSystemActor(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::OldConfigurationNumber(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PQDoesNotContainAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PQEmpty(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ParentFinalityAlreadyCommitted(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::QuorumAlreadyProcessed(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SignatureReplay(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SubnetNotActive(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SubnetNotFound(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ZeroMembershipWeight(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for GatewayRouterFacetErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <AddressEmptyCode as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <AddressInsufficientBalance as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <AddressShouldBeValidator as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <BatchAlreadyExists as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <BatchNotCreated as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <BatchWithNoMessages as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CannotConfirmFutureChanges as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CheckpointAlreadyExists as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CheckpointNotCreated as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <FailedAddIncompleteQuorum as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <FailedAddSignatory as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <FailedInnerCall as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <FailedRemoveIncompleteQuorum as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidActorAddress as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidBatchEpoch as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidBatchSource as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidCheckpointEpoch as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidCheckpointSource as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidConfigurationNumber as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidCrossMsgDstSubnet as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidCrossMsgNonce as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidRetentionHeight as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidSignature as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidSubnet as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <MaxMsgsPerBatchExceeded as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotAuthorized as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotEnoughBalance as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotEnoughSubnetCircSupply as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotRegisteredSubnet as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotSystemActor as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <OldConfigurationNumber as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <PQDoesNotContainAddress as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <PQEmpty as ::ethers::contract::EthError>::selector() => true,
                _ if selector
                    == <ParentFinalityAlreadyCommitted as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <QuorumAlreadyProcessed as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <SignatureReplay as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <SubnetNotActive as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <SubnetNotFound as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ZeroMembershipWeight as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for GatewayRouterFacetErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AddressEmptyCode(element) => ::core::fmt::Display::fmt(element, f),
                Self::AddressInsufficientBalance(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AddressShouldBeValidator(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::BatchAlreadyExists(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::BatchNotCreated(element) => ::core::fmt::Display::fmt(element, f),
                Self::BatchWithNoMessages(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotConfirmFutureChanges(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CheckpointAlreadyExists(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CheckpointNotCreated(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::FailedAddIncompleteQuorum(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::FailedAddSignatory(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::FailedInnerCall(element) => ::core::fmt::Display::fmt(element, f),
                Self::FailedRemoveIncompleteQuorum(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidActorAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidBatchEpoch(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidBatchSource(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidCheckpointEpoch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidCheckpointSource(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidConfigurationNumber(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidCrossMsgDstSubnet(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidCrossMsgNonce(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidRetentionHeight(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidSignature(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidSubnet(element) => ::core::fmt::Display::fmt(element, f),
                Self::MaxMsgsPerBatchExceeded(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotAuthorized(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotEnoughBalance(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotEnoughSubnetCircSupply(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotRegisteredSubnet(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotSystemActor(element) => ::core::fmt::Display::fmt(element, f),
                Self::OldConfigurationNumber(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PQDoesNotContainAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PQEmpty(element) => ::core::fmt::Display::fmt(element, f),
                Self::ParentFinalityAlreadyCommitted(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::QuorumAlreadyProcessed(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SignatureReplay(element) => ::core::fmt::Display::fmt(element, f),
                Self::SubnetNotActive(element) => ::core::fmt::Display::fmt(element, f),
                Self::SubnetNotFound(element) => ::core::fmt::Display::fmt(element, f),
                Self::ZeroMembershipWeight(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for GatewayRouterFacetErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<AddressEmptyCode> for GatewayRouterFacetErrors {
        fn from(value: AddressEmptyCode) -> Self {
            Self::AddressEmptyCode(value)
        }
    }
    impl ::core::convert::From<AddressInsufficientBalance> for GatewayRouterFacetErrors {
        fn from(value: AddressInsufficientBalance) -> Self {
            Self::AddressInsufficientBalance(value)
        }
    }
    impl ::core::convert::From<AddressShouldBeValidator> for GatewayRouterFacetErrors {
        fn from(value: AddressShouldBeValidator) -> Self {
            Self::AddressShouldBeValidator(value)
        }
    }
    impl ::core::convert::From<BatchAlreadyExists> for GatewayRouterFacetErrors {
        fn from(value: BatchAlreadyExists) -> Self {
            Self::BatchAlreadyExists(value)
        }
    }
    impl ::core::convert::From<BatchNotCreated> for GatewayRouterFacetErrors {
        fn from(value: BatchNotCreated) -> Self {
            Self::BatchNotCreated(value)
        }
    }
    impl ::core::convert::From<BatchWithNoMessages> for GatewayRouterFacetErrors {
        fn from(value: BatchWithNoMessages) -> Self {
            Self::BatchWithNoMessages(value)
        }
    }
    impl ::core::convert::From<CannotConfirmFutureChanges> for GatewayRouterFacetErrors {
        fn from(value: CannotConfirmFutureChanges) -> Self {
            Self::CannotConfirmFutureChanges(value)
        }
    }
    impl ::core::convert::From<CheckpointAlreadyExists> for GatewayRouterFacetErrors {
        fn from(value: CheckpointAlreadyExists) -> Self {
            Self::CheckpointAlreadyExists(value)
        }
    }
    impl ::core::convert::From<CheckpointNotCreated> for GatewayRouterFacetErrors {
        fn from(value: CheckpointNotCreated) -> Self {
            Self::CheckpointNotCreated(value)
        }
    }
    impl ::core::convert::From<FailedAddIncompleteQuorum> for GatewayRouterFacetErrors {
        fn from(value: FailedAddIncompleteQuorum) -> Self {
            Self::FailedAddIncompleteQuorum(value)
        }
    }
    impl ::core::convert::From<FailedAddSignatory> for GatewayRouterFacetErrors {
        fn from(value: FailedAddSignatory) -> Self {
            Self::FailedAddSignatory(value)
        }
    }
    impl ::core::convert::From<FailedInnerCall> for GatewayRouterFacetErrors {
        fn from(value: FailedInnerCall) -> Self {
            Self::FailedInnerCall(value)
        }
    }
    impl ::core::convert::From<FailedRemoveIncompleteQuorum>
    for GatewayRouterFacetErrors {
        fn from(value: FailedRemoveIncompleteQuorum) -> Self {
            Self::FailedRemoveIncompleteQuorum(value)
        }
    }
    impl ::core::convert::From<InvalidActorAddress> for GatewayRouterFacetErrors {
        fn from(value: InvalidActorAddress) -> Self {
            Self::InvalidActorAddress(value)
        }
    }
    impl ::core::convert::From<InvalidBatchEpoch> for GatewayRouterFacetErrors {
        fn from(value: InvalidBatchEpoch) -> Self {
            Self::InvalidBatchEpoch(value)
        }
    }
    impl ::core::convert::From<InvalidBatchSource> for GatewayRouterFacetErrors {
        fn from(value: InvalidBatchSource) -> Self {
            Self::InvalidBatchSource(value)
        }
    }
    impl ::core::convert::From<InvalidCheckpointEpoch> for GatewayRouterFacetErrors {
        fn from(value: InvalidCheckpointEpoch) -> Self {
            Self::InvalidCheckpointEpoch(value)
        }
    }
    impl ::core::convert::From<InvalidCheckpointSource> for GatewayRouterFacetErrors {
        fn from(value: InvalidCheckpointSource) -> Self {
            Self::InvalidCheckpointSource(value)
        }
    }
    impl ::core::convert::From<InvalidConfigurationNumber> for GatewayRouterFacetErrors {
        fn from(value: InvalidConfigurationNumber) -> Self {
            Self::InvalidConfigurationNumber(value)
        }
    }
    impl ::core::convert::From<InvalidCrossMsgDstSubnet> for GatewayRouterFacetErrors {
        fn from(value: InvalidCrossMsgDstSubnet) -> Self {
            Self::InvalidCrossMsgDstSubnet(value)
        }
    }
    impl ::core::convert::From<InvalidCrossMsgNonce> for GatewayRouterFacetErrors {
        fn from(value: InvalidCrossMsgNonce) -> Self {
            Self::InvalidCrossMsgNonce(value)
        }
    }
    impl ::core::convert::From<InvalidRetentionHeight> for GatewayRouterFacetErrors {
        fn from(value: InvalidRetentionHeight) -> Self {
            Self::InvalidRetentionHeight(value)
        }
    }
    impl ::core::convert::From<InvalidSignature> for GatewayRouterFacetErrors {
        fn from(value: InvalidSignature) -> Self {
            Self::InvalidSignature(value)
        }
    }
    impl ::core::convert::From<InvalidSubnet> for GatewayRouterFacetErrors {
        fn from(value: InvalidSubnet) -> Self {
            Self::InvalidSubnet(value)
        }
    }
    impl ::core::convert::From<MaxMsgsPerBatchExceeded> for GatewayRouterFacetErrors {
        fn from(value: MaxMsgsPerBatchExceeded) -> Self {
            Self::MaxMsgsPerBatchExceeded(value)
        }
    }
    impl ::core::convert::From<NotAuthorized> for GatewayRouterFacetErrors {
        fn from(value: NotAuthorized) -> Self {
            Self::NotAuthorized(value)
        }
    }
    impl ::core::convert::From<NotEnoughBalance> for GatewayRouterFacetErrors {
        fn from(value: NotEnoughBalance) -> Self {
            Self::NotEnoughBalance(value)
        }
    }
    impl ::core::convert::From<NotEnoughSubnetCircSupply> for GatewayRouterFacetErrors {
        fn from(value: NotEnoughSubnetCircSupply) -> Self {
            Self::NotEnoughSubnetCircSupply(value)
        }
    }
    impl ::core::convert::From<NotRegisteredSubnet> for GatewayRouterFacetErrors {
        fn from(value: NotRegisteredSubnet) -> Self {
            Self::NotRegisteredSubnet(value)
        }
    }
    impl ::core::convert::From<NotSystemActor> for GatewayRouterFacetErrors {
        fn from(value: NotSystemActor) -> Self {
            Self::NotSystemActor(value)
        }
    }
    impl ::core::convert::From<OldConfigurationNumber> for GatewayRouterFacetErrors {
        fn from(value: OldConfigurationNumber) -> Self {
            Self::OldConfigurationNumber(value)
        }
    }
    impl ::core::convert::From<PQDoesNotContainAddress> for GatewayRouterFacetErrors {
        fn from(value: PQDoesNotContainAddress) -> Self {
            Self::PQDoesNotContainAddress(value)
        }
    }
    impl ::core::convert::From<PQEmpty> for GatewayRouterFacetErrors {
        fn from(value: PQEmpty) -> Self {
            Self::PQEmpty(value)
        }
    }
    impl ::core::convert::From<ParentFinalityAlreadyCommitted>
    for GatewayRouterFacetErrors {
        fn from(value: ParentFinalityAlreadyCommitted) -> Self {
            Self::ParentFinalityAlreadyCommitted(value)
        }
    }
    impl ::core::convert::From<QuorumAlreadyProcessed> for GatewayRouterFacetErrors {
        fn from(value: QuorumAlreadyProcessed) -> Self {
            Self::QuorumAlreadyProcessed(value)
        }
    }
    impl ::core::convert::From<SignatureReplay> for GatewayRouterFacetErrors {
        fn from(value: SignatureReplay) -> Self {
            Self::SignatureReplay(value)
        }
    }
    impl ::core::convert::From<SubnetNotActive> for GatewayRouterFacetErrors {
        fn from(value: SubnetNotActive) -> Self {
            Self::SubnetNotActive(value)
        }
    }
    impl ::core::convert::From<SubnetNotFound> for GatewayRouterFacetErrors {
        fn from(value: SubnetNotFound) -> Self {
            Self::SubnetNotFound(value)
        }
    }
    impl ::core::convert::From<ZeroMembershipWeight> for GatewayRouterFacetErrors {
        fn from(value: ZeroMembershipWeight) -> Self {
            Self::ZeroMembershipWeight(value)
        }
    }
    ///Container type for all input parameters for the `addBottomUpMsgBatchSignature` function with signature `addBottomUpMsgBatchSignature(uint256,bytes32[],uint256,bytes)` and selector `0x0db0f77c`
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
        name = "addBottomUpMsgBatchSignature",
        abi = "addBottomUpMsgBatchSignature(uint256,bytes32[],uint256,bytes)"
    )]
    pub struct AddBottomUpMsgBatchSignatureCall {
        pub height: ::ethers::core::types::U256,
        pub membership_proof: ::std::vec::Vec<[u8; 32]>,
        pub weight: ::ethers::core::types::U256,
        pub signature: ::ethers::core::types::Bytes,
    }
    ///Container type for all input parameters for the `addCheckpointSignature` function with signature `addCheckpointSignature(uint256,bytes32[],uint256,bytes)` and selector `0x53b4e7bf`
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
        name = "addCheckpointSignature",
        abi = "addCheckpointSignature(uint256,bytes32[],uint256,bytes)"
    )]
    pub struct AddCheckpointSignatureCall {
        pub height: ::ethers::core::types::U256,
        pub membership_proof: ::std::vec::Vec<[u8; 32]>,
        pub weight: ::ethers::core::types::U256,
        pub signature: ::ethers::core::types::Bytes,
    }
    ///Container type for all input parameters for the `applyCrossMessages` function with signature `applyCrossMessages(((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes,uint256),bool)[])` and selector `0xc62eb4d5`
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
        name = "applyCrossMessages",
        abi = "applyCrossMessages(((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes,uint256),bool)[])"
    )]
    pub struct ApplyCrossMessagesCall {
        pub cross_msgs: ::std::vec::Vec<CrossMsg>,
    }
    ///Container type for all input parameters for the `applyFinalityChanges` function with signature `applyFinalityChanges()` and selector `0x0df14461`
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
    #[ethcall(name = "applyFinalityChanges", abi = "applyFinalityChanges()")]
    pub struct ApplyFinalityChangesCall;
    ///Container type for all input parameters for the `commitCheckpoint` function with signature `commitCheckpoint(((uint64,address[]),uint256,bytes32,uint64))` and selector `0x47dc9b4f`
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
        name = "commitCheckpoint",
        abi = "commitCheckpoint(((uint64,address[]),uint256,bytes32,uint64))"
    )]
    pub struct CommitCheckpointCall {
        pub checkpoint: BottomUpCheckpoint,
    }
    ///Container type for all input parameters for the `commitParentFinality` function with signature `commitParentFinality((uint256,bytes32))` and selector `0x11196974`
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
        name = "commitParentFinality",
        abi = "commitParentFinality((uint256,bytes32))"
    )]
    pub struct CommitParentFinalityCall {
        pub finality: ParentFinality,
    }
    ///Container type for all input parameters for the `createBottomUpCheckpoint` function with signature `createBottomUpCheckpoint(((uint64,address[]),uint256,bytes32,uint64),bytes32,uint256)` and selector `0x74303771`
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
        name = "createBottomUpCheckpoint",
        abi = "createBottomUpCheckpoint(((uint64,address[]),uint256,bytes32,uint64),bytes32,uint256)"
    )]
    pub struct CreateBottomUpCheckpointCall {
        pub checkpoint: BottomUpCheckpoint,
        pub membership_root_hash: [u8; 32],
        pub membership_weight: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `createBottomUpMsgBatch` function with signature `createBottomUpMsgBatch(((uint64,address[]),uint256,((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes,uint256),bool)[]),bytes32,uint256)` and selector `0x32e7661f`
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
        name = "createBottomUpMsgBatch",
        abi = "createBottomUpMsgBatch(((uint64,address[]),uint256,((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes,uint256),bool)[]),bytes32,uint256)"
    )]
    pub struct CreateBottomUpMsgBatchCall {
        pub batch: BottomUpMsgBatch,
        pub membership_root_hash: [u8; 32],
        pub membership_weight: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `execBottomUpMsgBatch` function with signature `execBottomUpMsgBatch(((uint64,address[]),uint256,((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes,uint256),bool)[]))` and selector `0x0bed7615`
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
        name = "execBottomUpMsgBatch",
        abi = "execBottomUpMsgBatch(((uint64,address[]),uint256,((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes,uint256),bool)[]))"
    )]
    pub struct ExecBottomUpMsgBatchCall {
        pub batch: BottomUpMsgBatch,
    }
    ///Container type for all input parameters for the `pruneBottomUpCheckpoints` function with signature `pruneBottomUpCheckpoints(uint256)` and selector `0xac818379`
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
        name = "pruneBottomUpCheckpoints",
        abi = "pruneBottomUpCheckpoints(uint256)"
    )]
    pub struct PruneBottomUpCheckpointsCall {
        pub new_retention_height: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `pruneBottomUpMsgBatches` function with signature `pruneBottomUpMsgBatches(uint256)` and selector `0xbacc656d`
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
        name = "pruneBottomUpMsgBatches",
        abi = "pruneBottomUpMsgBatches(uint256)"
    )]
    pub struct PruneBottomUpMsgBatchesCall {
        pub new_retention_height: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `storeValidatorChanges` function with signature `storeValidatorChanges(((uint8,bytes,address),uint64)[])` and selector `0xe49a547d`
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
        name = "storeValidatorChanges",
        abi = "storeValidatorChanges(((uint8,bytes,address),uint64)[])"
    )]
    pub struct StoreValidatorChangesCall {
        pub change_requests: ::std::vec::Vec<StakingChangeRequest>,
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum GatewayRouterFacetCalls {
        AddBottomUpMsgBatchSignature(AddBottomUpMsgBatchSignatureCall),
        AddCheckpointSignature(AddCheckpointSignatureCall),
        ApplyCrossMessages(ApplyCrossMessagesCall),
        ApplyFinalityChanges(ApplyFinalityChangesCall),
        CommitCheckpoint(CommitCheckpointCall),
        CommitParentFinality(CommitParentFinalityCall),
        CreateBottomUpCheckpoint(CreateBottomUpCheckpointCall),
        CreateBottomUpMsgBatch(CreateBottomUpMsgBatchCall),
        ExecBottomUpMsgBatch(ExecBottomUpMsgBatchCall),
        PruneBottomUpCheckpoints(PruneBottomUpCheckpointsCall),
        PruneBottomUpMsgBatches(PruneBottomUpMsgBatchesCall),
        StoreValidatorChanges(StoreValidatorChangesCall),
    }
    impl ::ethers::core::abi::AbiDecode for GatewayRouterFacetCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <AddBottomUpMsgBatchSignatureCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AddBottomUpMsgBatchSignature(decoded));
            }
            if let Ok(decoded) = <AddCheckpointSignatureCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AddCheckpointSignature(decoded));
            }
            if let Ok(decoded) = <ApplyCrossMessagesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ApplyCrossMessages(decoded));
            }
            if let Ok(decoded) = <ApplyFinalityChangesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ApplyFinalityChanges(decoded));
            }
            if let Ok(decoded) = <CommitCheckpointCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CommitCheckpoint(decoded));
            }
            if let Ok(decoded) = <CommitParentFinalityCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CommitParentFinality(decoded));
            }
            if let Ok(decoded) = <CreateBottomUpCheckpointCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CreateBottomUpCheckpoint(decoded));
            }
            if let Ok(decoded) = <CreateBottomUpMsgBatchCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::CreateBottomUpMsgBatch(decoded));
            }
            if let Ok(decoded) = <ExecBottomUpMsgBatchCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::ExecBottomUpMsgBatch(decoded));
            }
            if let Ok(decoded) = <PruneBottomUpCheckpointsCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PruneBottomUpCheckpoints(decoded));
            }
            if let Ok(decoded) = <PruneBottomUpMsgBatchesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PruneBottomUpMsgBatches(decoded));
            }
            if let Ok(decoded) = <StoreValidatorChangesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::StoreValidatorChanges(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for GatewayRouterFacetCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::AddBottomUpMsgBatchSignature(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AddCheckpointSignature(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ApplyCrossMessages(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ApplyFinalityChanges(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CommitCheckpoint(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CommitParentFinality(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CreateBottomUpCheckpoint(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CreateBottomUpMsgBatch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ExecBottomUpMsgBatch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PruneBottomUpCheckpoints(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PruneBottomUpMsgBatches(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::StoreValidatorChanges(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for GatewayRouterFacetCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AddBottomUpMsgBatchSignature(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AddCheckpointSignature(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ApplyCrossMessages(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ApplyFinalityChanges(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CommitCheckpoint(element) => ::core::fmt::Display::fmt(element, f),
                Self::CommitParentFinality(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CreateBottomUpCheckpoint(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CreateBottomUpMsgBatch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ExecBottomUpMsgBatch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PruneBottomUpCheckpoints(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PruneBottomUpMsgBatches(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::StoreValidatorChanges(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<AddBottomUpMsgBatchSignatureCall>
    for GatewayRouterFacetCalls {
        fn from(value: AddBottomUpMsgBatchSignatureCall) -> Self {
            Self::AddBottomUpMsgBatchSignature(value)
        }
    }
    impl ::core::convert::From<AddCheckpointSignatureCall> for GatewayRouterFacetCalls {
        fn from(value: AddCheckpointSignatureCall) -> Self {
            Self::AddCheckpointSignature(value)
        }
    }
    impl ::core::convert::From<ApplyCrossMessagesCall> for GatewayRouterFacetCalls {
        fn from(value: ApplyCrossMessagesCall) -> Self {
            Self::ApplyCrossMessages(value)
        }
    }
    impl ::core::convert::From<ApplyFinalityChangesCall> for GatewayRouterFacetCalls {
        fn from(value: ApplyFinalityChangesCall) -> Self {
            Self::ApplyFinalityChanges(value)
        }
    }
    impl ::core::convert::From<CommitCheckpointCall> for GatewayRouterFacetCalls {
        fn from(value: CommitCheckpointCall) -> Self {
            Self::CommitCheckpoint(value)
        }
    }
    impl ::core::convert::From<CommitParentFinalityCall> for GatewayRouterFacetCalls {
        fn from(value: CommitParentFinalityCall) -> Self {
            Self::CommitParentFinality(value)
        }
    }
    impl ::core::convert::From<CreateBottomUpCheckpointCall>
    for GatewayRouterFacetCalls {
        fn from(value: CreateBottomUpCheckpointCall) -> Self {
            Self::CreateBottomUpCheckpoint(value)
        }
    }
    impl ::core::convert::From<CreateBottomUpMsgBatchCall> for GatewayRouterFacetCalls {
        fn from(value: CreateBottomUpMsgBatchCall) -> Self {
            Self::CreateBottomUpMsgBatch(value)
        }
    }
    impl ::core::convert::From<ExecBottomUpMsgBatchCall> for GatewayRouterFacetCalls {
        fn from(value: ExecBottomUpMsgBatchCall) -> Self {
            Self::ExecBottomUpMsgBatch(value)
        }
    }
    impl ::core::convert::From<PruneBottomUpCheckpointsCall>
    for GatewayRouterFacetCalls {
        fn from(value: PruneBottomUpCheckpointsCall) -> Self {
            Self::PruneBottomUpCheckpoints(value)
        }
    }
    impl ::core::convert::From<PruneBottomUpMsgBatchesCall> for GatewayRouterFacetCalls {
        fn from(value: PruneBottomUpMsgBatchesCall) -> Self {
            Self::PruneBottomUpMsgBatches(value)
        }
    }
    impl ::core::convert::From<StoreValidatorChangesCall> for GatewayRouterFacetCalls {
        fn from(value: StoreValidatorChangesCall) -> Self {
            Self::StoreValidatorChanges(value)
        }
    }
    ///Container type for all return fields from the `applyFinalityChanges` function with signature `applyFinalityChanges()` and selector `0x0df14461`
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
    pub struct ApplyFinalityChangesReturn(pub u64);
    ///Container type for all return fields from the `commitParentFinality` function with signature `commitParentFinality((uint256,bytes32))` and selector `0x11196974`
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
    pub struct CommitParentFinalityReturn {
        pub has_committed_before: bool,
        pub previous_finality: ParentFinality,
    }
    ///`BottomUpCheckpoint((uint64,address[]),uint256,bytes32,uint64)`
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
    pub struct BottomUpCheckpoint {
        pub subnet_id: SubnetID,
        pub block_height: ::ethers::core::types::U256,
        pub block_hash: [u8; 32],
        pub next_configuration_number: u64,
    }
    ///`BottomUpMsgBatch((uint64,address[]),uint256,((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes,uint256),bool)[])`
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
    pub struct BottomUpMsgBatch {
        pub subnet_id: SubnetID,
        pub block_height: ::ethers::core::types::U256,
        pub msgs: ::std::vec::Vec<CrossMsg>,
    }
    ///`CrossMsg((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes,uint256),bool)`
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
        Hash
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
        Hash
    )]
    pub struct Ipcaddress {
        pub subnet_id: SubnetID,
        pub raw_address: FvmAddress,
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
        Hash
    )]
    pub struct ParentFinality {
        pub height: ::ethers::core::types::U256,
        pub block_hash: [u8; 32],
    }
    ///`StakingChange(uint8,bytes,address)`
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
    pub struct StakingChange {
        pub op: u8,
        pub payload: ::ethers::core::types::Bytes,
        pub validator: ::ethers::core::types::Address,
    }
    ///`StakingChangeRequest((uint8,bytes,address),uint64)`
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
    pub struct StakingChangeRequest {
        pub change: StakingChange,
        pub configuration_number: u64,
    }
    ///`StorableMsg(((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes,uint256)`
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
    pub struct StorableMsg {
        pub from: Ipcaddress,
        pub to: Ipcaddress,
        pub value: ::ethers::core::types::U256,
        pub nonce: u64,
        pub method: [u8; 4],
        pub params: ::ethers::core::types::Bytes,
        pub fee: ::ethers::core::types::U256,
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
}
