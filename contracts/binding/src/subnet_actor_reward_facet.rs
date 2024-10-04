pub use subnet_actor_reward_facet::*;
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
pub mod subnet_actor_reward_facet {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::core::convert::From::from([(
                ::std::borrow::ToOwned::to_owned("claim"),
                ::std::vec![::ethers::core::abi::ethabi::Function {
                    name: ::std::borrow::ToOwned::to_owned("claim"),
                    inputs: ::std::vec![],
                    outputs: ::std::vec![],
                    constant: ::core::option::Option::None,
                    state_mutability: ::ethers::core::abi::ethabi::StateMutability::NonPayable,
                },],
            )]),
            events: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("CollateralClaimed"),
                    ::std::vec![::ethers::core::abi::ethabi::Event {
                        name: ::std::borrow::ToOwned::to_owned("CollateralClaimed"),
                        inputs: ::std::vec![
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("validator"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                indexed: false,
                            },
                            ::ethers::core::abi::ethabi::EventParam {
                                name: ::std::borrow::ToOwned::to_owned("amount"),
                                kind: ::ethers::core::abi::ethabi::ParamType::Uint(256usize,),
                                indexed: false,
                            },
                        ],
                        anonymous: false,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("Paused"),
                    ::std::vec![::ethers::core::abi::ethabi::Event {
                        name: ::std::borrow::ToOwned::to_owned("Paused"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::EventParam {
                            name: ::std::borrow::ToOwned::to_owned("account"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Address,
                            indexed: false,
                        },],
                        anonymous: false,
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("Unpaused"),
                    ::std::vec![::ethers::core::abi::ethabi::Event {
                        name: ::std::borrow::ToOwned::to_owned("Unpaused"),
                        inputs: ::std::vec![::ethers::core::abi::ethabi::EventParam {
                            name: ::std::borrow::ToOwned::to_owned("account"),
                            kind: ::ethers::core::abi::ethabi::ParamType::Address,
                            indexed: false,
                        },],
                        anonymous: false,
                    },],
                ),
            ]),
            errors: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("EnforcedPause"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("EnforcedPause"),
                        inputs: ::std::vec![],
                    },],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("ExpectedPause"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("ExpectedPause"),
                        inputs: ::std::vec![],
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
                    ::std::borrow::ToOwned::to_owned("NotEnoughBalance"),
                    ::std::vec![::ethers::core::abi::ethabi::AbiError {
                        name: ::std::borrow::ToOwned::to_owned("NotEnoughBalance"),
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
            ]),
            receive: false,
            fallback: false,
        }
    }
    ///The parsed JSON ABI of the contract.
    pub static SUBNETACTORREWARDFACET_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> =
        ::ethers::contract::Lazy::new(__abi);
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = b"`\x80\x80`@R4`\x15Wa\x04\xA6\x90\x81a\0\x1B\x829\xF3[`\0\x80\xFD\xFE`\x80`@R`\x046\x10\x15a\0\x12W`\0\x80\xFD[`\x005`\xE0\x1CcNq\xD9-\x14a\0'W`\0\x80\xFD[4a\x02\xA0W`\x006`\x03\x19\x01\x12a\x02\xA0W`\x01\x7Fi\x1B\xB0?\xFC\x16\xC5o\xC9k\x82\xFD\x16\xCD\x1B7\x15\xF0\xBC<\xDCd\x07\0_I\xBBb\x05\x86\0\x95T\x14a\x02\x8FW`\x01\x7Fi\x1B\xB0?\xFC\x16\xC5o\xC9k\x82\xFD\x16\xCD\x1B7\x15\xF0\xBC<\xDCd\x07\0_I\xBBb\x05\x86\0\x95U`\xFF\x7F\xC4Q\xC9B\x9C'\xDBh\xF2\x86\xAB\x8Ah\xF3\x11\xF1\xDC\xCA\xB7\x03\xBA\x94#\xAE\xD2\x9C\xD3\x97\xAEc\xF8cT\x16a\x02~W3`\0\x90\x81R`\x17` R`@\x90 \x80Ta\xFF\xFF\x81\x16\x90\x81\x15a\x02mW\x90\x91`\0\x91`\x10\x1Ca\xFF\xFF\x16\x83`\x01\x83\x01[\x81a\xFF\xFF\x84\x16\x10a\x01\xE2W[PPc\xFF\xFF\0\0\x92\x93a\xFF\xFF\x83T\x91\x16\x93\x84\x92`\x10\x1B\x16\x90c\xFF\xFF\xFF\xFF\x19\x16\x17\x17\x90U\x15a\x01\xCDW[`@\x80Q3\x81R` \x81\x01\x83\x90R\x7F\x19|XcS\xEA\xED\n\x1CS\xE6\xE5@D[\x94\xBE\xFA\xB8\xF92\xC8\x11]\x11!\x15\xEC\xBE\xEE\xD5\x14\x91\x90\xA1\x80a\x01pW[`\0\x7Fi\x1B\xB0?\xFC\x16\xC5o\xC9k\x82\xFD\x16\xCD\x1B7\x15\xF0\xBC<\xDCd\x07\0_I\xBBb\x05\x86\0\x95U\0[`@Q\x90a\x01}\x82a\x02\xA5V[`\x08T\x91`\xFF\x83\x16\x92`\x02\x84\x10\x15a\x01\xB7W\x92\x81R`\x08\x92\x90\x92\x1C`\x01`\x01`\xA0\x1B\x03\x16` \x83\x01Ra\x01\xB1\x913\x90a\x02\xF9V[Pa\x01JV[cNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[3`\0\x90\x81R`\x17` R`@\x81 Ua\x01\x12V[\x90\x91\x93` `@`\0a\xFF\xFF\x88\x16\x81R\x84\x83R `@Qa\x02\x02\x81a\x02\xA5V[`\x01\x82T\x92\x83\x83R\x01T\x92\x83\x91\x01RC\x10a\x02eW\x81\x01\x80\x91\x11a\x02OWa\xFF\xFF`\x01\x81\x92\x96`\0\x82`@\x82\x86\x85\x16\x81R\x88` R \x82\x81U\x01U\x01\x16\x95`\0\x19\x01\x16\x91\x90\x94\x91\x94a\0\xDDV[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[P\x93\x91a\0\xE9V[cd\xB0U\x7F`\xE0\x1B`\0R`\x04`\0\xFD[c\xD9<\x06e`\xE0\x1B`\0R`\x04`\0\xFD[c)\xF7E\xA7`\xE0\x1B`\0R`\x04`\0\xFD[`\0\x80\xFD[`@\x81\x01\x90\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x02\xC1W`@RV[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[\x90`\x1F\x80\x19\x91\x01\x16\x81\x01\x90\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x02\xC1W`@RV[\x92\x91\x90`\0``\x85Q\x95`\x02\x87\x10\x15a\x01\xB7W`\0\x96a\x03]WPPP\x81G\x10a\x03NW\x83\x91\x82\x91\x82\x91\x82\x91`\x01`\x01`\xA0\x1B\x03\x16Z\xF1a\x038a\x040V[P`@Q\x92a\x03H` \x85a\x02\xD7V[\x83R\x91\x90V[cV\x9DE\xCF`\xE1\x1B\x84R`\x04\x84\xFD[\x80\x96\x92\x96\x95\x94\x93\x91\x95Q`\x02\x81\x10\x15a\x04\x1CW`\x01\x14a\x03}WPPPPV[` \x90\x81\x01Q`@\x80Q`\x01`\x01`\xA0\x1B\x03\x94\x85\x16\x93\x81\x01\x93\x90\x93R\x82\x81\x01\x95\x90\x95R\x93\x81R\x95P\x92\x93P\x91\x16a\x03\xB5``\x85a\x02\xD7V[`@Q` \x81\x01\x90c\xA9\x05\x9C\xBB`\xE0\x1B\x82R\x85Q\x84[\x81\x81\x10a\x04\x06WP\x84\x95\x96P\x90a\x03\xF6`$\x82\x87\x94\x81\x01\x85\x83\x82\x01R\x03\x01`\x1F\x19\x81\x01\x83R\x82a\x02\xD7V[Q\x92Z\xF1\x90a\x04\x03a\x040V[\x90V[\x80` \x80\x92\x8A\x01\x01Q`$\x82\x86\x01\x01R\x01a\x03\xCBV[cNH{q`\xE0\x1B\x84R`!`\x04R`$\x84\xFD[=\x15a\x04kW=\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11a\x02\xC1W`@Q\x91a\x04_`\x1F\x82\x01`\x1F\x19\x16` \x01\x84a\x02\xD7V[\x82R=`\0` \x84\x01>V[``\x90V\xFE\xA2dipfsX\"\x12 \xF1\xB7o\x86\x04\xD1M\x17\xD9t\x98'\xCD\xE8\xF7\x08\x87\x01&\x1C-H\xA3\x14\x8B\xD1d\xE6\xFE\xF6\x10\xE5dsolcC\0\x08\x1A\x003";
    /// The bytecode of the contract.
    pub static SUBNETACTORREWARDFACET_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__BYTECODE);
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`\x80`@R`\x046\x10\x15a\0\x12W`\0\x80\xFD[`\x005`\xE0\x1CcNq\xD9-\x14a\0'W`\0\x80\xFD[4a\x02\xA0W`\x006`\x03\x19\x01\x12a\x02\xA0W`\x01\x7Fi\x1B\xB0?\xFC\x16\xC5o\xC9k\x82\xFD\x16\xCD\x1B7\x15\xF0\xBC<\xDCd\x07\0_I\xBBb\x05\x86\0\x95T\x14a\x02\x8FW`\x01\x7Fi\x1B\xB0?\xFC\x16\xC5o\xC9k\x82\xFD\x16\xCD\x1B7\x15\xF0\xBC<\xDCd\x07\0_I\xBBb\x05\x86\0\x95U`\xFF\x7F\xC4Q\xC9B\x9C'\xDBh\xF2\x86\xAB\x8Ah\xF3\x11\xF1\xDC\xCA\xB7\x03\xBA\x94#\xAE\xD2\x9C\xD3\x97\xAEc\xF8cT\x16a\x02~W3`\0\x90\x81R`\x17` R`@\x90 \x80Ta\xFF\xFF\x81\x16\x90\x81\x15a\x02mW\x90\x91`\0\x91`\x10\x1Ca\xFF\xFF\x16\x83`\x01\x83\x01[\x81a\xFF\xFF\x84\x16\x10a\x01\xE2W[PPc\xFF\xFF\0\0\x92\x93a\xFF\xFF\x83T\x91\x16\x93\x84\x92`\x10\x1B\x16\x90c\xFF\xFF\xFF\xFF\x19\x16\x17\x17\x90U\x15a\x01\xCDW[`@\x80Q3\x81R` \x81\x01\x83\x90R\x7F\x19|XcS\xEA\xED\n\x1CS\xE6\xE5@D[\x94\xBE\xFA\xB8\xF92\xC8\x11]\x11!\x15\xEC\xBE\xEE\xD5\x14\x91\x90\xA1\x80a\x01pW[`\0\x7Fi\x1B\xB0?\xFC\x16\xC5o\xC9k\x82\xFD\x16\xCD\x1B7\x15\xF0\xBC<\xDCd\x07\0_I\xBBb\x05\x86\0\x95U\0[`@Q\x90a\x01}\x82a\x02\xA5V[`\x08T\x91`\xFF\x83\x16\x92`\x02\x84\x10\x15a\x01\xB7W\x92\x81R`\x08\x92\x90\x92\x1C`\x01`\x01`\xA0\x1B\x03\x16` \x83\x01Ra\x01\xB1\x913\x90a\x02\xF9V[Pa\x01JV[cNH{q`\xE0\x1B`\0R`!`\x04R`$`\0\xFD[3`\0\x90\x81R`\x17` R`@\x81 Ua\x01\x12V[\x90\x91\x93` `@`\0a\xFF\xFF\x88\x16\x81R\x84\x83R `@Qa\x02\x02\x81a\x02\xA5V[`\x01\x82T\x92\x83\x83R\x01T\x92\x83\x91\x01RC\x10a\x02eW\x81\x01\x80\x91\x11a\x02OWa\xFF\xFF`\x01\x81\x92\x96`\0\x82`@\x82\x86\x85\x16\x81R\x88` R \x82\x81U\x01U\x01\x16\x95`\0\x19\x01\x16\x91\x90\x94\x91\x94a\0\xDDV[cNH{q`\xE0\x1B`\0R`\x11`\x04R`$`\0\xFD[P\x93\x91a\0\xE9V[cd\xB0U\x7F`\xE0\x1B`\0R`\x04`\0\xFD[c\xD9<\x06e`\xE0\x1B`\0R`\x04`\0\xFD[c)\xF7E\xA7`\xE0\x1B`\0R`\x04`\0\xFD[`\0\x80\xFD[`@\x81\x01\x90\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x02\xC1W`@RV[cNH{q`\xE0\x1B`\0R`A`\x04R`$`\0\xFD[\x90`\x1F\x80\x19\x91\x01\x16\x81\x01\x90\x81\x10g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11\x17a\x02\xC1W`@RV[\x92\x91\x90`\0``\x85Q\x95`\x02\x87\x10\x15a\x01\xB7W`\0\x96a\x03]WPPP\x81G\x10a\x03NW\x83\x91\x82\x91\x82\x91\x82\x91`\x01`\x01`\xA0\x1B\x03\x16Z\xF1a\x038a\x040V[P`@Q\x92a\x03H` \x85a\x02\xD7V[\x83R\x91\x90V[cV\x9DE\xCF`\xE1\x1B\x84R`\x04\x84\xFD[\x80\x96\x92\x96\x95\x94\x93\x91\x95Q`\x02\x81\x10\x15a\x04\x1CW`\x01\x14a\x03}WPPPPV[` \x90\x81\x01Q`@\x80Q`\x01`\x01`\xA0\x1B\x03\x94\x85\x16\x93\x81\x01\x93\x90\x93R\x82\x81\x01\x95\x90\x95R\x93\x81R\x95P\x92\x93P\x91\x16a\x03\xB5``\x85a\x02\xD7V[`@Q` \x81\x01\x90c\xA9\x05\x9C\xBB`\xE0\x1B\x82R\x85Q\x84[\x81\x81\x10a\x04\x06WP\x84\x95\x96P\x90a\x03\xF6`$\x82\x87\x94\x81\x01\x85\x83\x82\x01R\x03\x01`\x1F\x19\x81\x01\x83R\x82a\x02\xD7V[Q\x92Z\xF1\x90a\x04\x03a\x040V[\x90V[\x80` \x80\x92\x8A\x01\x01Q`$\x82\x86\x01\x01R\x01a\x03\xCBV[cNH{q`\xE0\x1B\x84R`!`\x04R`$\x84\xFD[=\x15a\x04kW=\x90g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x82\x11a\x02\xC1W`@Q\x91a\x04_`\x1F\x82\x01`\x1F\x19\x16` \x01\x84a\x02\xD7V[\x82R=`\0` \x84\x01>V[``\x90V\xFE\xA2dipfsX\"\x12 \xF1\xB7o\x86\x04\xD1M\x17\xD9t\x98'\xCD\xE8\xF7\x08\x87\x01&\x1C-H\xA3\x14\x8B\xD1d\xE6\xFE\xF6\x10\xE5dsolcC\0\x08\x1A\x003";
    /// The deployed bytecode of the contract.
    pub static SUBNETACTORREWARDFACET_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes =
        ::ethers::core::types::Bytes::from_static(__DEPLOYED_BYTECODE);
    pub struct SubnetActorRewardFacet<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for SubnetActorRewardFacet<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for SubnetActorRewardFacet<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for SubnetActorRewardFacet<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for SubnetActorRewardFacet<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(SubnetActorRewardFacet))
                .field(&self.address())
                .finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> SubnetActorRewardFacet<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(::ethers::contract::Contract::new(
                address.into(),
                SUBNETACTORREWARDFACET_ABI.clone(),
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
                SUBNETACTORREWARDFACET_ABI.clone(),
                SUBNETACTORREWARDFACET_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Calls the contract's `claim` (0x4e71d92d) function
        pub fn claim(&self) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([78, 113, 217, 45], ())
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `CollateralClaimed` event
        pub fn collateral_claimed_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, CollateralClaimedFilter>
        {
            self.0.event()
        }
        ///Gets the contract's `Paused` event
        pub fn paused_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, PausedFilter> {
            self.0.event()
        }
        ///Gets the contract's `Unpaused` event
        pub fn unpaused_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, UnpausedFilter> {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<::std::sync::Arc<M>, M, SubnetActorRewardFacetEvents>
        {
            self.0
                .event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
        for SubnetActorRewardFacet<M>
    {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `EnforcedPause` with signature `EnforcedPause()` and selector `0xd93c0665`
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
    #[etherror(name = "EnforcedPause", abi = "EnforcedPause()")]
    pub struct EnforcedPause;
    ///Custom Error type `ExpectedPause` with signature `ExpectedPause()` and selector `0x8dfc202b`
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
    #[etherror(name = "ExpectedPause", abi = "ExpectedPause()")]
    pub struct ExpectedPause;
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
    ///Custom Error type `NotEnoughBalance` with signature `NotEnoughBalance()` and selector `0xad3a8b9e`
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
    #[etherror(name = "NotEnoughBalance", abi = "NotEnoughBalance()")]
    pub struct NotEnoughBalance;
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
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum SubnetActorRewardFacetErrors {
        EnforcedPause(EnforcedPause),
        ExpectedPause(ExpectedPause),
        NoCollateralToWithdraw(NoCollateralToWithdraw),
        NotEnoughBalance(NotEnoughBalance),
        ReentrancyError(ReentrancyError),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for SubnetActorRewardFacetErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) =
                <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded) = <EnforcedPause as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EnforcedPause(decoded));
            }
            if let Ok(decoded) = <ExpectedPause as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ExpectedPause(decoded));
            }
            if let Ok(decoded) =
                <NoCollateralToWithdraw as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NoCollateralToWithdraw(decoded));
            }
            if let Ok(decoded) = <NotEnoughBalance as ::ethers::core::abi::AbiDecode>::decode(data)
            {
                return Ok(Self::NotEnoughBalance(decoded));
            }
            if let Ok(decoded) = <ReentrancyError as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ReentrancyError(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for SubnetActorRewardFacetErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::EnforcedPause(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ExpectedPause(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::NoCollateralToWithdraw(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotEnoughBalance(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ReentrancyError(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for SubnetActorRewardFacetErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector == <EnforcedPause as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector == <ExpectedPause as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NoCollateralToWithdraw as ::ethers::contract::EthError>::selector() =>
                {
                    true
                }
                _ if selector == <NotEnoughBalance as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector == <ReentrancyError as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for SubnetActorRewardFacetErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::EnforcedPause(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExpectedPause(element) => ::core::fmt::Display::fmt(element, f),
                Self::NoCollateralToWithdraw(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotEnoughBalance(element) => ::core::fmt::Display::fmt(element, f),
                Self::ReentrancyError(element) => ::core::fmt::Display::fmt(element, f),
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for SubnetActorRewardFacetErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<EnforcedPause> for SubnetActorRewardFacetErrors {
        fn from(value: EnforcedPause) -> Self {
            Self::EnforcedPause(value)
        }
    }
    impl ::core::convert::From<ExpectedPause> for SubnetActorRewardFacetErrors {
        fn from(value: ExpectedPause) -> Self {
            Self::ExpectedPause(value)
        }
    }
    impl ::core::convert::From<NoCollateralToWithdraw> for SubnetActorRewardFacetErrors {
        fn from(value: NoCollateralToWithdraw) -> Self {
            Self::NoCollateralToWithdraw(value)
        }
    }
    impl ::core::convert::From<NotEnoughBalance> for SubnetActorRewardFacetErrors {
        fn from(value: NotEnoughBalance) -> Self {
            Self::NotEnoughBalance(value)
        }
    }
    impl ::core::convert::From<ReentrancyError> for SubnetActorRewardFacetErrors {
        fn from(value: ReentrancyError) -> Self {
            Self::ReentrancyError(value)
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
    #[ethevent(name = "CollateralClaimed", abi = "CollateralClaimed(address,uint256)")]
    pub struct CollateralClaimedFilter {
        pub validator: ::ethers::core::types::Address,
        pub amount: ::ethers::core::types::U256,
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
    #[ethevent(name = "Paused", abi = "Paused(address)")]
    pub struct PausedFilter {
        pub account: ::ethers::core::types::Address,
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
    #[ethevent(name = "Unpaused", abi = "Unpaused(address)")]
    pub struct UnpausedFilter {
        pub account: ::ethers::core::types::Address,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum SubnetActorRewardFacetEvents {
        CollateralClaimedFilter(CollateralClaimedFilter),
        PausedFilter(PausedFilter),
        UnpausedFilter(UnpausedFilter),
    }
    impl ::ethers::contract::EthLogDecode for SubnetActorRewardFacetEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = CollateralClaimedFilter::decode_log(log) {
                return Ok(SubnetActorRewardFacetEvents::CollateralClaimedFilter(
                    decoded,
                ));
            }
            if let Ok(decoded) = PausedFilter::decode_log(log) {
                return Ok(SubnetActorRewardFacetEvents::PausedFilter(decoded));
            }
            if let Ok(decoded) = UnpausedFilter::decode_log(log) {
                return Ok(SubnetActorRewardFacetEvents::UnpausedFilter(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for SubnetActorRewardFacetEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::CollateralClaimedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::PausedFilter(element) => ::core::fmt::Display::fmt(element, f),
                Self::UnpausedFilter(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<CollateralClaimedFilter> for SubnetActorRewardFacetEvents {
        fn from(value: CollateralClaimedFilter) -> Self {
            Self::CollateralClaimedFilter(value)
        }
    }
    impl ::core::convert::From<PausedFilter> for SubnetActorRewardFacetEvents {
        fn from(value: PausedFilter) -> Self {
            Self::PausedFilter(value)
        }
    }
    impl ::core::convert::From<UnpausedFilter> for SubnetActorRewardFacetEvents {
        fn from(value: UnpausedFilter) -> Self {
            Self::UnpausedFilter(value)
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
}
