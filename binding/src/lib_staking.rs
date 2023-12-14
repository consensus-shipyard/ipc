pub use lib_staking::*;
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
pub mod lib_staking {
    #[allow(deprecated)]
    fn __abi() -> ::ethers::core::abi::Abi {
        ::ethers::core::abi::ethabi::Contract {
            constructor: ::core::option::Option::None,
            functions: ::std::collections::BTreeMap::new(),
            events: ::core::convert::From::from([
                (
                    ::std::borrow::ToOwned::to_owned("CollateralClaimed"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned("CollateralClaimed"),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("validator"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Address,
                                    indexed: false,
                                },
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("amount"),
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
                    ::std::borrow::ToOwned::to_owned("ConfigurationNumberConfirmed"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::Event {
                            name: ::std::borrow::ToOwned::to_owned(
                                "ConfigurationNumberConfirmed",
                            ),
                            inputs: ::std::vec![
                                ::ethers::core::abi::ethabi::EventParam {
                                    name: ::std::borrow::ToOwned::to_owned("number"),
                                    kind: ::ethers::core::abi::ethabi::ParamType::Uint(64usize),
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
                    ::std::borrow::ToOwned::to_owned("FailedInnerCall"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("FailedInnerCall"),
                            inputs: ::std::vec![],
                        },
                    ],
                ),
                (
                    ::std::borrow::ToOwned::to_owned("NoRewardToWithdraw"),
                    ::std::vec![
                        ::ethers::core::abi::ethabi::AbiError {
                            name: ::std::borrow::ToOwned::to_owned("NoRewardToWithdraw"),
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
    pub static LIBSTAKING_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(
        __abi,
    );
    #[rustfmt::skip]
    const __BYTECODE: &[u8] = b"`\x80\x80`@R4a\0\x1CWa\x02\x14\x90\x81a\0\"\x8290\x81`.\x01R\xF3[`\0\x80\xFD\xFE`@`\x80\x81R`\x04\x806\x10\x15a\0\x14W`\0\x80\xFD[`\0\x91\x825`\xE0\x1Cc\x82\x98^\x0B\x14a\0+W`\0\x80\xFD[0\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x14a\x01\xDAW` 6`\x03\x19\x01\x12a\x01\xDAW\x815`\x01`\x01`\xA0\x1B\x03\x80\x82\x16\x90\x81\x83\x03a\x01\xD6W`\x01`\x01`\xA0\x1B\x03\x83\x16`\0\x90\x81R`\x1C` R`@\x90 T\x92\x83\x15a\x01\xC6W`\x01`\x01`\xA0\x1B\x03\x16`\0\x90\x81R`\x1C` R`@\x90 \x86\x90U`\tT\x86\x91\x16\x80;\x15a\x01\xC2W\x81\x90`$\x86Q\x80\x94\x81\x93clq*\xB9`\xE1\x1B\x83R\x88\x8B\x84\x01RZ\xF1\x80\x15a\x01\xB8Wa\x01\x8AW[P\x81G\x10a\x01tW\x84\x80\x80\x93\x81\x93Z\xF1=\x15a\x01oWg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF=\x81\x81\x11a\x01\\W\x83Q\x91`\x1F\x82\x01`\x1F\x19\x90\x81\x16`?\x01\x16\x83\x01\x90\x81\x11\x83\x82\x10\x17a\x01GW\x84R\x81R\x84` =\x92\x01>[\x15a\x01:W\x82\x80\xF3[Qc\n\x12\xF5!`\xE1\x1B\x81R\xFD[`A\x86cNH{q`\xE0\x1B`\0RR`$`\0\xFD[cNH{q`\xE0\x1B\x86R`A\x85R`$\x86\xFD[a\x011V[\x82Qc\xCDx`Y`\xE0\x1B\x81R0\x81\x86\x01R`$\x90\xFD[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x96\x92\x96\x11a\x01\xA5W\x83R\x938a\0\xE0V[cNH{q`\xE0\x1B\x82R`A\x85R`$\x82\xFD[\x84Q=\x88\x82>=\x90\xFD[P\x80\xFD[\x84Qcg0\x0F\x91`\xE1\x1B\x81R\x86\x90\xFD[\x85\x80\xFD[\x82\x80\xFD\xFE\xA2dipfsX\"\x12 3tm\x92\x93dk\x9F\xBBEQ\x1D\xA6j\xFA\x0FR\xA1HI0\xF6 P\x04fK{\xD7\x8A9xdsolcC\0\x08\x13\x003";
    /// The bytecode of the contract.
    pub static LIBSTAKING_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __BYTECODE,
    );
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = b"`@`\x80\x81R`\x04\x806\x10\x15a\0\x14W`\0\x80\xFD[`\0\x91\x825`\xE0\x1Cc\x82\x98^\x0B\x14a\0+W`\0\x80\xFD[0\x7F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\x14a\x01\xDAW` 6`\x03\x19\x01\x12a\x01\xDAW\x815`\x01`\x01`\xA0\x1B\x03\x80\x82\x16\x90\x81\x83\x03a\x01\xD6W`\x01`\x01`\xA0\x1B\x03\x83\x16`\0\x90\x81R`\x1C` R`@\x90 T\x92\x83\x15a\x01\xC6W`\x01`\x01`\xA0\x1B\x03\x16`\0\x90\x81R`\x1C` R`@\x90 \x86\x90U`\tT\x86\x91\x16\x80;\x15a\x01\xC2W\x81\x90`$\x86Q\x80\x94\x81\x93clq*\xB9`\xE1\x1B\x83R\x88\x8B\x84\x01RZ\xF1\x80\x15a\x01\xB8Wa\x01\x8AW[P\x81G\x10a\x01tW\x84\x80\x80\x93\x81\x93Z\xF1=\x15a\x01oWg\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF=\x81\x81\x11a\x01\\W\x83Q\x91`\x1F\x82\x01`\x1F\x19\x90\x81\x16`?\x01\x16\x83\x01\x90\x81\x11\x83\x82\x10\x17a\x01GW\x84R\x81R\x84` =\x92\x01>[\x15a\x01:W\x82\x80\xF3[Qc\n\x12\xF5!`\xE1\x1B\x81R\xFD[`A\x86cNH{q`\xE0\x1B`\0RR`$`\0\xFD[cNH{q`\xE0\x1B\x86R`A\x85R`$\x86\xFD[a\x011V[\x82Qc\xCDx`Y`\xE0\x1B\x81R0\x81\x86\x01R`$\x90\xFD[g\xFF\xFF\xFF\xFF\xFF\xFF\xFF\xFF\x81\x96\x92\x96\x11a\x01\xA5W\x83R\x938a\0\xE0V[cNH{q`\xE0\x1B\x82R`A\x85R`$\x82\xFD[\x84Q=\x88\x82>=\x90\xFD[P\x80\xFD[\x84Qcg0\x0F\x91`\xE1\x1B\x81R\x86\x90\xFD[\x85\x80\xFD[\x82\x80\xFD\xFE\xA2dipfsX\"\x12 3tm\x92\x93dk\x9F\xBBEQ\x1D\xA6j\xFA\x0FR\xA1HI0\xF6 P\x04fK{\xD7\x8A9xdsolcC\0\x08\x13\x003";
    /// The deployed bytecode of the contract.
    pub static LIBSTAKING_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __DEPLOYED_BYTECODE,
    );
    pub struct LibStaking<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for LibStaking<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for LibStaking<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for LibStaking<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for LibStaking<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(::core::stringify!(LibStaking)).field(&self.address()).finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> LibStaking<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    LIBSTAKING_ABI.clone(),
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
                LIBSTAKING_ABI.clone(),
                LIBSTAKING_BYTECODE.clone().into(),
                client,
            );
            let deployer = factory.deploy(constructor_args)?;
            let deployer = ::ethers::contract::ContractDeployer::new(deployer);
            Ok(deployer)
        }
        ///Gets the contract's `CollateralClaimed` event
        pub fn collateral_claimed_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            CollateralClaimedFilter,
        > {
            self.0.event()
        }
        ///Gets the contract's `ConfigurationNumberConfirmed` event
        pub fn configuration_number_confirmed_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            ConfigurationNumberConfirmedFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            LibStakingEvents,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for LibStaking<M> {
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
        Hash
    )]
    #[etherror(
        name = "AddressInsufficientBalance",
        abi = "AddressInsufficientBalance(address)"
    )]
    pub struct AddressInsufficientBalance {
        pub account: ::ethers::core::types::Address,
    }
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
    ///Custom Error type `NoRewardToWithdraw` with signature `NoRewardToWithdraw()` and selector `0xce601f22`
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
    #[etherror(name = "NoRewardToWithdraw", abi = "NoRewardToWithdraw()")]
    pub struct NoRewardToWithdraw;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum LibStakingErrors {
        AddressInsufficientBalance(AddressInsufficientBalance),
        FailedInnerCall(FailedInnerCall),
        NoRewardToWithdraw(NoRewardToWithdraw),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for LibStakingErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded) = <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded) = <AddressInsufficientBalance as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::AddressInsufficientBalance(decoded));
            }
            if let Ok(decoded) = <FailedInnerCall as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::FailedInnerCall(decoded));
            }
            if let Ok(decoded) = <NoRewardToWithdraw as ::ethers::core::abi::AbiDecode>::decode(
                data,
            ) {
                return Ok(Self::NoRewardToWithdraw(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for LibStakingErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::AddressInsufficientBalance(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FailedInnerCall(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NoRewardToWithdraw(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for LibStakingErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <AddressInsufficientBalance as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <FailedInnerCall as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NoRewardToWithdraw as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for LibStakingErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AddressInsufficientBalance(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::FailedInnerCall(element) => ::core::fmt::Display::fmt(element, f),
                Self::NoRewardToWithdraw(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for LibStakingErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<AddressInsufficientBalance> for LibStakingErrors {
        fn from(value: AddressInsufficientBalance) -> Self {
            Self::AddressInsufficientBalance(value)
        }
    }
    impl ::core::convert::From<FailedInnerCall> for LibStakingErrors {
        fn from(value: FailedInnerCall) -> Self {
            Self::FailedInnerCall(value)
        }
    }
    impl ::core::convert::From<NoRewardToWithdraw> for LibStakingErrors {
        fn from(value: NoRewardToWithdraw) -> Self {
            Self::NoRewardToWithdraw(value)
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
        Hash
    )]
    #[ethevent(
        name = "ConfigurationNumberConfirmed",
        abi = "ConfigurationNumberConfirmed(uint64)"
    )]
    pub struct ConfigurationNumberConfirmedFilter {
        pub number: u64,
    }
    ///Container type for all of the contract's events
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum LibStakingEvents {
        CollateralClaimedFilter(CollateralClaimedFilter),
        ConfigurationNumberConfirmedFilter(ConfigurationNumberConfirmedFilter),
    }
    impl ::ethers::contract::EthLogDecode for LibStakingEvents {
        fn decode_log(
            log: &::ethers::core::abi::RawLog,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::Error> {
            if let Ok(decoded) = CollateralClaimedFilter::decode_log(log) {
                return Ok(LibStakingEvents::CollateralClaimedFilter(decoded));
            }
            if let Ok(decoded) = ConfigurationNumberConfirmedFilter::decode_log(log) {
                return Ok(LibStakingEvents::ConfigurationNumberConfirmedFilter(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData)
        }
    }
    impl ::core::fmt::Display for LibStakingEvents {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::CollateralClaimedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ConfigurationNumberConfirmedFilter(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<CollateralClaimedFilter> for LibStakingEvents {
        fn from(value: CollateralClaimedFilter) -> Self {
            Self::CollateralClaimedFilter(value)
        }
    }
    impl ::core::convert::From<ConfigurationNumberConfirmedFilter> for LibStakingEvents {
        fn from(value: ConfigurationNumberConfirmedFilter) -> Self {
            Self::ConfigurationNumberConfirmedFilter(value)
        }
    }
}
