pub use gateway_diamond::*;
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
pub mod gateway_diamond {
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[{\"internalType\":\"struct IDiamond.FacetCut[]\",\"name\":\"_diamondCut\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"address\",\"name\":\"facetAddress\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"enum IDiamond.FacetCutAction\",\"name\":\"action\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes4[]\",\"name\":\"functionSelectors\",\"type\":\"bytes4[]\",\"components\":[]}]},{\"internalType\":\"struct GatewayDiamond.ConstructorParams\",\"name\":\"params\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"networkName\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"bottomUpCheckPeriod\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"topDownCheckPeriod\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"msgFee\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint8\",\"name\":\"majorityPercentage\",\"type\":\"uint8\",\"components\":[]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes4\",\"name\":\"_selector\",\"type\":\"bytes4\",\"components\":[]}],\"type\":\"error\",\"name\":\"CannotAddFunctionToDiamondThatAlreadyExists\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes4[]\",\"name\":\"_selectors\",\"type\":\"bytes4[]\",\"components\":[]}],\"type\":\"error\",\"name\":\"CannotAddSelectorsToZeroAddress\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"bytes4\",\"name\":\"_functionSelector\",\"type\":\"bytes4\",\"components\":[]}],\"type\":\"error\",\"name\":\"FunctionNotFound\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"enum IDiamond.FacetCutAction\",\"name\":\"_action\",\"type\":\"uint8\",\"components\":[]}],\"type\":\"error\",\"name\":\"IncorrectFacetCutAction\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_initializationContractAddress\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"_calldata\",\"type\":\"bytes\",\"components\":[]}],\"type\":\"error\",\"name\":\"InitializationFunctionReverted\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidMajorityPercentage\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_contractAddress\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"string\",\"name\":\"_message\",\"type\":\"string\",\"components\":[]}],\"type\":\"error\",\"name\":\"NoBytecodeAtAddress\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_facetAddress\",\"type\":\"address\",\"components\":[]}],\"type\":\"error\",\"name\":\"NoSelectorsProvidedForFacetForCut\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"payable\",\"type\":\"fallback\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"MIN_CHECKPOINT_PERIOD\",\"outputs\":[{\"internalType\":\"uint8\",\"name\":\"\",\"type\":\"uint8\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"MIN_COLLATERAL_AMOUNT\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"payable\",\"type\":\"receive\",\"outputs\":[]}]";
    ///The parsed JSON ABI of the contract.
    pub static GATEWAYDIAMOND_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(||
    ::ethers::core::utils::__serde_json::from_str(__ABI).expect("ABI is always valid"));
    #[rustfmt::skip]
    const __DEPLOYED_BYTECODE: &[u8] = &[
        96,
        128,
        96,
        64,
        82,
        96,
        4,
        54,
        16,
        21,
        96,
        21,
        87,
        91,
        54,
        96,
        113,
        87,
        96,
        113,
        86,
        91,
        96,
        0,
        128,
        53,
        96,
        224,
        28,
        128,
        99,
        145,
        190,
        77,
        65,
        20,
        96,
        80,
        87,
        99,
        161,
        173,
        163,
        3,
        20,
        96,
        52,
        87,
        80,
        96,
        13,
        86,
        91,
        52,
        96,
        77,
        87,
        128,
        96,
        3,
        25,
        54,
        1,
        18,
        96,
        77,
        87,
        96,
        32,
        96,
        64,
        81,
        96,
        10,
        129,
        82,
        243,
        91,
        128,
        253,
        91,
        80,
        52,
        96,
        77,
        87,
        128,
        96,
        3,
        25,
        54,
        1,
        18,
        96,
        77,
        87,
        103,
        13,
        224,
        182,
        179,
        167,
        100,
        0,
        0,
        96,
        128,
        82,
        96,
        32,
        96,
        128,
        243,
        91,
        96,
        0,
        128,
        53,
        96,
        1,
        96,
        1,
        96,
        224,
        27,
        3,
        25,
        22,
        128,
        130,
        82,
        127,
        128,
        110,
        12,
        187,
        159,
        206,
        41,
        107,
        188,
        51,
        106,
        72,
        244,
        43,
        241,
        219,
        198,
        151,
        34,
        209,
        141,
        144,
        214,
        254,
        112,
        91,
        117,
        130,
        194,
        187,
        75,
        210,
        96,
        32,
        82,
        96,
        64,
        130,
        32,
        84,
        96,
        1,
        96,
        1,
        96,
        160,
        27,
        3,
        22,
        144,
        129,
        21,
        96,
        215,
        87,
        80,
        129,
        128,
        145,
        54,
        130,
        128,
        55,
        129,
        54,
        145,
        90,
        244,
        61,
        130,
        128,
        62,
        21,
        96,
        211,
        87,
        61,
        144,
        243,
        91,
        61,
        144,
        253,
        91,
        96,
        36,
        144,
        96,
        64,
        81,
        144,
        99,
        10,
        130,
        221,
        115,
        96,
        227,
        27,
        130,
        82,
        96,
        4,
        130,
        1,
        82,
        253,
        254,
        162,
        100,
        105,
        112,
        102,
        115,
        88,
        34,
        18,
        32,
        67,
        152,
        61,
        224,
        214,
        176,
        230,
        237,
        248,
        60,
        69,
        226,
        30,
        46,
        144,
        233,
        11,
        69,
        204,
        217,
        0,
        126,
        186,
        185,
        198,
        22,
        218,
        59,
        243,
        68,
        226,
        112,
        100,
        115,
        111,
        108,
        99,
        67,
        0,
        8,
        19,
        0,
        51,
    ];
    ///The deployed bytecode of the contract.
    pub static GATEWAYDIAMOND_DEPLOYED_BYTECODE: ::ethers::core::types::Bytes = ::ethers::core::types::Bytes::from_static(
        __DEPLOYED_BYTECODE,
    );
    pub struct GatewayDiamond<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for GatewayDiamond<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for GatewayDiamond<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for GatewayDiamond<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for GatewayDiamond<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(GatewayDiamond)).field(&self.address()).finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> GatewayDiamond<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    GATEWAYDIAMOND_ABI.clone(),
                    client,
                ),
            )
        }
        ///Calls the contract's `MIN_CHECKPOINT_PERIOD` (0xa1ada303) function
        pub fn min_checkpoint_period(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u8> {
            self.0
                .method_hash([161, 173, 163, 3], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `MIN_COLLATERAL_AMOUNT` (0x91be4d41) function
        pub fn min_collateral_amount(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([145, 190, 77, 65], ())
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for GatewayDiamond<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `CannotAddFunctionToDiamondThatAlreadyExists` with signature `CannotAddFunctionToDiamondThatAlreadyExists(bytes4)` and selector `0xebbf5d07`
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
        name = "CannotAddFunctionToDiamondThatAlreadyExists",
        abi = "CannotAddFunctionToDiamondThatAlreadyExists(bytes4)"
    )]
    pub struct CannotAddFunctionToDiamondThatAlreadyExists {
        pub selector: [u8; 4],
    }
    ///Custom Error type `CannotAddSelectorsToZeroAddress` with signature `CannotAddSelectorsToZeroAddress(bytes4[])` and selector `0x0ae3681c`
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
        name = "CannotAddSelectorsToZeroAddress",
        abi = "CannotAddSelectorsToZeroAddress(bytes4[])"
    )]
    pub struct CannotAddSelectorsToZeroAddress {
        pub selectors: ::std::vec::Vec<[u8; 4]>,
    }
    ///Custom Error type `FunctionNotFound` with signature `FunctionNotFound(bytes4)` and selector `0x5416eb98`
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
    #[etherror(name = "FunctionNotFound", abi = "FunctionNotFound(bytes4)")]
    pub struct FunctionNotFound {
        pub function_selector: [u8; 4],
    }
    ///Custom Error type `IncorrectFacetCutAction` with signature `IncorrectFacetCutAction(uint8)` and selector `0x7fe9a41e`
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
    #[etherror(name = "IncorrectFacetCutAction", abi = "IncorrectFacetCutAction(uint8)")]
    pub struct IncorrectFacetCutAction {
        pub action: u8,
    }
    ///Custom Error type `InitializationFunctionReverted` with signature `InitializationFunctionReverted(address,bytes)` and selector `0x192105d7`
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
        name = "InitializationFunctionReverted",
        abi = "InitializationFunctionReverted(address,bytes)"
    )]
    pub struct InitializationFunctionReverted {
        pub initialization_contract_address: ::ethers::core::types::Address,
        pub calldata: ::ethers::core::types::Bytes,
    }
    ///Custom Error type `InvalidMajorityPercentage` with signature `InvalidMajorityPercentage()` and selector `0x75c3b427`
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
    #[etherror(name = "InvalidMajorityPercentage", abi = "InvalidMajorityPercentage()")]
    pub struct InvalidMajorityPercentage;
    ///Custom Error type `NoBytecodeAtAddress` with signature `NoBytecodeAtAddress(address,string)` and selector `0x919834b9`
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
        name = "NoBytecodeAtAddress",
        abi = "NoBytecodeAtAddress(address,string)"
    )]
    pub struct NoBytecodeAtAddress {
        pub contract_address: ::ethers::core::types::Address,
        pub message: ::std::string::String,
    }
    ///Custom Error type `NoSelectorsProvidedForFacetForCut` with signature `NoSelectorsProvidedForFacetForCut(address)` and selector `0xe767f91f`
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
        name = "NoSelectorsProvidedForFacetForCut",
        abi = "NoSelectorsProvidedForFacetForCut(address)"
    )]
    pub struct NoSelectorsProvidedForFacetForCut {
        pub facet_address: ::ethers::core::types::Address,
    }
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum GatewayDiamondErrors {
        CannotAddFunctionToDiamondThatAlreadyExists(
            CannotAddFunctionToDiamondThatAlreadyExists,
        ),
        CannotAddSelectorsToZeroAddress(CannotAddSelectorsToZeroAddress),
        FunctionNotFound(FunctionNotFound),
        IncorrectFacetCutAction(IncorrectFacetCutAction),
        InitializationFunctionReverted(InitializationFunctionReverted),
        InvalidMajorityPercentage(InvalidMajorityPercentage),
        NoBytecodeAtAddress(NoBytecodeAtAddress),
        NoSelectorsProvidedForFacetForCut(NoSelectorsProvidedForFacetForCut),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for GatewayDiamondErrors {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded)
                = <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded)
                = <CannotAddFunctionToDiamondThatAlreadyExists as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::CannotAddFunctionToDiamondThatAlreadyExists(decoded));
            }
            if let Ok(decoded)
                = <CannotAddSelectorsToZeroAddress as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::CannotAddSelectorsToZeroAddress(decoded));
            }
            if let Ok(decoded)
                = <FunctionNotFound as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::FunctionNotFound(decoded));
            }
            if let Ok(decoded)
                = <IncorrectFacetCutAction as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::IncorrectFacetCutAction(decoded));
            }
            if let Ok(decoded)
                = <InitializationFunctionReverted as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::InitializationFunctionReverted(decoded));
            }
            if let Ok(decoded)
                = <InvalidMajorityPercentage as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::InvalidMajorityPercentage(decoded));
            }
            if let Ok(decoded)
                = <NoBytecodeAtAddress as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NoBytecodeAtAddress(decoded));
            }
            if let Ok(decoded)
                = <NoSelectorsProvidedForFacetForCut as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::NoSelectorsProvidedForFacetForCut(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for GatewayDiamondErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::CannotAddFunctionToDiamondThatAlreadyExists(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotAddSelectorsToZeroAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FunctionNotFound(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::IncorrectFacetCutAction(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitializationFunctionReverted(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidMajorityPercentage(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NoBytecodeAtAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NoSelectorsProvidedForFacetForCut(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for GatewayDiamondErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <CannotAddFunctionToDiamondThatAlreadyExists as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CannotAddSelectorsToZeroAddress as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <FunctionNotFound as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <IncorrectFacetCutAction as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InitializationFunctionReverted as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidMajorityPercentage as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NoBytecodeAtAddress as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NoSelectorsProvidedForFacetForCut as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for GatewayDiamondErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::CannotAddFunctionToDiamondThatAlreadyExists(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CannotAddSelectorsToZeroAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::FunctionNotFound(element) => ::core::fmt::Display::fmt(element, f),
                Self::IncorrectFacetCutAction(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitializationFunctionReverted(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidMajorityPercentage(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NoBytecodeAtAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NoSelectorsProvidedForFacetForCut(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for GatewayDiamondErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<CannotAddFunctionToDiamondThatAlreadyExists>
    for GatewayDiamondErrors {
        fn from(value: CannotAddFunctionToDiamondThatAlreadyExists) -> Self {
            Self::CannotAddFunctionToDiamondThatAlreadyExists(value)
        }
    }
    impl ::core::convert::From<CannotAddSelectorsToZeroAddress>
    for GatewayDiamondErrors {
        fn from(value: CannotAddSelectorsToZeroAddress) -> Self {
            Self::CannotAddSelectorsToZeroAddress(value)
        }
    }
    impl ::core::convert::From<FunctionNotFound> for GatewayDiamondErrors {
        fn from(value: FunctionNotFound) -> Self {
            Self::FunctionNotFound(value)
        }
    }
    impl ::core::convert::From<IncorrectFacetCutAction> for GatewayDiamondErrors {
        fn from(value: IncorrectFacetCutAction) -> Self {
            Self::IncorrectFacetCutAction(value)
        }
    }
    impl ::core::convert::From<InitializationFunctionReverted> for GatewayDiamondErrors {
        fn from(value: InitializationFunctionReverted) -> Self {
            Self::InitializationFunctionReverted(value)
        }
    }
    impl ::core::convert::From<InvalidMajorityPercentage> for GatewayDiamondErrors {
        fn from(value: InvalidMajorityPercentage) -> Self {
            Self::InvalidMajorityPercentage(value)
        }
    }
    impl ::core::convert::From<NoBytecodeAtAddress> for GatewayDiamondErrors {
        fn from(value: NoBytecodeAtAddress) -> Self {
            Self::NoBytecodeAtAddress(value)
        }
    }
    impl ::core::convert::From<NoSelectorsProvidedForFacetForCut>
    for GatewayDiamondErrors {
        fn from(value: NoSelectorsProvidedForFacetForCut) -> Self {
            Self::NoSelectorsProvidedForFacetForCut(value)
        }
    }
    ///Container type for all input parameters for the `MIN_CHECKPOINT_PERIOD` function with signature `MIN_CHECKPOINT_PERIOD()` and selector `0xa1ada303`
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
    #[ethcall(name = "MIN_CHECKPOINT_PERIOD", abi = "MIN_CHECKPOINT_PERIOD()")]
    pub struct MinCheckpointPeriodCall;
    ///Container type for all input parameters for the `MIN_COLLATERAL_AMOUNT` function with signature `MIN_COLLATERAL_AMOUNT()` and selector `0x91be4d41`
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
    #[ethcall(name = "MIN_COLLATERAL_AMOUNT", abi = "MIN_COLLATERAL_AMOUNT()")]
    pub struct MinCollateralAmountCall;
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum GatewayDiamondCalls {
        MinCheckpointPeriod(MinCheckpointPeriodCall),
        MinCollateralAmount(MinCollateralAmountCall),
    }
    impl ::ethers::core::abi::AbiDecode for GatewayDiamondCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded)
                = <MinCheckpointPeriodCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::MinCheckpointPeriod(decoded));
            }
            if let Ok(decoded)
                = <MinCollateralAmountCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::MinCollateralAmount(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for GatewayDiamondCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::MinCheckpointPeriod(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MinCollateralAmount(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for GatewayDiamondCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::MinCheckpointPeriod(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::MinCollateralAmount(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
            }
        }
    }
    impl ::core::convert::From<MinCheckpointPeriodCall> for GatewayDiamondCalls {
        fn from(value: MinCheckpointPeriodCall) -> Self {
            Self::MinCheckpointPeriod(value)
        }
    }
    impl ::core::convert::From<MinCollateralAmountCall> for GatewayDiamondCalls {
        fn from(value: MinCollateralAmountCall) -> Self {
            Self::MinCollateralAmount(value)
        }
    }
    ///Container type for all return fields from the `MIN_CHECKPOINT_PERIOD` function with signature `MIN_CHECKPOINT_PERIOD()` and selector `0xa1ada303`
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
    pub struct MinCheckpointPeriodReturn(pub u8);
    ///Container type for all return fields from the `MIN_COLLATERAL_AMOUNT` function with signature `MIN_COLLATERAL_AMOUNT()` and selector `0x91be4d41`
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
    pub struct MinCollateralAmountReturn(pub ::ethers::core::types::U256);
}
