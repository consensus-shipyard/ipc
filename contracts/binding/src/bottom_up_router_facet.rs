pub use bottom_up_router_facet::*;
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
pub mod bottom_up_router_facet {
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
    pub static BOTTOMUPROUTERFACET_ABI: ::ethers::contract::Lazy<
        ::ethers::core::abi::Abi,
    > = ::ethers::contract::Lazy::new(__abi);
    pub struct BottomUpRouterFacet<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for BottomUpRouterFacet<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for BottomUpRouterFacet<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for BottomUpRouterFacet<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for BottomUpRouterFacet<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(BottomUpRouterFacet))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> BottomUpRouterFacet<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    BOTTOMUPROUTERFACET_ABI.clone(),
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
        ///Calls the contract's `pruneBottomUpMsgBatches` (0xbacc656d) function
        pub fn prune_bottom_up_msg_batches(
            &self,
            new_retention_height: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([186, 204, 101, 109], new_retention_height)
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for BottomUpRouterFacet<M> {
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
    pub enum BottomUpRouterFacetErrors {
        AddressEmptyCode(AddressEmptyCode),
        AddressInsufficientBalance(AddressInsufficientBalance),
        BatchAlreadyExists(BatchAlreadyExists),
        BatchNotCreated(BatchNotCreated),
        BatchWithNoMessages(BatchWithNoMessages),
        FailedAddIncompleteQuorum(FailedAddIncompleteQuorum),
        FailedAddSignatory(FailedAddSignatory),
        FailedInnerCall(FailedInnerCall),
        FailedRemoveIncompleteQuorum(FailedRemoveIncompleteQuorum),
        InvalidActorAddress(InvalidActorAddress),
        InvalidBatchEpoch(InvalidBatchEpoch),
        InvalidBatchSource(InvalidBatchSource),
        InvalidCrossMsgDstSubnet(InvalidCrossMsgDstSubnet),
        InvalidCrossMsgNonce(InvalidCrossMsgNonce),
        InvalidRetentionHeight(InvalidRetentionHeight),
        InvalidSignature(InvalidSignature),
        MaxMsgsPerBatchExceeded(MaxMsgsPerBatchExceeded),
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
    impl ::ethers::core::abi::AbiDecode for BottomUpRouterFacetErrors {
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
    impl ::ethers::core::abi::AbiEncode for BottomUpRouterFacetErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::AddressEmptyCode(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AddressInsufficientBalance(element) => {
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
                Self::MaxMsgsPerBatchExceeded(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotAuthorized(element) => {
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
                Self::QuorumAlreadyProcessed(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SignatureReplay(element) => {
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
    impl ::ethers::contract::ContractRevert for BottomUpRouterFacetErrors {
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
                    == <MaxMsgsPerBatchExceeded as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotAuthorized as ::ethers::contract::EthError>::selector() => {
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
                    == <QuorumAlreadyProcessed as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <SignatureReplay as ::ethers::contract::EthError>::selector() => {
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
    impl ::core::fmt::Display for BottomUpRouterFacetErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AddressEmptyCode(element) => ::core::fmt::Display::fmt(element, f),
                Self::AddressInsufficientBalance(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::BatchAlreadyExists(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::BatchNotCreated(element) => ::core::fmt::Display::fmt(element, f),
                Self::BatchWithNoMessages(element) => {
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
                Self::MaxMsgsPerBatchExceeded(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotAuthorized(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotEnoughSubnetCircSupply(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotRegisteredSubnet(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotSystemActor(element) => ::core::fmt::Display::fmt(element, f),
                Self::QuorumAlreadyProcessed(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SignatureReplay(element) => ::core::fmt::Display::fmt(element, f),
                Self::SubnetNotFound(element) => ::core::fmt::Display::fmt(element, f),
                Self::ZeroMembershipWeight(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for BottomUpRouterFacetErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<AddressEmptyCode> for BottomUpRouterFacetErrors {
        fn from(value: AddressEmptyCode) -> Self {
            Self::AddressEmptyCode(value)
        }
    }
    impl ::core::convert::From<AddressInsufficientBalance>
    for BottomUpRouterFacetErrors {
        fn from(value: AddressInsufficientBalance) -> Self {
            Self::AddressInsufficientBalance(value)
        }
    }
    impl ::core::convert::From<BatchAlreadyExists> for BottomUpRouterFacetErrors {
        fn from(value: BatchAlreadyExists) -> Self {
            Self::BatchAlreadyExists(value)
        }
    }
    impl ::core::convert::From<BatchNotCreated> for BottomUpRouterFacetErrors {
        fn from(value: BatchNotCreated) -> Self {
            Self::BatchNotCreated(value)
        }
    }
    impl ::core::convert::From<BatchWithNoMessages> for BottomUpRouterFacetErrors {
        fn from(value: BatchWithNoMessages) -> Self {
            Self::BatchWithNoMessages(value)
        }
    }
    impl ::core::convert::From<FailedAddIncompleteQuorum> for BottomUpRouterFacetErrors {
        fn from(value: FailedAddIncompleteQuorum) -> Self {
            Self::FailedAddIncompleteQuorum(value)
        }
    }
    impl ::core::convert::From<FailedAddSignatory> for BottomUpRouterFacetErrors {
        fn from(value: FailedAddSignatory) -> Self {
            Self::FailedAddSignatory(value)
        }
    }
    impl ::core::convert::From<FailedInnerCall> for BottomUpRouterFacetErrors {
        fn from(value: FailedInnerCall) -> Self {
            Self::FailedInnerCall(value)
        }
    }
    impl ::core::convert::From<FailedRemoveIncompleteQuorum>
    for BottomUpRouterFacetErrors {
        fn from(value: FailedRemoveIncompleteQuorum) -> Self {
            Self::FailedRemoveIncompleteQuorum(value)
        }
    }
    impl ::core::convert::From<InvalidActorAddress> for BottomUpRouterFacetErrors {
        fn from(value: InvalidActorAddress) -> Self {
            Self::InvalidActorAddress(value)
        }
    }
    impl ::core::convert::From<InvalidBatchEpoch> for BottomUpRouterFacetErrors {
        fn from(value: InvalidBatchEpoch) -> Self {
            Self::InvalidBatchEpoch(value)
        }
    }
    impl ::core::convert::From<InvalidBatchSource> for BottomUpRouterFacetErrors {
        fn from(value: InvalidBatchSource) -> Self {
            Self::InvalidBatchSource(value)
        }
    }
    impl ::core::convert::From<InvalidCrossMsgDstSubnet> for BottomUpRouterFacetErrors {
        fn from(value: InvalidCrossMsgDstSubnet) -> Self {
            Self::InvalidCrossMsgDstSubnet(value)
        }
    }
    impl ::core::convert::From<InvalidCrossMsgNonce> for BottomUpRouterFacetErrors {
        fn from(value: InvalidCrossMsgNonce) -> Self {
            Self::InvalidCrossMsgNonce(value)
        }
    }
    impl ::core::convert::From<InvalidRetentionHeight> for BottomUpRouterFacetErrors {
        fn from(value: InvalidRetentionHeight) -> Self {
            Self::InvalidRetentionHeight(value)
        }
    }
    impl ::core::convert::From<InvalidSignature> for BottomUpRouterFacetErrors {
        fn from(value: InvalidSignature) -> Self {
            Self::InvalidSignature(value)
        }
    }
    impl ::core::convert::From<MaxMsgsPerBatchExceeded> for BottomUpRouterFacetErrors {
        fn from(value: MaxMsgsPerBatchExceeded) -> Self {
            Self::MaxMsgsPerBatchExceeded(value)
        }
    }
    impl ::core::convert::From<NotAuthorized> for BottomUpRouterFacetErrors {
        fn from(value: NotAuthorized) -> Self {
            Self::NotAuthorized(value)
        }
    }
    impl ::core::convert::From<NotEnoughSubnetCircSupply> for BottomUpRouterFacetErrors {
        fn from(value: NotEnoughSubnetCircSupply) -> Self {
            Self::NotEnoughSubnetCircSupply(value)
        }
    }
    impl ::core::convert::From<NotRegisteredSubnet> for BottomUpRouterFacetErrors {
        fn from(value: NotRegisteredSubnet) -> Self {
            Self::NotRegisteredSubnet(value)
        }
    }
    impl ::core::convert::From<NotSystemActor> for BottomUpRouterFacetErrors {
        fn from(value: NotSystemActor) -> Self {
            Self::NotSystemActor(value)
        }
    }
    impl ::core::convert::From<QuorumAlreadyProcessed> for BottomUpRouterFacetErrors {
        fn from(value: QuorumAlreadyProcessed) -> Self {
            Self::QuorumAlreadyProcessed(value)
        }
    }
    impl ::core::convert::From<SignatureReplay> for BottomUpRouterFacetErrors {
        fn from(value: SignatureReplay) -> Self {
            Self::SignatureReplay(value)
        }
    }
    impl ::core::convert::From<SubnetNotFound> for BottomUpRouterFacetErrors {
        fn from(value: SubnetNotFound) -> Self {
            Self::SubnetNotFound(value)
        }
    }
    impl ::core::convert::From<ZeroMembershipWeight> for BottomUpRouterFacetErrors {
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
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum BottomUpRouterFacetCalls {
        AddBottomUpMsgBatchSignature(AddBottomUpMsgBatchSignatureCall),
        CreateBottomUpMsgBatch(CreateBottomUpMsgBatchCall),
        ExecBottomUpMsgBatch(ExecBottomUpMsgBatchCall),
        PruneBottomUpMsgBatches(PruneBottomUpMsgBatchesCall),
    }
    impl ::ethers::core::abi::AbiDecode for BottomUpRouterFacetCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <AddBottomUpMsgBatchSignatureCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AddBottomUpMsgBatchSignature(decoded));
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
            if let Ok(decoded) = <PruneBottomUpMsgBatchesCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::PruneBottomUpMsgBatches(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for BottomUpRouterFacetCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::AddBottomUpMsgBatchSignature(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CreateBottomUpMsgBatch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ExecBottomUpMsgBatch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PruneBottomUpMsgBatches(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for BottomUpRouterFacetCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AddBottomUpMsgBatchSignature(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CreateBottomUpMsgBatch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ExecBottomUpMsgBatch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::PruneBottomUpMsgBatches(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<AddBottomUpMsgBatchSignatureCall>
    for BottomUpRouterFacetCalls {
        fn from(value: AddBottomUpMsgBatchSignatureCall) -> Self {
            Self::AddBottomUpMsgBatchSignature(value)
        }
    }
    impl ::core::convert::From<CreateBottomUpMsgBatchCall> for BottomUpRouterFacetCalls {
        fn from(value: CreateBottomUpMsgBatchCall) -> Self {
            Self::CreateBottomUpMsgBatch(value)
        }
    }
    impl ::core::convert::From<ExecBottomUpMsgBatchCall> for BottomUpRouterFacetCalls {
        fn from(value: ExecBottomUpMsgBatchCall) -> Self {
            Self::ExecBottomUpMsgBatch(value)
        }
    }
    impl ::core::convert::From<PruneBottomUpMsgBatchesCall>
    for BottomUpRouterFacetCalls {
        fn from(value: PruneBottomUpMsgBatchesCall) -> Self {
            Self::PruneBottomUpMsgBatches(value)
        }
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
