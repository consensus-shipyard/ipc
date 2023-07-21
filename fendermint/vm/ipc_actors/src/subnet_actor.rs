pub use subnet_actor::*;
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
pub mod subnet_actor {
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[{\"internalType\":\"struct SubnetActor.ConstructParams\",\"name\":\"params\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"parentId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"bytes32\",\"name\":\"name\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"ipcGatewayAddr\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"enum ConsensusType\",\"name\":\"consensus\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"minActivationCollateral\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"minValidators\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"bottomUpCheckPeriod\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"topDownCheckPeriod\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint8\",\"name\":\"majorityPercentage\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"genesis\",\"type\":\"bytes\",\"components\":[]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"type\":\"error\",\"name\":\"AddressInsufficientBalance\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"CallerHasNoStake\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"CheckpointNotChained\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"CollateralIsZero\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"CollateralStillLockedInSubnet\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"EpochAlreadyExecuted\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"EpochNotVotable\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"FailedInnerCall\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"GatewayCannotBeZero\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidMajorityPercentage\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"MessagesNotSorted\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NoRewardToWithdraw\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NoValidatorsInSubnet\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotAccount\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotAllValidatorsHaveLeft\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotEnoughBalanceForRewards\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotGateway\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotValidator\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"ReentrancyGuardReentrantCall\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"SubnetAlreadyKilled\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"SubnetNotActive\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"ValidatorAlreadyVoted\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"WorkerAddressInvalid\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"WrongCheckpointSource\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"MIN_CHECKPOINT_PERIOD\",\"outputs\":[{\"internalType\":\"uint8\",\"name\":\"\",\"type\":\"uint8\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"MIN_COLLATERAL_AMOUNT\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"accumulatedRewards\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"bottomUpCheckPeriod\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"bottomUpCheckpointAtEpoch\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"exists\",\"type\":\"bool\",\"components\":[]},{\"internalType\":\"struct BottomUpCheckpoint\",\"name\":\"checkpoint\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"fee\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"struct CrossMsg[]\",\"name\":\"crossMsgs\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct StorableMsg\",\"name\":\"message\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct IPCAddress\",\"name\":\"from\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"struct IPCAddress\",\"name\":\"to\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes4\",\"name\":\"method\",\"type\":\"bytes4\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"params\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"bool\",\"name\":\"wrapped\",\"type\":\"bool\",\"components\":[]}]},{\"internalType\":\"struct ChildCheck[]\",\"name\":\"children\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"bytes32[]\",\"name\":\"checks\",\"type\":\"bytes32[]\",\"components\":[]}]},{\"internalType\":\"bytes32\",\"name\":\"prevHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\",\"components\":[]}]}]},{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"bottomUpCheckpointHashAtEpoch\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"committedCheckpoints\",\"outputs\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"fee\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"prevHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"configurationNumber\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"consensus\",\"outputs\":[{\"internalType\":\"enum ConsensusType\",\"name\":\"\",\"type\":\"uint8\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"currentSubnetHash\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"executableQueue\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"period\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"first\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"last\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"genesis\",\"outputs\":[{\"internalType\":\"bytes\",\"name\":\"\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getGenesisEpoch\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getParent\",\"outputs\":[{\"internalType\":\"struct SubnetID\",\"name\":\"\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getValidatorSet\",\"outputs\":[{\"internalType\":\"struct SubnetActor.ValidatorSet\",\"name\":\"\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetActor.ValidatorInfo[]\",\"name\":\"validators\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"address\",\"name\":\"addr\",\"type\":\"address\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"weight\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"struct FvmAddress\",\"name\":\"workerAddr\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"string\",\"name\":\"netAddresses\",\"type\":\"string\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"configurationNumber\",\"type\":\"uint64\",\"components\":[]}]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getValidators\",\"outputs\":[{\"internalType\":\"address[]\",\"name\":\"\",\"type\":\"address[]\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"submitter\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"hasValidatorVotedForSubmission\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"ipcGatewayAddr\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"string\",\"name\":\"netAddr\",\"type\":\"string\",\"components\":[]},{\"internalType\":\"struct FvmAddress\",\"name\":\"workerAddr\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}],\"stateMutability\":\"payable\",\"type\":\"function\",\"name\":\"join\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"kill\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"lastVotingExecutedEpoch\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"leave\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"majorityPercentage\",\"outputs\":[{\"internalType\":\"uint8\",\"name\":\"\",\"type\":\"uint8\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"minActivationCollateral\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"minValidators\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"name\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"prevExecutedCheckpointHash\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"reward\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"stake\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"status\",\"outputs\":[{\"internalType\":\"enum Status\",\"name\":\"\",\"type\":\"uint8\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"submissionPeriod\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"struct BottomUpCheckpoint\",\"name\":\"checkpoint\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"fee\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"struct CrossMsg[]\",\"name\":\"crossMsgs\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct StorableMsg\",\"name\":\"message\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct IPCAddress\",\"name\":\"from\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"struct IPCAddress\",\"name\":\"to\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes4\",\"name\":\"method\",\"type\":\"bytes4\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"params\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"bool\",\"name\":\"wrapped\",\"type\":\"bool\",\"components\":[]}]},{\"internalType\":\"struct ChildCheck[]\",\"name\":\"children\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"bytes32[]\",\"name\":\"checks\",\"type\":\"bytes32[]\",\"components\":[]}]},{\"internalType\":\"bytes32\",\"name\":\"prevHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\",\"components\":[]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"submitCheckpoint\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"topDownCheckPeriod\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalStake\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"index\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"validatorAt\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"validatorCount\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"validatorNetAddresses\",\"outputs\":[{\"internalType\":\"string\",\"name\":\"\",\"type\":\"string\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"validatorWorkerAddresses\",\"outputs\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"withdraw\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"payable\",\"type\":\"receive\",\"outputs\":[]}]";
    ///The parsed JSON ABI of the contract.
    pub static SUBNETACTOR_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(||
    ::ethers::core::utils::__serde_json::from_str(__ABI).expect("ABI is always valid"));
    pub struct SubnetActor<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for SubnetActor<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for SubnetActor<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for SubnetActor<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for SubnetActor<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(SubnetActor)).field(&self.address()).finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> SubnetActor<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    SUBNETACTOR_ABI.clone(),
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
        ///Calls the contract's `accumulatedRewards` (0x73f273fc) function
        pub fn accumulated_rewards(
            &self,
            p0: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([115, 242, 115, 252], p0)
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
        ///Calls the contract's `committedCheckpoints` (0x98903748) function
        pub fn committed_checkpoints(
            &self,
            p0: u64,
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
                .method_hash([152, 144, 55, 72], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `configurationNumber` (0x04fda3d4) function
        pub fn configuration_number(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([4, 253, 163, 212], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `consensus` (0x8ef3f761) function
        pub fn consensus(&self) -> ::ethers::contract::builders::ContractCall<M, u8> {
            self.0
                .method_hash([142, 243, 247, 97], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `currentSubnetHash` (0xc18f64aa) function
        pub fn current_subnet_hash(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([193, 143, 100, 170], ())
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
        ///Calls the contract's `getGenesisEpoch` (0x51392fc0) function
        pub fn get_genesis_epoch(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([81, 57, 47, 192], ())
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
        ///Calls the contract's `lastVotingExecutedEpoch` (0xad81e244) function
        pub fn last_voting_executed_epoch(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([173, 129, 226, 68], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `leave` (0xd66d9e19) function
        pub fn leave(&self) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([214, 109, 158, 25], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `majorityPercentage` (0x599c7bd1) function
        pub fn majority_percentage(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u8> {
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
        ///Calls the contract's `reward` (0xa9fb763c) function
        pub fn reward(
            &self,
            amount: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([169, 251, 118, 60], amount)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `stake` (0x26476204) function
        pub fn stake(
            &self,
            p0: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([38, 71, 98, 4], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `status` (0x200d2ed2) function
        pub fn status(&self) -> ::ethers::contract::builders::ContractCall<M, u8> {
            self.0
                .method_hash([32, 13, 46, 210], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `submissionPeriod` (0x185fde7e) function
        pub fn submission_period(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([24, 95, 222, 126], ())
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
        ///Calls the contract's `validatorNetAddresses` (0x23a35705) function
        pub fn validator_net_addresses(
            &self,
            p0: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::string::String> {
            self.0
                .method_hash([35, 163, 87, 5], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `validatorWorkerAddresses` (0xd4e25bab) function
        pub fn validator_worker_addresses(
            &self,
            p0: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (u8, ::ethers::core::types::Bytes),
        > {
            self.0
                .method_hash([212, 226, 91, 171], p0)
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
    for SubnetActor<M> {
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
    ///Custom Error type `CallerHasNoStake` with signature `CallerHasNoStake()` and selector `0x5083a7f9`
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
    #[etherror(name = "CallerHasNoStake", abi = "CallerHasNoStake()")]
    pub struct CallerHasNoStake;
    ///Custom Error type `CheckpointNotChained` with signature `CheckpointNotChained()` and selector `0x85fdfc09`
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
    #[etherror(name = "CheckpointNotChained", abi = "CheckpointNotChained()")]
    pub struct CheckpointNotChained;
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
    ///Custom Error type `CollateralStillLockedInSubnet` with signature `CollateralStillLockedInSubnet()` and selector `0xcc91bf05`
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
        name = "CollateralStillLockedInSubnet",
        abi = "CollateralStillLockedInSubnet()"
    )]
    pub struct CollateralStillLockedInSubnet;
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
    ///Custom Error type `GatewayCannotBeZero` with signature `GatewayCannotBeZero()` and selector `0x8b3ddc33`
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
    #[etherror(name = "GatewayCannotBeZero", abi = "GatewayCannotBeZero()")]
    pub struct GatewayCannotBeZero;
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
    ///Custom Error type `NotAccount` with signature `NotAccount()` and selector `0xb7150de5`
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
    #[etherror(name = "NotAccount", abi = "NotAccount()")]
    pub struct NotAccount;
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
    ///Custom Error type `ReentrancyGuardReentrantCall` with signature `ReentrancyGuardReentrantCall()` and selector `0x3ee5aeb5`
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
        name = "ReentrancyGuardReentrantCall",
        abi = "ReentrancyGuardReentrantCall()"
    )]
    pub struct ReentrancyGuardReentrantCall;
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
    ///Custom Error type `WorkerAddressInvalid` with signature `WorkerAddressInvalid()` and selector `0x92b21e66`
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
    #[etherror(name = "WorkerAddressInvalid", abi = "WorkerAddressInvalid()")]
    pub struct WorkerAddressInvalid;
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
    pub enum SubnetActorErrors {
        AddressInsufficientBalance(AddressInsufficientBalance),
        CallerHasNoStake(CallerHasNoStake),
        CheckpointNotChained(CheckpointNotChained),
        CollateralIsZero(CollateralIsZero),
        CollateralStillLockedInSubnet(CollateralStillLockedInSubnet),
        EpochAlreadyExecuted(EpochAlreadyExecuted),
        EpochNotVotable(EpochNotVotable),
        FailedInnerCall(FailedInnerCall),
        GatewayCannotBeZero(GatewayCannotBeZero),
        InvalidMajorityPercentage(InvalidMajorityPercentage),
        MessagesNotSorted(MessagesNotSorted),
        NoRewardToWithdraw(NoRewardToWithdraw),
        NoValidatorsInSubnet(NoValidatorsInSubnet),
        NotAccount(NotAccount),
        NotAllValidatorsHaveLeft(NotAllValidatorsHaveLeft),
        NotEnoughBalanceForRewards(NotEnoughBalanceForRewards),
        NotGateway(NotGateway),
        NotValidator(NotValidator),
        ReentrancyGuardReentrantCall(ReentrancyGuardReentrantCall),
        SubnetAlreadyKilled(SubnetAlreadyKilled),
        SubnetNotActive(SubnetNotActive),
        ValidatorAlreadyVoted(ValidatorAlreadyVoted),
        WorkerAddressInvalid(WorkerAddressInvalid),
        WrongCheckpointSource(WrongCheckpointSource),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for SubnetActorErrors {
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
                = <CallerHasNoStake as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::CallerHasNoStake(decoded));
            }
            if let Ok(decoded)
                = <CheckpointNotChained as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::CheckpointNotChained(decoded));
            }
            if let Ok(decoded)
                = <CollateralIsZero as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::CollateralIsZero(decoded));
            }
            if let Ok(decoded)
                = <CollateralStillLockedInSubnet as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::CollateralStillLockedInSubnet(decoded));
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
                = <GatewayCannotBeZero as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GatewayCannotBeZero(decoded));
            }
            if let Ok(decoded)
                = <InvalidMajorityPercentage as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::InvalidMajorityPercentage(decoded));
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
                = <NotAccount as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NotAccount(decoded));
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
                = <ReentrancyGuardReentrantCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ReentrancyGuardReentrantCall(decoded));
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
                = <WorkerAddressInvalid as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::WorkerAddressInvalid(decoded));
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
    impl ::ethers::core::abi::AbiEncode for SubnetActorErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::AddressInsufficientBalance(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CallerHasNoStake(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CheckpointNotChained(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CollateralIsZero(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CollateralStillLockedInSubnet(element) => {
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
                Self::GatewayCannotBeZero(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidMajorityPercentage(element) => {
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
                Self::NotAccount(element) => {
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
                Self::ReentrancyGuardReentrantCall(element) => {
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
                Self::WorkerAddressInvalid(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::WrongCheckpointSource(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for SubnetActorErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <AddressInsufficientBalance as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CallerHasNoStake as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CheckpointNotChained as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CollateralIsZero as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CollateralStillLockedInSubnet as ::ethers::contract::EthError>::selector() => {
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
                    == <GatewayCannotBeZero as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidMajorityPercentage as ::ethers::contract::EthError>::selector() => {
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
                    == <NotAccount as ::ethers::contract::EthError>::selector() => true,
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
                    == <ReentrancyGuardReentrantCall as ::ethers::contract::EthError>::selector() => {
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
                    == <WorkerAddressInvalid as ::ethers::contract::EthError>::selector() => {
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
    impl ::core::fmt::Display for SubnetActorErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AddressInsufficientBalance(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CallerHasNoStake(element) => ::core::fmt::Display::fmt(element, f),
                Self::CheckpointNotChained(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CollateralIsZero(element) => ::core::fmt::Display::fmt(element, f),
                Self::CollateralStillLockedInSubnet(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::EpochAlreadyExecuted(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::EpochNotVotable(element) => ::core::fmt::Display::fmt(element, f),
                Self::FailedInnerCall(element) => ::core::fmt::Display::fmt(element, f),
                Self::GatewayCannotBeZero(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidMajorityPercentage(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::MessagesNotSorted(element) => ::core::fmt::Display::fmt(element, f),
                Self::NoRewardToWithdraw(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NoValidatorsInSubnet(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotAccount(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotAllValidatorsHaveLeft(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotEnoughBalanceForRewards(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotGateway(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotValidator(element) => ::core::fmt::Display::fmt(element, f),
                Self::ReentrancyGuardReentrantCall(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SubnetAlreadyKilled(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SubnetNotActive(element) => ::core::fmt::Display::fmt(element, f),
                Self::ValidatorAlreadyVoted(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::WorkerAddressInvalid(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::WrongCheckpointSource(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for SubnetActorErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<AddressInsufficientBalance> for SubnetActorErrors {
        fn from(value: AddressInsufficientBalance) -> Self {
            Self::AddressInsufficientBalance(value)
        }
    }
    impl ::core::convert::From<CallerHasNoStake> for SubnetActorErrors {
        fn from(value: CallerHasNoStake) -> Self {
            Self::CallerHasNoStake(value)
        }
    }
    impl ::core::convert::From<CheckpointNotChained> for SubnetActorErrors {
        fn from(value: CheckpointNotChained) -> Self {
            Self::CheckpointNotChained(value)
        }
    }
    impl ::core::convert::From<CollateralIsZero> for SubnetActorErrors {
        fn from(value: CollateralIsZero) -> Self {
            Self::CollateralIsZero(value)
        }
    }
    impl ::core::convert::From<CollateralStillLockedInSubnet> for SubnetActorErrors {
        fn from(value: CollateralStillLockedInSubnet) -> Self {
            Self::CollateralStillLockedInSubnet(value)
        }
    }
    impl ::core::convert::From<EpochAlreadyExecuted> for SubnetActorErrors {
        fn from(value: EpochAlreadyExecuted) -> Self {
            Self::EpochAlreadyExecuted(value)
        }
    }
    impl ::core::convert::From<EpochNotVotable> for SubnetActorErrors {
        fn from(value: EpochNotVotable) -> Self {
            Self::EpochNotVotable(value)
        }
    }
    impl ::core::convert::From<FailedInnerCall> for SubnetActorErrors {
        fn from(value: FailedInnerCall) -> Self {
            Self::FailedInnerCall(value)
        }
    }
    impl ::core::convert::From<GatewayCannotBeZero> for SubnetActorErrors {
        fn from(value: GatewayCannotBeZero) -> Self {
            Self::GatewayCannotBeZero(value)
        }
    }
    impl ::core::convert::From<InvalidMajorityPercentage> for SubnetActorErrors {
        fn from(value: InvalidMajorityPercentage) -> Self {
            Self::InvalidMajorityPercentage(value)
        }
    }
    impl ::core::convert::From<MessagesNotSorted> for SubnetActorErrors {
        fn from(value: MessagesNotSorted) -> Self {
            Self::MessagesNotSorted(value)
        }
    }
    impl ::core::convert::From<NoRewardToWithdraw> for SubnetActorErrors {
        fn from(value: NoRewardToWithdraw) -> Self {
            Self::NoRewardToWithdraw(value)
        }
    }
    impl ::core::convert::From<NoValidatorsInSubnet> for SubnetActorErrors {
        fn from(value: NoValidatorsInSubnet) -> Self {
            Self::NoValidatorsInSubnet(value)
        }
    }
    impl ::core::convert::From<NotAccount> for SubnetActorErrors {
        fn from(value: NotAccount) -> Self {
            Self::NotAccount(value)
        }
    }
    impl ::core::convert::From<NotAllValidatorsHaveLeft> for SubnetActorErrors {
        fn from(value: NotAllValidatorsHaveLeft) -> Self {
            Self::NotAllValidatorsHaveLeft(value)
        }
    }
    impl ::core::convert::From<NotEnoughBalanceForRewards> for SubnetActorErrors {
        fn from(value: NotEnoughBalanceForRewards) -> Self {
            Self::NotEnoughBalanceForRewards(value)
        }
    }
    impl ::core::convert::From<NotGateway> for SubnetActorErrors {
        fn from(value: NotGateway) -> Self {
            Self::NotGateway(value)
        }
    }
    impl ::core::convert::From<NotValidator> for SubnetActorErrors {
        fn from(value: NotValidator) -> Self {
            Self::NotValidator(value)
        }
    }
    impl ::core::convert::From<ReentrancyGuardReentrantCall> for SubnetActorErrors {
        fn from(value: ReentrancyGuardReentrantCall) -> Self {
            Self::ReentrancyGuardReentrantCall(value)
        }
    }
    impl ::core::convert::From<SubnetAlreadyKilled> for SubnetActorErrors {
        fn from(value: SubnetAlreadyKilled) -> Self {
            Self::SubnetAlreadyKilled(value)
        }
    }
    impl ::core::convert::From<SubnetNotActive> for SubnetActorErrors {
        fn from(value: SubnetNotActive) -> Self {
            Self::SubnetNotActive(value)
        }
    }
    impl ::core::convert::From<ValidatorAlreadyVoted> for SubnetActorErrors {
        fn from(value: ValidatorAlreadyVoted) -> Self {
            Self::ValidatorAlreadyVoted(value)
        }
    }
    impl ::core::convert::From<WorkerAddressInvalid> for SubnetActorErrors {
        fn from(value: WorkerAddressInvalid) -> Self {
            Self::WorkerAddressInvalid(value)
        }
    }
    impl ::core::convert::From<WrongCheckpointSource> for SubnetActorErrors {
        fn from(value: WrongCheckpointSource) -> Self {
            Self::WrongCheckpointSource(value)
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
    pub struct AccumulatedRewardsCall(pub ::ethers::core::types::Address);
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
    pub struct CommittedCheckpointsCall(pub u64);
    ///Container type for all input parameters for the `configurationNumber` function with signature `configurationNumber()` and selector `0x04fda3d4`
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
    #[ethcall(name = "configurationNumber", abi = "configurationNumber()")]
    pub struct ConfigurationNumberCall;
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
    ///Container type for all input parameters for the `currentSubnetHash` function with signature `currentSubnetHash()` and selector `0xc18f64aa`
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
    #[ethcall(name = "currentSubnetHash", abi = "currentSubnetHash()")]
    pub struct CurrentSubnetHashCall;
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
    ///Container type for all input parameters for the `getGenesisEpoch` function with signature `getGenesisEpoch()` and selector `0x51392fc0`
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
    #[ethcall(name = "getGenesisEpoch", abi = "getGenesisEpoch()")]
    pub struct GetGenesisEpochCall;
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
    pub struct StakeCall(pub ::ethers::core::types::Address);
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
    ///Container type for all input parameters for the `submissionPeriod` function with signature `submissionPeriod()` and selector `0x185fde7e`
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
    #[ethcall(name = "submissionPeriod", abi = "submissionPeriod()")]
    pub struct SubmissionPeriodCall;
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
    ///Container type for all input parameters for the `validatorNetAddresses` function with signature `validatorNetAddresses(address)` and selector `0x23a35705`
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
    #[ethcall(name = "validatorNetAddresses", abi = "validatorNetAddresses(address)")]
    pub struct ValidatorNetAddressesCall(pub ::ethers::core::types::Address);
    ///Container type for all input parameters for the `validatorWorkerAddresses` function with signature `validatorWorkerAddresses(address)` and selector `0xd4e25bab`
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
        name = "validatorWorkerAddresses",
        abi = "validatorWorkerAddresses(address)"
    )]
    pub struct ValidatorWorkerAddressesCall(pub ::ethers::core::types::Address);
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
    pub enum SubnetActorCalls {
        MinCheckpointPeriod(MinCheckpointPeriodCall),
        MinCollateralAmount(MinCollateralAmountCall),
        AccumulatedRewards(AccumulatedRewardsCall),
        BottomUpCheckPeriod(BottomUpCheckPeriodCall),
        BottomUpCheckpointAtEpoch(BottomUpCheckpointAtEpochCall),
        BottomUpCheckpointHashAtEpoch(BottomUpCheckpointHashAtEpochCall),
        CommittedCheckpoints(CommittedCheckpointsCall),
        ConfigurationNumber(ConfigurationNumberCall),
        Consensus(ConsensusCall),
        CurrentSubnetHash(CurrentSubnetHashCall),
        ExecutableQueue(ExecutableQueueCall),
        Genesis(GenesisCall),
        GetGenesisEpoch(GetGenesisEpochCall),
        GetParent(GetParentCall),
        GetValidatorSet(GetValidatorSetCall),
        GetValidators(GetValidatorsCall),
        HasValidatorVotedForSubmission(HasValidatorVotedForSubmissionCall),
        IpcGatewayAddr(IpcGatewayAddrCall),
        Join(JoinCall),
        Kill(KillCall),
        LastVotingExecutedEpoch(LastVotingExecutedEpochCall),
        Leave(LeaveCall),
        MajorityPercentage(MajorityPercentageCall),
        MinActivationCollateral(MinActivationCollateralCall),
        MinValidators(MinValidatorsCall),
        Name(NameCall),
        PrevExecutedCheckpointHash(PrevExecutedCheckpointHashCall),
        Reward(RewardCall),
        Stake(StakeCall),
        Status(StatusCall),
        SubmissionPeriod(SubmissionPeriodCall),
        SubmitCheckpoint(SubmitCheckpointCall),
        TopDownCheckPeriod(TopDownCheckPeriodCall),
        TotalStake(TotalStakeCall),
        ValidatorAt(ValidatorAtCall),
        ValidatorCount(ValidatorCountCall),
        ValidatorNetAddresses(ValidatorNetAddressesCall),
        ValidatorWorkerAddresses(ValidatorWorkerAddressesCall),
        Withdraw(WithdrawCall),
    }
    impl ::ethers::core::abi::AbiDecode for SubnetActorCalls {
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
                = <CommittedCheckpointsCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::CommittedCheckpoints(decoded));
            }
            if let Ok(decoded)
                = <ConfigurationNumberCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ConfigurationNumber(decoded));
            }
            if let Ok(decoded)
                = <ConsensusCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Consensus(decoded));
            }
            if let Ok(decoded)
                = <CurrentSubnetHashCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::CurrentSubnetHash(decoded));
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
                = <GetGenesisEpochCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetGenesisEpoch(decoded));
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
                = <HasValidatorVotedForSubmissionCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::HasValidatorVotedForSubmission(decoded));
            }
            if let Ok(decoded)
                = <IpcGatewayAddrCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::IpcGatewayAddr(decoded));
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
                = <LastVotingExecutedEpochCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::LastVotingExecutedEpoch(decoded));
            }
            if let Ok(decoded)
                = <LeaveCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Leave(decoded));
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
                = <RewardCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Reward(decoded));
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
                = <SubmissionPeriodCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::SubmissionPeriod(decoded));
            }
            if let Ok(decoded)
                = <SubmitCheckpointCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::SubmitCheckpoint(decoded));
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
            if let Ok(decoded)
                = <ValidatorNetAddressesCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ValidatorNetAddresses(decoded));
            }
            if let Ok(decoded)
                = <ValidatorWorkerAddressesCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ValidatorWorkerAddresses(decoded));
            }
            if let Ok(decoded)
                = <WithdrawCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Withdraw(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for SubnetActorCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::MinCheckpointPeriod(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MinCollateralAmount(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
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
                Self::CommittedCheckpoints(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ConfigurationNumber(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Consensus(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CurrentSubnetHash(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ExecutableQueue(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Genesis(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetGenesisEpoch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetParent(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetValidatorSet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetValidators(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::HasValidatorVotedForSubmission(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::IpcGatewayAddr(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Join(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Kill(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::LastVotingExecutedEpoch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Leave(element) => ::ethers::core::abi::AbiEncode::encode(element),
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
                Self::Reward(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Stake(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Status(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::SubmissionPeriod(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SubmitCheckpoint(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
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
                Self::ValidatorNetAddresses(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ValidatorWorkerAddresses(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Withdraw(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for SubnetActorCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::MinCheckpointPeriod(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::MinCollateralAmount(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
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
                Self::CommittedCheckpoints(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ConfigurationNumber(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::Consensus(element) => ::core::fmt::Display::fmt(element, f),
                Self::CurrentSubnetHash(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExecutableQueue(element) => ::core::fmt::Display::fmt(element, f),
                Self::Genesis(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetGenesisEpoch(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetParent(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetValidatorSet(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetValidators(element) => ::core::fmt::Display::fmt(element, f),
                Self::HasValidatorVotedForSubmission(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::IpcGatewayAddr(element) => ::core::fmt::Display::fmt(element, f),
                Self::Join(element) => ::core::fmt::Display::fmt(element, f),
                Self::Kill(element) => ::core::fmt::Display::fmt(element, f),
                Self::LastVotingExecutedEpoch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::Leave(element) => ::core::fmt::Display::fmt(element, f),
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
                Self::Reward(element) => ::core::fmt::Display::fmt(element, f),
                Self::Stake(element) => ::core::fmt::Display::fmt(element, f),
                Self::Status(element) => ::core::fmt::Display::fmt(element, f),
                Self::SubmissionPeriod(element) => ::core::fmt::Display::fmt(element, f),
                Self::SubmitCheckpoint(element) => ::core::fmt::Display::fmt(element, f),
                Self::TopDownCheckPeriod(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TotalStake(element) => ::core::fmt::Display::fmt(element, f),
                Self::ValidatorAt(element) => ::core::fmt::Display::fmt(element, f),
                Self::ValidatorCount(element) => ::core::fmt::Display::fmt(element, f),
                Self::ValidatorNetAddresses(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ValidatorWorkerAddresses(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::Withdraw(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<MinCheckpointPeriodCall> for SubnetActorCalls {
        fn from(value: MinCheckpointPeriodCall) -> Self {
            Self::MinCheckpointPeriod(value)
        }
    }
    impl ::core::convert::From<MinCollateralAmountCall> for SubnetActorCalls {
        fn from(value: MinCollateralAmountCall) -> Self {
            Self::MinCollateralAmount(value)
        }
    }
    impl ::core::convert::From<AccumulatedRewardsCall> for SubnetActorCalls {
        fn from(value: AccumulatedRewardsCall) -> Self {
            Self::AccumulatedRewards(value)
        }
    }
    impl ::core::convert::From<BottomUpCheckPeriodCall> for SubnetActorCalls {
        fn from(value: BottomUpCheckPeriodCall) -> Self {
            Self::BottomUpCheckPeriod(value)
        }
    }
    impl ::core::convert::From<BottomUpCheckpointAtEpochCall> for SubnetActorCalls {
        fn from(value: BottomUpCheckpointAtEpochCall) -> Self {
            Self::BottomUpCheckpointAtEpoch(value)
        }
    }
    impl ::core::convert::From<BottomUpCheckpointHashAtEpochCall> for SubnetActorCalls {
        fn from(value: BottomUpCheckpointHashAtEpochCall) -> Self {
            Self::BottomUpCheckpointHashAtEpoch(value)
        }
    }
    impl ::core::convert::From<CommittedCheckpointsCall> for SubnetActorCalls {
        fn from(value: CommittedCheckpointsCall) -> Self {
            Self::CommittedCheckpoints(value)
        }
    }
    impl ::core::convert::From<ConfigurationNumberCall> for SubnetActorCalls {
        fn from(value: ConfigurationNumberCall) -> Self {
            Self::ConfigurationNumber(value)
        }
    }
    impl ::core::convert::From<ConsensusCall> for SubnetActorCalls {
        fn from(value: ConsensusCall) -> Self {
            Self::Consensus(value)
        }
    }
    impl ::core::convert::From<CurrentSubnetHashCall> for SubnetActorCalls {
        fn from(value: CurrentSubnetHashCall) -> Self {
            Self::CurrentSubnetHash(value)
        }
    }
    impl ::core::convert::From<ExecutableQueueCall> for SubnetActorCalls {
        fn from(value: ExecutableQueueCall) -> Self {
            Self::ExecutableQueue(value)
        }
    }
    impl ::core::convert::From<GenesisCall> for SubnetActorCalls {
        fn from(value: GenesisCall) -> Self {
            Self::Genesis(value)
        }
    }
    impl ::core::convert::From<GetGenesisEpochCall> for SubnetActorCalls {
        fn from(value: GetGenesisEpochCall) -> Self {
            Self::GetGenesisEpoch(value)
        }
    }
    impl ::core::convert::From<GetParentCall> for SubnetActorCalls {
        fn from(value: GetParentCall) -> Self {
            Self::GetParent(value)
        }
    }
    impl ::core::convert::From<GetValidatorSetCall> for SubnetActorCalls {
        fn from(value: GetValidatorSetCall) -> Self {
            Self::GetValidatorSet(value)
        }
    }
    impl ::core::convert::From<GetValidatorsCall> for SubnetActorCalls {
        fn from(value: GetValidatorsCall) -> Self {
            Self::GetValidators(value)
        }
    }
    impl ::core::convert::From<HasValidatorVotedForSubmissionCall> for SubnetActorCalls {
        fn from(value: HasValidatorVotedForSubmissionCall) -> Self {
            Self::HasValidatorVotedForSubmission(value)
        }
    }
    impl ::core::convert::From<IpcGatewayAddrCall> for SubnetActorCalls {
        fn from(value: IpcGatewayAddrCall) -> Self {
            Self::IpcGatewayAddr(value)
        }
    }
    impl ::core::convert::From<JoinCall> for SubnetActorCalls {
        fn from(value: JoinCall) -> Self {
            Self::Join(value)
        }
    }
    impl ::core::convert::From<KillCall> for SubnetActorCalls {
        fn from(value: KillCall) -> Self {
            Self::Kill(value)
        }
    }
    impl ::core::convert::From<LastVotingExecutedEpochCall> for SubnetActorCalls {
        fn from(value: LastVotingExecutedEpochCall) -> Self {
            Self::LastVotingExecutedEpoch(value)
        }
    }
    impl ::core::convert::From<LeaveCall> for SubnetActorCalls {
        fn from(value: LeaveCall) -> Self {
            Self::Leave(value)
        }
    }
    impl ::core::convert::From<MajorityPercentageCall> for SubnetActorCalls {
        fn from(value: MajorityPercentageCall) -> Self {
            Self::MajorityPercentage(value)
        }
    }
    impl ::core::convert::From<MinActivationCollateralCall> for SubnetActorCalls {
        fn from(value: MinActivationCollateralCall) -> Self {
            Self::MinActivationCollateral(value)
        }
    }
    impl ::core::convert::From<MinValidatorsCall> for SubnetActorCalls {
        fn from(value: MinValidatorsCall) -> Self {
            Self::MinValidators(value)
        }
    }
    impl ::core::convert::From<NameCall> for SubnetActorCalls {
        fn from(value: NameCall) -> Self {
            Self::Name(value)
        }
    }
    impl ::core::convert::From<PrevExecutedCheckpointHashCall> for SubnetActorCalls {
        fn from(value: PrevExecutedCheckpointHashCall) -> Self {
            Self::PrevExecutedCheckpointHash(value)
        }
    }
    impl ::core::convert::From<RewardCall> for SubnetActorCalls {
        fn from(value: RewardCall) -> Self {
            Self::Reward(value)
        }
    }
    impl ::core::convert::From<StakeCall> for SubnetActorCalls {
        fn from(value: StakeCall) -> Self {
            Self::Stake(value)
        }
    }
    impl ::core::convert::From<StatusCall> for SubnetActorCalls {
        fn from(value: StatusCall) -> Self {
            Self::Status(value)
        }
    }
    impl ::core::convert::From<SubmissionPeriodCall> for SubnetActorCalls {
        fn from(value: SubmissionPeriodCall) -> Self {
            Self::SubmissionPeriod(value)
        }
    }
    impl ::core::convert::From<SubmitCheckpointCall> for SubnetActorCalls {
        fn from(value: SubmitCheckpointCall) -> Self {
            Self::SubmitCheckpoint(value)
        }
    }
    impl ::core::convert::From<TopDownCheckPeriodCall> for SubnetActorCalls {
        fn from(value: TopDownCheckPeriodCall) -> Self {
            Self::TopDownCheckPeriod(value)
        }
    }
    impl ::core::convert::From<TotalStakeCall> for SubnetActorCalls {
        fn from(value: TotalStakeCall) -> Self {
            Self::TotalStake(value)
        }
    }
    impl ::core::convert::From<ValidatorAtCall> for SubnetActorCalls {
        fn from(value: ValidatorAtCall) -> Self {
            Self::ValidatorAt(value)
        }
    }
    impl ::core::convert::From<ValidatorCountCall> for SubnetActorCalls {
        fn from(value: ValidatorCountCall) -> Self {
            Self::ValidatorCount(value)
        }
    }
    impl ::core::convert::From<ValidatorNetAddressesCall> for SubnetActorCalls {
        fn from(value: ValidatorNetAddressesCall) -> Self {
            Self::ValidatorNetAddresses(value)
        }
    }
    impl ::core::convert::From<ValidatorWorkerAddressesCall> for SubnetActorCalls {
        fn from(value: ValidatorWorkerAddressesCall) -> Self {
            Self::ValidatorWorkerAddresses(value)
        }
    }
    impl ::core::convert::From<WithdrawCall> for SubnetActorCalls {
        fn from(value: WithdrawCall) -> Self {
            Self::Withdraw(value)
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
    ///Container type for all return fields from the `configurationNumber` function with signature `configurationNumber()` and selector `0x04fda3d4`
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
    pub struct ConfigurationNumberReturn(pub u64);
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
    ///Container type for all return fields from the `currentSubnetHash` function with signature `currentSubnetHash()` and selector `0xc18f64aa`
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
    pub struct CurrentSubnetHashReturn(pub [u8; 32]);
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
    pub struct ExecutableQueueReturn {
        pub period: u64,
        pub first: u64,
        pub last: u64,
    }
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
    ///Container type for all return fields from the `getGenesisEpoch` function with signature `getGenesisEpoch()` and selector `0x51392fc0`
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
    pub struct GetGenesisEpochReturn(pub u64);
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
    pub struct MajorityPercentageReturn(pub u8);
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
    ///Container type for all return fields from the `submissionPeriod` function with signature `submissionPeriod()` and selector `0x185fde7e`
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
    pub struct SubmissionPeriodReturn(pub u64);
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
    ///Container type for all return fields from the `validatorNetAddresses` function with signature `validatorNetAddresses(address)` and selector `0x23a35705`
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
    pub struct ValidatorNetAddressesReturn(pub ::std::string::String);
    ///Container type for all return fields from the `validatorWorkerAddresses` function with signature `validatorWorkerAddresses(address)` and selector `0xd4e25bab`
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
    pub struct ValidatorWorkerAddressesReturn {
        pub addr_type: u8,
        pub payload: ::ethers::core::types::Bytes,
    }
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
