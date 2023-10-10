pub use subnet_actor_getter_facet::*;
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
pub mod subnet_actor_getter_facet {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("bootstrapped"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("bootstrapped"),
                        inputs: ::std::vec![],
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
                (
                    ::std::borrow::ToOwned::to_owned("bottomUpCheckPeriod"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("bottomUpCheckPeriod",),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint64"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("bottomUpCheckpointAtEpoch"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("bottomUpCheckpointAtEpoch",),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("epoch"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint64"),
                            ),
                        },],
                        outputs: ::std::vec![
                            ::ethers::core::abi::ethabi::Param {
                                name: ::std::borrow::ToOwned::to_owned("exists"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Bool,
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("bool"),
                                ),
                            },
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
                        ],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("bottomUpCheckpointHashAtEpoch"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("bottomUpCheckpointHashAtEpoch",),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("epoch"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint64"),
                            ),
                        },],
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
                                kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize,),
                                internal_type: ::core::option::Option::Some(
                                    ::std::borrow::ToOwned::to_owned("bytes32"),
                                ),
                            },
                        ],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("consensus"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("consensus"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("enum ConsensusType"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("genesisValidators"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("genesisValidators"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Array(
                                ::std::boxed::Box::new(
                                    ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                        ::ethers::core::abi::ethabi::ParamType::Uint(256usize),
                                        ::ethers::core::abi::ethabi::ParamType::Address,
                                        ::ethers::core::abi::ethabi::ParamType::Bytes,
                                    ],),
                                ),
                            ),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("struct Validator[]"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getConfigurationNumbers"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("getConfigurationNumbers",),
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
                        ],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("getParent"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("getParent"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Tuple(::std::vec![
                                ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                                ::ethers::core::abi::ethabi::ParamType::Array(
                                    ::std::boxed::Box::new(
                                        ::ethers::core::abi::ethabi::ParamType::Address,
                                    ),
                                ),
                            ],),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("struct SubnetID"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
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
                                ::std::borrow::ToOwned::to_owned("struct ValidatorInfo"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ipcGatewayAddr"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("ipcGatewayAddr"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
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
                    ::std::borrow::ToOwned::to_owned("isActiveValidator"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("isActiveValidator"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::borrow::ToOwned::to_owned("validator"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Address,
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("address"),
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
                (
                    ::std::borrow::ToOwned::to_owned("killed"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("killed"),
                        inputs: ::std::vec![],
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
                (
                    ::std::borrow::ToOwned::to_owned("minActivationCollateral"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("minActivationCollateral",),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint256"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("minValidators"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("minValidators"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("uint64"),
                            ),
                        },],
                        constant: ::core::option::Option::None,
                        state_mutability: ::ethers::core::abi::ethabi::StateMutability::View,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("name"),
                    ::std::vec![::ethers::core::abi::ethabi::Function {
                        name: ::std::borrow::ToOwned::to_owned("name"),
                        inputs: ::std::vec![],
                        outputs: ::std::vec![::ethers::core::abi::ethabi::Param {
                            name: ::std::string::String::new(),
                            kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize,),
                            internal_type: ::core::option::Option::Some(
                                ::std::borrow::ToOwned::to_owned("bytes32"),
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
    pub static SUBNETACTORGETTERFACET_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(__abi);
    pub struct SubnetActorGetterFacet<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for SubnetActorGetterFacet<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for SubnetActorGetterFacet<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for SubnetActorGetterFacet<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for SubnetActorGetterFacet<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(SubnetActorGetterFacet))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> SubnetActorGetterFacet<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                SUBNETACTORGETTERFACET_ABI.clone(),
                client,
            ))
        }
        ///Calls the contract's `bootstrapped` (0x35142c8c) function
        pub fn bootstrapped(&self) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([53, 20, 44, 140], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `bottomUpCheckPeriod` (0x06c46853) function
        pub fn bottom_up_check_period(&self) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([6, 196, 104, 83], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `bottomUpCheckpointAtEpoch` (0x6cb2ecee) function
        pub fn bottom_up_checkpoint_at_epoch(
            &self,
            epoch: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, (bool, BottomUpCheckpoint)> {
            self.0
                .method_hash([108, 178, 236, 238], epoch)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `bottomUpCheckpointHashAtEpoch` (0x133f74ea) function
        pub fn bottom_up_checkpoint_hash_at_epoch(
            &self,
            epoch: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, (bool, [u8; 32])> {
            self.0
                .method_hash([19, 63, 116, 234], epoch)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `consensus` (0x8ef3f761) function
        pub fn consensus(&self) -> ::ethers::contract::builders::ContractCall<M, u8> {
            self.0
                .method_hash([142, 243, 247, 97], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `genesisValidators` (0xd92e8f12) function
        pub fn genesis_validators(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::vec::Vec<Validator>> {
            self.0
                .method_hash([217, 46, 143, 18], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getConfigurationNumbers` (0x38a210b3) function
        pub fn get_configuration_numbers(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, (u64, u64)> {
            self.0
                .method_hash([56, 162, 16, 179], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getParent` (0x80f76021) function
        pub fn get_parent(&self) -> ::ethers::contract::builders::ContractCall<M, SubnetID> {
            self.0
                .method_hash([128, 247, 96, 33], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getValidator` (0x1904bb2e) function
        pub fn get_validator(
            &self,
            validator_address: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ValidatorInfo> {
            self.0
                .method_hash([25, 4, 187, 46], validator_address)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `ipcGatewayAddr` (0xcfca2824) function
        pub fn ipc_gateway_addr(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::Address> {
            self.0
                .method_hash([207, 202, 40, 36], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `isActiveValidator` (0x40550a1c) function
        pub fn is_active_validator(
            &self,
            validator: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([64, 85, 10, 28], validator)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `killed` (0x1f3a0e41) function
        pub fn killed(&self) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([31, 58, 14, 65], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `minActivationCollateral` (0x9e33bd02) function
        pub fn min_activation_collateral(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([158, 51, 189, 2], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `minValidators` (0xc5ab2241) function
        pub fn min_validators(&self) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([197, 171, 34, 65], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `name` (0x06fdde03) function
        pub fn name(&self) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([6, 253, 222, 3], ())
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
        for SubnetActorGetterFacet<M>
    {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Container type for all input parameters for the `bootstrapped` function with signature `bootstrapped()` and selector `0x35142c8c`
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
    #[ethcall(name = "bootstrapped", abi = "bootstrapped()")]
    pub struct BootstrappedCall;
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
    ///Container type for all input parameters for the `bottomUpCheckpointAtEpoch` function with signature `bottomUpCheckpointAtEpoch(uint64)` and selector `0x6cb2ecee`
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
        name = "bottomUpCheckpointAtEpoch",
        abi = "bottomUpCheckpointAtEpoch(uint64)"
    )]
    pub struct BottomUpCheckpointAtEpochCall {
        pub epoch: u64,
    }
    ///Container type for all input parameters for the `bottomUpCheckpointHashAtEpoch` function with signature `bottomUpCheckpointHashAtEpoch(uint64)` and selector `0x133f74ea`
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
        name = "bottomUpCheckpointHashAtEpoch",
        abi = "bottomUpCheckpointHashAtEpoch(uint64)"
    )]
    pub struct BottomUpCheckpointHashAtEpochCall {
        pub epoch: u64,
    }
    ///Container type for all input parameters for the `consensus` function with signature `consensus()` and selector `0x8ef3f761`
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
    #[ethcall(name = "consensus", abi = "consensus()")]
    pub struct ConsensusCall;
    ///Container type for all input parameters for the `genesisValidators` function with signature `genesisValidators()` and selector `0xd92e8f12`
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
    #[ethcall(name = "genesisValidators", abi = "genesisValidators()")]
    pub struct GenesisValidatorsCall;
    ///Container type for all input parameters for the `getConfigurationNumbers` function with signature `getConfigurationNumbers()` and selector `0x38a210b3`
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
    #[ethcall(name = "getConfigurationNumbers", abi = "getConfigurationNumbers()")]
    pub struct GetConfigurationNumbersCall;
    ///Container type for all input parameters for the `getParent` function with signature `getParent()` and selector `0x80f76021`
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
    #[ethcall(name = "getParent", abi = "getParent()")]
    pub struct GetParentCall;
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
    ///Container type for all input parameters for the `ipcGatewayAddr` function with signature `ipcGatewayAddr()` and selector `0xcfca2824`
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
    #[ethcall(name = "ipcGatewayAddr", abi = "ipcGatewayAddr()")]
    pub struct IpcGatewayAddrCall;
    ///Container type for all input parameters for the `isActiveValidator` function with signature `isActiveValidator(address)` and selector `0x40550a1c`
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
    #[ethcall(name = "isActiveValidator", abi = "isActiveValidator(address)")]
    pub struct IsActiveValidatorCall {
        pub validator: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `killed` function with signature `killed()` and selector `0x1f3a0e41`
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
    #[ethcall(name = "killed", abi = "killed()")]
    pub struct KilledCall;
    ///Container type for all input parameters for the `minActivationCollateral` function with signature `minActivationCollateral()` and selector `0x9e33bd02`
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
    #[ethcall(name = "minActivationCollateral", abi = "minActivationCollateral()")]
    pub struct MinActivationCollateralCall;
    ///Container type for all input parameters for the `minValidators` function with signature `minValidators()` and selector `0xc5ab2241`
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
    #[ethcall(name = "minValidators", abi = "minValidators()")]
    pub struct MinValidatorsCall;
    ///Container type for all input parameters for the `name` function with signature `name()` and selector `0x06fdde03`
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
    #[ethcall(name = "name", abi = "name()")]
    pub struct NameCall;
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum SubnetActorGetterFacetCalls {
        Bootstrapped(BootstrappedCall),
        BottomUpCheckPeriod(BottomUpCheckPeriodCall),
        BottomUpCheckpointAtEpoch(BottomUpCheckpointAtEpochCall),
        BottomUpCheckpointHashAtEpoch(BottomUpCheckpointHashAtEpochCall),
        Consensus(ConsensusCall),
        GenesisValidators(GenesisValidatorsCall),
        GetConfigurationNumbers(GetConfigurationNumbersCall),
        GetParent(GetParentCall),
        GetValidator(GetValidatorCall),
        IpcGatewayAddr(IpcGatewayAddrCall),
        IsActiveValidator(IsActiveValidatorCall),
        Killed(KilledCall),
        MinActivationCollateral(MinActivationCollateralCall),
        MinValidators(MinValidatorsCall),
        Name(NameCall),
    }
    impl ::ethers::core::abi::AbiDecode for SubnetActorGetterFacetCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <BootstrappedCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::Bootstrapped(decoded));
            }
            if let Ok(decoded) =
                <BottomUpCheckPeriodCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::BottomUpCheckPeriod(decoded));
            }
            if let Ok(decoded) =
                <BottomUpCheckpointAtEpochCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::BottomUpCheckpointAtEpoch(decoded));
            }
            if let Ok(decoded) =
                <BottomUpCheckpointHashAtEpochCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::BottomUpCheckpointHashAtEpoch(decoded));
            }
            if let Ok(decoded) = <ConsensusCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Consensus(decoded));
            }
            if let Ok(decoded) =
                <GenesisValidatorsCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GenesisValidators(decoded));
            }
            if let Ok(decoded) =
                <GetConfigurationNumbersCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetConfigurationNumbers(decoded));
            }
            if let Ok(decoded) = <GetParentCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetParent(decoded));
            }
            if let Ok(decoded) = <GetValidatorCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::GetValidator(decoded));
            }
            if let Ok(decoded) =
                <IpcGatewayAddrCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::IpcGatewayAddr(decoded));
            }
            if let Ok(decoded) =
                <IsActiveValidatorCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::IsActiveValidator(decoded));
            }
            if let Ok(decoded) = <KilledCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Killed(decoded));
            }
            if let Ok(decoded) =
                <MinActivationCollateralCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::MinActivationCollateral(decoded));
            }
            if let Ok(decoded) = <MinValidatorsCall as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::MinValidators(decoded));
            }
            if let Ok(decoded) = <NameCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Name(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for SubnetActorGetterFacetCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::Bootstrapped(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::BottomUpCheckPeriod(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BottomUpCheckpointAtEpoch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BottomUpCheckpointHashAtEpoch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Consensus(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GenesisValidators(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetConfigurationNumbers(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetParent(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetValidator(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::IpcGatewayAddr(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::IsActiveValidator(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Killed(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::MinActivationCollateral(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MinValidators(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Name(element) => ::ethers::core::abi::AbiEncode::encode(element),
            }
        }
    }
    impl ::core::fmt::Display for SubnetActorGetterFacetCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::Bootstrapped(element) => ::core::fmt::Display::fmt(element, f),
                Self::BottomUpCheckPeriod(element) => ::core::fmt::Display::fmt(element, f),
                Self::BottomUpCheckpointAtEpoch(element) => ::core::fmt::Display::fmt(element, f),
                Self::BottomUpCheckpointHashAtEpoch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::Consensus(element) => ::core::fmt::Display::fmt(element, f),
                Self::GenesisValidators(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetConfigurationNumbers(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetParent(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetValidator(element) => ::core::fmt::Display::fmt(element, f),
                Self::IpcGatewayAddr(element) => ::core::fmt::Display::fmt(element, f),
                Self::IsActiveValidator(element) => ::core::fmt::Display::fmt(element, f),
                Self::Killed(element) => ::core::fmt::Display::fmt(element, f),
                Self::MinActivationCollateral(element) => ::core::fmt::Display::fmt(element, f),
                Self::MinValidators(element) => ::core::fmt::Display::fmt(element, f),
                Self::Name(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<BootstrappedCall> for SubnetActorGetterFacetCalls {
        fn from(value: BootstrappedCall) -> Self {
            Self::Bootstrapped(value)
        }
    }
    impl ::core::convert::From<BottomUpCheckPeriodCall> for SubnetActorGetterFacetCalls {
        fn from(value: BottomUpCheckPeriodCall) -> Self {
            Self::BottomUpCheckPeriod(value)
        }
    }
    impl ::core::convert::From<BottomUpCheckpointAtEpochCall> for SubnetActorGetterFacetCalls {
        fn from(value: BottomUpCheckpointAtEpochCall) -> Self {
            Self::BottomUpCheckpointAtEpoch(value)
        }
    }
    impl ::core::convert::From<BottomUpCheckpointHashAtEpochCall> for SubnetActorGetterFacetCalls {
        fn from(value: BottomUpCheckpointHashAtEpochCall) -> Self {
            Self::BottomUpCheckpointHashAtEpoch(value)
        }
    }
    impl ::core::convert::From<ConsensusCall> for SubnetActorGetterFacetCalls {
        fn from(value: ConsensusCall) -> Self {
            Self::Consensus(value)
        }
    }
    impl ::core::convert::From<GenesisValidatorsCall> for SubnetActorGetterFacetCalls {
        fn from(value: GenesisValidatorsCall) -> Self {
            Self::GenesisValidators(value)
        }
    }
    impl ::core::convert::From<GetConfigurationNumbersCall> for SubnetActorGetterFacetCalls {
        fn from(value: GetConfigurationNumbersCall) -> Self {
            Self::GetConfigurationNumbers(value)
        }
    }
    impl ::core::convert::From<GetParentCall> for SubnetActorGetterFacetCalls {
        fn from(value: GetParentCall) -> Self {
            Self::GetParent(value)
        }
    }
    impl ::core::convert::From<GetValidatorCall> for SubnetActorGetterFacetCalls {
        fn from(value: GetValidatorCall) -> Self {
            Self::GetValidator(value)
        }
    }
    impl ::core::convert::From<IpcGatewayAddrCall> for SubnetActorGetterFacetCalls {
        fn from(value: IpcGatewayAddrCall) -> Self {
            Self::IpcGatewayAddr(value)
        }
    }
    impl ::core::convert::From<IsActiveValidatorCall> for SubnetActorGetterFacetCalls {
        fn from(value: IsActiveValidatorCall) -> Self {
            Self::IsActiveValidator(value)
        }
    }
    impl ::core::convert::From<KilledCall> for SubnetActorGetterFacetCalls {
        fn from(value: KilledCall) -> Self {
            Self::Killed(value)
        }
    }
    impl ::core::convert::From<MinActivationCollateralCall> for SubnetActorGetterFacetCalls {
        fn from(value: MinActivationCollateralCall) -> Self {
            Self::MinActivationCollateral(value)
        }
    }
    impl ::core::convert::From<MinValidatorsCall> for SubnetActorGetterFacetCalls {
        fn from(value: MinValidatorsCall) -> Self {
            Self::MinValidators(value)
        }
    }
    impl ::core::convert::From<NameCall> for SubnetActorGetterFacetCalls {
        fn from(value: NameCall) -> Self {
            Self::Name(value)
        }
    }
    ///Container type for all return fields from the `bootstrapped` function with signature `bootstrapped()` and selector `0x35142c8c`
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
    pub struct BootstrappedReturn(pub bool);
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
    ///Container type for all return fields from the `bottomUpCheckpointAtEpoch` function with signature `bottomUpCheckpointAtEpoch(uint64)` and selector `0x6cb2ecee`
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
    pub struct BottomUpCheckpointAtEpochReturn {
        pub exists: bool,
        pub checkpoint: BottomUpCheckpoint,
    }
    ///Container type for all return fields from the `bottomUpCheckpointHashAtEpoch` function with signature `bottomUpCheckpointHashAtEpoch(uint64)` and selector `0x133f74ea`
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
    pub struct BottomUpCheckpointHashAtEpochReturn(pub bool, pub [u8; 32]);
    ///Container type for all return fields from the `consensus` function with signature `consensus()` and selector `0x8ef3f761`
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
    pub struct ConsensusReturn(pub u8);
    ///Container type for all return fields from the `genesisValidators` function with signature `genesisValidators()` and selector `0xd92e8f12`
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
    pub struct GenesisValidatorsReturn(pub ::std::vec::Vec<Validator>);
    ///Container type for all return fields from the `getConfigurationNumbers` function with signature `getConfigurationNumbers()` and selector `0x38a210b3`
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
    pub struct GetConfigurationNumbersReturn(pub u64, pub u64);
    ///Container type for all return fields from the `getParent` function with signature `getParent()` and selector `0x80f76021`
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
    pub struct GetParentReturn(pub SubnetID);
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
        pub validator: ValidatorInfo,
    }
    ///Container type for all return fields from the `ipcGatewayAddr` function with signature `ipcGatewayAddr()` and selector `0xcfca2824`
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
    pub struct IpcGatewayAddrReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `isActiveValidator` function with signature `isActiveValidator(address)` and selector `0x40550a1c`
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
    pub struct IsActiveValidatorReturn(pub bool);
    ///Container type for all return fields from the `killed` function with signature `killed()` and selector `0x1f3a0e41`
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
    pub struct KilledReturn(pub bool);
    ///Container type for all return fields from the `minActivationCollateral` function with signature `minActivationCollateral()` and selector `0x9e33bd02`
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
    pub struct MinActivationCollateralReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `minValidators` function with signature `minValidators()` and selector `0xc5ab2241`
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
    pub struct MinValidatorsReturn(pub u64);
    ///Container type for all return fields from the `name` function with signature `name()` and selector `0x06fdde03`
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
    pub struct NameReturn(pub [u8; 32]);
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
    ///`Validator(uint256,address,bytes)`
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
        pub addr: ::ethers::core::types::Address,
        pub metadata: ::ethers::core::types::Bytes,
    }
    ///`ValidatorInfo(uint256,uint256,bytes)`
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
    pub struct ValidatorInfo {
        pub confirmed_collateral: ::ethers::core::types::U256,
        pub total_collateral: ::ethers::core::types::U256,
        pub metadata: ::ethers::core::types::Bytes,
    }
}
