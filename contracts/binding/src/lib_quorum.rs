pub use lib_quorum::*;
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
pub mod lib_quorum {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::std::collections::BTreeMap::new(),
            events: ::core::convert::From::from([
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
            errors: ::std::collections::BTreeMap::new(),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static LIBQUORUM_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(
        __abi,
    );
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = b"`\x80\x80`@R4a\0\x1AWa\x04Q\x90\x81a\0 \x8290\x81PP\xF3[`\0\x80\xFD\xFE`\x80\x80`@R`\x046\x10\x15a\0\x13W`\0\x80\xFD[`\x005`\xE0\x1Cc\x08\xA6\xAD%\x14a\0(W`\0\x80\xFD[`@6`\x03\x19\x01\x12a\x03\x80W`\x80\x81a\0B`\0\x93a\x03\x85V[\x82\x81R\x82` \x82\x01R\x82`@\x82\x01R\x82``\x82\x01R\x01R`$5`\0R`\x02`\x045\x01` R`@`\0 `\xFF`\x04`@Q\x92a\0~\x84a\x03\x85V[\x80T\x84R`\x01\x81\x01T` \x85\x01R`\x02\x81\x01T`@\x85\x01R`\x03\x81\x01T``\x85\x01R\x01T\x16\x15\x15`\x80\x82\x01R`$5`\0R`\x05`\x045\x01` R`@`\0 \x90`@Q\x90\x81\x82` \x85T\x92\x83\x81R\x01\x80\x95`\0R` `\0 \x92`\0[\x81\x81\x10a\x03gWPPa\0\xF1\x92P\x03\x83a\x03\xB7V[\x81Q\x90a\0\xFD\x82a\x03\xD9V[\x91a\x01\x0B`@Q\x93\x84a\x03\xB7V[\x80\x83Ra\x01\x17\x81a\x03\xD9V[`\0[`\x1F\x19\x82\x01\x81\x10a\x03VWPP`\0[\x81\x81\x10a\x026WPP`@Q\x92`\x80`\xE0\x85\x01\x92\x80Q\x86R` \x81\x01Q` \x87\x01R`@\x81\x01Q`@\x87\x01R``\x81\x01Q``\x87\x01R\x01Q\x15\x15`\x80\x85\x01R`\xE0`\xA0\x85\x01RQ\x80\x91Ra\x01\0\x83\x01\x93\x90`\0[\x81\x81\x10a\x02\x17WPPP\x81\x83\x03`\xC0\x83\x01R\x80Q\x80\x84R` \x84\x01\x90` \x80\x82`\x05\x1B\x87\x01\x01\x93\x01\x91`\0\x95[\x82\x87\x10a\x01\xB8W\x85\x85\x03\x86\xF3[\x90\x91\x92\x93`\x1F\x19\x82\x82\x03\x01\x83R\x84Q\x80Q\x90\x81\x83R`\0[\x82\x81\x10a\x02\x02WPP` \x80\x83\x83`\0\x83\x80\x96`\x01\x98\x01\x01R`\x1F\x80\x19\x91\x01\x16\x01\x01\x96\x01\x93\x01\x96\x01\x95\x90\x92\x91\x92a\x01\xABV[\x80` \x80\x92\x84\x01\x01Q\x82\x82\x87\x01\x01R\x01a\x01\xD0V[\x82Q`\x01`\x01`\xA0\x1B\x03\x16\x86R` \x95\x86\x01\x95\x90\x92\x01\x91`\x01\x01a\x01~V[`$5`\0\x90\x81R`\x045`\x06\x01` R`@\x90 `\x01`\x01`\xA0\x1B\x03a\x02]\x83\x88a\x03\xF1V[Q\x16`\0R` R`@`\0 `@Q\x90`\0\x90\x80T\x90\x81`\x01\x1C\x91`\x01\x81\x16\x15a\x03LW[` \x83\x10`\x01\x82\x16\x14a\x036W\x82\x85R`\x01\x81\x16\x90\x81\x15a\x03\x0FWP`\x01\x14a\x02\xD5W[PP\x90a\x02\xB9\x81`\x01\x94\x93\x03\x82a\x03\xB7V[a\x02\xC3\x82\x87a\x03\xF1V[Ra\x02\xCE\x81\x86a\x03\xF1V[P\x01a\x01*V[`\0\x90\x81R` \x81 \x90\x92P[\x81\x83\x10a\x02\xF9WPP\x81\x01` \x01a\x02\xB9\x82a\x02\xA7V[`\x01\x81` \x92T\x83\x86\x88\x01\x01R\x01\x92\x01\x91a\x02\xE2V[`\xFF\x19\x16` \x80\x87\x01\x91\x90\x91R\x92\x15\x15`\x05\x1B\x85\x01\x90\x92\x01\x92Pa\x02\xB9\x91P\x83\x90Pa\x02\xA7V[cNH{q`\xE0\x1B`\0R`\"`\x04R`$`\0\xFD[\x91`\x7F\x16\x91a\x02\x83V[\x80``` \x80\x93\x88\x01\x01R\x01a\x01\x1AV[\x84T\x83R`\x01\x94\x85\x01\x94\x87\x94P` \x90\x93\x01\x92\x01a\0\xDCV[`\0\x80\xFD[`\xA0\x81\x01\x90\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x03\xA1W`@RV[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[\x90`\x1F\x80\x19\x91\x01\x16\x81\x01\x90\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x03\xA1W`@RV[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x03\xA1W`\x05\x1B` \x01\x90V[\x80Q\x82\x10\x15a\x04\x05W` \x91`\x05\x1B\x01\x01\x90V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD\xFE\xA2dipfsX\"\x12 \x9E\xD2\xF9\xCE\x15\xA8s\xF1\xF3e\xBB\xC4\xE2o\xC3\xAA\x06\xF0eU\xDF\xF5t\xF2T\x1Avh\x93a\xB5\xE9dsolcC\0\x08\x13\x003";
    /// The bytecode of the contract.
    pub static LIBQUORUM_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __BYTECODE,
    );
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80\x80`@R`\x046\x10\x15a\0\x13W`\0\x80\xFD[`\x005`\xE0\x1Cc\x08\xA6\xAD%\x14a\0(W`\0\x80\xFD[`@6`\x03\x19\x01\x12a\x03\x80W`\x80\x81a\0B`\0\x93a\x03\x85V[\x82\x81R\x82` \x82\x01R\x82`@\x82\x01R\x82``\x82\x01R\x01R`$5`\0R`\x02`\x045\x01` R`@`\0 `\xFF`\x04`@Q\x92a\0~\x84a\x03\x85V[\x80T\x84R`\x01\x81\x01T` \x85\x01R`\x02\x81\x01T`@\x85\x01R`\x03\x81\x01T``\x85\x01R\x01T\x16\x15\x15`\x80\x82\x01R`$5`\0R`\x05`\x045\x01` R`@`\0 \x90`@Q\x90\x81\x82` \x85T\x92\x83\x81R\x01\x80\x95`\0R` `\0 \x92`\0[\x81\x81\x10a\x03gWPPa\0\xF1\x92P\x03\x83a\x03\xB7V[\x81Q\x90a\0\xFD\x82a\x03\xD9V[\x91a\x01\x0B`@Q\x93\x84a\x03\xB7V[\x80\x83Ra\x01\x17\x81a\x03\xD9V[`\0[`\x1F\x19\x82\x01\x81\x10a\x03VWPP`\0[\x81\x81\x10a\x026WPP`@Q\x92`\x80`\xE0\x85\x01\x92\x80Q\x86R` \x81\x01Q` \x87\x01R`@\x81\x01Q`@\x87\x01R``\x81\x01Q``\x87\x01R\x01Q\x15\x15`\x80\x85\x01R`\xE0`\xA0\x85\x01RQ\x80\x91Ra\x01\0\x83\x01\x93\x90`\0[\x81\x81\x10a\x02\x17WPPP\x81\x83\x03`\xC0\x83\x01R\x80Q\x80\x84R` \x84\x01\x90` \x80\x82`\x05\x1B\x87\x01\x01\x93\x01\x91`\0\x95[\x82\x87\x10a\x01\xB8W\x85\x85\x03\x86\xF3[\x90\x91\x92\x93`\x1F\x19\x82\x82\x03\x01\x83R\x84Q\x80Q\x90\x81\x83R`\0[\x82\x81\x10a\x02\x02WPP` \x80\x83\x83`\0\x83\x80\x96`\x01\x98\x01\x01R`\x1F\x80\x19\x91\x01\x16\x01\x01\x96\x01\x93\x01\x96\x01\x95\x90\x92\x91\x92a\x01\xABV[\x80` \x80\x92\x84\x01\x01Q\x82\x82\x87\x01\x01R\x01a\x01\xD0V[\x82Q`\x01`\x01`\xA0\x1B\x03\x16\x86R` \x95\x86\x01\x95\x90\x92\x01\x91`\x01\x01a\x01~V[`$5`\0\x90\x81R`\x045`\x06\x01` R`@\x90 `\x01`\x01`\xA0\x1B\x03a\x02]\x83\x88a\x03\xF1V[Q\x16`\0R` R`@`\0 `@Q\x90`\0\x90\x80T\x90\x81`\x01\x1C\x91`\x01\x81\x16\x15a\x03LW[` \x83\x10`\x01\x82\x16\x14a\x036W\x82\x85R`\x01\x81\x16\x90\x81\x15a\x03\x0FWP`\x01\x14a\x02\xD5W[PP\x90a\x02\xB9\x81`\x01\x94\x93\x03\x82a\x03\xB7V[a\x02\xC3\x82\x87a\x03\xF1V[Ra\x02\xCE\x81\x86a\x03\xF1V[P\x01a\x01*V[`\0\x90\x81R` \x81 \x90\x92P[\x81\x83\x10a\x02\xF9WPP\x81\x01` \x01a\x02\xB9\x82a\x02\xA7V[`\x01\x81` \x92T\x83\x86\x88\x01\x01R\x01\x92\x01\x91a\x02\xE2V[`\xFF\x19\x16` \x80\x87\x01\x91\x90\x91R\x92\x15\x15`\x05\x1B\x85\x01\x90\x92\x01\x92Pa\x02\xB9\x91P\x83\x90Pa\x02\xA7V[cNH{q`\xE0\x1B`\0R`\"`\x04R`$`\0\xFD[\x91`\x7F\x16\x91a\x02\x83V[\x80``` \x80\x93\x88\x01\x01R\x01a\x01\x1AV[\x84T\x83R`\x01\x94\x85\x01\x94\x87\x94P` \x90\x93\x01\x92\x01a\0\xDCV[`\0\x80\xFD[`\xA0\x81\x01\x90\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x03\xA1W`@RV[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[\x90`\x1F\x80\x19\x91\x01\x16\x81\x01\x90\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x03\xA1W`@RV[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x03\xA1W`\x05\x1B` \x01\x90V[\x80Q\x82\x10\x15a\x04\x05W` \x91`\x05\x1B\x01\x01\x90V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD\xFE\xA2dipfsX\"\x12 \x9E\xD2\xF9\xCE\x15\xA8s\xF1\xF3e\xBB\xC4\xE2o\xC3\xAA\x06\xF0eU\xDF\xF5t\xF2T\x1Avh\x93a\xB5\xE9dsolcC\0\x08\x13\x003";
    /// The deployed bytecode of the contract.
    pub static LIBQUORUM_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __DEPLOYED_BYTECODE,
    );
    pub struct LibQuorum<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for LibQuorum<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for LibQuorum<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for LibQuorum<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for LibQuorum<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(LibQuorum)).field(&self.address()).finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> LibQuorum<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    LIBQUORUM_ABI.clone(),
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
                LIBQUORUM_ABI.clone(),
                LIBQUORUM_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Gets the contract's `QuorumReached` event
        pub fn quorum_reached_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            QuorumReachedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `QuorumWeightUpdated` event
        pub fn quorum_weight_updated_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            QuorumWeightUpdatedFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            LibQuorumEvents,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for LibQuorum<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
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
        Hash
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
        Hash
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
    pub enum LibQuorumEvents {
        QuorumReachedFilter(QuorumReachedFilter),
        QuorumWeightUpdatedFilter(QuorumWeightUpdatedFilter),
    }
    impl ::ethers::contract::EthLogDecode for LibQuorumEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = QuorumReachedFilter::decode_log(log) {
                return Ok(LibQuorumEvents::QuorumReachedFilter(decoded));
            }
            if let Ok(decoded) = QuorumWeightUpdatedFilter::decode_log(log) {
                return Ok(LibQuorumEvents::QuorumWeightUpdatedFilter(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for LibQuorumEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::QuorumReachedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::QuorumWeightUpdatedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<QuorumReachedFilter> for LibQuorumEvents {
        fn from(value: QuorumReachedFilter) -> Self {
            Self::QuorumReachedFilter(value)
        }
    }
    impl ::core::convert::From<QuorumWeightUpdatedFilter> for LibQuorumEvents {
        fn from(value: QuorumWeightUpdatedFilter) -> Self {
            Self::QuorumWeightUpdatedFilter(value)
        }
    }
}
