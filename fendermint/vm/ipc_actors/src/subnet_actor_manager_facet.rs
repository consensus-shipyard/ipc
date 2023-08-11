pub use subnet_actor_manager_facet::*;
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
pub mod subnet_actor_manager_facet {
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"type\":\"error\",\"name\":\"AddressInsufficientBalance\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"CollateralIsZero\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"EpochAlreadyExecuted\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"EpochNotVotable\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"FailedInnerCall\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"MessagesNotSorted\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NoRewardToWithdraw\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NoValidatorsInSubnet\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotAllValidatorsHaveLeft\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotEnoughBalanceForRewards\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotGateway\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotValidator\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"ReentrancyError\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"SubnetAlreadyKilled\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"SubnetNotActive\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"ValidatorAlreadyVoted\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"WrongCheckpointSource\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"e\",\"type\":\"uint64\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"committedCheckpoints\",\"outputs\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"fee\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"prevHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"submitter\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"hasValidatorVotedForSubmission\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"netAddr\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"struct FvmAddress\",\"name\":\"workerAddr\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}],\"stateMutability\":\"payable\",\"type\":\"function\",\"name\":\"join\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"kill\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"leave\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"reward\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"struct BottomUpCheckpoint\",\"name\":\"checkpoint\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"fee\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"struct CrossMsg[]\",\"name\":\"crossMsgs\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct StorableMsg\",\"name\":\"message\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct IPCAddress\",\"name\":\"from\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"struct IPCAddress\",\"name\":\"to\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes4\",\"name\":\"method\",\"type\":\"bytes4\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"params\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"bool\",\"name\":\"wrapped\",\"type\":\"bool\",\"components\":[]}]},{\"internalType\":\"struct ChildCheck[]\",\"name\":\"children\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"bytes32[]\",\"name\":\"checks\",\"type\":\"bytes32[]\",\"components\":[]}]},{\"internalType\":\"bytes32\",\"name\":\"prevHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\",\"components\":[]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"submitCheckpoint\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"withdraw\",\"outputs\":[]}]";
    ///The parsed JSON ABI of the contract.
    pub static SUBNETACTORMANAGERFACET_ABI: ::ethers::contract::Lazy<
        ::ethers::core::abi::Abi,
    > = ::ethers::contract::Lazy::new(|| {
        ::ethers::core::utils::__serde_json::from_str(__ABI)
            .expect("ABI is always valid")
    });
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
            f.debug_tuple(stringify!(SubnetActorManagerFacet))
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
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    SUBNETACTORMANAGERFACET_ABI.clone(),
                    client,
                ),
            )
        }
        ///Calls the contract's `committedCheckpoints` (0x98903748) function
        pub fn committed_checkpoints(
            &self,
            e: u64,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (
                SubnetID,
                u64,
                ::ethers::core::types::U256,
                [u8; 32],
                ::ethers::core::types::Bytes,
            ),
        > {
            self.0
                .method_hash([152, 144, 55, 72], e)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `hasValidatorVotedForSubmission` (0x66d7bbbc) function
        pub fn has_validator_voted_for_submission(
            &self,
            epoch: u64,
            submitter: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([102, 215, 187, 188], (epoch, submitter))
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
        ///Calls the contract's `kill` (0x41c0e1b5) function
        pub fn kill(&self) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([65, 192, 225, 181], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `leave` (0xd66d9e19) function
        pub fn leave(&self) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([214, 109, 158, 25], ())
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
        ///Calls the contract's `submitCheckpoint` (0xf6fd8381) function
        pub fn submit_checkpoint(
            &self,
            checkpoint: BottomUpCheckpoint,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([246, 253, 131, 129], (checkpoint,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `withdraw` (0x3ccfd60b) function
        pub fn withdraw(&self) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([60, 207, 214, 11], ())
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for SubnetActorManagerFacet<M> {
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
    ///Custom Error type `CollateralIsZero` with signature `CollateralIsZero()` and selector `0xb4f18b02`
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
    #[etherror(name = "CollateralIsZero", abi = "CollateralIsZero()")]
    pub struct CollateralIsZero;
    ///Custom Error type `EpochAlreadyExecuted` with signature `EpochAlreadyExecuted()` and selector `0x7cc3318c`
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
    #[etherror(name = "EpochAlreadyExecuted", abi = "EpochAlreadyExecuted()")]
    pub struct EpochAlreadyExecuted;
    ///Custom Error type `EpochNotVotable` with signature `EpochNotVotable()` and selector `0xb4f68f97`
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
    #[etherror(name = "EpochNotVotable", abi = "EpochNotVotable()")]
    pub struct EpochNotVotable;
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
    ///Custom Error type `MessagesNotSorted` with signature `MessagesNotSorted()` and selector `0x0bd9169f`
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
    #[etherror(name = "MessagesNotSorted", abi = "MessagesNotSorted()")]
    pub struct MessagesNotSorted;
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
    ///Custom Error type `NoValidatorsInSubnet` with signature `NoValidatorsInSubnet()` and selector `0xefa9c8f1`
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
        Hash
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
        Hash
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
        Hash
    )]
    #[etherror(name = "NotGateway", abi = "NotGateway()")]
    pub struct NotGateway;
    ///Custom Error type `NotValidator` with signature `NotValidator()` and selector `0x2ec5b449`
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
        Hash
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
        Hash
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
        Hash
    )]
    #[etherror(name = "SubnetNotActive", abi = "SubnetNotActive()")]
    pub struct SubnetNotActive;
    ///Custom Error type `ValidatorAlreadyVoted` with signature `ValidatorAlreadyVoted()` and selector `0x6e271ebe`
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
    #[etherror(name = "ValidatorAlreadyVoted", abi = "ValidatorAlreadyVoted()")]
    pub struct ValidatorAlreadyVoted;
    ///Custom Error type `WrongCheckpointSource` with signature `WrongCheckpointSource()` and selector `0x75ecc72d`
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
    #[etherror(name = "WrongCheckpointSource", abi = "WrongCheckpointSource()")]
    pub struct WrongCheckpointSource;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum SubnetActorManagerFacetErrors {
        AddressInsufficientBalance(AddressInsufficientBalance),
        CollateralIsZero(CollateralIsZero),
        EpochAlreadyExecuted(EpochAlreadyExecuted),
        EpochNotVotable(EpochNotVotable),
        FailedInnerCall(FailedInnerCall),
        MessagesNotSorted(MessagesNotSorted),
        NoRewardToWithdraw(NoRewardToWithdraw),
        NoValidatorsInSubnet(NoValidatorsInSubnet),
        NotAllValidatorsHaveLeft(NotAllValidatorsHaveLeft),
        NotEnoughBalanceForRewards(NotEnoughBalanceForRewards),
        NotGateway(NotGateway),
        NotValidator(NotValidator),
        ReentrancyError(ReentrancyError),
        SubnetAlreadyKilled(SubnetAlreadyKilled),
        SubnetNotActive(SubnetNotActive),
        ValidatorAlreadyVoted(ValidatorAlreadyVoted),
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
            if let Ok(decoded)
                = <::std::string::String as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::RevertString(decoded));
            }
            if let Ok(decoded)
                = <AddressInsufficientBalance as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::AddressInsufficientBalance(decoded));
            }
            if let Ok(decoded)
                = <CollateralIsZero as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::CollateralIsZero(decoded));
            }
            if let Ok(decoded)
                = <EpochAlreadyExecuted as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::EpochAlreadyExecuted(decoded));
            }
            if let Ok(decoded)
                = <EpochNotVotable as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EpochNotVotable(decoded));
            }
            if let Ok(decoded)
                = <FailedInnerCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::FailedInnerCall(decoded));
            }
            if let Ok(decoded)
                = <MessagesNotSorted as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::MessagesNotSorted(decoded));
            }
            if let Ok(decoded)
                = <NoRewardToWithdraw as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NoRewardToWithdraw(decoded));
            }
            if let Ok(decoded)
                = <NoValidatorsInSubnet as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::NoValidatorsInSubnet(decoded));
            }
            if let Ok(decoded)
                = <NotAllValidatorsHaveLeft as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::NotAllValidatorsHaveLeft(decoded));
            }
            if let Ok(decoded)
                = <NotEnoughBalanceForRewards as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::NotEnoughBalanceForRewards(decoded));
            }
            if let Ok(decoded)
                = <NotGateway as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NotGateway(decoded));
            }
            if let Ok(decoded)
                = <NotValidator as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NotValidator(decoded));
            }
            if let Ok(decoded)
                = <ReentrancyError as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ReentrancyError(decoded));
            }
            if let Ok(decoded)
                = <SubnetAlreadyKilled as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SubnetAlreadyKilled(decoded));
            }
            if let Ok(decoded)
                = <SubnetNotActive as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SubnetNotActive(decoded));
            }
            if let Ok(decoded)
                = <ValidatorAlreadyVoted as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ValidatorAlreadyVoted(decoded));
            }
            if let Ok(decoded)
                = <WrongCheckpointSource as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
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
                Self::CollateralIsZero(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::EpochAlreadyExecuted(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::EpochNotVotable(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::FailedInnerCall(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MessagesNotSorted(element) => {
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
                Self::NotGateway(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotValidator(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ReentrancyError(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SubnetAlreadyKilled(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SubnetNotActive(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ValidatorAlreadyVoted(element) => {
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
                    == <AddressInsufficientBalance as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CollateralIsZero as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <EpochAlreadyExecuted as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <EpochNotVotable as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <FailedInnerCall as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <MessagesNotSorted as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NoRewardToWithdraw as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NoValidatorsInSubnet as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotAllValidatorsHaveLeft as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotEnoughBalanceForRewards as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotGateway as ::ethers::contract::EthError>::selector() => true,
                _ if selector
                    == <NotValidator as ::ethers::contract::EthError>::selector() => true,
                _ if selector
                    == <ReentrancyError as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <SubnetAlreadyKilled as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <SubnetNotActive as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ValidatorAlreadyVoted as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <WrongCheckpointSource as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for SubnetActorManagerFacetErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AddressInsufficientBalance(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CollateralIsZero(element) => ::core::fmt::Display::fmt(element, f),
                Self::EpochAlreadyExecuted(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::EpochNotVotable(element) => ::core::fmt::Display::fmt(element, f),
                Self::FailedInnerCall(element) => ::core::fmt::Display::fmt(element, f),
                Self::MessagesNotSorted(element) => ::core::fmt::Display::fmt(element, f),
                Self::NoRewardToWithdraw(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NoValidatorsInSubnet(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotAllValidatorsHaveLeft(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotEnoughBalanceForRewards(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotGateway(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotValidator(element) => ::core::fmt::Display::fmt(element, f),
                Self::ReentrancyError(element) => ::core::fmt::Display::fmt(element, f),
                Self::SubnetAlreadyKilled(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SubnetNotActive(element) => ::core::fmt::Display::fmt(element, f),
                Self::ValidatorAlreadyVoted(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::WrongCheckpointSource(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for SubnetActorManagerFacetErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<AddressInsufficientBalance>
    for SubnetActorManagerFacetErrors {
        fn from(value: AddressInsufficientBalance) -> Self {
            Self::AddressInsufficientBalance(value)
        }
    }
    impl ::core::convert::From<CollateralIsZero> for SubnetActorManagerFacetErrors {
        fn from(value: CollateralIsZero) -> Self {
            Self::CollateralIsZero(value)
        }
    }
    impl ::core::convert::From<EpochAlreadyExecuted> for SubnetActorManagerFacetErrors {
        fn from(value: EpochAlreadyExecuted) -> Self {
            Self::EpochAlreadyExecuted(value)
        }
    }
    impl ::core::convert::From<EpochNotVotable> for SubnetActorManagerFacetErrors {
        fn from(value: EpochNotVotable) -> Self {
            Self::EpochNotVotable(value)
        }
    }
    impl ::core::convert::From<FailedInnerCall> for SubnetActorManagerFacetErrors {
        fn from(value: FailedInnerCall) -> Self {
            Self::FailedInnerCall(value)
        }
    }
    impl ::core::convert::From<MessagesNotSorted> for SubnetActorManagerFacetErrors {
        fn from(value: MessagesNotSorted) -> Self {
            Self::MessagesNotSorted(value)
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
    impl ::core::convert::From<NotAllValidatorsHaveLeft>
    for SubnetActorManagerFacetErrors {
        fn from(value: NotAllValidatorsHaveLeft) -> Self {
            Self::NotAllValidatorsHaveLeft(value)
        }
    }
    impl ::core::convert::From<NotEnoughBalanceForRewards>
    for SubnetActorManagerFacetErrors {
        fn from(value: NotEnoughBalanceForRewards) -> Self {
            Self::NotEnoughBalanceForRewards(value)
        }
    }
    impl ::core::convert::From<NotGateway> for SubnetActorManagerFacetErrors {
        fn from(value: NotGateway) -> Self {
            Self::NotGateway(value)
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
    impl ::core::convert::From<ValidatorAlreadyVoted> for SubnetActorManagerFacetErrors {
        fn from(value: ValidatorAlreadyVoted) -> Self {
            Self::ValidatorAlreadyVoted(value)
        }
    }
    impl ::core::convert::From<WrongCheckpointSource> for SubnetActorManagerFacetErrors {
        fn from(value: WrongCheckpointSource) -> Self {
            Self::WrongCheckpointSource(value)
        }
    }
    ///Container type for all input parameters for the `committedCheckpoints` function with signature `committedCheckpoints(uint64)` and selector `0x98903748`
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
    #[ethcall(name = "committedCheckpoints", abi = "committedCheckpoints(uint64)")]
    pub struct CommittedCheckpointsCall {
        pub e: u64,
    }
    ///Container type for all input parameters for the `hasValidatorVotedForSubmission` function with signature `hasValidatorVotedForSubmission(uint64,address)` and selector `0x66d7bbbc`
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
        name = "hasValidatorVotedForSubmission",
        abi = "hasValidatorVotedForSubmission(uint64,address)"
    )]
    pub struct HasValidatorVotedForSubmissionCall {
        pub epoch: u64,
        pub submitter: ::ethers::core::types::Address,
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
        Hash
    )]
    #[ethcall(name = "join", abi = "join(string,(uint8,bytes))")]
    pub struct JoinCall {
        pub net_addr: ::std::string::String,
        pub worker_addr: FvmAddress,
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
        Hash
    )]
    #[ethcall(name = "kill", abi = "kill()")]
    pub struct KillCall;
    ///Container type for all input parameters for the `leave` function with signature `leave()` and selector `0xd66d9e19`
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
    #[ethcall(name = "leave", abi = "leave()")]
    pub struct LeaveCall;
    ///Container type for all input parameters for the `reward` function with signature `reward(uint256)` and selector `0xa9fb763c`
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
    #[ethcall(name = "reward", abi = "reward(uint256)")]
    pub struct RewardCall {
        pub amount: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `submitCheckpoint` function with signature `submitCheckpoint(((uint64,address[]),uint64,uint256,((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes),bool)[],((uint64,address[]),bytes32[])[],bytes32,bytes))` and selector `0xf6fd8381`
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
        name = "submitCheckpoint",
        abi = "submitCheckpoint(((uint64,address[]),uint64,uint256,((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes),bool)[],((uint64,address[]),bytes32[])[],bytes32,bytes))"
    )]
    pub struct SubmitCheckpointCall {
        pub checkpoint: BottomUpCheckpoint,
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
        Hash
    )]
    #[ethcall(name = "withdraw", abi = "withdraw()")]
    pub struct WithdrawCall;
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum SubnetActorManagerFacetCalls {
        CommittedCheckpoints(CommittedCheckpointsCall),
        HasValidatorVotedForSubmission(HasValidatorVotedForSubmissionCall),
        Join(JoinCall),
        Kill(KillCall),
        Leave(LeaveCall),
        Reward(RewardCall),
        SubmitCheckpoint(SubmitCheckpointCall),
        Withdraw(WithdrawCall),
    }
    impl ::ethers::core::abi::AbiDecode for SubnetActorManagerFacetCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded)
                = <CommittedCheckpointsCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::CommittedCheckpoints(decoded));
            }
            if let Ok(decoded)
                = <HasValidatorVotedForSubmissionCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::HasValidatorVotedForSubmission(decoded));
            }
            if let Ok(decoded)
                = <JoinCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Join(decoded));
            }
            if let Ok(decoded)
                = <KillCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Kill(decoded));
            }
            if let Ok(decoded)
                = <LeaveCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Leave(decoded));
            }
            if let Ok(decoded)
                = <RewardCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Reward(decoded));
            }
            if let Ok(decoded)
                = <SubmitCheckpointCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::SubmitCheckpoint(decoded));
            }
            if let Ok(decoded)
                = <WithdrawCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Withdraw(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for SubnetActorManagerFacetCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::CommittedCheckpoints(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::HasValidatorVotedForSubmission(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Join(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Kill(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Leave(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Reward(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::SubmitCheckpoint(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Withdraw(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for SubnetActorManagerFacetCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::CommittedCheckpoints(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::HasValidatorVotedForSubmission(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::Join(element) => ::core::fmt::Display::fmt(element, f),
                Self::Kill(element) => ::core::fmt::Display::fmt(element, f),
                Self::Leave(element) => ::core::fmt::Display::fmt(element, f),
                Self::Reward(element) => ::core::fmt::Display::fmt(element, f),
                Self::SubmitCheckpoint(element) => ::core::fmt::Display::fmt(element, f),
                Self::Withdraw(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<CommittedCheckpointsCall>
    for SubnetActorManagerFacetCalls {
        fn from(value: CommittedCheckpointsCall) -> Self {
            Self::CommittedCheckpoints(value)
        }
    }
    impl ::core::convert::From<HasValidatorVotedForSubmissionCall>
    for SubnetActorManagerFacetCalls {
        fn from(value: HasValidatorVotedForSubmissionCall) -> Self {
            Self::HasValidatorVotedForSubmission(value)
        }
    }
    impl ::core::convert::From<JoinCall> for SubnetActorManagerFacetCalls {
        fn from(value: JoinCall) -> Self {
            Self::Join(value)
        }
    }
    impl ::core::convert::From<KillCall> for SubnetActorManagerFacetCalls {
        fn from(value: KillCall) -> Self {
            Self::Kill(value)
        }
    }
    impl ::core::convert::From<LeaveCall> for SubnetActorManagerFacetCalls {
        fn from(value: LeaveCall) -> Self {
            Self::Leave(value)
        }
    }
    impl ::core::convert::From<RewardCall> for SubnetActorManagerFacetCalls {
        fn from(value: RewardCall) -> Self {
            Self::Reward(value)
        }
    }
    impl ::core::convert::From<SubmitCheckpointCall> for SubnetActorManagerFacetCalls {
        fn from(value: SubmitCheckpointCall) -> Self {
            Self::SubmitCheckpoint(value)
        }
    }
    impl ::core::convert::From<WithdrawCall> for SubnetActorManagerFacetCalls {
        fn from(value: WithdrawCall) -> Self {
            Self::Withdraw(value)
        }
    }
    ///Container type for all return fields from the `committedCheckpoints` function with signature `committedCheckpoints(uint64)` and selector `0x98903748`
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
    pub struct CommittedCheckpointsReturn {
        pub source: SubnetID,
        pub epoch: u64,
        pub fee: ::ethers::core::types::U256,
        pub prev_hash: [u8; 32],
        pub proof: ::ethers::core::types::Bytes,
    }
    ///Container type for all return fields from the `hasValidatorVotedForSubmission` function with signature `hasValidatorVotedForSubmission(uint64,address)` and selector `0x66d7bbbc`
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
    pub struct HasValidatorVotedForSubmissionReturn(pub bool);
    ///`BottomUpCheckpoint((uint64,address[]),uint64,uint256,((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes),bool)[],((uint64,address[]),bytes32[])[],bytes32,bytes)`
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
    pub struct BottomUpCheckpoint {
        pub source: SubnetID,
        pub epoch: u64,
        pub fee: ::ethers::core::types::U256,
        pub cross_msgs: ::std::vec::Vec<CrossMsg>,
        pub children: ::std::vec::Vec<ChildCheck>,
        pub prev_hash: [u8; 32],
        pub proof: ::ethers::core::types::Bytes,
    }
    ///`ChildCheck((uint64,address[]),bytes32[])`
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
    pub struct ChildCheck {
        pub source: SubnetID,
        pub checks: ::std::vec::Vec<[u8; 32]>,
    }
    ///`CrossMsg((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes),bool)`
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
    ///`StorableMsg(((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes)`
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
