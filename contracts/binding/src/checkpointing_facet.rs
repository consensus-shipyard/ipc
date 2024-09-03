pub use checkpointing_facet::*;
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
pub mod checkpointing_facet {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([
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
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
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
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                        ],
                                                    ),
                                                ),
                                            ),
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
                                            ::ethers::core::abi::ethabi::ParamType::Array(
                                                ::std::boxed::Box::new(
                                                    ::ethers::core::abi::ethabi::ParamType::Tuple(
                                                        ::std::vec![
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
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
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                                        ],
                                                    ),
                                                ),
                                            ),
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
            ]),
            events: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("NewBottomUpMsgBatch"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "NewBottomUpMsgBatch",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("epoch"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    indexed: true,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NewTopDownMessage"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("NewTopDownMessage"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("subnet"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: true,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("message"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Tuple(
                                        ::std::vec![
                                            ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
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
                                            ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                            ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                            ::ethers::core::abi::ethabi::ParamType::Bytes,
                                        ],
                                    ),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("QuorumReached"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("QuorumReached"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("objKind"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("height"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("objHash"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("quorumWeight"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("QuorumWeightUpdated"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "QuorumWeightUpdated",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("objKind"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("height"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("objHash"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(
                                        32usize,
                                    ),
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("newWeight"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(
                                        256usize,
                                    ),
                                    indexed: false,
                                },
                            ],
                            anonymous: false,
                        },
                    ],
                ),
            ]),
            errors: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("CannotSendCrossMsgToItself"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned(
                                "CannotSendCrossMsgToItself",
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
                    ::std::borrow::ToOwned::to_owned("InvalidXnetMessage"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("InvalidXnetMessage"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("reason"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    internal_type: ::core::option::Option::Some(
                                        ::std::borrow::ToOwned::to_owned(
                                            "enum InvalidXnetMessageReason",
                                        ),
                                    ),
                                },
                            ],
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
                    ::std::borrow::ToOwned::to_owned("MethodNotAllowed"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("MethodNotAllowed"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::Param {
                                    name: ::std::borrow::ToOwned::to_owned("reason"),
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
    pub static CHECKPOINTINGFACET_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(__abi);
    pub struct CheckpointingFacet<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for CheckpointingFacet<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for CheckpointingFacet<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for CheckpointingFacet<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for CheckpointingFacet<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(CheckpointingFacet))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> CheckpointingFacet<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                CHECKPOINTINGFACET_ABI.clone(),
                client,
            ))
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
        ///Calls the contract's `commitCheckpoint` (0xfba0fa4d) function
        pub fn commit_checkpoint(
            &self,
            checkpoint: BottomUpCheckpoint,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([251, 160, 250, 77], (checkpoint,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `createBottomUpCheckpoint` (0xdc749b05) function
        pub fn create_bottom_up_checkpoint(
            &self,
            checkpoint: BottomUpCheckpoint,
            membership_root_hash: [u8; 32],
            membership_weight: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [220, 116, 155, 5],
                    (checkpoint, membership_root_hash, membership_weight),
                )
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
        ///Gets the contract's `NewBottomUpMsgBatch` event
        pub fn new_bottom_up_msg_batch_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, NewBottomUpMsgBatchFilter>
        {
            self.0.event()
        }
        ///Gets the contract's `NewTopDownMessage` event
        pub fn new_top_down_message_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, NewTopDownMessageFilter>
        {
            self.0.event()
        }
        ///Gets the contract's `QuorumReached` event
        pub fn quorum_reached_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, QuorumReachedFilter>
        {
            self.0.event()
        }
        ///Gets the contract's `QuorumWeightUpdated` event
        pub fn quorum_weight_updated_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, QuorumWeightUpdatedFilter>
        {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, CheckpointingFacetEvents>
        {
            self.0
                .event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
        for CheckpointingFacet<M>
    {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `CannotSendCrossMsgToItself` with signature `CannotSendCrossMsgToItself()` and selector `0xbcccd7fc`
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
        name = "CannotSendCrossMsgToItself",
        abi = "CannotSendCrossMsgToItself()"
    )]
    pub struct CannotSendCrossMsgToItself;
    ///Custom Error type `CheckpointAlreadyExists` with signature `CheckpointAlreadyExists()` and selector `0xb8a1eae1`
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
        Hash,
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
        Hash,
    )]
    #[etherror(
        name = "FailedAddIncompleteQuorum",
        abi = "FailedAddIncompleteQuorum()"
    )]
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
        Hash,
    )]
    #[etherror(name = "FailedAddSignatory", abi = "FailedAddSignatory()")]
    pub struct FailedAddSignatory;
    ///Custom Error type `FailedRemoveIncompleteQuorum` with signature `FailedRemoveIncompleteQuorum()` and selector `0x894f690e`
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
        Hash,
    )]
    #[etherror(name = "InvalidActorAddress", abi = "InvalidActorAddress()")]
    pub struct InvalidActorAddress;
    ///Custom Error type `InvalidCheckpointSource` with signature `InvalidCheckpointSource()` and selector `0xfe72264e`
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
    #[etherror(name = "InvalidCheckpointSource", abi = "InvalidCheckpointSource()")]
    pub struct InvalidCheckpointSource;
    ///Custom Error type `InvalidRetentionHeight` with signature `InvalidRetentionHeight()` and selector `0x6819a3a9`
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
        Hash,
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
        Hash,
    )]
    #[etherror(name = "InvalidSubnet", abi = "InvalidSubnet()")]
    pub struct InvalidSubnet;
    ///Custom Error type `InvalidXnetMessage` with signature `InvalidXnetMessage(uint8)` and selector `0xbc0f01cf`
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
    #[etherror(name = "InvalidXnetMessage", abi = "InvalidXnetMessage(uint8)")]
    pub struct InvalidXnetMessage {
        pub reason: u8,
    }
    ///Custom Error type `MaxMsgsPerBatchExceeded` with signature `MaxMsgsPerBatchExceeded()` and selector `0x351c7007`
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
    #[etherror(name = "MaxMsgsPerBatchExceeded", abi = "MaxMsgsPerBatchExceeded()")]
    pub struct MaxMsgsPerBatchExceeded;
    ///Custom Error type `MethodNotAllowed` with signature `MethodNotAllowed(string)` and selector `0x015538b1`
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
    #[etherror(name = "MethodNotAllowed", abi = "MethodNotAllowed(string)")]
    pub struct MethodNotAllowed {
        pub reason: ::std::string::String,
    }
    ///Custom Error type `NotAuthorized` with signature `NotAuthorized(address)` and selector `0x4a0bfec1`
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
    #[etherror(name = "NotAuthorized", abi = "NotAuthorized(address)")]
    pub struct NotAuthorized(pub ::ethers::core::types::Address);
    ///Custom Error type `NotEnoughSubnetCircSupply` with signature `NotEnoughSubnetCircSupply()` and selector `0x74db2854`
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
        name = "NotEnoughSubnetCircSupply",
        abi = "NotEnoughSubnetCircSupply()"
    )]
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
        Hash,
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
        Hash,
    )]
    #[etherror(name = "NotSystemActor", abi = "NotSystemActor()")]
    pub struct NotSystemActor;
    ///Custom Error type `QuorumAlreadyProcessed` with signature `QuorumAlreadyProcessed()` and selector `0x042384dc`
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
        Hash,
    )]
    #[etherror(name = "SignatureReplay", abi = "SignatureReplay()")]
    pub struct SignatureReplay;
    ///Custom Error type `SubnetNotFound` with signature `SubnetNotFound()` and selector `0x00476ad8`
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
        Hash,
    )]
    #[etherror(name = "ZeroMembershipWeight", abi = "ZeroMembershipWeight()")]
    pub struct ZeroMembershipWeight;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum CheckpointingFacetErrors {
        CannotSendCrossMsgToItself(CannotSendCrossMsgToItself),
        CheckpointAlreadyExists(CheckpointAlreadyExists),
        CheckpointNotCreated(CheckpointNotCreated),
        FailedAddIncompleteQuorum(FailedAddIncompleteQuorum),
        FailedAddSignatory(FailedAddSignatory),
        FailedRemoveIncompleteQuorum(FailedRemoveIncompleteQuorum),
        InvalidActorAddress(InvalidActorAddress),
        InvalidCheckpointSource(InvalidCheckpointSource),
        InvalidRetentionHeight(InvalidRetentionHeight),
        InvalidSignature(InvalidSignature),
        InvalidSubnet(InvalidSubnet),
        InvalidXnetMessage(InvalidXnetMessage),
        MaxMsgsPerBatchExceeded(MaxMsgsPerBatchExceeded),
        MethodNotAllowed(MethodNotAllowed),
        NotAuthorized(NotAuthorized),
        NotEnoughSubnetCircSupply(NotEnoughSubnetCircSupply),
        NotRegisteredSubnet(NotRegisteredSubnet),
        NotSystemActor(NotSystemActor),
        QuorumAlreadyProcessed(QuorumAlreadyProcessed),
        SignatureReplay(SignatureReplay),
        SubnetNotFound(SubnetNotFound),
        ZeroMembershipWeight(ZeroMembershipWeight),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for CheckpointingFacetErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) =
                <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded) =
                <CannotSendCrossMsgToItself as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::CannotSendCrossMsgToItself(decoded));
            }
            if let Ok(decoded) =
                <CheckpointAlreadyExists as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::CheckpointAlreadyExists(decoded));
            }
            if let Ok(decoded) =
                <CheckpointNotCreated as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::CheckpointNotCreated(decoded));
            }
            if let Ok(decoded) =
                <FailedAddIncompleteQuorum as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::FailedAddIncompleteQuorum(decoded));
            }
            if let Ok(decoded) =
                <FailedAddSignatory as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::FailedAddSignatory(decoded));
            }
            if let Ok(decoded) =
                <FailedRemoveIncompleteQuorum as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::FailedRemoveIncompleteQuorum(decoded));
            }
            if let Ok(decoded) =
                <InvalidActorAddress as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::InvalidActorAddress(decoded));
            }
            if let Ok(decoded) =
                <InvalidCheckpointSource as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::InvalidCheckpointSource(decoded));
            }
            if let Ok(decoded) =
                <InvalidRetentionHeight as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::InvalidRetentionHeight(decoded));
            }
            if let Ok(decoded) = <InvalidSignature as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::InvalidSignature(decoded));
            }
            if let Ok(decoded) = <InvalidSubnet as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::InvalidSubnet(decoded));
            }
            if let Ok(decoded) =
                <InvalidXnetMessage as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::InvalidXnetMessage(decoded));
            }
            if let Ok(decoded) =
                <MaxMsgsPerBatchExceeded as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::MaxMsgsPerBatchExceeded(decoded));
            }
            if let Ok(decoded) = <MethodNotAllowed as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::MethodNotAllowed(decoded));
            }
            if let Ok(decoded) = <NotAuthorized as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NotAuthorized(decoded));
            }
            if let Ok(decoded) =
                <NotEnoughSubnetCircSupply as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NotEnoughSubnetCircSupply(decoded));
            }
            if let Ok(decoded) =
                <NotRegisteredSubnet as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NotRegisteredSubnet(decoded));
            }
            if let Ok(decoded) = <NotSystemActor as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NotSystemActor(decoded));
            }
            if let Ok(decoded) =
                <QuorumAlreadyProcessed as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::QuorumAlreadyProcessed(decoded));
            }
            if let Ok(decoded) = <SignatureReplay as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SignatureReplay(decoded));
            }
            if let Ok(decoded) = <SubnetNotFound as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SubnetNotFound(decoded));
            }
            if let Ok(decoded) =
                <ZeroMembershipWeight as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ZeroMembershipWeight(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for CheckpointingFacetErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::CannotSendCrossMsgToItself(element) => {
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
                Self::FailedRemoveIncompleteQuorum(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidActorAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidCheckpointSource(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidRetentionHeight(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidSignature(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::InvalidSubnet(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::InvalidXnetMessage(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MaxMsgsPerBatchExceeded(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MethodNotAllowed(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::NotAuthorized(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::NotEnoughSubnetCircSupply(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotRegisteredSubnet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotSystemActor(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::QuorumAlreadyProcessed(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SignatureReplay(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::SubnetNotFound(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ZeroMembershipWeight(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for CheckpointingFacetErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <CannotSendCrossMsgToItself as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <CheckpointAlreadyExists as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <CheckpointNotCreated as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <FailedAddIncompleteQuorum as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <FailedAddSignatory as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <FailedRemoveIncompleteQuorum as ::ethers::contract::EthError>::selector(
                    ) =>
                {
                    true
                }
                _ if selector
                    == <InvalidActorAddress as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <InvalidCheckpointSource as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <InvalidRetentionHeight as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector == <InvalidSignature as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector == <InvalidSubnet as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidXnetMessage as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <MaxMsgsPerBatchExceeded as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector == <MethodNotAllowed as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector == <NotAuthorized as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotEnoughSubnetCircSupply as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <NotRegisteredSubnet as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector == <NotSystemActor as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <QuorumAlreadyProcessed as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector == <SignatureReplay as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector == <SubnetNotFound as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ZeroMembershipWeight as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for CheckpointingFacetErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::CannotSendCrossMsgToItself(element) => ::core::fmt::Display::fmt(element, f),
                Self::CheckpointAlreadyExists(element) => ::core::fmt::Display::fmt(element, f),
                Self::CheckpointNotCreated(element) => ::core::fmt::Display::fmt(element, f),
                Self::FailedAddIncompleteQuorum(element) => ::core::fmt::Display::fmt(element, f),
                Self::FailedAddSignatory(element) => ::core::fmt::Display::fmt(element, f),
                Self::FailedRemoveIncompleteQuorum(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidActorAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidCheckpointSource(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidRetentionHeight(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidSignature(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidSubnet(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidXnetMessage(element) => ::core::fmt::Display::fmt(element, f),
                Self::MaxMsgsPerBatchExceeded(element) => ::core::fmt::Display::fmt(element, f),
                Self::MethodNotAllowed(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotAuthorized(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotEnoughSubnetCircSupply(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotRegisteredSubnet(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotSystemActor(element) => ::core::fmt::Display::fmt(element, f),
                Self::QuorumAlreadyProcessed(element) => ::core::fmt::Display::fmt(element, f),
                Self::SignatureReplay(element) => ::core::fmt::Display::fmt(element, f),
                Self::SubnetNotFound(element) => ::core::fmt::Display::fmt(element, f),
                Self::ZeroMembershipWeight(element) => ::core::fmt::Display::fmt(element, f),
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for CheckpointingFacetErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<CannotSendCrossMsgToItself> for CheckpointingFacetErrors {
        fn from(value: CannotSendCrossMsgToItself) -> Self {
            Self::CannotSendCrossMsgToItself(value)
        }
    }
    impl ::core::convert::From<CheckpointAlreadyExists> for CheckpointingFacetErrors {
        fn from(value: CheckpointAlreadyExists) -> Self {
            Self::CheckpointAlreadyExists(value)
        }
    }
    impl ::core::convert::From<CheckpointNotCreated> for CheckpointingFacetErrors {
        fn from(value: CheckpointNotCreated) -> Self {
            Self::CheckpointNotCreated(value)
        }
    }
    impl ::core::convert::From<FailedAddIncompleteQuorum> for CheckpointingFacetErrors {
        fn from(value: FailedAddIncompleteQuorum) -> Self {
            Self::FailedAddIncompleteQuorum(value)
        }
    }
    impl ::core::convert::From<FailedAddSignatory> for CheckpointingFacetErrors {
        fn from(value: FailedAddSignatory) -> Self {
            Self::FailedAddSignatory(value)
        }
    }
    impl ::core::convert::From<FailedRemoveIncompleteQuorum> for CheckpointingFacetErrors {
        fn from(value: FailedRemoveIncompleteQuorum) -> Self {
            Self::FailedRemoveIncompleteQuorum(value)
        }
    }
    impl ::core::convert::From<InvalidActorAddress> for CheckpointingFacetErrors {
        fn from(value: InvalidActorAddress) -> Self {
            Self::InvalidActorAddress(value)
        }
    }
    impl ::core::convert::From<InvalidCheckpointSource> for CheckpointingFacetErrors {
        fn from(value: InvalidCheckpointSource) -> Self {
            Self::InvalidCheckpointSource(value)
        }
    }
    impl ::core::convert::From<InvalidRetentionHeight> for CheckpointingFacetErrors {
        fn from(value: InvalidRetentionHeight) -> Self {
            Self::InvalidRetentionHeight(value)
        }
    }
    impl ::core::convert::From<InvalidSignature> for CheckpointingFacetErrors {
        fn from(value: InvalidSignature) -> Self {
            Self::InvalidSignature(value)
        }
    }
    impl ::core::convert::From<InvalidSubnet> for CheckpointingFacetErrors {
        fn from(value: InvalidSubnet) -> Self {
            Self::InvalidSubnet(value)
        }
    }
    impl ::core::convert::From<InvalidXnetMessage> for CheckpointingFacetErrors {
        fn from(value: InvalidXnetMessage) -> Self {
            Self::InvalidXnetMessage(value)
        }
    }
    impl ::core::convert::From<MaxMsgsPerBatchExceeded> for CheckpointingFacetErrors {
        fn from(value: MaxMsgsPerBatchExceeded) -> Self {
            Self::MaxMsgsPerBatchExceeded(value)
        }
    }
    impl ::core::convert::From<MethodNotAllowed> for CheckpointingFacetErrors {
        fn from(value: MethodNotAllowed) -> Self {
            Self::MethodNotAllowed(value)
        }
    }
    impl ::core::convert::From<NotAuthorized> for CheckpointingFacetErrors {
        fn from(value: NotAuthorized) -> Self {
            Self::NotAuthorized(value)
        }
    }
    impl ::core::convert::From<NotEnoughSubnetCircSupply> for CheckpointingFacetErrors {
        fn from(value: NotEnoughSubnetCircSupply) -> Self {
            Self::NotEnoughSubnetCircSupply(value)
        }
    }
    impl ::core::convert::From<NotRegisteredSubnet> for CheckpointingFacetErrors {
        fn from(value: NotRegisteredSubnet) -> Self {
            Self::NotRegisteredSubnet(value)
        }
    }
    impl ::core::convert::From<NotSystemActor> for CheckpointingFacetErrors {
        fn from(value: NotSystemActor) -> Self {
            Self::NotSystemActor(value)
        }
    }
    impl ::core::convert::From<QuorumAlreadyProcessed> for CheckpointingFacetErrors {
        fn from(value: QuorumAlreadyProcessed) -> Self {
            Self::QuorumAlreadyProcessed(value)
        }
    }
    impl ::core::convert::From<SignatureReplay> for CheckpointingFacetErrors {
        fn from(value: SignatureReplay) -> Self {
            Self::SignatureReplay(value)
        }
    }
    impl ::core::convert::From<SubnetNotFound> for CheckpointingFacetErrors {
        fn from(value: SubnetNotFound) -> Self {
            Self::SubnetNotFound(value)
        }
    }
    impl ::core::convert::From<ZeroMembershipWeight> for CheckpointingFacetErrors {
        fn from(value: ZeroMembershipWeight) -> Self {
            Self::ZeroMembershipWeight(value)
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
    #[ethevent(name = "NewBottomUpMsgBatch", abi = "NewBottomUpMsgBatch(uint256)")]
    pub struct NewBottomUpMsgBatchFilter {
        #[ethevent(indexed)]
        pub epoch: ::ethers::core::types::U256,
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
        name = "NewTopDownMessage",
        abi = "NewTopDownMessage(address,(uint8,((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint64,uint256,bytes))"
    )]
    pub struct NewTopDownMessageFilter {
        #[ethevent(indexed)]
        pub subnet: ::ethers::core::types::Address,
        pub message: IpcEnvelope,
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
        name = "QuorumReached",
        abi = "QuorumReached(uint8,uint256,bytes32,uint256)"
    )]
    pub struct QuorumReachedFilter {
        pub obj_kind: u8,
        pub height: ::ethers::core::types::U256,
        pub obj_hash: [u8; 32],
        pub quorum_weight: ::ethers::core::types::U256,
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
        name = "QuorumWeightUpdated",
        abi = "QuorumWeightUpdated(uint8,uint256,bytes32,uint256)"
    )]
    pub struct QuorumWeightUpdatedFilter {
        pub obj_kind: u8,
        pub height: ::ethers::core::types::U256,
        pub obj_hash: [u8; 32],
        pub new_weight: ::ethers::core::types::U256,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum CheckpointingFacetEvents {
        NewBottomUpMsgBatchFilter(NewBottomUpMsgBatchFilter),
        NewTopDownMessageFilter(NewTopDownMessageFilter),
        QuorumReachedFilter(QuorumReachedFilter),
        QuorumWeightUpdatedFilter(QuorumWeightUpdatedFilter),
    }
    impl ::ethers::contract::EthLogDecode for CheckpointingFacetEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = NewBottomUpMsgBatchFilter::decode_log(log) {
                return Ok(CheckpointingFacetEvents::NewBottomUpMsgBatchFilter(decoded));
            }
            if let Ok(decoded) = NewTopDownMessageFilter::decode_log(log) {
                return Ok(CheckpointingFacetEvents::NewTopDownMessageFilter(decoded));
            }
            if let Ok(decoded) = QuorumReachedFilter::decode_log(log) {
                return Ok(CheckpointingFacetEvents::QuorumReachedFilter(decoded));
            }
            if let Ok(decoded) = QuorumWeightUpdatedFilter::decode_log(log) {
                return Ok(CheckpointingFacetEvents::QuorumWeightUpdatedFilter(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for CheckpointingFacetEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::NewBottomUpMsgBatchFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::NewTopDownMessageFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::QuorumReachedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::QuorumWeightUpdatedFilter(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<NewBottomUpMsgBatchFilter> for CheckpointingFacetEvents {
        fn from(value: NewBottomUpMsgBatchFilter) -> Self {
            Self::NewBottomUpMsgBatchFilter(value)
        }
    }
    impl ::core::convert::From<NewTopDownMessageFilter> for CheckpointingFacetEvents {
        fn from(value: NewTopDownMessageFilter) -> Self {
            Self::NewTopDownMessageFilter(value)
        }
    }
    impl ::core::convert::From<QuorumReachedFilter> for CheckpointingFacetEvents {
        fn from(value: QuorumReachedFilter) -> Self {
            Self::QuorumReachedFilter(value)
        }
    }
    impl ::core::convert::From<QuorumWeightUpdatedFilter> for CheckpointingFacetEvents {
        fn from(value: QuorumWeightUpdatedFilter) -> Self {
            Self::QuorumWeightUpdatedFilter(value)
        }
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
        Hash,
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
    ///Container type for all input parameters for the `commitCheckpoint` function with signature `commitCheckpoint(((uint64,address[]),uint256,bytes32,uint64,(uint8,((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint64,uint256,bytes)[]))` and selector `0xfba0fa4d`
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
        name = "commitCheckpoint",
        abi = "commitCheckpoint(((uint64,address[]),uint256,bytes32,uint64,(uint8,((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint64,uint256,bytes)[]))"
    )]
    pub struct CommitCheckpointCall {
        pub checkpoint: BottomUpCheckpoint,
    }
    ///Container type for all input parameters for the `createBottomUpCheckpoint` function with signature `createBottomUpCheckpoint(((uint64,address[]),uint256,bytes32,uint64,(uint8,((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint64,uint256,bytes)[]),bytes32,uint256)` and selector `0xdc749b05`
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
        name = "createBottomUpCheckpoint",
        abi = "createBottomUpCheckpoint(((uint64,address[]),uint256,bytes32,uint64,(uint8,((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint64,uint256,bytes)[]),bytes32,uint256)"
    )]
    pub struct CreateBottomUpCheckpointCall {
        pub checkpoint: BottomUpCheckpoint,
        pub membership_root_hash: [u8; 32],
        pub membership_weight: ::ethers::core::types::U256,
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
        Hash,
    )]
    #[ethcall(
        name = "pruneBottomUpCheckpoints",
        abi = "pruneBottomUpCheckpoints(uint256)"
    )]
    pub struct PruneBottomUpCheckpointsCall {
        pub new_retention_height: ::ethers::core::types::U256,
    }
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum CheckpointingFacetCalls {
        AddCheckpointSignature(AddCheckpointSignatureCall),
        CommitCheckpoint(CommitCheckpointCall),
        CreateBottomUpCheckpoint(CreateBottomUpCheckpointCall),
        PruneBottomUpCheckpoints(PruneBottomUpCheckpointsCall),
    }
    impl ::ethers::core::abi::AbiDecode for CheckpointingFacetCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) =
                <AddCheckpointSignatureCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::AddCheckpointSignature(decoded));
            }
            if let Ok(decoded) =
                <CommitCheckpointCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::CommitCheckpoint(decoded));
            }
            if let Ok(decoded) =
                <CreateBottomUpCheckpointCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::CreateBottomUpCheckpoint(decoded));
            }
            if let Ok(decoded) =
                <PruneBottomUpCheckpointsCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::PruneBottomUpCheckpoints(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for CheckpointingFacetCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::AddCheckpointSignature(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CommitCheckpoint(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::CreateBottomUpCheckpoint(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PruneBottomUpCheckpoints(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for CheckpointingFacetCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AddCheckpointSignature(element) => ::core::fmt::Display::fmt(element, f),
                Self::CommitCheckpoint(element) => ::core::fmt::Display::fmt(element, f),
                Self::CreateBottomUpCheckpoint(element) => ::core::fmt::Display::fmt(element, f),
                Self::PruneBottomUpCheckpoints(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<AddCheckpointSignatureCall> for CheckpointingFacetCalls {
        fn from(value: AddCheckpointSignatureCall) -> Self {
            Self::AddCheckpointSignature(value)
        }
    }
    impl ::core::convert::From<CommitCheckpointCall> for CheckpointingFacetCalls {
        fn from(value: CommitCheckpointCall) -> Self {
            Self::CommitCheckpoint(value)
        }
    }
    impl ::core::convert::From<CreateBottomUpCheckpointCall> for CheckpointingFacetCalls {
        fn from(value: CreateBottomUpCheckpointCall) -> Self {
            Self::CreateBottomUpCheckpoint(value)
        }
    }
    impl ::core::convert::From<PruneBottomUpCheckpointsCall> for CheckpointingFacetCalls {
        fn from(value: PruneBottomUpCheckpointsCall) -> Self {
            Self::PruneBottomUpCheckpoints(value)
        }
    }
    ///`BottomUpCheckpoint((uint64,address[]),uint256,bytes32,uint64,(uint8,((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint64,uint256,bytes)[])`
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
        pub block_height: ::ethers::core::types::U256,
        pub block_hash: [u8; 32],
        pub next_configuration_number: u64,
        pub msgs: ::std::vec::Vec<IpcEnvelope>,
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
    ///`IpcEnvelope(uint8,((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint64,uint256,bytes)`
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
    pub struct IpcEnvelope {
        pub kind: u8,
        pub to: Ipcaddress,
        pub from: Ipcaddress,
        pub nonce: u64,
        pub value: ::ethers::core::types::U256,
        pub message: ::ethers::core::types::Bytes,
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
}
