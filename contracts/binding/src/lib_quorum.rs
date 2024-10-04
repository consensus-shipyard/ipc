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
    const __BYTECODE: &[u8] = b"`\x80\x80`@R4`\x19Wa\x04D\x90\x81a\0\x1F\x8290\x81PP\xF3[`\0\x80\xFD\xFE`\x80\x80`@R`\x046\x10\x15a\0\x13W`\0\x80\xFD[`\x005`\xE0\x1Cc\x08\xA6\xAD%\x14a\0(W`\0\x80\xFD[`@6`\x03\x19\x01\x12a\x03sW`\x045`\0`\x80`$5\x93a\0H\x81a\x03xV[\x82\x81R\x82` \x82\x01R\x82`@\x82\x01R\x82``\x82\x01R\x01R\x81`\0R`\x02\x81\x01` R`@`\0 \x91`@Qa\0|\x81a\x03xV[\x83T\x81R`\x01\x84\x01T\x92` \x82\x01\x93\x84R`\x02\x85\x01T\x94`@\x83\x01\x95\x86R`\xFF`\x04`\x03\x83\x01T\x92``\x86\x01\x93\x84R\x01T\x16\x90`\x80\x84\x01\x91\x15\x15\x82R\x84`\0R`\x05\x83\x01` R`@`\0 \x96`@Q\x93\x84\x85` \x8BT\x92\x83\x81R\x01\x80\x9B`\0R` `\0 \x92`\0[\x81\x81\x10a\x03ZWPPa\0\xFB\x92P\x03\x86a\x03\xAAV[\x84Q\x90a\x01\x07\x82a\x03\xCCV[\x97a\x01\x15`@Q\x99\x8Aa\x03\xAAV[\x82\x89R`\x1F\x19a\x01$\x84a\x03\xCCV[\x01`\0[\x8A\x82\x82\x10a\x03JWPPP`\x06`\0\x92\x01\x91[\x83\x81\x10a\x029WPPPP`@Q\x96`\xE0\x88\x01\x95Q\x88RQ` \x88\x01RQ`@\x87\x01RQ``\x86\x01RQ\x15\x15`\x80\x85\x01R`\xE0`\xA0\x85\x01RQ\x80\x91Ra\x01\0\x83\x01\x93\x90`\0[\x81\x81\x10a\x02\x1AWPPP\x81\x83\x03`\xC0\x83\x01R\x80Q\x80\x84R` \x84\x01\x90` \x80\x82`\x05\x1B\x87\x01\x01\x93\x01\x91`\0\x95[\x82\x87\x10a\x01\xBBW\x85\x85\x03\x86\xF3[\x90\x91\x92\x93`\x1F\x19\x82\x82\x03\x01\x83R\x84Q\x80Q\x90\x81\x83R`\0[\x82\x81\x10a\x02\x05WPP` \x80\x83\x83`\0\x83\x80\x96`\x01\x98\x01\x01R`\x1F\x80\x19\x91\x01\x16\x01\x01\x96\x01\x93\x01\x96\x01\x95\x90\x92\x91\x92a\x01\xAEV[\x80` \x80\x92\x84\x01\x01Q\x82\x82\x87\x01\x01R\x01a\x01\xD3V[\x82Q`\x01`\x01`\xA0\x1B\x03\x16\x86R` \x95\x86\x01\x95\x90\x92\x01\x91`\x01\x01a\x01\x81V[`\0\x82\x81R` \x84\x90R`@\x90 `\x01`\x01`\xA0\x1B\x03a\x02Y\x83\x8Ba\x03\xE4V[Q\x16`\0R` R`@`\0 `@Q\x90`\0\x90\x80T\x90\x81`\x01\x1C\x91`\x01\x81\x16\x80\x15a\x03@W[` \x84\x10\x81\x14a\x03,W\x83\x86R\x90\x81\x15a\x03\x05WP`\x01\x14a\x02\xCBW[PP\x90a\x02\xAF\x81`\x01\x94\x93\x03\x82a\x03\xAAV[a\x02\xB9\x82\x8Da\x03\xE4V[Ra\x02\xC4\x81\x8Ca\x03\xE4V[P\x01a\x01;V[`\0\x90\x81R` \x81 \x90\x92P[\x81\x83\x10a\x02\xEFWPP\x81\x01` \x01a\x02\xAF\x82a\x02\x9DV[`\x01\x81` \x92T\x83\x86\x88\x01\x01R\x01\x92\x01\x91a\x02\xD8V[`\xFF\x19\x16` \x80\x87\x01\x91\x90\x91R\x92\x15\x15`\x05\x1B\x85\x01\x90\x92\x01\x92Pa\x02\xAF\x91P\x83\x90Pa\x02\x9DV[cNH{q`\xE0\x1B\x85R`\"`\x04R`$\x85\xFD[\x92`\x7F\x16\x92a\x02\x80V[``` \x91\x83\x01\x82\x01R\x01a\x01(V[\x84T\x83R`\x01\x94\x85\x01\x94\x8A\x94P` \x90\x93\x01\x92\x01a\0\xE6V[`\0\x80\xFD[`\xA0\x81\x01\x90\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x03\x94W`@RV[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[\x90`\x1F\x80\x19\x91\x01\x16\x81\x01\x90\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x03\x94W`@RV[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x03\x94W`\x05\x1B` \x01\x90V[\x80Q\x82\x10\x15a\x03\xF8W` \x91`\x05\x1B\x01\x01\x90V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD\xFE\xA2dipfsX\"\x12 8k\xB2G\x18\xF6K3\x0F<)\x0F\xAF\xCE\x0C\xEB{\xD0O|\x1C\xC5n\xC5sY\xA1\x05\x9Cn%TdsolcC\0\x08\x1A\x003";
    /// The bytecode of the contract.
    pub static LIBQUORUM_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__BYTECODE);
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80\x80`@R`\x046\x10\x15a\0\x13W`\0\x80\xFD[`\x005`\xE0\x1Cc\x08\xA6\xAD%\x14a\0(W`\0\x80\xFD[`@6`\x03\x19\x01\x12a\x03sW`\x045`\0`\x80`$5\x93a\0H\x81a\x03xV[\x82\x81R\x82` \x82\x01R\x82`@\x82\x01R\x82``\x82\x01R\x01R\x81`\0R`\x02\x81\x01` R`@`\0 \x91`@Qa\0|\x81a\x03xV[\x83T\x81R`\x01\x84\x01T\x92` \x82\x01\x93\x84R`\x02\x85\x01T\x94`@\x83\x01\x95\x86R`\xFF`\x04`\x03\x83\x01T\x92``\x86\x01\x93\x84R\x01T\x16\x90`\x80\x84\x01\x91\x15\x15\x82R\x84`\0R`\x05\x83\x01` R`@`\0 \x96`@Q\x93\x84\x85` \x8BT\x92\x83\x81R\x01\x80\x9B`\0R` `\0 \x92`\0[\x81\x81\x10a\x03ZWPPa\0\xFB\x92P\x03\x86a\x03\xAAV[\x84Q\x90a\x01\x07\x82a\x03\xCCV[\x97a\x01\x15`@Q\x99\x8Aa\x03\xAAV[\x82\x89R`\x1F\x19a\x01$\x84a\x03\xCCV[\x01`\0[\x8A\x82\x82\x10a\x03JWPPP`\x06`\0\x92\x01\x91[\x83\x81\x10a\x029WPPPP`@Q\x96`\xE0\x88\x01\x95Q\x88RQ` \x88\x01RQ`@\x87\x01RQ``\x86\x01RQ\x15\x15`\x80\x85\x01R`\xE0`\xA0\x85\x01RQ\x80\x91Ra\x01\0\x83\x01\x93\x90`\0[\x81\x81\x10a\x02\x1AWPPP\x81\x83\x03`\xC0\x83\x01R\x80Q\x80\x84R` \x84\x01\x90` \x80\x82`\x05\x1B\x87\x01\x01\x93\x01\x91`\0\x95[\x82\x87\x10a\x01\xBBW\x85\x85\x03\x86\xF3[\x90\x91\x92\x93`\x1F\x19\x82\x82\x03\x01\x83R\x84Q\x80Q\x90\x81\x83R`\0[\x82\x81\x10a\x02\x05WPP` \x80\x83\x83`\0\x83\x80\x96`\x01\x98\x01\x01R`\x1F\x80\x19\x91\x01\x16\x01\x01\x96\x01\x93\x01\x96\x01\x95\x90\x92\x91\x92a\x01\xAEV[\x80` \x80\x92\x84\x01\x01Q\x82\x82\x87\x01\x01R\x01a\x01\xD3V[\x82Q`\x01`\x01`\xA0\x1B\x03\x16\x86R` \x95\x86\x01\x95\x90\x92\x01\x91`\x01\x01a\x01\x81V[`\0\x82\x81R` \x84\x90R`@\x90 `\x01`\x01`\xA0\x1B\x03a\x02Y\x83\x8Ba\x03\xE4V[Q\x16`\0R` R`@`\0 `@Q\x90`\0\x90\x80T\x90\x81`\x01\x1C\x91`\x01\x81\x16\x80\x15a\x03@W[` \x84\x10\x81\x14a\x03,W\x83\x86R\x90\x81\x15a\x03\x05WP`\x01\x14a\x02\xCBW[PP\x90a\x02\xAF\x81`\x01\x94\x93\x03\x82a\x03\xAAV[a\x02\xB9\x82\x8Da\x03\xE4V[Ra\x02\xC4\x81\x8Ca\x03\xE4V[P\x01a\x01;V[`\0\x90\x81R` \x81 \x90\x92P[\x81\x83\x10a\x02\xEFWPP\x81\x01` \x01a\x02\xAF\x82a\x02\x9DV[`\x01\x81` \x92T\x83\x86\x88\x01\x01R\x01\x92\x01\x91a\x02\xD8V[`\xFF\x19\x16` \x80\x87\x01\x91\x90\x91R\x92\x15\x15`\x05\x1B\x85\x01\x90\x92\x01\x92Pa\x02\xAF\x91P\x83\x90Pa\x02\x9DV[cNH{q`\xE0\x1B\x85R`\"`\x04R`$\x85\xFD[\x92`\x7F\x16\x92a\x02\x80V[``` \x91\x83\x01\x82\x01R\x01a\x01(V[\x84T\x83R`\x01\x94\x85\x01\x94\x8A\x94P` \x90\x93\x01\x92\x01a\0\xE6V[`\0\x80\xFD[`\xA0\x81\x01\x90\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x03\x94W`@RV[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[\x90`\x1F\x80\x19\x91\x01\x16\x81\x01\x90\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x03\x94W`@RV[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x11a\x03\x94W`\x05\x1B` \x01\x90V[\x80Q\x82\x10\x15a\x03\xF8W` \x91`\x05\x1B\x01\x01\x90V[cNH{q`\xE0\x1B`\0R`2`\x04R`$`\0\xFD\xFE\xA2dipfsX\"\x12 8k\xB2G\x18\xF6K3\x0F<)\x0F\xAF\xCE\x0C\xEB{\xD0O|\x1C\xC5n\xC5sY\xA1\x05\x9Cn%TdsolcC\0\x08\x1A\x003";
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
