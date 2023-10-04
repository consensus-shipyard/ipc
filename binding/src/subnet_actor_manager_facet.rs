pub use subnet_actor_manager_facet::*;
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
pub mod subnet_actor_manager_facet {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("claim"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("claim"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getValidator"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("getValidator"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("validatorAddress"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Address,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("address"),
                            ),
                        },],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("validator"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                ::ethers::core::abi::ethabi::ParamType::Bytes,
                            ],),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("struct Validator"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("join"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("join"),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("netAddr"),
                                kind: ::ethers::core::abi::ethabi::ParamType::String,
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("string"),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("workerAddr"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                    ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                    ::ethers::core::abi::ethabi::ParamType::Bytes,
                                ],),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("struct FvmAddress"),
                                ),
                            },
                        ],
                        outputs: ::std::vec![],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::Payable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("join2"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("join2"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("data"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("bytes"),
                            ),
                        },],
                        outputs: ::std::vec![],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::Payable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("kill"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("kill"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("kill2"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("kill2"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("leave"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("leave"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("leave2"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("leave2"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("reward"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("reward"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("amount"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint256"),
                            ),
                        },],
                        outputs: ::std::vec![],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("setMetadata"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("setMetadata"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("metadata"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("bytes"),
                            ),
                        },],
                        outputs: ::std::vec![],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("setValidatorNetAddr"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("setValidatorNetAddr",),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("newNetAddr"),
                            kind: ::ethers::core::abi::ethabi::ParamType::String,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("string"),
                            ),
                        },],
                        outputs: ::std::vec![],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("setValidatorWorkerAddr"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("setValidatorWorkerAddr",),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("newWorkerAddr"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                ::ethers::core::abi::ethabi::ParamType::Bytes,
                            ],),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("struct FvmAddress"),
                            ),
                        },],
                        outputs: ::std::vec![],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("stake"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("stake"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::Payable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("submitCheckpoint"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("submitCheckpoint"),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("checkpoint"),
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
                                    ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                ],),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("struct BottomUpCheckpoint",),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("membershipRootHash",),
                                kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize,),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("bytes32"),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("membershipWeight"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("uint256"),
                                ),
                            },
                        ],
                        outputs: ::std::vec![],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("validateCheckpoint"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("validateCheckpoint"),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("signatories"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                    ::std::boxed::Box::new(
                                        ::ethers::core::abi::ethabi::ParamType::Address,
                                    ),
                                ),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("address[]"),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("hash"),
                                kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize,),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("bytes32"),
                                ),
                            },
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("signatures"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Bytes,
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("bytes"),
                                ),
                            },
                        ],
                        outputs: ::std::vec![],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("withdraw"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("withdraw"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                    },],
                ),
            ]),
            events: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("BottomUpCheckpointExecuted"),
                    ::std::vec![::ethers::core::abi::ethabi::Event {
                        name: ::std::borrow::ToOwned::to_owned("BottomUpCheckpointExecuted",),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("epoch"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                indexed: false,
                            },
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("submitter"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                indexed: false,
                            },
                        ],
                        anonymous: false,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("BottomUpCheckpointSubmitted"),
                    ::std::vec![::ethers::core::abi::ethabi::Event {
                        name: ::std::borrow::ToOwned::to_owned("BottomUpCheckpointSubmitted",),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("checkpoint"),
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
                                    ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                    ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                    ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize),
                                ],),
                                indexed: false,
                            },
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("submitter"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                indexed: false,
                            },
                        ],
                        anonymous: false,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NextBottomUpCheckpointExecuted"),
                    ::std::vec![::ethers::core::abi::ethabi::Event {
                        name: ::std::borrow::ToOwned::to_owned("NextBottomUpCheckpointExecuted",),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("epoch"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                indexed: false,
                            },
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("submitter"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                indexed: false,
                            },
                        ],
                        anonymous: false,
                    },],
                ),
            ]),
            errors: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("AddressInsufficientBalance"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("AddressInsufficientBalance",),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("account"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Address,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("address"),
                            ),
                        },],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("CollateralIsZero"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("CollateralIsZero"),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("EmptyAddress"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("EmptyAddress"),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("FailedInnerCall"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("FailedInnerCall"),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("InvalidSignatureErr"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("InvalidSignatureErr",),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint8"),
                            ),
                        },],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NoCollateralToWithdraw"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("NoCollateralToWithdraw",),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NoRewardToWithdraw"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("NoRewardToWithdraw"),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NoValidatorsInSubnet"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("NoValidatorsInSubnet",),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NotAllValidatorsHaveLeft"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("NotAllValidatorsHaveLeft",),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NotEnoughBalanceForRewards"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("NotEnoughBalanceForRewards",),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NotGateway"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("NotGateway"),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NotStakedBefore"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("NotStakedBefore"),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NotValidator"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("NotValidator"),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ReentrancyError"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("ReentrancyError"),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("SubnetAlreadyKilled"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("SubnetAlreadyKilled",),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("SubnetNotActive"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("SubnetNotActive"),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("WithdrawExceedingCollateral"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("WithdrawExceedingCollateral",),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("WrongCheckpointSource"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("WrongCheckpointSource",),
                        inputs: ::std::vec![],
                    },],
                ),
            ]),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static SUBNETACTORMANAGERFACET_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(__abi);
    pub struct SubnetActorManagerFacet<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for SubnetActorManagerFacet<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for SubnetActorManagerFacet<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for SubnetActorManagerFacet<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for SubnetActorManagerFacet<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(SubnetActorManagerFacet))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> SubnetActorManagerFacet<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                SUBNETACTORMANAGERFACET_ABI.clone(),
                client,
            ))
        }
        ///Calls the contract's `claim` (0x4e71d92d) function
        pub fn claim(&self) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([78, 113, 217, 45], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getValidator` (0x1904bb2e) function
        pub fn get_validator(
            &self,
            validator_address: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, Validator> {
            self.0
                .method_hash([25, 4, 187, 46], validator_address)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `join` (0x6cf6970a) function
        pub fn join(
            &self,
            net_addr: ::std::string::String,
            worker_addr: FvmAddress,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([108, 246, 151, 10], (net_addr, worker_addr))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `join2` (0xedb0ff83) function
        pub fn join_2(
            &self,
            data: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([237, 176, 255, 131], data)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `kill` (0x41c0e1b5) function
        pub fn kill(&self) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([65, 192, 225, 181], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `kill2` (0x2bb685bc) function
        pub fn kill_2(&self) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([43, 182, 133, 188], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `leave` (0xd66d9e19) function
        pub fn leave(&self) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([214, 109, 158, 25], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `leave2` (0xf5904fcf) function
        pub fn leave_2(&self) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([245, 144, 79, 207], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `reward` (0xa9fb763c) function
        pub fn reward(
            &self,
            amount: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([169, 251, 118, 60], amount)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setMetadata` (0xee57e36f) function
        pub fn set_metadata(
            &self,
            metadata: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([238, 87, 227, 111], metadata)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setValidatorNetAddr` (0x7cc4fc43) function
        pub fn set_validator_net_addr(
            &self,
            new_net_addr: ::std::string::String,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([124, 196, 252, 67], new_net_addr)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `setValidatorWorkerAddr` (0xe8ef872f) function
        pub fn set_validator_worker_addr(
            &self,
            new_worker_addr: FvmAddress,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([232, 239, 135, 47], (new_worker_addr,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `stake` (0x3a4b66f1) function
        pub fn stake(&self) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([58, 75, 102, 241], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `submitCheckpoint` (0xf2d108c1) function
        pub fn submit_checkpoint(
            &self,
            checkpoint: BottomUpCheckpoint,
            membership_root_hash: [u8; 32],
            membership_weight: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash(
                    [242, 209, 8, 193],
                    (checkpoint, membership_root_hash, membership_weight),
                )
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `validateCheckpoint` (0x46ab0f48) function
        pub fn validate_checkpoint(
            &self,
            signatories: ::std::vec::Vec<::ethers::core::types::Address>,
            hash: [u8; 32],
            signatures: ::ethers::core::types::Bytes,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([70, 171, 15, 72], (signatories, hash, signatures))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `withdraw` (0x3ccfd60b) function
        pub fn withdraw(&self) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([60, 207, 214, 11], ())
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `BottomUpCheckpointExecuted` event
        pub fn bottom_up_checkpoint_executed_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            BottomUpCheckpointExecutedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `BottomUpCheckpointSubmitted` event
        pub fn bottom_up_checkpoint_submitted_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            BottomUpCheckpointSubmittedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `NextBottomUpCheckpointExecuted` event
        pub fn next_bottom_up_checkpoint_executed_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            NextBottomUpCheckpointExecutedFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            SubnetActorManagerFacetEvents,
        > {
            self.0
                .event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
        for SubnetActorManagerFacet<M>
    {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
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
        Hash,
    )]
    #[etherror(
        name = "AddressInsufficientBalance",
        abi = "AddressInsufficientBalance(address)"
    )]
    pub struct AddressInsufficientBalance {
        pub account: ::ethers::core::types::Address,
    }
    ///Custom Error type `CollateralIsZero` with signature `CollateralIsZero()` and selector `0xb4f18b02`
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
    #[etherror(name = "CollateralIsZero", abi = "CollateralIsZero()")]
    pub struct CollateralIsZero;
    ///Custom Error type `EmptyAddress` with signature `EmptyAddress()` and selector `0x7138356f`
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
    #[etherror(name = "EmptyAddress", abi = "EmptyAddress()")]
    pub struct EmptyAddress;
    ///Custom Error type `FailedInnerCall` with signature `FailedInnerCall()` and selector `0x1425ea42`
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
    #[etherror(name = "FailedInnerCall", abi = "FailedInnerCall()")]
    pub struct FailedInnerCall;
    ///Custom Error type `InvalidSignatureErr` with signature `InvalidSignatureErr(uint8)` and selector `0x282ef1c1`
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
    #[etherror(name = "InvalidSignatureErr", abi = "InvalidSignatureErr(uint8)")]
    pub struct InvalidSignatureErr(pub u8);
    ///Custom Error type `NoCollateralToWithdraw` with signature `NoCollateralToWithdraw()` and selector `0x64b0557f`
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
    #[etherror(name = "NoCollateralToWithdraw", abi = "NoCollateralToWithdraw()")]
    pub struct NoCollateralToWithdraw;
    ///Custom Error type `NoRewardToWithdraw` with signature `NoRewardToWithdraw()` and selector `0xce601f22`
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
    #[etherror(name = "NoRewardToWithdraw", abi = "NoRewardToWithdraw()")]
    pub struct NoRewardToWithdraw;
    ///Custom Error type `NoValidatorsInSubnet` with signature `NoValidatorsInSubnet()` and selector `0xefa9c8f1`
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
    #[etherror(name = "NoValidatorsInSubnet", abi = "NoValidatorsInSubnet()")]
    pub struct NoValidatorsInSubnet;
    ///Custom Error type `NotAllValidatorsHaveLeft` with signature `NotAllValidatorsHaveLeft()` and selector `0xd6c44aa2`
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
    #[etherror(name = "NotAllValidatorsHaveLeft", abi = "NotAllValidatorsHaveLeft()")]
    pub struct NotAllValidatorsHaveLeft;
    ///Custom Error type `NotEnoughBalanceForRewards` with signature `NotEnoughBalanceForRewards()` and selector `0x60e9957e`
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
        name = "NotEnoughBalanceForRewards",
        abi = "NotEnoughBalanceForRewards()"
    )]
    pub struct NotEnoughBalanceForRewards;
    ///Custom Error type `NotGateway` with signature `NotGateway()` and selector `0xe7e601db`
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
    #[etherror(name = "NotGateway", abi = "NotGateway()")]
    pub struct NotGateway;
    ///Custom Error type `NotStakedBefore` with signature `NotStakedBefore()` and selector `0x528fc165`
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
    #[etherror(name = "NotStakedBefore", abi = "NotStakedBefore()")]
    pub struct NotStakedBefore;
    ///Custom Error type `NotValidator` with signature `NotValidator()` and selector `0x2ec5b449`
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
    #[etherror(name = "NotValidator", abi = "NotValidator()")]
    pub struct NotValidator;
    ///Custom Error type `ReentrancyError` with signature `ReentrancyError()` and selector `0x29f745a7`
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
    #[etherror(name = "ReentrancyError", abi = "ReentrancyError()")]
    pub struct ReentrancyError;
    ///Custom Error type `SubnetAlreadyKilled` with signature `SubnetAlreadyKilled()` and selector `0x49191df6`
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
    #[etherror(name = "SubnetAlreadyKilled", abi = "SubnetAlreadyKilled()")]
    pub struct SubnetAlreadyKilled;
    ///Custom Error type `SubnetNotActive` with signature `SubnetNotActive()` and selector `0xc18316bf`
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
    #[etherror(name = "SubnetNotActive", abi = "SubnetNotActive()")]
    pub struct SubnetNotActive;
    ///Custom Error type `WithdrawExceedingCollateral` with signature `WithdrawExceedingCollateral()` and selector `0xac693603`
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
        name = "WithdrawExceedingCollateral",
        abi = "WithdrawExceedingCollateral()"
    )]
    pub struct WithdrawExceedingCollateral;
    ///Custom Error type `WrongCheckpointSource` with signature `WrongCheckpointSource()` and selector `0x75ecc72d`
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
    #[etherror(name = "WrongCheckpointSource", abi = "WrongCheckpointSource()")]
    pub struct WrongCheckpointSource;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum SubnetActorManagerFacetErrors {
        AddressInsufficientBalance(AddressInsufficientBalance),
        CollateralIsZero(CollateralIsZero),
        EmptyAddress(EmptyAddress),
        FailedInnerCall(FailedInnerCall),
        InvalidSignatureErr(InvalidSignatureErr),
        NoCollateralToWithdraw(NoCollateralToWithdraw),
        NoRewardToWithdraw(NoRewardToWithdraw),
        NoValidatorsInSubnet(NoValidatorsInSubnet),
        NotAllValidatorsHaveLeft(NotAllValidatorsHaveLeft),
        NotEnoughBalanceForRewards(NotEnoughBalanceForRewards),
        NotGateway(NotGateway),
        NotStakedBefore(NotStakedBefore),
        NotValidator(NotValidator),
        ReentrancyError(ReentrancyError),
        SubnetAlreadyKilled(SubnetAlreadyKilled),
        SubnetNotActive(SubnetNotActive),
        WithdrawExceedingCollateral(WithdrawExceedingCollateral),
        WrongCheckpointSource(WrongCheckpointSource),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for SubnetActorManagerFacetErrors {
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
                <AddressInsufficientBalance as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::AddressInsufficientBalance(decoded));
            }
            if let Ok(decoded) = <CollateralIsZero as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::CollateralIsZero(decoded));
            }
            if let Ok(decoded) = <EmptyAddress as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EmptyAddress(decoded));
            }
            if let Ok(decoded) = <FailedInnerCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::FailedInnerCall(decoded));
            }
            if let Ok(decoded) =
                <InvalidSignatureErr as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::InvalidSignatureErr(decoded));
            }
            if let Ok(decoded) =
                <NoCollateralToWithdraw as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NoCollateralToWithdraw(decoded));
            }
            if let Ok(decoded) =
                <NoRewardToWithdraw as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NoRewardToWithdraw(decoded));
            }
            if let Ok(decoded) =
                <NoValidatorsInSubnet as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NoValidatorsInSubnet(decoded));
            }
            if let Ok(decoded) =
                <NotAllValidatorsHaveLeft as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NotAllValidatorsHaveLeft(decoded));
            }
            if let Ok(decoded) =
                <NotEnoughBalanceForRewards as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NotEnoughBalanceForRewards(decoded));
            }
            if let Ok(decoded) = <NotGateway as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NotGateway(decoded));
            }
            if let Ok(decoded) = <NotStakedBefore as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NotStakedBefore(decoded));
            }
            if let Ok(decoded) = <NotValidator as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NotValidator(decoded));
            }
            if let Ok(decoded) = <ReentrancyError as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ReentrancyError(decoded));
            }
            if let Ok(decoded) =
                <SubnetAlreadyKilled as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::SubnetAlreadyKilled(decoded));
            }
            if let Ok(decoded) = <SubnetNotActive as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SubnetNotActive(decoded));
            }
            if let Ok(decoded) =
                <WithdrawExceedingCollateral as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::WithdrawExceedingCollateral(decoded));
            }
            if let Ok(decoded) =
                <WrongCheckpointSource as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::WrongCheckpointSource(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for SubnetActorManagerFacetErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::AddressInsufficientBalance(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CollateralIsZero(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::EmptyAddress(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::FailedInnerCall(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::InvalidSignatureErr(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NoCollateralToWithdraw(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NoRewardToWithdraw(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NoValidatorsInSubnet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotAllValidatorsHaveLeft(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotEnoughBalanceForRewards(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotGateway(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::NotStakedBefore(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::NotValidator(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ReentrancyError(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::SubnetAlreadyKilled(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SubnetNotActive(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::WithdrawExceedingCollateral(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::WrongCheckpointSource(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for SubnetActorManagerFacetErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <AddressInsufficientBalance as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector == <CollateralIsZero as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector == <EmptyAddress as ::ethers::contract::EthError>::selector() => true,
                _ if selector == <FailedInnerCall as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidSignatureErr as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <NoCollateralToWithdraw as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <NoRewardToWithdraw as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <NoValidatorsInSubnet as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <NotAllValidatorsHaveLeft as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector
                    == <NotEnoughBalanceForRewards as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector == <NotGateway as ::ethers::contract::EthError>::selector() => true,
                _ if selector == <NotStakedBefore as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector == <NotValidator as ::ethers::contract::EthError>::selector() => true,
                _ if selector == <ReentrancyError as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <SubnetAlreadyKilled as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector == <SubnetNotActive as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <WithdrawExceedingCollateral as ::ethers::contract::EthError>::selector(
                    ) =>
                {
                    true
                }
                _ if selector
                    == <WrongCheckpointSource as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for SubnetActorManagerFacetErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AddressInsufficientBalance(element) => ::core::fmt::Display::fmt(element, f),
                Self::CollateralIsZero(element) => ::core::fmt::Display::fmt(element, f),
                Self::EmptyAddress(element) => ::core::fmt::Display::fmt(element, f),
                Self::FailedInnerCall(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidSignatureErr(element) => ::core::fmt::Display::fmt(element, f),
                Self::NoCollateralToWithdraw(element) => ::core::fmt::Display::fmt(element, f),
                Self::NoRewardToWithdraw(element) => ::core::fmt::Display::fmt(element, f),
                Self::NoValidatorsInSubnet(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotAllValidatorsHaveLeft(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotEnoughBalanceForRewards(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotGateway(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotStakedBefore(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotValidator(element) => ::core::fmt::Display::fmt(element, f),
                Self::ReentrancyError(element) => ::core::fmt::Display::fmt(element, f),
                Self::SubnetAlreadyKilled(element) => ::core::fmt::Display::fmt(element, f),
                Self::SubnetNotActive(element) => ::core::fmt::Display::fmt(element, f),
                Self::WithdrawExceedingCollateral(element) => ::core::fmt::Display::fmt(element, f),
                Self::WrongCheckpointSource(element) => ::core::fmt::Display::fmt(element, f),
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for SubnetActorManagerFacetErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<AddressInsufficientBalance> for SubnetActorManagerFacetErrors {
        fn from(value: AddressInsufficientBalance) -> Self {
            Self::AddressInsufficientBalance(value)
        }
    }
    impl ::core::convert::From<CollateralIsZero> for SubnetActorManagerFacetErrors {
        fn from(value: CollateralIsZero) -> Self {
            Self::CollateralIsZero(value)
        }
    }
    impl ::core::convert::From<EmptyAddress> for SubnetActorManagerFacetErrors {
        fn from(value: EmptyAddress) -> Self {
            Self::EmptyAddress(value)
        }
    }
    impl ::core::convert::From<FailedInnerCall> for SubnetActorManagerFacetErrors {
        fn from(value: FailedInnerCall) -> Self {
            Self::FailedInnerCall(value)
        }
    }
    impl ::core::convert::From<InvalidSignatureErr> for SubnetActorManagerFacetErrors {
        fn from(value: InvalidSignatureErr) -> Self {
            Self::InvalidSignatureErr(value)
        }
    }
    impl ::core::convert::From<NoCollateralToWithdraw> for SubnetActorManagerFacetErrors {
        fn from(value: NoCollateralToWithdraw) -> Self {
            Self::NoCollateralToWithdraw(value)
        }
    }
    impl ::core::convert::From<NoRewardToWithdraw> for SubnetActorManagerFacetErrors {
        fn from(value: NoRewardToWithdraw) -> Self {
            Self::NoRewardToWithdraw(value)
        }
    }
    impl ::core::convert::From<NoValidatorsInSubnet> for SubnetActorManagerFacetErrors {
        fn from(value: NoValidatorsInSubnet) -> Self {
            Self::NoValidatorsInSubnet(value)
        }
    }
    impl ::core::convert::From<NotAllValidatorsHaveLeft> for SubnetActorManagerFacetErrors {
        fn from(value: NotAllValidatorsHaveLeft) -> Self {
            Self::NotAllValidatorsHaveLeft(value)
        }
    }
    impl ::core::convert::From<NotEnoughBalanceForRewards> for SubnetActorManagerFacetErrors {
        fn from(value: NotEnoughBalanceForRewards) -> Self {
            Self::NotEnoughBalanceForRewards(value)
        }
    }
    impl ::core::convert::From<NotGateway> for SubnetActorManagerFacetErrors {
        fn from(value: NotGateway) -> Self {
            Self::NotGateway(value)
        }
    }
    impl ::core::convert::From<NotStakedBefore> for SubnetActorManagerFacetErrors {
        fn from(value: NotStakedBefore) -> Self {
            Self::NotStakedBefore(value)
        }
    }
    impl ::core::convert::From<NotValidator> for SubnetActorManagerFacetErrors {
        fn from(value: NotValidator) -> Self {
            Self::NotValidator(value)
        }
    }
    impl ::core::convert::From<ReentrancyError> for SubnetActorManagerFacetErrors {
        fn from(value: ReentrancyError) -> Self {
            Self::ReentrancyError(value)
        }
    }
    impl ::core::convert::From<SubnetAlreadyKilled> for SubnetActorManagerFacetErrors {
        fn from(value: SubnetAlreadyKilled) -> Self {
            Self::SubnetAlreadyKilled(value)
        }
    }
    impl ::core::convert::From<SubnetNotActive> for SubnetActorManagerFacetErrors {
        fn from(value: SubnetNotActive) -> Self {
            Self::SubnetNotActive(value)
        }
    }
    impl ::core::convert::From<WithdrawExceedingCollateral> for SubnetActorManagerFacetErrors {
        fn from(value: WithdrawExceedingCollateral) -> Self {
            Self::WithdrawExceedingCollateral(value)
        }
    }
    impl ::core::convert::From<WrongCheckpointSource> for SubnetActorManagerFacetErrors {
        fn from(value: WrongCheckpointSource) -> Self {
            Self::WrongCheckpointSource(value)
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
        name = "BottomUpCheckpointExecuted",
        abi = "BottomUpCheckpointExecuted(uint64,address)"
    )]
    pub struct BottomUpCheckpointExecutedFilter {
        pub epoch: u64,
        pub submitter: ::ethers::core::types::Address,
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
        name = "BottomUpCheckpointSubmitted",
        abi = "BottomUpCheckpointSubmitted(((uint64,address[]),uint64,bytes32,uint64,bytes32),address)"
    )]
    pub struct BottomUpCheckpointSubmittedFilter {
        pub checkpoint: BottomUpCheckpoint,
        pub submitter: ::ethers::core::types::Address,
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
        name = "NextBottomUpCheckpointExecuted",
        abi = "NextBottomUpCheckpointExecuted(uint64,address)"
    )]
    pub struct NextBottomUpCheckpointExecutedFilter {
        pub epoch: u64,
        pub submitter: ::ethers::core::types::Address,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum SubnetActorManagerFacetEvents {
        BottomUpCheckpointExecutedFilter(BottomUpCheckpointExecutedFilter),
        BottomUpCheckpointSubmittedFilter(BottomUpCheckpointSubmittedFilter),
        NextBottomUpCheckpointExecutedFilter(NextBottomUpCheckpointExecutedFilter),
    }
    impl ::ethers::contract::EthLogDecode for SubnetActorManagerFacetEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = BottomUpCheckpointExecutedFilter::decode_log(log) {
                return Ok(
                    SubnetActorManagerFacetEvents::BottomUpCheckpointExecutedFilter(decoded),
                );
            }
            if let Ok(decoded) = BottomUpCheckpointSubmittedFilter::decode_log(log) {
                return Ok(
                    SubnetActorManagerFacetEvents::BottomUpCheckpointSubmittedFilter(decoded),
                );
            }
            if let Ok(decoded) = NextBottomUpCheckpointExecutedFilter::decode_log(log) {
                return Ok(
                    SubnetActorManagerFacetEvents::NextBottomUpCheckpointExecutedFilter(decoded),
                );
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for SubnetActorManagerFacetEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::BottomUpCheckpointExecutedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::BottomUpCheckpointSubmittedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NextBottomUpCheckpointExecutedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<BottomUpCheckpointExecutedFilter> for SubnetActorManagerFacetEvents {
        fn from(value: BottomUpCheckpointExecutedFilter) -> Self {
            Self::BottomUpCheckpointExecutedFilter(value)
        }
    }
    impl ::core::convert::From<BottomUpCheckpointSubmittedFilter> for SubnetActorManagerFacetEvents {
        fn from(value: BottomUpCheckpointSubmittedFilter) -> Self {
            Self::BottomUpCheckpointSubmittedFilter(value)
        }
    }
    impl ::core::convert::From<NextBottomUpCheckpointExecutedFilter> for SubnetActorManagerFacetEvents {
        fn from(value: NextBottomUpCheckpointExecutedFilter) -> Self {
            Self::NextBottomUpCheckpointExecutedFilter(value)
        }
    }
    ///Container type for all input parameters for the `claim` function with signature `claim()` and selector `0x4e71d92d`
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
    #[ethcall(name = "claim", abi = "claim()")]
    pub struct ClaimCall;
    ///Container type for all input parameters for the `getValidator` function with signature `getValidator(address)` and selector `0x1904bb2e`
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
    #[ethcall(name = "getValidator", abi = "getValidator(address)")]
    pub struct GetValidatorCall {
        pub validator_address: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `join` function with signature `join(string,(uint8,bytes))` and selector `0x6cf6970a`
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
    #[ethcall(name = "join", abi = "join(string,(uint8,bytes))")]
    pub struct JoinCall {
        pub net_addr: ::std::string::String,
        pub worker_addr: FvmAddress,
    }
    ///Container type for all input parameters for the `join2` function with signature `join2(bytes)` and selector `0xedb0ff83`
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
    #[ethcall(name = "join2", abi = "join2(bytes)")]
    pub struct Join2Call {
        pub data: ::ethers::core::types::Bytes,
    }
    ///Container type for all input parameters for the `kill` function with signature `kill()` and selector `0x41c0e1b5`
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
    #[ethcall(name = "kill", abi = "kill()")]
    pub struct KillCall;
    ///Container type for all input parameters for the `kill2` function with signature `kill2()` and selector `0x2bb685bc`
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
    #[ethcall(name = "kill2", abi = "kill2()")]
    pub struct Kill2Call;
    ///Container type for all input parameters for the `leave` function with signature `leave()` and selector `0xd66d9e19`
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
    #[ethcall(name = "leave", abi = "leave()")]
    pub struct LeaveCall;
    ///Container type for all input parameters for the `leave2` function with signature `leave2()` and selector `0xf5904fcf`
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
    #[ethcall(name = "leave2", abi = "leave2()")]
    pub struct Leave2Call;
    ///Container type for all input parameters for the `reward` function with signature `reward(uint256)` and selector `0xa9fb763c`
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
    #[ethcall(name = "reward", abi = "reward(uint256)")]
    pub struct RewardCall {
        pub amount: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `setMetadata` function with signature `setMetadata(bytes)` and selector `0xee57e36f`
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
    #[ethcall(name = "setMetadata", abi = "setMetadata(bytes)")]
    pub struct SetMetadataCall {
        pub metadata: ::ethers::core::types::Bytes,
    }
    ///Container type for all input parameters for the `setValidatorNetAddr` function with signature `setValidatorNetAddr(string)` and selector `0x7cc4fc43`
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
    #[ethcall(name = "setValidatorNetAddr", abi = "setValidatorNetAddr(string)")]
    pub struct SetValidatorNetAddrCall {
        pub new_net_addr: ::std::string::String,
    }
    ///Container type for all input parameters for the `setValidatorWorkerAddr` function with signature `setValidatorWorkerAddr((uint8,bytes))` and selector `0xe8ef872f`
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
        name = "setValidatorWorkerAddr",
        abi = "setValidatorWorkerAddr((uint8,bytes))"
    )]
    pub struct SetValidatorWorkerAddrCall {
        pub new_worker_addr: FvmAddress,
    }
    ///Container type for all input parameters for the `stake` function with signature `stake()` and selector `0x3a4b66f1`
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
    #[ethcall(name = "stake", abi = "stake()")]
    pub struct StakeCall;
    ///Container type for all input parameters for the `submitCheckpoint` function with signature `submitCheckpoint(((uint64,address[]),uint64,bytes32,uint64,bytes32),bytes32,uint256)` and selector `0xf2d108c1`
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
        name = "submitCheckpoint",
        abi = "submitCheckpoint(((uint64,address[]),uint64,bytes32,uint64,bytes32),bytes32,uint256)"
    )]
    pub struct SubmitCheckpointCall {
        pub checkpoint: BottomUpCheckpoint,
        pub membership_root_hash: [u8; 32],
        pub membership_weight: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `validateCheckpoint` function with signature `validateCheckpoint(address[],bytes32,bytes)` and selector `0x46ab0f48`
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
        name = "validateCheckpoint",
        abi = "validateCheckpoint(address[],bytes32,bytes)"
    )]
    pub struct ValidateCheckpointCall {
        pub signatories: ::std::vec::Vec<::ethers::core::types::Address>,
        pub hash: [u8; 32],
        pub signatures: ::ethers::core::types::Bytes,
    }
    ///Container type for all input parameters for the `withdraw` function with signature `withdraw()` and selector `0x3ccfd60b`
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
    #[ethcall(name = "withdraw", abi = "withdraw()")]
    pub struct WithdrawCall;
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum SubnetActorManagerFacetCalls {
        Claim(ClaimCall),
        GetValidator(GetValidatorCall),
        Join(JoinCall),
        Join2(Join2Call),
        Kill(KillCall),
        Kill2(Kill2Call),
        Leave(LeaveCall),
        Leave2(Leave2Call),
        Reward(RewardCall),
        SetMetadata(SetMetadataCall),
        SetValidatorNetAddr(SetValidatorNetAddrCall),
        SetValidatorWorkerAddr(SetValidatorWorkerAddrCall),
        Stake(StakeCall),
        SubmitCheckpoint(SubmitCheckpointCall),
        ValidateCheckpoint(ValidateCheckpointCall),
        Withdraw(WithdrawCall),
    }
    impl ::ethers::core::abi::AbiDecode for SubnetActorManagerFacetCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <ClaimCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Claim(decoded));
            }
            if let Ok(decoded) = <GetValidatorCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetValidator(decoded));
            }
            if let Ok(decoded) = <JoinCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Join(decoded));
            }
            if let Ok(decoded) = <Join2Call as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Join2(decoded));
            }
            if let Ok(decoded) = <KillCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Kill(decoded));
            }
            if let Ok(decoded) = <Kill2Call as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Kill2(decoded));
            }
            if let Ok(decoded) = <LeaveCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Leave(decoded));
            }
            if let Ok(decoded) = <Leave2Call as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Leave2(decoded));
            }
            if let Ok(decoded) = <RewardCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Reward(decoded));
            }
            if let Ok(decoded) = <SetMetadataCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SetMetadata(decoded));
            }
            if let Ok(decoded) =
                <SetValidatorNetAddrCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::SetValidatorNetAddr(decoded));
            }
            if let Ok(decoded) =
                <SetValidatorWorkerAddrCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::SetValidatorWorkerAddr(decoded));
            }
            if let Ok(decoded) = <StakeCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Stake(decoded));
            }
            if let Ok(decoded) =
                <SubmitCheckpointCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::SubmitCheckpoint(decoded));
            }
            if let Ok(decoded) =
                <ValidateCheckpointCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::ValidateCheckpoint(decoded));
            }
            if let Ok(decoded) = <WithdrawCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Withdraw(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for SubnetActorManagerFacetCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::Claim(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetValidator(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Join(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Join2(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Kill(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Kill2(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Leave(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Leave2(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Reward(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::SetMetadata(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::SetValidatorNetAddr(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SetValidatorWorkerAddr(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Stake(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::SubmitCheckpoint(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ValidateCheckpoint(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Withdraw(element) => ::ethers::core::abi::AbiEncode::encode(element),
            }
        }
    }
    impl ::core::fmt::Display for SubnetActorManagerFacetCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::Claim(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetValidator(element) => ::core::fmt::Display::fmt(element, f),
                Self::Join(element) => ::core::fmt::Display::fmt(element, f),
                Self::Join2(element) => ::core::fmt::Display::fmt(element, f),
                Self::Kill(element) => ::core::fmt::Display::fmt(element, f),
                Self::Kill2(element) => ::core::fmt::Display::fmt(element, f),
                Self::Leave(element) => ::core::fmt::Display::fmt(element, f),
                Self::Leave2(element) => ::core::fmt::Display::fmt(element, f),
                Self::Reward(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetMetadata(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetValidatorNetAddr(element) => ::core::fmt::Display::fmt(element, f),
                Self::SetValidatorWorkerAddr(element) => ::core::fmt::Display::fmt(element, f),
                Self::Stake(element) => ::core::fmt::Display::fmt(element, f),
                Self::SubmitCheckpoint(element) => ::core::fmt::Display::fmt(element, f),
                Self::ValidateCheckpoint(element) => ::core::fmt::Display::fmt(element, f),
                Self::Withdraw(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<ClaimCall> for SubnetActorManagerFacetCalls {
        fn from(value: ClaimCall) -> Self {
            Self::Claim(value)
        }
    }
    impl ::core::convert::From<GetValidatorCall> for SubnetActorManagerFacetCalls {
        fn from(value: GetValidatorCall) -> Self {
            Self::GetValidator(value)
        }
    }
    impl ::core::convert::From<JoinCall> for SubnetActorManagerFacetCalls {
        fn from(value: JoinCall) -> Self {
            Self::Join(value)
        }
    }
    impl ::core::convert::From<Join2Call> for SubnetActorManagerFacetCalls {
        fn from(value: Join2Call) -> Self {
            Self::Join2(value)
        }
    }
    impl ::core::convert::From<KillCall> for SubnetActorManagerFacetCalls {
        fn from(value: KillCall) -> Self {
            Self::Kill(value)
        }
    }
    impl ::core::convert::From<Kill2Call> for SubnetActorManagerFacetCalls {
        fn from(value: Kill2Call) -> Self {
            Self::Kill2(value)
        }
    }
    impl ::core::convert::From<LeaveCall> for SubnetActorManagerFacetCalls {
        fn from(value: LeaveCall) -> Self {
            Self::Leave(value)
        }
    }
    impl ::core::convert::From<Leave2Call> for SubnetActorManagerFacetCalls {
        fn from(value: Leave2Call) -> Self {
            Self::Leave2(value)
        }
    }
    impl ::core::convert::From<RewardCall> for SubnetActorManagerFacetCalls {
        fn from(value: RewardCall) -> Self {
            Self::Reward(value)
        }
    }
    impl ::core::convert::From<SetMetadataCall> for SubnetActorManagerFacetCalls {
        fn from(value: SetMetadataCall) -> Self {
            Self::SetMetadata(value)
        }
    }
    impl ::core::convert::From<SetValidatorNetAddrCall> for SubnetActorManagerFacetCalls {
        fn from(value: SetValidatorNetAddrCall) -> Self {
            Self::SetValidatorNetAddr(value)
        }
    }
    impl ::core::convert::From<SetValidatorWorkerAddrCall> for SubnetActorManagerFacetCalls {
        fn from(value: SetValidatorWorkerAddrCall) -> Self {
            Self::SetValidatorWorkerAddr(value)
        }
    }
    impl ::core::convert::From<StakeCall> for SubnetActorManagerFacetCalls {
        fn from(value: StakeCall) -> Self {
            Self::Stake(value)
        }
    }
    impl ::core::convert::From<SubmitCheckpointCall> for SubnetActorManagerFacetCalls {
        fn from(value: SubmitCheckpointCall) -> Self {
            Self::SubmitCheckpoint(value)
        }
    }
    impl ::core::convert::From<ValidateCheckpointCall> for SubnetActorManagerFacetCalls {
        fn from(value: ValidateCheckpointCall) -> Self {
            Self::ValidateCheckpoint(value)
        }
    }
    impl ::core::convert::From<WithdrawCall> for SubnetActorManagerFacetCalls {
        fn from(value: WithdrawCall) -> Self {
            Self::Withdraw(value)
        }
    }
    ///Container type for all return fields from the `getValidator` function with signature `getValidator(address)` and selector `0x1904bb2e`
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
    pub struct GetValidatorReturn {
        pub validator: Validator,
    }
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
    ///`Validator(uint256,uint256,bytes)`
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
        pub confirmed_collateral: ::ethers::core::types::U256,
        pub total_collateral: ::ethers::core::types::U256,
        pub data: ::ethers::core::types::Bytes,
    }
}
