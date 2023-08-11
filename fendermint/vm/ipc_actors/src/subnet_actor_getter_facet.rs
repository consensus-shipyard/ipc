pub use subnet_actor_getter_facet::*;
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
pub mod subnet_actor_getter_facet {
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[{\"internalType\":\"address\",\"name\":\"a\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"accumulatedRewards\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"bottomUpCheckPeriod\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"bottomUpCheckpointAtEpoch\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"exists\",\"type\":\"bool\",\"components\":[]},{\"internalType\":\"struct BottomUpCheckpoint\",\"name\":\"checkpoint\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"fee\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"struct CrossMsg[]\",\"name\":\"crossMsgs\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct StorableMsg\",\"name\":\"message\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct IPCAddress\",\"name\":\"from\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"struct IPCAddress\",\"name\":\"to\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes4\",\"name\":\"method\",\"type\":\"bytes4\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"params\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"bool\",\"name\":\"wrapped\",\"type\":\"bool\",\"components\":[]}]},{\"internalType\":\"struct ChildCheck[]\",\"name\":\"children\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"bytes32[]\",\"name\":\"checks\",\"type\":\"bytes32[]\",\"components\":[]}]},{\"internalType\":\"bytes32\",\"name\":\"prevHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\",\"components\":[]}]}]},{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"bottomUpCheckpointHashAtEpoch\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"consensus\",\"outputs\":[{\"internalType\":\"enum ConsensusType\",\"name\":\"\",\"type\":\"uint8\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"executableQueue\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"genesis\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getParent\",\"outputs\":[{\"internalType\":\"struct SubnetID\",\"name\":\"\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getValidatorSet\",\"outputs\":[{\"internalType\":\"struct ValidatorSet\",\"name\":\"\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct ValidatorInfo[]\",\"name\":\"validators\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"address\",\"name\":\"addr\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"weight\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"struct FvmAddress\",\"name\":\"workerAddr\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"string\",\"name\":\"netAddresses\",\"type\":\"string\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"configurationNumber\",\"type\":\"uint64\",\"components\":[]}]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getValidators\",\"outputs\":[{\"internalType\":\"address[]\",\"name\":\"\",\"type\":\"address[]\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"ipcGatewayAddr\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"lastVotingExecutedEpoch\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"fromEpoch\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"toEpoch\",\"type\":\"uint64\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"listBottomUpCheckpoints\",\"outputs\":[{\"internalType\":\"struct BottomUpCheckpoint[]\",\"name\":\"\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"fee\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"struct CrossMsg[]\",\"name\":\"crossMsgs\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct StorableMsg\",\"name\":\"message\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct IPCAddress\",\"name\":\"from\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"struct IPCAddress\",\"name\":\"to\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes4\",\"name\":\"method\",\"type\":\"bytes4\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"params\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"bool\",\"name\":\"wrapped\",\"type\":\"bool\",\"components\":[]}]},{\"internalType\":\"struct ChildCheck[]\",\"name\":\"children\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"bytes32[]\",\"name\":\"checks\",\"type\":\"bytes32[]\",\"components\":[]}]},{\"internalType\":\"bytes32\",\"name\":\"prevHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\",\"components\":[]}]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"majorityPercentage\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"minActivationCollateral\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"minValidators\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"name\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"prevExecutedCheckpointHash\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"a\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"stake\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"status\",\"outputs\":[{\"internalType\":\"enum Status\",\"name\":\"\",\"type\":\"uint8\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"topDownCheckPeriod\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalStake\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"index\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"validatorAt\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"validatorCount\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]}]";
    ///The parsed JSON ABI of the contract.
    pub static SUBNETACTORGETTERFACET_ABI: ::ethers::contract::Lazy<
        ::ethers::core::abi::Abi,
    > = ::ethers::contract::Lazy::new(|| {
        ::ethers::core::utils::__serde_json::from_str(__ABI)
            .expect("ABI is always valid")
    });
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
            f.debug_tuple(stringify!(SubnetActorGetterFacet))
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
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    SUBNETACTORGETTERFACET_ABI.clone(),
                    client,
                ),
            )
        }
        ///Calls the contract's `accumulatedRewards` (0x73f273fc) function
        pub fn accumulated_rewards(
            &self,
            a: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([115, 242, 115, 252], a)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `bottomUpCheckPeriod` (0x06c46853) function
        pub fn bottom_up_check_period(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
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
        ///Calls the contract's `executableQueue` (0x10d500e1) function
        pub fn executable_queue(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, (u64, u64, u64)> {
            self.0
                .method_hash([16, 213, 0, 225], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `genesis` (0xa7f0b3de) function
        pub fn genesis(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Bytes,
        > {
            self.0
                .method_hash([167, 240, 179, 222], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getParent` (0x80f76021) function
        pub fn get_parent(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, SubnetID> {
            self.0
                .method_hash([128, 247, 96, 33], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getValidatorSet` (0xcf331250) function
        pub fn get_validator_set(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ValidatorSet> {
            self.0
                .method_hash([207, 51, 18, 80], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getValidators` (0xb7ab4db5) function
        pub fn get_validators(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::std::vec::Vec<::ethers::core::types::Address>,
        > {
            self.0
                .method_hash([183, 171, 77, 181], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `ipcGatewayAddr` (0xcfca2824) function
        pub fn ipc_gateway_addr(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([207, 202, 40, 36], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `lastVotingExecutedEpoch` (0xad81e244) function
        pub fn last_voting_executed_epoch(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([173, 129, 226, 68], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `listBottomUpCheckpoints` (0xac9c2a6f) function
        pub fn list_bottom_up_checkpoints(
            &self,
            from_epoch: u64,
            to_epoch: u64,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::std::vec::Vec<BottomUpCheckpoint>,
        > {
            self.0
                .method_hash([172, 156, 42, 111], (from_epoch, to_epoch))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `majorityPercentage` (0x599c7bd1) function
        pub fn majority_percentage(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([89, 156, 123, 209], ())
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
        pub fn min_validators(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
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
        ///Calls the contract's `prevExecutedCheckpointHash` (0x5f832dbf) function
        pub fn prev_executed_checkpoint_hash(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([95, 131, 45, 191], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `stake` (0x26476204) function
        pub fn stake(
            &self,
            a: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([38, 71, 98, 4], a)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `status` (0x200d2ed2) function
        pub fn status(&self) -> ::ethers::contract::builders::ContractCall<M, u8> {
            self.0
                .method_hash([32, 13, 46, 210], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `topDownCheckPeriod` (0x7d9740f4) function
        pub fn top_down_check_period(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([125, 151, 64, 244], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `totalStake` (0x8b0e9f3f) function
        pub fn total_stake(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([139, 14, 159, 63], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `validatorAt` (0x32e0aa1f) function
        pub fn validator_at(
            &self,
            index: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            ::ethers::core::types::Address,
        > {
            self.0
                .method_hash([50, 224, 170, 31], index)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `validatorCount` (0x0f43a677) function
        pub fn validator_count(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([15, 67, 166, 119], ())
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for SubnetActorGetterFacet<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Container type for all input parameters for the `accumulatedRewards` function with signature `accumulatedRewards(address)` and selector `0x73f273fc`
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
    #[ethcall(name = "accumulatedRewards", abi = "accumulatedRewards(address)")]
    pub struct AccumulatedRewardsCall {
        pub a: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `bottomUpCheckPeriod` function with signature `bottomUpCheckPeriod()` and selector `0x06c46853`
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
        Hash
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
        Hash
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
        Hash
    )]
    #[ethcall(name = "consensus", abi = "consensus()")]
    pub struct ConsensusCall;
    ///Container type for all input parameters for the `executableQueue` function with signature `executableQueue()` and selector `0x10d500e1`
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
    #[ethcall(name = "executableQueue", abi = "executableQueue()")]
    pub struct ExecutableQueueCall;
    ///Container type for all input parameters for the `genesis` function with signature `genesis()` and selector `0xa7f0b3de`
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
    #[ethcall(name = "genesis", abi = "genesis()")]
    pub struct GenesisCall;
    ///Container type for all input parameters for the `getParent` function with signature `getParent()` and selector `0x80f76021`
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
    #[ethcall(name = "getParent", abi = "getParent()")]
    pub struct GetParentCall;
    ///Container type for all input parameters for the `getValidatorSet` function with signature `getValidatorSet()` and selector `0xcf331250`
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
    #[ethcall(name = "getValidatorSet", abi = "getValidatorSet()")]
    pub struct GetValidatorSetCall;
    ///Container type for all input parameters for the `getValidators` function with signature `getValidators()` and selector `0xb7ab4db5`
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
    #[ethcall(name = "getValidators", abi = "getValidators()")]
    pub struct GetValidatorsCall;
    ///Container type for all input parameters for the `ipcGatewayAddr` function with signature `ipcGatewayAddr()` and selector `0xcfca2824`
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
    #[ethcall(name = "ipcGatewayAddr", abi = "ipcGatewayAddr()")]
    pub struct IpcGatewayAddrCall;
    ///Container type for all input parameters for the `lastVotingExecutedEpoch` function with signature `lastVotingExecutedEpoch()` and selector `0xad81e244`
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
    #[ethcall(name = "lastVotingExecutedEpoch", abi = "lastVotingExecutedEpoch()")]
    pub struct LastVotingExecutedEpochCall;
    ///Container type for all input parameters for the `listBottomUpCheckpoints` function with signature `listBottomUpCheckpoints(uint64,uint64)` and selector `0xac9c2a6f`
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
        name = "listBottomUpCheckpoints",
        abi = "listBottomUpCheckpoints(uint64,uint64)"
    )]
    pub struct ListBottomUpCheckpointsCall {
        pub from_epoch: u64,
        pub to_epoch: u64,
    }
    ///Container type for all input parameters for the `majorityPercentage` function with signature `majorityPercentage()` and selector `0x599c7bd1`
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
    #[ethcall(name = "majorityPercentage", abi = "majorityPercentage()")]
    pub struct MajorityPercentageCall;
    ///Container type for all input parameters for the `minActivationCollateral` function with signature `minActivationCollateral()` and selector `0x9e33bd02`
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
        Hash
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
        Hash
    )]
    #[ethcall(name = "name", abi = "name()")]
    pub struct NameCall;
    ///Container type for all input parameters for the `prevExecutedCheckpointHash` function with signature `prevExecutedCheckpointHash()` and selector `0x5f832dbf`
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
    #[ethcall(name = "prevExecutedCheckpointHash", abi = "prevExecutedCheckpointHash()")]
    pub struct PrevExecutedCheckpointHashCall;
    ///Container type for all input parameters for the `stake` function with signature `stake(address)` and selector `0x26476204`
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
    #[ethcall(name = "stake", abi = "stake(address)")]
    pub struct StakeCall {
        pub a: ::ethers::core::types::Address,
    }
    ///Container type for all input parameters for the `status` function with signature `status()` and selector `0x200d2ed2`
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
    #[ethcall(name = "status", abi = "status()")]
    pub struct StatusCall;
    ///Container type for all input parameters for the `topDownCheckPeriod` function with signature `topDownCheckPeriod()` and selector `0x7d9740f4`
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
    #[ethcall(name = "topDownCheckPeriod", abi = "topDownCheckPeriod()")]
    pub struct TopDownCheckPeriodCall;
    ///Container type for all input parameters for the `totalStake` function with signature `totalStake()` and selector `0x8b0e9f3f`
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
    #[ethcall(name = "totalStake", abi = "totalStake()")]
    pub struct TotalStakeCall;
    ///Container type for all input parameters for the `validatorAt` function with signature `validatorAt(uint256)` and selector `0x32e0aa1f`
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
    #[ethcall(name = "validatorAt", abi = "validatorAt(uint256)")]
    pub struct ValidatorAtCall {
        pub index: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `validatorCount` function with signature `validatorCount()` and selector `0x0f43a677`
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
    #[ethcall(name = "validatorCount", abi = "validatorCount()")]
    pub struct ValidatorCountCall;
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum SubnetActorGetterFacetCalls {
        AccumulatedRewards(AccumulatedRewardsCall),
        BottomUpCheckPeriod(BottomUpCheckPeriodCall),
        BottomUpCheckpointAtEpoch(BottomUpCheckpointAtEpochCall),
        BottomUpCheckpointHashAtEpoch(BottomUpCheckpointHashAtEpochCall),
        Consensus(ConsensusCall),
        ExecutableQueue(ExecutableQueueCall),
        Genesis(GenesisCall),
        GetParent(GetParentCall),
        GetValidatorSet(GetValidatorSetCall),
        GetValidators(GetValidatorsCall),
        IpcGatewayAddr(IpcGatewayAddrCall),
        LastVotingExecutedEpoch(LastVotingExecutedEpochCall),
        ListBottomUpCheckpoints(ListBottomUpCheckpointsCall),
        MajorityPercentage(MajorityPercentageCall),
        MinActivationCollateral(MinActivationCollateralCall),
        MinValidators(MinValidatorsCall),
        Name(NameCall),
        PrevExecutedCheckpointHash(PrevExecutedCheckpointHashCall),
        Stake(StakeCall),
        Status(StatusCall),
        TopDownCheckPeriod(TopDownCheckPeriodCall),
        TotalStake(TotalStakeCall),
        ValidatorAt(ValidatorAtCall),
        ValidatorCount(ValidatorCountCall),
    }
    impl ::ethers::core::abi::AbiDecode for SubnetActorGetterFacetCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::core::result::Result<Self, ::ethers::core::abi::AbiError> {
            let data = data.as_ref();
            if let Ok(decoded)
                = <AccumulatedRewardsCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::AccumulatedRewards(decoded));
            }
            if let Ok(decoded)
                = <BottomUpCheckPeriodCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::BottomUpCheckPeriod(decoded));
            }
            if let Ok(decoded)
                = <BottomUpCheckpointAtEpochCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::BottomUpCheckpointAtEpoch(decoded));
            }
            if let Ok(decoded)
                = <BottomUpCheckpointHashAtEpochCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::BottomUpCheckpointHashAtEpoch(decoded));
            }
            if let Ok(decoded)
                = <ConsensusCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Consensus(decoded));
            }
            if let Ok(decoded)
                = <ExecutableQueueCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ExecutableQueue(decoded));
            }
            if let Ok(decoded)
                = <GenesisCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Genesis(decoded));
            }
            if let Ok(decoded)
                = <GetParentCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetParent(decoded));
            }
            if let Ok(decoded)
                = <GetValidatorSetCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetValidatorSet(decoded));
            }
            if let Ok(decoded)
                = <GetValidatorsCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetValidators(decoded));
            }
            if let Ok(decoded)
                = <IpcGatewayAddrCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::IpcGatewayAddr(decoded));
            }
            if let Ok(decoded)
                = <LastVotingExecutedEpochCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::LastVotingExecutedEpoch(decoded));
            }
            if let Ok(decoded)
                = <ListBottomUpCheckpointsCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ListBottomUpCheckpoints(decoded));
            }
            if let Ok(decoded)
                = <MajorityPercentageCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::MajorityPercentage(decoded));
            }
            if let Ok(decoded)
                = <MinActivationCollateralCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::MinActivationCollateral(decoded));
            }
            if let Ok(decoded)
                = <MinValidatorsCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::MinValidators(decoded));
            }
            if let Ok(decoded)
                = <NameCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Name(decoded));
            }
            if let Ok(decoded)
                = <PrevExecutedCheckpointHashCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::PrevExecutedCheckpointHash(decoded));
            }
            if let Ok(decoded)
                = <StakeCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Stake(decoded));
            }
            if let Ok(decoded)
                = <StatusCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Status(decoded));
            }
            if let Ok(decoded)
                = <TopDownCheckPeriodCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::TopDownCheckPeriod(decoded));
            }
            if let Ok(decoded)
                = <TotalStakeCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::TotalStake(decoded));
            }
            if let Ok(decoded)
                = <ValidatorAtCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ValidatorAt(decoded));
            }
            if let Ok(decoded)
                = <ValidatorCountCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ValidatorCount(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for SubnetActorGetterFacetCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::AccumulatedRewards(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BottomUpCheckPeriod(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BottomUpCheckpointAtEpoch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BottomUpCheckpointHashAtEpoch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Consensus(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ExecutableQueue(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Genesis(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetParent(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetValidatorSet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetValidators(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::IpcGatewayAddr(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::LastVotingExecutedEpoch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ListBottomUpCheckpoints(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MajorityPercentage(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MinActivationCollateral(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MinValidators(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Name(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::PrevExecutedCheckpointHash(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Stake(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Status(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::TopDownCheckPeriod(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TotalStake(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ValidatorAt(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ValidatorCount(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for SubnetActorGetterFacetCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AccumulatedRewards(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::BottomUpCheckPeriod(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::BottomUpCheckpointAtEpoch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::BottomUpCheckpointHashAtEpoch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::Consensus(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExecutableQueue(element) => ::core::fmt::Display::fmt(element, f),
                Self::Genesis(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetParent(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetValidatorSet(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetValidators(element) => ::core::fmt::Display::fmt(element, f),
                Self::IpcGatewayAddr(element) => ::core::fmt::Display::fmt(element, f),
                Self::LastVotingExecutedEpoch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ListBottomUpCheckpoints(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::MajorityPercentage(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::MinActivationCollateral(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::MinValidators(element) => ::core::fmt::Display::fmt(element, f),
                Self::Name(element) => ::core::fmt::Display::fmt(element, f),
                Self::PrevExecutedCheckpointHash(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::Stake(element) => ::core::fmt::Display::fmt(element, f),
                Self::Status(element) => ::core::fmt::Display::fmt(element, f),
                Self::TopDownCheckPeriod(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TotalStake(element) => ::core::fmt::Display::fmt(element, f),
                Self::ValidatorAt(element) => ::core::fmt::Display::fmt(element, f),
                Self::ValidatorCount(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<AccumulatedRewardsCall> for SubnetActorGetterFacetCalls {
        fn from(value: AccumulatedRewardsCall) -> Self {
            Self::AccumulatedRewards(value)
        }
    }
    impl ::core::convert::From<BottomUpCheckPeriodCall> for SubnetActorGetterFacetCalls {
        fn from(value: BottomUpCheckPeriodCall) -> Self {
            Self::BottomUpCheckPeriod(value)
        }
    }
    impl ::core::convert::From<BottomUpCheckpointAtEpochCall>
    for SubnetActorGetterFacetCalls {
        fn from(value: BottomUpCheckpointAtEpochCall) -> Self {
            Self::BottomUpCheckpointAtEpoch(value)
        }
    }
    impl ::core::convert::From<BottomUpCheckpointHashAtEpochCall>
    for SubnetActorGetterFacetCalls {
        fn from(value: BottomUpCheckpointHashAtEpochCall) -> Self {
            Self::BottomUpCheckpointHashAtEpoch(value)
        }
    }
    impl ::core::convert::From<ConsensusCall> for SubnetActorGetterFacetCalls {
        fn from(value: ConsensusCall) -> Self {
            Self::Consensus(value)
        }
    }
    impl ::core::convert::From<ExecutableQueueCall> for SubnetActorGetterFacetCalls {
        fn from(value: ExecutableQueueCall) -> Self {
            Self::ExecutableQueue(value)
        }
    }
    impl ::core::convert::From<GenesisCall> for SubnetActorGetterFacetCalls {
        fn from(value: GenesisCall) -> Self {
            Self::Genesis(value)
        }
    }
    impl ::core::convert::From<GetParentCall> for SubnetActorGetterFacetCalls {
        fn from(value: GetParentCall) -> Self {
            Self::GetParent(value)
        }
    }
    impl ::core::convert::From<GetValidatorSetCall> for SubnetActorGetterFacetCalls {
        fn from(value: GetValidatorSetCall) -> Self {
            Self::GetValidatorSet(value)
        }
    }
    impl ::core::convert::From<GetValidatorsCall> for SubnetActorGetterFacetCalls {
        fn from(value: GetValidatorsCall) -> Self {
            Self::GetValidators(value)
        }
    }
    impl ::core::convert::From<IpcGatewayAddrCall> for SubnetActorGetterFacetCalls {
        fn from(value: IpcGatewayAddrCall) -> Self {
            Self::IpcGatewayAddr(value)
        }
    }
    impl ::core::convert::From<LastVotingExecutedEpochCall>
    for SubnetActorGetterFacetCalls {
        fn from(value: LastVotingExecutedEpochCall) -> Self {
            Self::LastVotingExecutedEpoch(value)
        }
    }
    impl ::core::convert::From<ListBottomUpCheckpointsCall>
    for SubnetActorGetterFacetCalls {
        fn from(value: ListBottomUpCheckpointsCall) -> Self {
            Self::ListBottomUpCheckpoints(value)
        }
    }
    impl ::core::convert::From<MajorityPercentageCall> for SubnetActorGetterFacetCalls {
        fn from(value: MajorityPercentageCall) -> Self {
            Self::MajorityPercentage(value)
        }
    }
    impl ::core::convert::From<MinActivationCollateralCall>
    for SubnetActorGetterFacetCalls {
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
    impl ::core::convert::From<PrevExecutedCheckpointHashCall>
    for SubnetActorGetterFacetCalls {
        fn from(value: PrevExecutedCheckpointHashCall) -> Self {
            Self::PrevExecutedCheckpointHash(value)
        }
    }
    impl ::core::convert::From<StakeCall> for SubnetActorGetterFacetCalls {
        fn from(value: StakeCall) -> Self {
            Self::Stake(value)
        }
    }
    impl ::core::convert::From<StatusCall> for SubnetActorGetterFacetCalls {
        fn from(value: StatusCall) -> Self {
            Self::Status(value)
        }
    }
    impl ::core::convert::From<TopDownCheckPeriodCall> for SubnetActorGetterFacetCalls {
        fn from(value: TopDownCheckPeriodCall) -> Self {
            Self::TopDownCheckPeriod(value)
        }
    }
    impl ::core::convert::From<TotalStakeCall> for SubnetActorGetterFacetCalls {
        fn from(value: TotalStakeCall) -> Self {
            Self::TotalStake(value)
        }
    }
    impl ::core::convert::From<ValidatorAtCall> for SubnetActorGetterFacetCalls {
        fn from(value: ValidatorAtCall) -> Self {
            Self::ValidatorAt(value)
        }
    }
    impl ::core::convert::From<ValidatorCountCall> for SubnetActorGetterFacetCalls {
        fn from(value: ValidatorCountCall) -> Self {
            Self::ValidatorCount(value)
        }
    }
    ///Container type for all return fields from the `accumulatedRewards` function with signature `accumulatedRewards(address)` and selector `0x73f273fc`
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
    pub struct AccumulatedRewardsReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `bottomUpCheckPeriod` function with signature `bottomUpCheckPeriod()` and selector `0x06c46853`
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
        Hash
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
        Hash
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
        Hash
    )]
    pub struct ConsensusReturn(pub u8);
    ///Container type for all return fields from the `executableQueue` function with signature `executableQueue()` and selector `0x10d500e1`
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
    pub struct ExecutableQueueReturn(pub u64, pub u64, pub u64);
    ///Container type for all return fields from the `genesis` function with signature `genesis()` and selector `0xa7f0b3de`
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
    pub struct GenesisReturn(pub ::ethers::core::types::Bytes);
    ///Container type for all return fields from the `getParent` function with signature `getParent()` and selector `0x80f76021`
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
    pub struct GetParentReturn(pub SubnetID);
    ///Container type for all return fields from the `getValidatorSet` function with signature `getValidatorSet()` and selector `0xcf331250`
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
    pub struct GetValidatorSetReturn(pub ValidatorSet);
    ///Container type for all return fields from the `getValidators` function with signature `getValidators()` and selector `0xb7ab4db5`
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
    pub struct GetValidatorsReturn(pub ::std::vec::Vec<::ethers::core::types::Address>);
    ///Container type for all return fields from the `ipcGatewayAddr` function with signature `ipcGatewayAddr()` and selector `0xcfca2824`
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
    pub struct IpcGatewayAddrReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `lastVotingExecutedEpoch` function with signature `lastVotingExecutedEpoch()` and selector `0xad81e244`
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
    pub struct LastVotingExecutedEpochReturn(pub u64);
    ///Container type for all return fields from the `listBottomUpCheckpoints` function with signature `listBottomUpCheckpoints(uint64,uint64)` and selector `0xac9c2a6f`
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
    pub struct ListBottomUpCheckpointsReturn(pub ::std::vec::Vec<BottomUpCheckpoint>);
    ///Container type for all return fields from the `majorityPercentage` function with signature `majorityPercentage()` and selector `0x599c7bd1`
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
    pub struct MajorityPercentageReturn(pub u64);
    ///Container type for all return fields from the `minActivationCollateral` function with signature `minActivationCollateral()` and selector `0x9e33bd02`
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
        Hash
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
        Hash
    )]
    pub struct NameReturn(pub [u8; 32]);
    ///Container type for all return fields from the `prevExecutedCheckpointHash` function with signature `prevExecutedCheckpointHash()` and selector `0x5f832dbf`
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
    pub struct PrevExecutedCheckpointHashReturn(pub [u8; 32]);
    ///Container type for all return fields from the `stake` function with signature `stake(address)` and selector `0x26476204`
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
    pub struct StakeReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `status` function with signature `status()` and selector `0x200d2ed2`
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
    pub struct StatusReturn(pub u8);
    ///Container type for all return fields from the `topDownCheckPeriod` function with signature `topDownCheckPeriod()` and selector `0x7d9740f4`
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
    pub struct TopDownCheckPeriodReturn(pub u64);
    ///Container type for all return fields from the `totalStake` function with signature `totalStake()` and selector `0x8b0e9f3f`
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
    pub struct TotalStakeReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `validatorAt` function with signature `validatorAt(uint256)` and selector `0x32e0aa1f`
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
    pub struct ValidatorAtReturn(pub ::ethers::core::types::Address);
    ///Container type for all return fields from the `validatorCount` function with signature `validatorCount()` and selector `0x0f43a677`
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
    pub struct ValidatorCountReturn(pub ::ethers::core::types::U256);
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
    ///`ValidatorInfo(address,uint256,(uint8,bytes),string)`
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
    pub struct ValidatorInfo {
        pub addr: ::ethers::core::types::Address,
        pub weight: ::ethers::core::types::U256,
        pub worker_addr: FvmAddress,
        pub net_addresses: ::std::string::String,
    }
    ///`ValidatorSet((address,uint256,(uint8,bytes),string)[],uint64)`
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
    pub struct ValidatorSet {
        pub validators: ::std::vec::Vec<ValidatorInfo>,
        pub configuration_number: u64,
    }
}
