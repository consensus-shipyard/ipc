pub use subnet_registry::*;
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
pub mod subnet_registry {
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"_gateway\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"UnknownSubnet\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"WrongGateway\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"ZeroGatewayAddress\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"subnetAddr\",\"type\":\"address\",\"components\":[],\"indexed\":false}],\"type\":\"event\",\"name\":\"SubnetDeployed\",\"outputs\":[],\"anonymous\":false},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"gateway\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"owner\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getSubnetDeployedByNonce\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"subnet\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"owner\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"latestSubnetDeployed\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"subnet\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"struct SubnetActor.ConstructParams\",\"name\":\"params\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"parentId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"bytes32\",\"name\":\"name\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"ipcGatewayAddr\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"enum ConsensusType\",\"name\":\"consensus\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"minActivationCollateral\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"minValidators\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"bottomUpCheckPeriod\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"topDownCheckPeriod\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint8\",\"name\":\"majorityPercentage\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"genesis\",\"type\":\"bytes\",\"components\":[]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"newSubnetActor\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"subnetAddr\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"subnets\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"userNonces\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]}]";
    ///The parsed JSON ABI of the contract.
    pub static SUBNETREGISTRY_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(||
    ::ethers::core::utils::__serde_json::from_str(__ABI).expect("ABI is always valid"));
    pub struct SubnetRegistry<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for SubnetRegistry<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for SubnetRegistry<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for SubnetRegistry<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for SubnetRegistry<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(SubnetRegistry)).field(&self.address()).finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> SubnetRegistry<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    SUBNETREGISTRY_ABI.clone(),
                    client,
                ),
            )
        }
        ///Calls the contract's `gateway` (0x116191b6) function
        pub fn gateway(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([17, 97, 145, 182], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getSubnetDeployedByNonce` (0x9836b75f) function
        pub fn get_subnet_deployed_by_nonce(
            &self,
            owner: ::ethers::core::types::Address,
            nonce: u64,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([152, 54, 183, 95], (owner, nonce))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `latestSubnetDeployed` (0x1163dca5) function
        pub fn latest_subnet_deployed(
            &self,
            owner: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([17, 99, 220, 165], owner)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `newSubnetActor` (0xf9daddd2) function
        pub fn new_subnet_actor(
            &self,
            params: ConstructParams,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([249, 218, 221, 210], (params,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `subnets` (0xb67c7b4a) function
        pub fn subnets(
            &self,
            p0: ::ethers::core::types::Address,
            p1: u64,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([182, 124, 123, 74], (p0, p1))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `userNonces` (0x2f7801f4) function
        pub fn user_nonces(
            &self,
            p0: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([47, 120, 1, 244], p0)
                .expect("method not found (this should never happen)")
        }
        ///Gets the contract's `SubnetDeployed` event
        pub fn subnet_deployed_filter(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            SubnetDeployedFilter,
        > {
            self.0.event()
        }
        /// Returns an `Event` builder for all the events of this contract.
        pub fn events(
            &self,
        ) -> ::ethers::contract::builders::Event<
            ::std::sync::Arc<M>,
            M,
            SubnetDeployedFilter,
        > {
            self.0.event_with_filter(::core::default::Default::default())
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for SubnetRegistry<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `UnknownSubnet` with signature `UnknownSubnet()` and selector `0x63b0e022`
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
    #[etherror(name = "UnknownSubnet", abi = "UnknownSubnet()")]
    pub struct UnknownSubnet;
    ///Custom Error type `WrongGateway` with signature `WrongGateway()` and selector `0x3bed0499`
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
    #[etherror(name = "WrongGateway", abi = "WrongGateway()")]
    pub struct WrongGateway;
    ///Custom Error type `ZeroGatewayAddress` with signature `ZeroGatewayAddress()` and selector `0x609a8af9`
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
    #[etherror(name = "ZeroGatewayAddress", abi = "ZeroGatewayAddress()")]
    pub struct ZeroGatewayAddress;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum SubnetRegistryErrors {
        UnknownSubnet(UnknownSubnet),
        WrongGateway(WrongGateway),
        ZeroGatewayAddress(ZeroGatewayAddress),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for SubnetRegistryErrors {
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
                = <UnknownSubnet as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::UnknownSubnet(decoded));
            }
            if let Ok(decoded)
                = <WrongGateway as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::WrongGateway(decoded));
            }
            if let Ok(decoded)
                = <ZeroGatewayAddress as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ZeroGatewayAddress(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for SubnetRegistryErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::UnknownSubnet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::WrongGateway(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ZeroGatewayAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for SubnetRegistryErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <UnknownSubnet as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <WrongGateway as ::ethers::contract::EthError>::selector() => true,
                _ if selector
                    == <ZeroGatewayAddress as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for SubnetRegistryErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::UnknownSubnet(element) => ::core::fmt::Display::fmt(element, f),
                Self::WrongGateway(element) => ::core::fmt::Display::fmt(element, f),
                Self::ZeroGatewayAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for SubnetRegistryErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<UnknownSubnet> for SubnetRegistryErrors {
        fn from(value: UnknownSubnet) -> Self {
            Self::UnknownSubnet(value)
        }
    }
    impl ::core::convert::From<WrongGateway> for SubnetRegistryErrors {
        fn from(value: WrongGateway) -> Self {
            Self::WrongGateway(value)
        }
    }
    impl ::core::convert::From<ZeroGatewayAddress> for SubnetRegistryErrors {
        fn from(value: ZeroGatewayAddress) -> Self {
            Self::ZeroGatewayAddress(value)
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
    #[ethevent(name = "SubnetDeployed", abi = "SubnetDeployed(address)")]
    pub struct SubnetDeployedFilter {
        pub subnet_addr: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `gateway` function with signature `gateway()` and selector `0x116191b6`
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
    #[ethcall(name = "gateway", abi = "gateway()")]
    pub struct GatewayCall;
    ///Container type for all input parameters for the `getSubnetDeployedByNonce` function with signature `getSubnetDeployedByNonce(address,uint64)` and selector `0x9836b75f`
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
        name = "getSubnetDeployedByNonce",
        abi = "getSubnetDeployedByNonce(address,uint64)"
    )]
    pub struct GetSubnetDeployedByNonceCall {
        pub owner: ::ethers::core::types::Address,
        pub nonce: u64,
    }
    ///Container type for all input parameters for the `latestSubnetDeployed` function with signature `latestSubnetDeployed(address)` and selector `0x1163dca5`
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
    #[ethcall(name = "latestSubnetDeployed", abi = "latestSubnetDeployed(address)")]
    pub struct LatestSubnetDeployedCall {
        pub owner: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `newSubnetActor` function with signature `newSubnetActor(((uint64,address[]),bytes32,address,uint8,uint256,uint64,uint64,uint64,uint8,bytes))` and selector `0xf9daddd2`
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
        name = "newSubnetActor",
        abi = "newSubnetActor(((uint64,address[]),bytes32,address,uint8,uint256,uint64,uint64,uint64,uint8,bytes))"
    )]
    pub struct NewSubnetActorCall {
        pub params: ConstructParams,
    }
    ///Container type for all input parameters for the `subnets` function with signature `subnets(address,uint64)` and selector `0xb67c7b4a`
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
    #[ethcall(name = "subnets", abi = "subnets(address,uint64)")]
    pub struct SubnetsCall(pub ::ethers::core::types::Address, pub u64);
    ///Container type for all input parameters for the `userNonces` function with signature `userNonces(address)` and selector `0x2f7801f4`
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
    #[ethcall(name = "userNonces", abi = "userNonces(address)")]
    pub struct UserNoncesCall(pub ::ethers::core::types::Address);
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum SubnetRegistryCalls {
        Gateway(GatewayCall),
        GetSubnetDeployedByNonce(GetSubnetDeployedByNonceCall),
        LatestSubnetDeployed(LatestSubnetDeployedCall),
        NewSubnetActor(NewSubnetActorCall),
        Subnets(SubnetsCall),
        UserNonces(UserNoncesCall),
    }
    impl ::ethers::core::abi::AbiDecode for SubnetRegistryCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded)
                = <GatewayCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Gateway(decoded));
            }
            if let Ok(decoded)
                = <GetSubnetDeployedByNonceCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::GetSubnetDeployedByNonce(decoded));
            }
            if let Ok(decoded)
                = <LatestSubnetDeployedCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::LatestSubnetDeployed(decoded));
            }
            if let Ok(decoded)
                = <NewSubnetActorCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NewSubnetActor(decoded));
            }
            if let Ok(decoded)
                = <SubnetsCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Subnets(decoded));
            }
            if let Ok(decoded)
                = <UserNoncesCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::UserNonces(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for SubnetRegistryCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::Gateway(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetSubnetDeployedByNonce(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::LatestSubnetDeployed(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NewSubnetActor(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Subnets(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::UserNonces(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for SubnetRegistryCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::Gateway(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetSubnetDeployedByNonce(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::LatestSubnetDeployed(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NewSubnetActor(element) => ::core::fmt::Display::fmt(element, f),
                Self::Subnets(element) => ::core::fmt::Display::fmt(element, f),
                Self::UserNonces(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<GatewayCall> for SubnetRegistryCalls {
        fn from(value: GatewayCall) -> Self {
            Self::Gateway(value)
        }
    }
    impl ::core::convert::From<GetSubnetDeployedByNonceCall> for SubnetRegistryCalls {
        fn from(value: GetSubnetDeployedByNonceCall) -> Self {
            Self::GetSubnetDeployedByNonce(value)
        }
    }
    impl ::core::convert::From<LatestSubnetDeployedCall> for SubnetRegistryCalls {
        fn from(value: LatestSubnetDeployedCall) -> Self {
            Self::LatestSubnetDeployed(value)
        }
    }
    impl ::core::convert::From<NewSubnetActorCall> for SubnetRegistryCalls {
        fn from(value: NewSubnetActorCall) -> Self {
            Self::NewSubnetActor(value)
        }
    }
    impl ::core::convert::From<SubnetsCall> for SubnetRegistryCalls {
        fn from(value: SubnetsCall) -> Self {
            Self::Subnets(value)
        }
    }
    impl ::core::convert::From<UserNoncesCall> for SubnetRegistryCalls {
        fn from(value: UserNoncesCall) -> Self {
            Self::UserNonces(value)
        }
    }
    ///Container type for all return fields from the `gateway` function with signature `gateway()` and selector `0x116191b6`
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
    pub struct GatewayReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `getSubnetDeployedByNonce` function with signature `getSubnetDeployedByNonce(address,uint64)` and selector `0x9836b75f`
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
    pub struct GetSubnetDeployedByNonceReturn {
        pub subnet: ::ethers::core::types::Address,
    }
    ///Container type for all return fields from the `latestSubnetDeployed` function with signature `latestSubnetDeployed(address)` and selector `0x1163dca5`
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
    pub struct LatestSubnetDeployedReturn {
        pub subnet: ::ethers::core::types::Address,
    }
    ///Container type for all return fields from the `newSubnetActor` function with signature `newSubnetActor(((uint64,address[]),bytes32,address,uint8,uint256,uint64,uint64,uint64,uint8,bytes))` and selector `0xf9daddd2`
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
    pub struct NewSubnetActorReturn {
        pub subnet_addr: ::ethers::core::types::Address,
    }
    ///Container type for all return fields from the `subnets` function with signature `subnets(address,uint64)` and selector `0xb67c7b4a`
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
    pub struct SubnetsReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `userNonces` function with signature `userNonces(address)` and selector `0x2f7801f4`
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
    pub struct UserNoncesReturn(pub u64);
    ///`ConstructParams((uint64,address[]),bytes32,address,uint8,uint256,uint64,uint64,uint64,uint8,bytes)`
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
    pub struct ConstructParams {
        pub parent_id: SubnetID,
        pub name: [u8; 32],
        pub ipc_gateway_addr: ::ethers::core::types::Address,
        pub consensus: u8,
        pub min_activation_collateral: ::ethers::core::types::U256,
        pub min_validators: u64,
        pub bottom_up_check_period: u64,
        pub top_down_check_period: u64,
        pub majority_percentage: u8,
        pub genesis: ::ethers::core::types::Bytes,
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
