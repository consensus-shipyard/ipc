pub use lib_quorum::*;
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
pub mod lib_quorum {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::std::collections::BTreeMap::new(),
            events: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("QuorumReached"),
                    ::std::vec![::ethers::core::abi::ethabi::Event {
                        name: ::std::borrow::ToOwned::to_owned("QuorumReached"),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("objKind"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                indexed: false,
                            },
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("height"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                                indexed: false,
                            },
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("objHash"),
                                kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize,),
                                indexed: false,
                            },
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("quorumWeight"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                                indexed: false,
                            },
                        ],
                        anonymous: false,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("QuorumWeightUpdated"),
                    ::std::vec![::ethers::core::abi::ethabi::Event {
                        name: ::std::borrow::ToOwned::to_owned("QuorumWeightUpdated",),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("objKind"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Uint(8usize),
                                indexed: false,
                            },
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("height"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                                indexed: false,
                            },
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("objHash"),
                                kind: ::ethers::core::abi::ethabi::ParamType::FixedBytes(32usize,),
                                indexed: false,
                            },
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("newWeight"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                                indexed: false,
                            },
                        ],
                        anonymous: false,
                    },],
                ),
            ]),
            errors: ::std::collections::BTreeMap::new(),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static LIBQUORUM_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(__abi);
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = b"a\x04\xACa\09`\x0B\x82\x82\x829\x80Q`\0\x1A`s\x14`,WcNH{q`\xE0\x1B`\0R`\0`\x04R`$`\0\xFD[0`\0R`s\x81S\x82\x81\xF3\xFEs\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x000\x14`\x80`@R`\x046\x10a\x005W`\x005`\xE0\x1C\x80c\x08\xA6\xAD%\x14a\0:W[`\0\x80\xFD[a\0Ma\0H6`\x04a\x02\xC4V[a\0eV[`@Qa\0\\\x93\x92\x91\x90a\x03yV[`@Q\x80\x91\x03\x90\xF3[`@\x80Q`\xA0\x80\x82\x01\x83R`\0\x80\x83R` \x80\x84\x01\x82\x90R\x83\x85\x01\x82\x90R``\x80\x85\x01\x83\x90R`\x80\x94\x85\x01\x83\x90R\x86\x83R`\x02\x88\x81\x01\x83R\x86\x84 \x87Q\x95\x86\x01\x88R\x80T\x86R`\x01\x81\x01T\x86\x85\x01R\x90\x81\x01T\x85\x88\x01R`\x03\x81\x01T\x85\x83\x01R`\x04\x01T`\xFF\x16\x15\x15\x94\x84\x01\x94\x90\x94R\x85\x82R`\x05\x87\x01\x90R\x92\x90\x92 \x81\x90a\0\xED\x90a\x02TV[\x80Q\x90\x92P\x80g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x01\x0BWa\x01\x0Ba\x04\x10V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x01>W\x81` \x01[``\x81R` \x01\x90`\x01\x90\x03\x90\x81a\x01)W\x90P[P\x91P`\0[\x81\x81\x10\x15a\x02KW\x86`\x06\x01`\0\x87\x81R` \x01\x90\x81R` \x01`\0 `\0\x85\x83\x81Q\x81\x10a\x01uWa\x01ua\x04&V[` \x02` \x01\x01Q`\x01`\x01`\xA0\x1B\x03\x16`\x01`\x01`\xA0\x1B\x03\x16\x81R` \x01\x90\x81R` \x01`\0 \x80Ta\x01\xA8\x90a\x04<V[\x80`\x1F\x01` \x80\x91\x04\x02` \x01`@Q\x90\x81\x01`@R\x80\x92\x91\x90\x81\x81R` \x01\x82\x80Ta\x01\xD4\x90a\x04<V[\x80\x15a\x02!W\x80`\x1F\x10a\x01\xF6Wa\x01\0\x80\x83T\x04\x02\x83R\x91` \x01\x91a\x02!V[\x82\x01\x91\x90`\0R` `\0 \x90[\x81T\x81R\x90`\x01\x01\x90` \x01\x80\x83\x11a\x02\x04W\x82\x90\x03`\x1F\x16\x82\x01\x91[PPPPP\x83\x82\x81Q\x81\x10a\x028Wa\x028a\x04&V[` \x90\x81\x02\x91\x90\x91\x01\x01R`\x01\x01a\x01DV[PP\x92P\x92P\x92V[```\0a\x02a\x83a\x02hV[\x93\x92PPPV[``\x81`\0\x01\x80T\x80` \x02` \x01`@Q\x90\x81\x01`@R\x80\x92\x91\x90\x81\x81R` \x01\x82\x80T\x80\x15a\x02\xB8W` \x02\x82\x01\x91\x90`\0R` `\0 \x90[\x81T\x81R` \x01\x90`\x01\x01\x90\x80\x83\x11a\x02\xA4W[PPPPP\x90P\x91\x90PV[`\0\x80`@\x83\x85\x03\x12\x15a\x02\xD7W`\0\x80\xFD[PP\x805\x92` \x90\x91\x015\x91PV[`\0\x82\x82Q\x80\x85R` \x85\x01\x94P` \x81`\x05\x1B\x83\x01\x01` \x85\x01`\0[\x83\x81\x10\x15a\x03mW\x84\x83\x03`\x1F\x19\x01\x88R\x81Q\x80Q\x80\x85R`\0[\x81\x81\x10\x15a\x03;W` \x81\x84\x01\x81\x01Q\x87\x83\x01\x82\x01R\x01a\x03\x1FV[P`\0` \x82\x87\x01\x01R` `\x1F\x19`\x1F\x83\x01\x16\x86\x01\x01\x94PPP` \x82\x01\x91P` \x88\x01\x97P`\x01\x81\x01\x90Pa\x03\x04V[P\x90\x96\x95PPPPPPV[`\0`\xE0\x82\x01\x85Q\x83R` \x86\x01Q` \x84\x01R`@\x86\x01Q`@\x84\x01R``\x86\x01Q``\x84\x01R`\x80\x86\x01Q\x15\x15`\x80\x84\x01R`\xE0`\xA0\x84\x01R\x80\x85Q\x80\x83Ra\x01\0\x85\x01\x91P` \x87\x01\x92P`\0[\x81\x81\x10\x15a\x03\xF1W\x83Q`\x01`\x01`\xA0\x1B\x03\x16\x83R` \x93\x84\x01\x93\x90\x92\x01\x91`\x01\x01a\x03\xCAV[PP\x83\x81\x03`\xC0\x85\x01Ra\x04\x05\x81\x86a\x02\xE6V[\x97\x96PPPPPPPV[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[`\x01\x81\x81\x1C\x90\x82\x16\x80a\x04PW`\x7F\x82\x16\x91P[` \x82\x10\x81\x03a\x04pWcNH{q`\xE0\x1B`\0R`\"`\x04R`$`\0\xFD[P\x91\x90PV\xFE\xA2dipfsX\"\x12 ,\r \x88\x92B\xA5:F\xE3\x150\xD0Vkxb\xE3`\x12IJ\xD2G\xD5{\xCB\xCDhe\xABZdsolcC\0\x08\x1A\x003";
    /// The bytecode of the contract.
    pub static LIBQUORUM_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__BYTECODE);
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"s\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x000\x14`\x80`@R`\x046\x10a\x005W`\x005`\xE0\x1C\x80c\x08\xA6\xAD%\x14a\0:W[`\0\x80\xFD[a\0Ma\0H6`\x04a\x02\xC4V[a\0eV[`@Qa\0\\\x93\x92\x91\x90a\x03yV[`@Q\x80\x91\x03\x90\xF3[`@\x80Q`\xA0\x80\x82\x01\x83R`\0\x80\x83R` \x80\x84\x01\x82\x90R\x83\x85\x01\x82\x90R``\x80\x85\x01\x83\x90R`\x80\x94\x85\x01\x83\x90R\x86\x83R`\x02\x88\x81\x01\x83R\x86\x84 \x87Q\x95\x86\x01\x88R\x80T\x86R`\x01\x81\x01T\x86\x85\x01R\x90\x81\x01T\x85\x88\x01R`\x03\x81\x01T\x85\x83\x01R`\x04\x01T`\xFF\x16\x15\x15\x94\x84\x01\x94\x90\x94R\x85\x82R`\x05\x87\x01\x90R\x92\x90\x92 \x81\x90a\0\xED\x90a\x02TV[\x80Q\x90\x92P\x80g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11\x15a\x01\x0BWa\x01\x0Ba\x04\x10V[`@Q\x90\x80\x82R\x80` \x02` \x01\x82\x01`@R\x80\x15a\x01>W\x81` \x01[``\x81R` \x01\x90`\x01\x90\x03\x90\x81a\x01)W\x90P[P\x91P`\0[\x81\x81\x10\x15a\x02KW\x86`\x06\x01`\0\x87\x81R` \x01\x90\x81R` \x01`\0 `\0\x85\x83\x81Q\x81\x10a\x01uWa\x01ua\x04&V[` \x02` \x01\x01Q`\x01`\x01`\xA0\x1B\x03\x16`\x01`\x01`\xA0\x1B\x03\x16\x81R` \x01\x90\x81R` \x01`\0 \x80Ta\x01\xA8\x90a\x04<V[\x80`\x1F\x01` \x80\x91\x04\x02` \x01`@Q\x90\x81\x01`@R\x80\x92\x91\x90\x81\x81R` \x01\x82\x80Ta\x01\xD4\x90a\x04<V[\x80\x15a\x02!W\x80`\x1F\x10a\x01\xF6Wa\x01\0\x80\x83T\x04\x02\x83R\x91` \x01\x91a\x02!V[\x82\x01\x91\x90`\0R` `\0 \x90[\x81T\x81R\x90`\x01\x01\x90` \x01\x80\x83\x11a\x02\x04W\x82\x90\x03`\x1F\x16\x82\x01\x91[PPPPP\x83\x82\x81Q\x81\x10a\x028Wa\x028a\x04&V[` \x90\x81\x02\x91\x90\x91\x01\x01R`\x01\x01a\x01DV[PP\x92P\x92P\x92V[```\0a\x02a\x83a\x02hV[\x93\x92PPPV[``\x81`\0\x01\x80T\x80` \x02` \x01`@Q\x90\x81\x01`@R\x80\x92\x91\x90\x81\x81R` \x01\x82\x80T\x80\x15a\x02\xB8W` \x02\x82\x01\x91\x90`\0R` `\0 \x90[\x81T\x81R` \x01\x90`\x01\x01\x90\x80\x83\x11a\x02\xA4W[PPPPP\x90P\x91\x90PV[`\0\x80`@\x83\x85\x03\x12\x15a\x02\xD7W`\0\x80\xFD[PP\x805\x92` \x90\x91\x015\x91PV[`\0\x82\x82Q\x80\x85R` \x85\x01\x94P` \x81`\x05\x1B\x83\x01\x01` \x85\x01`\0[\x83\x81\x10\x15a\x03mW\x84\x83\x03`\x1F\x19\x01\x88R\x81Q\x80Q\x80\x85R`\0[\x81\x81\x10\x15a\x03;W` \x81\x84\x01\x81\x01Q\x87\x83\x01\x82\x01R\x01a\x03\x1FV[P`\0` \x82\x87\x01\x01R` `\x1F\x19`\x1F\x83\x01\x16\x86\x01\x01\x94PPP` \x82\x01\x91P` \x88\x01\x97P`\x01\x81\x01\x90Pa\x03\x04V[P\x90\x96\x95PPPPPPV[`\0`\xE0\x82\x01\x85Q\x83R` \x86\x01Q` \x84\x01R`@\x86\x01Q`@\x84\x01R``\x86\x01Q``\x84\x01R`\x80\x86\x01Q\x15\x15`\x80\x84\x01R`\xE0`\xA0\x84\x01R\x80\x85Q\x80\x83Ra\x01\0\x85\x01\x91P` \x87\x01\x92P`\0[\x81\x81\x10\x15a\x03\xF1W\x83Q`\x01`\x01`\xA0\x1B\x03\x16\x83R` \x93\x84\x01\x93\x90\x92\x01\x91`\x01\x01a\x03\xCAV[PP\x83\x81\x03`\xC0\x85\x01Ra\x04\x05\x81\x86a\x02\xE6V[\x97\x96PPPPPPPV[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD[`\x01\x81\x81\x1C\x90\x82\x16\x80a\x04PW`\x7F\x82\x16\x91P[` \x82\x10\x81\x03a\x04pWcNH{q`\xE0\x1B`\0R`\"`\x04R`$`\0\xFD[P\x91\x90PV\xFE\xA2dipfsX\"\x12 ,\r \x88\x92B\xA5:F\xE3\x150\xD0Vkxb\xE3`\x12IJ\xD2G\xD5{\xCB\xCDhe\xABZdsolcC\0\x08\x1A\x003";
    /// The deployed bytecode of the contract.
    pub static LIBQUORUM_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__DEPLOYED_BYTECODE);
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
            f.debug_tuple(::core::stringify!(LibQuorum))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> LibQuorum<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                LIBQUORUM_ABI.clone(),
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
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, LibQuorumEvents> {
            self.0
                .event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>> for LibQuorum<M> {
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
                Self::QuorumReachedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::QuorumWeightUpdatedFilter(element) => ::core::fmt::Display::fmt(element, f),
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
