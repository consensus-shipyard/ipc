pub use gateway::*;
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
pub mod gateway {
    #[rustfmt::skip]
    const __ABI: &str = "[{\"inputs\":[{\"internalType\":\"struct Gateway.ConstructorParams\",\"name\":\"params\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"networkName\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"bottomUpCheckPeriod\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"topDownCheckPeriod\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"msgFee\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint8\",\"name\":\"majorityPercentage\",\"type\":\"uint8\",\"components\":[]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"constructor\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"target\",\"type\":\"address\",\"components\":[]}],\"type\":\"error\",\"name\":\"AddressEmptyCode\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"account\",\"type\":\"address\",\"components\":[]}],\"type\":\"error\",\"name\":\"AddressInsufficientBalance\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"AlreadyInitialized\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"AlreadyRegisteredSubnet\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"CallFailed\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"CannotReleaseZero\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"CannotSendCrossMsgToItself\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"EmptySubnet\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"EpochAlreadyExecuted\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"EpochNotVotable\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"FailedInnerCall\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InconsistentPrevCheckpoint\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InsufficientFunds\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidActorAddress\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidCheckpointEpoch\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidCheckpointSource\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidCrossMsgDestinationSubnet\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidCrossMsgFromSubnetId\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidCrossMsgNonce\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidCrossMsgsSortOrder\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"InvalidMajorityPercentage\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"MessagesNotSorted\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"MethodNotSupportedYet\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotEmptySubnetCircSupply\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotEnoughBalance\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotEnoughFee\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotEnoughFunds\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotEnoughFundsForMembership\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotEnoughFundsToRelease\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotEnoughSubnetCircSupply\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotInitialized\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotRegisteredSubnet\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotSignableAccount\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotSystemActor\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"NotValidator\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"PostboxNotExist\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"ReentrancyGuardReentrantCall\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"SubnetNotActive\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"ValidatorAlreadyVoted\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"ValidatorWeightIsZero\",\"outputs\":[]},{\"inputs\":[],\"type\":\"error\",\"name\":\"ValidatorsAndWeightsLengthMismatch\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"MIN_CHECKPOINT_PERIOD\",\"outputs\":[{\"internalType\":\"uint8\",\"name\":\"\",\"type\":\"uint8\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"MIN_COLLATERAL_AMOUNT\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"payable\",\"type\":\"function\",\"name\":\"addStake\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"appliedTopDownNonce\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"bottomUpCheckPeriod\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"bottomUpCheckpointAtEpoch\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"exists\",\"type\":\"bool\",\"components\":[]},{\"internalType\":\"struct BottomUpCheckpoint\",\"name\":\"checkpoint\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"fee\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"struct CrossMsg[]\",\"name\":\"crossMsgs\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct StorableMsg\",\"name\":\"message\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct IPCAddress\",\"name\":\"from\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"struct IPCAddress\",\"name\":\"to\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes4\",\"name\":\"method\",\"type\":\"bytes4\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"params\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"bool\",\"name\":\"wrapped\",\"type\":\"bool\",\"components\":[]}]},{\"internalType\":\"struct ChildCheck[]\",\"name\":\"children\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"bytes32[]\",\"name\":\"checks\",\"type\":\"bytes32[]\",\"components\":[]}]},{\"internalType\":\"bytes32\",\"name\":\"prevHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\",\"components\":[]}]}]},{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"bottomUpCheckpointHashAtEpoch\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"bottomUpCheckpoints\",\"outputs\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"fee\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"bytes32\",\"name\":\"prevHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"bottomUpNonce\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"struct BottomUpCheckpoint\",\"name\":\"commit\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"fee\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"struct CrossMsg[]\",\"name\":\"crossMsgs\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct StorableMsg\",\"name\":\"message\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct IPCAddress\",\"name\":\"from\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"struct IPCAddress\",\"name\":\"to\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes4\",\"name\":\"method\",\"type\":\"bytes4\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"params\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"bool\",\"name\":\"wrapped\",\"type\":\"bool\",\"components\":[]}]},{\"internalType\":\"struct ChildCheck[]\",\"name\":\"children\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"bytes32[]\",\"name\":\"checks\",\"type\":\"bytes32[]\",\"components\":[]}]},{\"internalType\":\"bytes32\",\"name\":\"prevHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\",\"components\":[]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"commitChildCheck\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"crossMsgFee\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"executableQueue\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"period\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"first\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"last\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"to\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}],\"stateMutability\":\"payable\",\"type\":\"function\",\"name\":\"fund\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getGenesisEpoch\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getNetworkName\",\"outputs\":[{\"internalType\":\"struct SubnetID\",\"name\":\"\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]}]},{\"inputs\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getSubnet\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]},{\"internalType\":\"struct Subnet\",\"name\":\"\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"enum Status\",\"name\":\"status\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"topDownNonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"appliedBottomUpNonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"stake\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"genesisEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"circSupply\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"struct SubnetID\",\"name\":\"id\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct BottomUpCheckpoint\",\"name\":\"prevCheckpoint\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"fee\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"struct CrossMsg[]\",\"name\":\"crossMsgs\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct StorableMsg\",\"name\":\"message\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct IPCAddress\",\"name\":\"from\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"struct IPCAddress\",\"name\":\"to\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes4\",\"name\":\"method\",\"type\":\"bytes4\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"params\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"bool\",\"name\":\"wrapped\",\"type\":\"bool\",\"components\":[]}]},{\"internalType\":\"struct ChildCheck[]\",\"name\":\"children\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"bytes32[]\",\"name\":\"checks\",\"type\":\"bytes32[]\",\"components\":[]}]},{\"internalType\":\"bytes32\",\"name\":\"prevHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"struct CrossMsg[]\",\"name\":\"topDownMsgs\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct StorableMsg\",\"name\":\"message\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct IPCAddress\",\"name\":\"from\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"struct IPCAddress\",\"name\":\"to\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes4\",\"name\":\"method\",\"type\":\"bytes4\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"params\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"bool\",\"name\":\"wrapped\",\"type\":\"bool\",\"components\":[]}]}]}]},{\"inputs\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"uint256\",\"name\":\"index\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getSubnetTopDownMsg\",\"outputs\":[{\"internalType\":\"struct CrossMsg\",\"name\":\"\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct StorableMsg\",\"name\":\"message\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct IPCAddress\",\"name\":\"from\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"struct IPCAddress\",\"name\":\"to\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes4\",\"name\":\"method\",\"type\":\"bytes4\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"params\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"bool\",\"name\":\"wrapped\",\"type\":\"bool\",\"components\":[]}]}]},{\"inputs\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getSubnetTopDownMsgsLength\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"fromNonce\",\"type\":\"uint64\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"getTopDownMsgs\",\"outputs\":[{\"internalType\":\"struct CrossMsg[]\",\"name\":\"\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct StorableMsg\",\"name\":\"message\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct IPCAddress\",\"name\":\"from\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"struct IPCAddress\",\"name\":\"to\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes4\",\"name\":\"method\",\"type\":\"bytes4\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"params\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"bool\",\"name\":\"wrapped\",\"type\":\"bool\",\"components\":[]}]}]},{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"submitter\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"hasValidatorVotedForSubmission\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint64\",\"name\":\"genesisEpoch\",\"type\":\"uint64\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"initGenesisEpoch\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"initialized\",\"outputs\":[{\"internalType\":\"bool\",\"name\":\"\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"kill\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"lastVotingExecutedEpoch\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"listSubnets\",\"outputs\":[{\"internalType\":\"struct Subnet[]\",\"name\":\"\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"enum Status\",\"name\":\"status\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"topDownNonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"appliedBottomUpNonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"stake\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"genesisEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"circSupply\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"struct SubnetID\",\"name\":\"id\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct BottomUpCheckpoint\",\"name\":\"prevCheckpoint\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"fee\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"struct CrossMsg[]\",\"name\":\"crossMsgs\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct StorableMsg\",\"name\":\"message\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct IPCAddress\",\"name\":\"from\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"struct IPCAddress\",\"name\":\"to\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes4\",\"name\":\"method\",\"type\":\"bytes4\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"params\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"bool\",\"name\":\"wrapped\",\"type\":\"bool\",\"components\":[]}]},{\"internalType\":\"struct ChildCheck[]\",\"name\":\"children\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"bytes32[]\",\"name\":\"checks\",\"type\":\"bytes32[]\",\"components\":[]}]},{\"internalType\":\"bytes32\",\"name\":\"prevHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"struct CrossMsg[]\",\"name\":\"topDownMsgs\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct StorableMsg\",\"name\":\"message\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct IPCAddress\",\"name\":\"from\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"struct IPCAddress\",\"name\":\"to\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes4\",\"name\":\"method\",\"type\":\"bytes4\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"params\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"bool\",\"name\":\"wrapped\",\"type\":\"bool\",\"components\":[]}]}]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"majorityPercentage\",\"outputs\":[{\"internalType\":\"uint8\",\"name\":\"\",\"type\":\"uint8\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"minStake\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"postbox\",\"outputs\":[{\"internalType\":\"struct StorableMsg\",\"name\":\"message\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct IPCAddress\",\"name\":\"from\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"struct IPCAddress\",\"name\":\"to\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes4\",\"name\":\"method\",\"type\":\"bytes4\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"params\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"bool\",\"name\":\"wrapped\",\"type\":\"bool\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"payable\",\"type\":\"function\",\"name\":\"register\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"struct FvmAddress\",\"name\":\"to\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}],\"stateMutability\":\"payable\",\"type\":\"function\",\"name\":\"release\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"releaseRewards\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"amount\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"releaseStake\",\"outputs\":[]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"submissionPeriod\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"struct TopDownCheckpoint\",\"name\":\"checkpoint\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"struct CrossMsg[]\",\"name\":\"topDownMsgs\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct StorableMsg\",\"name\":\"message\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct IPCAddress\",\"name\":\"from\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"struct IPCAddress\",\"name\":\"to\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes4\",\"name\":\"method\",\"type\":\"bytes4\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"params\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"bool\",\"name\":\"wrapped\",\"type\":\"bool\",\"components\":[]}]}]}],\"stateMutability\":\"nonpayable\",\"type\":\"function\",\"name\":\"submitTopDownCheckpoint\",\"outputs\":[]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"subnetKeys\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"bytes32\",\"name\":\"\",\"type\":\"bytes32\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"subnets\",\"outputs\":[{\"internalType\":\"enum Status\",\"name\":\"status\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"topDownNonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"appliedBottomUpNonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"stake\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"genesisEpoch\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"circSupply\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"struct SubnetID\",\"name\":\"id\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct BottomUpCheckpoint\",\"name\":\"prevCheckpoint\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"uint64\",\"name\":\"epoch\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"uint256\",\"name\":\"fee\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"struct CrossMsg[]\",\"name\":\"crossMsgs\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct StorableMsg\",\"name\":\"message\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct IPCAddress\",\"name\":\"from\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"struct IPCAddress\",\"name\":\"to\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"subnetId\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"struct FvmAddress\",\"name\":\"rawAddress\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint8\",\"name\":\"addrType\",\"type\":\"uint8\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"payload\",\"type\":\"bytes\",\"components\":[]}]}]},{\"internalType\":\"uint256\",\"name\":\"value\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"uint64\",\"name\":\"nonce\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"bytes4\",\"name\":\"method\",\"type\":\"bytes4\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"params\",\"type\":\"bytes\",\"components\":[]}]},{\"internalType\":\"bool\",\"name\":\"wrapped\",\"type\":\"bool\",\"components\":[]}]},{\"internalType\":\"struct ChildCheck[]\",\"name\":\"children\",\"type\":\"tuple[]\",\"components\":[{\"internalType\":\"struct SubnetID\",\"name\":\"source\",\"type\":\"tuple\",\"components\":[{\"internalType\":\"uint64\",\"name\":\"root\",\"type\":\"uint64\",\"components\":[]},{\"internalType\":\"address[]\",\"name\":\"route\",\"type\":\"address[]\",\"components\":[]}]},{\"internalType\":\"bytes32[]\",\"name\":\"checks\",\"type\":\"bytes32[]\",\"components\":[]}]},{\"internalType\":\"bytes32\",\"name\":\"prevHash\",\"type\":\"bytes32\",\"components\":[]},{\"internalType\":\"bytes\",\"name\":\"proof\",\"type\":\"bytes\",\"components\":[]}]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"topDownCheckPeriod\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalSubnets\",\"outputs\":[{\"internalType\":\"uint64\",\"name\":\"\",\"type\":\"uint64\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"totalWeight\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"validatorNonce\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]},{\"internalType\":\"address\",\"name\":\"\",\"type\":\"address\",\"components\":[]}],\"stateMutability\":\"view\",\"type\":\"function\",\"name\":\"validatorSet\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"\",\"type\":\"uint256\",\"components\":[]}]}]";
    ///The parsed JSON ABI of the contract.
    pub static GATEWAY_ABI: ::ethers::contract::Lazy<::ethers::core::abi::Abi> = ::ethers::contract::Lazy::new(||
    ::ethers::core::utils::__serde_json::from_str(__ABI).expect("ABI is always valid"));
    pub struct Gateway<M>(::ethers::contract::Contract<M>);
    impl<M> ::core::clone::Clone for Gateway<M> {
        fn clone(&self) -> Self {
            Self(::core::clone::Clone::clone(&self.0))
        }
    }
    impl<M> ::core::ops::Deref for Gateway<M> {
        type Target = ::ethers::contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> ::core::ops::DerefMut for Gateway<M> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl<M> ::core::fmt::Debug for Gateway<M> {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            f.debug_tuple(stringify!(Gateway)).field(&self.address()).finish()
        }
    }
    impl<M: ::ethers::providers::Middleware> Gateway<M> {
        /// Creates a new contract instance with the specified `ethers` client at
        /// `address`. The contract derefs to a `ethers::Contract` object.
        pub fn new<T: Into<::ethers::core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            Self(
                ::ethers::contract::Contract::new(
                    address.into(),
                    GATEWAY_ABI.clone(),
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
        ///Calls the contract's `addStake` (0x5a627dbc) function
        pub fn add_stake(&self) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([90, 98, 125, 188], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `appliedTopDownNonce` (0x8789f83b) function
        pub fn applied_top_down_nonce(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([135, 137, 248, 59], ())
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
        ///Calls the contract's `bottomUpCheckpoints` (0x2cc14ea2) function
        pub fn bottom_up_checkpoints(
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
                .method_hash([44, 193, 78, 162], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `bottomUpNonce` (0x41b6a2e8) function
        pub fn bottom_up_nonce(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([65, 182, 162, 232], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `commitChildCheck` (0xd4e149a8) function
        pub fn commit_child_check(
            &self,
            commit: BottomUpCheckpoint,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([212, 225, 73, 168], (commit,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `crossMsgFee` (0x24729425) function
        pub fn cross_msg_fee(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([36, 114, 148, 37], ())
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
        ///Calls the contract's `fund` (0x18f44b70) function
        pub fn fund(
            &self,
            subnet_id: SubnetID,
            to: FvmAddress,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([24, 244, 75, 112], (subnet_id, to))
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
        ///Calls the contract's `getNetworkName` (0x94074b03) function
        pub fn get_network_name(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, SubnetID> {
            self.0
                .method_hash([148, 7, 75, 3], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getSubnet` (0xc66c66a1) function
        pub fn get_subnet(
            &self,
            subnet_id: SubnetID,
        ) -> ::ethers::contract::builders::ContractCall<M, (bool, Subnet)> {
            self.0
                .method_hash([198, 108, 102, 161], (subnet_id,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getSubnetTopDownMsg` (0x0ea746f2) function
        pub fn get_subnet_top_down_msg(
            &self,
            subnet_id: SubnetID,
            index: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, CrossMsg> {
            self.0
                .method_hash([14, 167, 70, 242], (subnet_id, index))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getSubnetTopDownMsgsLength` (0x9d3070b5) function
        pub fn get_subnet_top_down_msgs_length(
            &self,
            subnet_id: SubnetID,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([157, 48, 112, 181], (subnet_id,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `getTopDownMsgs` (0x13549315) function
        pub fn get_top_down_msgs(
            &self,
            subnet_id: SubnetID,
            from_nonce: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::vec::Vec<CrossMsg>> {
            self.0
                .method_hash([19, 84, 147, 21], (subnet_id, from_nonce))
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
        ///Calls the contract's `initGenesisEpoch` (0x13f35388) function
        pub fn init_genesis_epoch(
            &self,
            genesis_epoch: u64,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([19, 243, 83, 136], genesis_epoch)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `initialized` (0x158ef93e) function
        pub fn initialized(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, bool> {
            self.0
                .method_hash([21, 142, 249, 62], ())
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
        ///Calls the contract's `listSubnets` (0x5d029685) function
        pub fn list_subnets(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::std::vec::Vec<Subnet>> {
            self.0
                .method_hash([93, 2, 150, 133], ())
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
        ///Calls the contract's `minStake` (0x375b3c0a) function
        pub fn min_stake(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([55, 91, 60, 10], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `postbox` (0x8cfd78e7) function
        pub fn postbox(
            &self,
            p0: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<M, (StorableMsg, bool)> {
            self.0
                .method_hash([140, 253, 120, 231], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `register` (0x1aa3a008) function
        pub fn register(&self) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([26, 163, 160, 8], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `release` (0x6b2c1eef) function
        pub fn release(
            &self,
            to: FvmAddress,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([107, 44, 30, 239], (to,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `releaseRewards` (0xf8703bb8) function
        pub fn release_rewards(
            &self,
            amount: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([248, 112, 59, 184], amount)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `releaseStake` (0x45f54485) function
        pub fn release_stake(
            &self,
            amount: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([69, 245, 68, 133], amount)
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
        ///Calls the contract's `submitTopDownCheckpoint` (0x986acf38) function
        pub fn submit_top_down_checkpoint(
            &self,
            checkpoint: TopDownCheckpoint,
        ) -> ::ethers::contract::builders::ContractCall<M, ()> {
            self.0
                .method_hash([152, 106, 207, 56], (checkpoint,))
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `subnetKeys` (0x548b3b38) function
        pub fn subnet_keys(
            &self,
            p0: ::ethers::core::types::U256,
        ) -> ::ethers::contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([84, 139, 59, 56], p0)
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `subnets` (0x02e30f9a) function
        pub fn subnets(
            &self,
            p0: [u8; 32],
        ) -> ::ethers::contract::builders::ContractCall<
            M,
            (
                u8,
                u64,
                u64,
                ::ethers::core::types::U256,
                ::ethers::core::types::U256,
                ::ethers::core::types::U256,
                SubnetID,
                BottomUpCheckpoint,
            ),
        > {
            self.0
                .method_hash([2, 227, 15, 154], p0)
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
        ///Calls the contract's `totalSubnets` (0xa2b67158) function
        pub fn total_subnets(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, u64> {
            self.0
                .method_hash([162, 182, 113, 88], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `totalWeight` (0x96c82e57) function
        pub fn total_weight(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([150, 200, 46, 87], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `validatorNonce` (0xe17a684f) function
        pub fn validator_nonce(
            &self,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([225, 122, 104, 79], ())
                .expect("method not found (this should never happen)")
        }
        ///Calls the contract's `validatorSet` (0x223d9056) function
        pub fn validator_set(
            &self,
            p0: ::ethers::core::types::U256,
            p1: ::ethers::core::types::Address,
        ) -> ::ethers::contract::builders::ContractCall<M, ::ethers::core::types::U256> {
            self.0
                .method_hash([34, 61, 144, 86], (p0, p1))
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ::ethers::providers::Middleware> From<::ethers::contract::Contract<M>>
    for Gateway<M> {
        fn from(contract: ::ethers::contract::Contract<M>) -> Self {
            Self::new(contract.address(), contract.client())
        }
    }
    ///Custom Error type `AddressEmptyCode` with signature `AddressEmptyCode(address)` and selector `0x9996b315`
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
    #[etherror(name = "AddressEmptyCode", abi = "AddressEmptyCode(address)")]
    pub struct AddressEmptyCode {
        pub target: ::ethers::core::types::Address,
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
    ///Custom Error type `AlreadyInitialized` with signature `AlreadyInitialized()` and selector `0x0dc149f0`
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
    #[etherror(name = "AlreadyInitialized", abi = "AlreadyInitialized()")]
    pub struct AlreadyInitialized;
    ///Custom Error type `AlreadyRegisteredSubnet` with signature `AlreadyRegisteredSubnet()` and selector `0x36a719be`
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
    #[etherror(name = "AlreadyRegisteredSubnet", abi = "AlreadyRegisteredSubnet()")]
    pub struct AlreadyRegisteredSubnet;
    ///Custom Error type `CallFailed` with signature `CallFailed()` and selector `0x3204506f`
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
    #[etherror(name = "CallFailed", abi = "CallFailed()")]
    pub struct CallFailed;
    ///Custom Error type `CannotReleaseZero` with signature `CannotReleaseZero()` and selector `0xc79cad7b`
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
    #[etherror(name = "CannotReleaseZero", abi = "CannotReleaseZero()")]
    pub struct CannotReleaseZero;
    ///Custom Error type `CannotSendCrossMsgToItself` with signature `CannotSendCrossMsgToItself()` and selector `0xbcccd7fc`
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
        name = "CannotSendCrossMsgToItself",
        abi = "CannotSendCrossMsgToItself()"
    )]
    pub struct CannotSendCrossMsgToItself;
    ///Custom Error type `EmptySubnet` with signature `EmptySubnet()` and selector `0x79e7ca82`
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
    #[etherror(name = "EmptySubnet", abi = "EmptySubnet()")]
    pub struct EmptySubnet;
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
    ///Custom Error type `InconsistentPrevCheckpoint` with signature `InconsistentPrevCheckpoint()` and selector `0x24465cba`
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
        name = "InconsistentPrevCheckpoint",
        abi = "InconsistentPrevCheckpoint()"
    )]
    pub struct InconsistentPrevCheckpoint;
    ///Custom Error type `InsufficientFunds` with signature `InsufficientFunds()` and selector `0x356680b7`
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
    #[etherror(name = "InsufficientFunds", abi = "InsufficientFunds()")]
    pub struct InsufficientFunds;
    ///Custom Error type `InvalidActorAddress` with signature `InvalidActorAddress()` and selector `0x70e45109`
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
    #[etherror(name = "InvalidActorAddress", abi = "InvalidActorAddress()")]
    pub struct InvalidActorAddress;
    ///Custom Error type `InvalidCheckpointEpoch` with signature `InvalidCheckpointEpoch()` and selector `0xfae4eadb`
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
    #[etherror(name = "InvalidCheckpointEpoch", abi = "InvalidCheckpointEpoch()")]
    pub struct InvalidCheckpointEpoch;
    ///Custom Error type `InvalidCheckpointSource` with signature `InvalidCheckpointSource()` and selector `0xfe72264e`
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
    #[etherror(name = "InvalidCheckpointSource", abi = "InvalidCheckpointSource()")]
    pub struct InvalidCheckpointSource;
    ///Custom Error type `InvalidCrossMsgDestinationSubnet` with signature `InvalidCrossMsgDestinationSubnet()` and selector `0x461e815d`
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
        name = "InvalidCrossMsgDestinationSubnet",
        abi = "InvalidCrossMsgDestinationSubnet()"
    )]
    pub struct InvalidCrossMsgDestinationSubnet;
    ///Custom Error type `InvalidCrossMsgFromSubnetId` with signature `InvalidCrossMsgFromSubnetId()` and selector `0x8481de49`
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
        name = "InvalidCrossMsgFromSubnetId",
        abi = "InvalidCrossMsgFromSubnetId()"
    )]
    pub struct InvalidCrossMsgFromSubnetId;
    ///Custom Error type `InvalidCrossMsgNonce` with signature `InvalidCrossMsgNonce()` and selector `0xa57cadff`
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
    #[etherror(name = "InvalidCrossMsgNonce", abi = "InvalidCrossMsgNonce()")]
    pub struct InvalidCrossMsgNonce;
    ///Custom Error type `InvalidCrossMsgsSortOrder` with signature `InvalidCrossMsgsSortOrder()` and selector `0xabc05942`
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
    #[etherror(name = "InvalidCrossMsgsSortOrder", abi = "InvalidCrossMsgsSortOrder()")]
    pub struct InvalidCrossMsgsSortOrder;
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
    ///Custom Error type `MethodNotSupportedYet` with signature `MethodNotSupportedYet()` and selector `0x2b843632`
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
    #[etherror(name = "MethodNotSupportedYet", abi = "MethodNotSupportedYet()")]
    pub struct MethodNotSupportedYet;
    ///Custom Error type `NotEmptySubnetCircSupply` with signature `NotEmptySubnetCircSupply()` and selector `0xf8cf8e02`
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
    #[etherror(name = "NotEmptySubnetCircSupply", abi = "NotEmptySubnetCircSupply()")]
    pub struct NotEmptySubnetCircSupply;
    ///Custom Error type `NotEnoughBalance` with signature `NotEnoughBalance()` and selector `0xad3a8b9e`
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
    #[etherror(name = "NotEnoughBalance", abi = "NotEnoughBalance()")]
    pub struct NotEnoughBalance;
    ///Custom Error type `NotEnoughFee` with signature `NotEnoughFee()` and selector `0x688e55ae`
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
    #[etherror(name = "NotEnoughFee", abi = "NotEnoughFee()")]
    pub struct NotEnoughFee;
    ///Custom Error type `NotEnoughFunds` with signature `NotEnoughFunds()` and selector `0x81b5ad68`
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
    #[etherror(name = "NotEnoughFunds", abi = "NotEnoughFunds()")]
    pub struct NotEnoughFunds;
    ///Custom Error type `NotEnoughFundsForMembership` with signature `NotEnoughFundsForMembership()` and selector `0xc26f1a27`
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
        name = "NotEnoughFundsForMembership",
        abi = "NotEnoughFundsForMembership()"
    )]
    pub struct NotEnoughFundsForMembership;
    ///Custom Error type `NotEnoughFundsToRelease` with signature `NotEnoughFundsToRelease()` and selector `0x79b33e79`
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
    #[etherror(name = "NotEnoughFundsToRelease", abi = "NotEnoughFundsToRelease()")]
    pub struct NotEnoughFundsToRelease;
    ///Custom Error type `NotEnoughSubnetCircSupply` with signature `NotEnoughSubnetCircSupply()` and selector `0x74db2854`
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
    #[etherror(name = "NotEnoughSubnetCircSupply", abi = "NotEnoughSubnetCircSupply()")]
    pub struct NotEnoughSubnetCircSupply;
    ///Custom Error type `NotInitialized` with signature `NotInitialized()` and selector `0x87138d5c`
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
    #[etherror(name = "NotInitialized", abi = "NotInitialized()")]
    pub struct NotInitialized;
    ///Custom Error type `NotRegisteredSubnet` with signature `NotRegisteredSubnet()` and selector `0xe991abd0`
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
    #[etherror(name = "NotRegisteredSubnet", abi = "NotRegisteredSubnet()")]
    pub struct NotRegisteredSubnet;
    ///Custom Error type `NotSignableAccount` with signature `NotSignableAccount()` and selector `0x511ed158`
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
    #[etherror(name = "NotSignableAccount", abi = "NotSignableAccount()")]
    pub struct NotSignableAccount;
    ///Custom Error type `NotSystemActor` with signature `NotSystemActor()` and selector `0xf0d97f3b`
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
    #[etherror(name = "NotSystemActor", abi = "NotSystemActor()")]
    pub struct NotSystemActor;
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
    ///Custom Error type `PostboxNotExist` with signature `PostboxNotExist()` and selector `0x24498941`
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
    #[etherror(name = "PostboxNotExist", abi = "PostboxNotExist()")]
    pub struct PostboxNotExist;
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
    ///Custom Error type `ValidatorWeightIsZero` with signature `ValidatorWeightIsZero()` and selector `0x389b457d`
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
    #[etherror(name = "ValidatorWeightIsZero", abi = "ValidatorWeightIsZero()")]
    pub struct ValidatorWeightIsZero;
    ///Custom Error type `ValidatorsAndWeightsLengthMismatch` with signature `ValidatorsAndWeightsLengthMismatch()` and selector `0x465f0a7d`
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
        name = "ValidatorsAndWeightsLengthMismatch",
        abi = "ValidatorsAndWeightsLengthMismatch()"
    )]
    pub struct ValidatorsAndWeightsLengthMismatch;
    ///Container type for all of the contract's custom errors
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum GatewayErrors {
        AddressEmptyCode(AddressEmptyCode),
        AddressInsufficientBalance(AddressInsufficientBalance),
        AlreadyInitialized(AlreadyInitialized),
        AlreadyRegisteredSubnet(AlreadyRegisteredSubnet),
        CallFailed(CallFailed),
        CannotReleaseZero(CannotReleaseZero),
        CannotSendCrossMsgToItself(CannotSendCrossMsgToItself),
        EmptySubnet(EmptySubnet),
        EpochAlreadyExecuted(EpochAlreadyExecuted),
        EpochNotVotable(EpochNotVotable),
        FailedInnerCall(FailedInnerCall),
        InconsistentPrevCheckpoint(InconsistentPrevCheckpoint),
        InsufficientFunds(InsufficientFunds),
        InvalidActorAddress(InvalidActorAddress),
        InvalidCheckpointEpoch(InvalidCheckpointEpoch),
        InvalidCheckpointSource(InvalidCheckpointSource),
        InvalidCrossMsgDestinationSubnet(InvalidCrossMsgDestinationSubnet),
        InvalidCrossMsgFromSubnetId(InvalidCrossMsgFromSubnetId),
        InvalidCrossMsgNonce(InvalidCrossMsgNonce),
        InvalidCrossMsgsSortOrder(InvalidCrossMsgsSortOrder),
        InvalidMajorityPercentage(InvalidMajorityPercentage),
        MessagesNotSorted(MessagesNotSorted),
        MethodNotSupportedYet(MethodNotSupportedYet),
        NotEmptySubnetCircSupply(NotEmptySubnetCircSupply),
        NotEnoughBalance(NotEnoughBalance),
        NotEnoughFee(NotEnoughFee),
        NotEnoughFunds(NotEnoughFunds),
        NotEnoughFundsForMembership(NotEnoughFundsForMembership),
        NotEnoughFundsToRelease(NotEnoughFundsToRelease),
        NotEnoughSubnetCircSupply(NotEnoughSubnetCircSupply),
        NotInitialized(NotInitialized),
        NotRegisteredSubnet(NotRegisteredSubnet),
        NotSignableAccount(NotSignableAccount),
        NotSystemActor(NotSystemActor),
        NotValidator(NotValidator),
        PostboxNotExist(PostboxNotExist),
        ReentrancyGuardReentrantCall(ReentrancyGuardReentrantCall),
        SubnetNotActive(SubnetNotActive),
        ValidatorAlreadyVoted(ValidatorAlreadyVoted),
        ValidatorWeightIsZero(ValidatorWeightIsZero),
        ValidatorsAndWeightsLengthMismatch(ValidatorsAndWeightsLengthMismatch),
        /// The standard solidity revert string, with selector
        /// Error(string) -- 0x08c379a0
        RevertString(::std::string::String),
    }
    impl ::ethers::core::abi::AbiDecode for GatewayErrors {
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
                = <AddressEmptyCode as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::AddressEmptyCode(decoded));
            }
            if let Ok(decoded)
                = <AddressInsufficientBalance as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::AddressInsufficientBalance(decoded));
            }
            if let Ok(decoded)
                = <AlreadyInitialized as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::AlreadyInitialized(decoded));
            }
            if let Ok(decoded)
                = <AlreadyRegisteredSubnet as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::AlreadyRegisteredSubnet(decoded));
            }
            if let Ok(decoded)
                = <CallFailed as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::CallFailed(decoded));
            }
            if let Ok(decoded)
                = <CannotReleaseZero as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::CannotReleaseZero(decoded));
            }
            if let Ok(decoded)
                = <CannotSendCrossMsgToItself as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::CannotSendCrossMsgToItself(decoded));
            }
            if let Ok(decoded)
                = <EmptySubnet as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::EmptySubnet(decoded));
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
                = <InconsistentPrevCheckpoint as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::InconsistentPrevCheckpoint(decoded));
            }
            if let Ok(decoded)
                = <InsufficientFunds as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::InsufficientFunds(decoded));
            }
            if let Ok(decoded)
                = <InvalidActorAddress as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::InvalidActorAddress(decoded));
            }
            if let Ok(decoded)
                = <InvalidCheckpointEpoch as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::InvalidCheckpointEpoch(decoded));
            }
            if let Ok(decoded)
                = <InvalidCheckpointSource as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::InvalidCheckpointSource(decoded));
            }
            if let Ok(decoded)
                = <InvalidCrossMsgDestinationSubnet as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::InvalidCrossMsgDestinationSubnet(decoded));
            }
            if let Ok(decoded)
                = <InvalidCrossMsgFromSubnetId as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::InvalidCrossMsgFromSubnetId(decoded));
            }
            if let Ok(decoded)
                = <InvalidCrossMsgNonce as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::InvalidCrossMsgNonce(decoded));
            }
            if let Ok(decoded)
                = <InvalidCrossMsgsSortOrder as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::InvalidCrossMsgsSortOrder(decoded));
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
                = <MethodNotSupportedYet as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::MethodNotSupportedYet(decoded));
            }
            if let Ok(decoded)
                = <NotEmptySubnetCircSupply as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::NotEmptySubnetCircSupply(decoded));
            }
            if let Ok(decoded)
                = <NotEnoughBalance as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NotEnoughBalance(decoded));
            }
            if let Ok(decoded)
                = <NotEnoughFee as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NotEnoughFee(decoded));
            }
            if let Ok(decoded)
                = <NotEnoughFunds as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NotEnoughFunds(decoded));
            }
            if let Ok(decoded)
                = <NotEnoughFundsForMembership as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::NotEnoughFundsForMembership(decoded));
            }
            if let Ok(decoded)
                = <NotEnoughFundsToRelease as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::NotEnoughFundsToRelease(decoded));
            }
            if let Ok(decoded)
                = <NotEnoughSubnetCircSupply as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::NotEnoughSubnetCircSupply(decoded));
            }
            if let Ok(decoded)
                = <NotInitialized as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NotInitialized(decoded));
            }
            if let Ok(decoded)
                = <NotRegisteredSubnet as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NotRegisteredSubnet(decoded));
            }
            if let Ok(decoded)
                = <NotSignableAccount as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NotSignableAccount(decoded));
            }
            if let Ok(decoded)
                = <NotSystemActor as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NotSystemActor(decoded));
            }
            if let Ok(decoded)
                = <NotValidator as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::NotValidator(decoded));
            }
            if let Ok(decoded)
                = <PostboxNotExist as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::PostboxNotExist(decoded));
            }
            if let Ok(decoded)
                = <ReentrancyGuardReentrantCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ReentrancyGuardReentrantCall(decoded));
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
                = <ValidatorWeightIsZero as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ValidatorWeightIsZero(decoded));
            }
            if let Ok(decoded)
                = <ValidatorsAndWeightsLengthMismatch as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::ValidatorsAndWeightsLengthMismatch(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for GatewayErrors {
        fn encode(self) -> ::std::vec::Vec<u8> {
            match self {
                Self::AddressEmptyCode(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AddressInsufficientBalance(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AlreadyInitialized(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AlreadyRegisteredSubnet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CallFailed(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotReleaseZero(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CannotSendCrossMsgToItself(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::EmptySubnet(element) => {
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
                Self::InconsistentPrevCheckpoint(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InsufficientFunds(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidActorAddress(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidCheckpointEpoch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidCheckpointSource(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidCrossMsgDestinationSubnet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidCrossMsgFromSubnetId(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidCrossMsgNonce(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidCrossMsgsSortOrder(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InvalidMajorityPercentage(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MessagesNotSorted(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MethodNotSupportedYet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotEmptySubnetCircSupply(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotEnoughBalance(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotEnoughFee(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotEnoughFunds(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotEnoughFundsForMembership(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotEnoughFundsToRelease(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotEnoughSubnetCircSupply(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotInitialized(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotRegisteredSubnet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotSignableAccount(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotSystemActor(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::NotValidator(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::PostboxNotExist(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ReentrancyGuardReentrantCall(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SubnetNotActive(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ValidatorAlreadyVoted(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ValidatorWeightIsZero(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ValidatorsAndWeightsLengthMismatch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::RevertString(s) => ::ethers::core::abi::AbiEncode::encode(s),
            }
        }
    }
    impl ::ethers::contract::ContractRevert for GatewayErrors {
        fn valid_selector(selector: [u8; 4]) -> bool {
            match selector {
                [0x08, 0xc3, 0x79, 0xa0] => true,
                _ if selector
                    == <AddressEmptyCode as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <AddressInsufficientBalance as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <AlreadyInitialized as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <AlreadyRegisteredSubnet as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CallFailed as ::ethers::contract::EthError>::selector() => true,
                _ if selector
                    == <CannotReleaseZero as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <CannotSendCrossMsgToItself as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <EmptySubnet as ::ethers::contract::EthError>::selector() => true,
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
                    == <InconsistentPrevCheckpoint as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InsufficientFunds as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidActorAddress as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidCheckpointEpoch as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidCheckpointSource as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidCrossMsgDestinationSubnet as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidCrossMsgFromSubnetId as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidCrossMsgNonce as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <InvalidCrossMsgsSortOrder as ::ethers::contract::EthError>::selector() => {
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
                    == <MethodNotSupportedYet as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotEmptySubnetCircSupply as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotEnoughBalance as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotEnoughFee as ::ethers::contract::EthError>::selector() => true,
                _ if selector
                    == <NotEnoughFunds as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotEnoughFundsForMembership as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotEnoughFundsToRelease as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotEnoughSubnetCircSupply as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotInitialized as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotRegisteredSubnet as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotSignableAccount as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotSystemActor as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <NotValidator as ::ethers::contract::EthError>::selector() => true,
                _ if selector
                    == <PostboxNotExist as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ReentrancyGuardReentrantCall as ::ethers::contract::EthError>::selector() => {
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
                    == <ValidatorWeightIsZero as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ if selector
                    == <ValidatorsAndWeightsLengthMismatch as ::ethers::contract::EthError>::selector() => {
                    true
                }
                _ => false,
            }
        }
    }
    impl ::core::fmt::Display for GatewayErrors {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::AddressEmptyCode(element) => ::core::fmt::Display::fmt(element, f),
                Self::AddressInsufficientBalance(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AlreadyInitialized(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AlreadyRegisteredSubnet(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::CallFailed(element) => ::core::fmt::Display::fmt(element, f),
                Self::CannotReleaseZero(element) => ::core::fmt::Display::fmt(element, f),
                Self::CannotSendCrossMsgToItself(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::EmptySubnet(element) => ::core::fmt::Display::fmt(element, f),
                Self::EpochAlreadyExecuted(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::EpochNotVotable(element) => ::core::fmt::Display::fmt(element, f),
                Self::FailedInnerCall(element) => ::core::fmt::Display::fmt(element, f),
                Self::InconsistentPrevCheckpoint(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InsufficientFunds(element) => ::core::fmt::Display::fmt(element, f),
                Self::InvalidActorAddress(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidCheckpointEpoch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidCheckpointSource(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidCrossMsgDestinationSubnet(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidCrossMsgFromSubnetId(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidCrossMsgNonce(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidCrossMsgsSortOrder(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InvalidMajorityPercentage(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::MessagesNotSorted(element) => ::core::fmt::Display::fmt(element, f),
                Self::MethodNotSupportedYet(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotEmptySubnetCircSupply(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotEnoughBalance(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotEnoughFee(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotEnoughFunds(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotEnoughFundsForMembership(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotEnoughFundsToRelease(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotEnoughSubnetCircSupply(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotInitialized(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotRegisteredSubnet(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotSignableAccount(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::NotSystemActor(element) => ::core::fmt::Display::fmt(element, f),
                Self::NotValidator(element) => ::core::fmt::Display::fmt(element, f),
                Self::PostboxNotExist(element) => ::core::fmt::Display::fmt(element, f),
                Self::ReentrancyGuardReentrantCall(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SubnetNotActive(element) => ::core::fmt::Display::fmt(element, f),
                Self::ValidatorAlreadyVoted(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ValidatorWeightIsZero(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ValidatorsAndWeightsLengthMismatch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::RevertString(s) => ::core::fmt::Display::fmt(s, f),
            }
        }
    }
    impl ::core::convert::From<::std::string::String> for GatewayErrors {
        fn from(value: String) -> Self {
            Self::RevertString(value)
        }
    }
    impl ::core::convert::From<AddressEmptyCode> for GatewayErrors {
        fn from(value: AddressEmptyCode) -> Self {
            Self::AddressEmptyCode(value)
        }
    }
    impl ::core::convert::From<AddressInsufficientBalance> for GatewayErrors {
        fn from(value: AddressInsufficientBalance) -> Self {
            Self::AddressInsufficientBalance(value)
        }
    }
    impl ::core::convert::From<AlreadyInitialized> for GatewayErrors {
        fn from(value: AlreadyInitialized) -> Self {
            Self::AlreadyInitialized(value)
        }
    }
    impl ::core::convert::From<AlreadyRegisteredSubnet> for GatewayErrors {
        fn from(value: AlreadyRegisteredSubnet) -> Self {
            Self::AlreadyRegisteredSubnet(value)
        }
    }
    impl ::core::convert::From<CallFailed> for GatewayErrors {
        fn from(value: CallFailed) -> Self {
            Self::CallFailed(value)
        }
    }
    impl ::core::convert::From<CannotReleaseZero> for GatewayErrors {
        fn from(value: CannotReleaseZero) -> Self {
            Self::CannotReleaseZero(value)
        }
    }
    impl ::core::convert::From<CannotSendCrossMsgToItself> for GatewayErrors {
        fn from(value: CannotSendCrossMsgToItself) -> Self {
            Self::CannotSendCrossMsgToItself(value)
        }
    }
    impl ::core::convert::From<EmptySubnet> for GatewayErrors {
        fn from(value: EmptySubnet) -> Self {
            Self::EmptySubnet(value)
        }
    }
    impl ::core::convert::From<EpochAlreadyExecuted> for GatewayErrors {
        fn from(value: EpochAlreadyExecuted) -> Self {
            Self::EpochAlreadyExecuted(value)
        }
    }
    impl ::core::convert::From<EpochNotVotable> for GatewayErrors {
        fn from(value: EpochNotVotable) -> Self {
            Self::EpochNotVotable(value)
        }
    }
    impl ::core::convert::From<FailedInnerCall> for GatewayErrors {
        fn from(value: FailedInnerCall) -> Self {
            Self::FailedInnerCall(value)
        }
    }
    impl ::core::convert::From<InconsistentPrevCheckpoint> for GatewayErrors {
        fn from(value: InconsistentPrevCheckpoint) -> Self {
            Self::InconsistentPrevCheckpoint(value)
        }
    }
    impl ::core::convert::From<InsufficientFunds> for GatewayErrors {
        fn from(value: InsufficientFunds) -> Self {
            Self::InsufficientFunds(value)
        }
    }
    impl ::core::convert::From<InvalidActorAddress> for GatewayErrors {
        fn from(value: InvalidActorAddress) -> Self {
            Self::InvalidActorAddress(value)
        }
    }
    impl ::core::convert::From<InvalidCheckpointEpoch> for GatewayErrors {
        fn from(value: InvalidCheckpointEpoch) -> Self {
            Self::InvalidCheckpointEpoch(value)
        }
    }
    impl ::core::convert::From<InvalidCheckpointSource> for GatewayErrors {
        fn from(value: InvalidCheckpointSource) -> Self {
            Self::InvalidCheckpointSource(value)
        }
    }
    impl ::core::convert::From<InvalidCrossMsgDestinationSubnet> for GatewayErrors {
        fn from(value: InvalidCrossMsgDestinationSubnet) -> Self {
            Self::InvalidCrossMsgDestinationSubnet(value)
        }
    }
    impl ::core::convert::From<InvalidCrossMsgFromSubnetId> for GatewayErrors {
        fn from(value: InvalidCrossMsgFromSubnetId) -> Self {
            Self::InvalidCrossMsgFromSubnetId(value)
        }
    }
    impl ::core::convert::From<InvalidCrossMsgNonce> for GatewayErrors {
        fn from(value: InvalidCrossMsgNonce) -> Self {
            Self::InvalidCrossMsgNonce(value)
        }
    }
    impl ::core::convert::From<InvalidCrossMsgsSortOrder> for GatewayErrors {
        fn from(value: InvalidCrossMsgsSortOrder) -> Self {
            Self::InvalidCrossMsgsSortOrder(value)
        }
    }
    impl ::core::convert::From<InvalidMajorityPercentage> for GatewayErrors {
        fn from(value: InvalidMajorityPercentage) -> Self {
            Self::InvalidMajorityPercentage(value)
        }
    }
    impl ::core::convert::From<MessagesNotSorted> for GatewayErrors {
        fn from(value: MessagesNotSorted) -> Self {
            Self::MessagesNotSorted(value)
        }
    }
    impl ::core::convert::From<MethodNotSupportedYet> for GatewayErrors {
        fn from(value: MethodNotSupportedYet) -> Self {
            Self::MethodNotSupportedYet(value)
        }
    }
    impl ::core::convert::From<NotEmptySubnetCircSupply> for GatewayErrors {
        fn from(value: NotEmptySubnetCircSupply) -> Self {
            Self::NotEmptySubnetCircSupply(value)
        }
    }
    impl ::core::convert::From<NotEnoughBalance> for GatewayErrors {
        fn from(value: NotEnoughBalance) -> Self {
            Self::NotEnoughBalance(value)
        }
    }
    impl ::core::convert::From<NotEnoughFee> for GatewayErrors {
        fn from(value: NotEnoughFee) -> Self {
            Self::NotEnoughFee(value)
        }
    }
    impl ::core::convert::From<NotEnoughFunds> for GatewayErrors {
        fn from(value: NotEnoughFunds) -> Self {
            Self::NotEnoughFunds(value)
        }
    }
    impl ::core::convert::From<NotEnoughFundsForMembership> for GatewayErrors {
        fn from(value: NotEnoughFundsForMembership) -> Self {
            Self::NotEnoughFundsForMembership(value)
        }
    }
    impl ::core::convert::From<NotEnoughFundsToRelease> for GatewayErrors {
        fn from(value: NotEnoughFundsToRelease) -> Self {
            Self::NotEnoughFundsToRelease(value)
        }
    }
    impl ::core::convert::From<NotEnoughSubnetCircSupply> for GatewayErrors {
        fn from(value: NotEnoughSubnetCircSupply) -> Self {
            Self::NotEnoughSubnetCircSupply(value)
        }
    }
    impl ::core::convert::From<NotInitialized> for GatewayErrors {
        fn from(value: NotInitialized) -> Self {
            Self::NotInitialized(value)
        }
    }
    impl ::core::convert::From<NotRegisteredSubnet> for GatewayErrors {
        fn from(value: NotRegisteredSubnet) -> Self {
            Self::NotRegisteredSubnet(value)
        }
    }
    impl ::core::convert::From<NotSignableAccount> for GatewayErrors {
        fn from(value: NotSignableAccount) -> Self {
            Self::NotSignableAccount(value)
        }
    }
    impl ::core::convert::From<NotSystemActor> for GatewayErrors {
        fn from(value: NotSystemActor) -> Self {
            Self::NotSystemActor(value)
        }
    }
    impl ::core::convert::From<NotValidator> for GatewayErrors {
        fn from(value: NotValidator) -> Self {
            Self::NotValidator(value)
        }
    }
    impl ::core::convert::From<PostboxNotExist> for GatewayErrors {
        fn from(value: PostboxNotExist) -> Self {
            Self::PostboxNotExist(value)
        }
    }
    impl ::core::convert::From<ReentrancyGuardReentrantCall> for GatewayErrors {
        fn from(value: ReentrancyGuardReentrantCall) -> Self {
            Self::ReentrancyGuardReentrantCall(value)
        }
    }
    impl ::core::convert::From<SubnetNotActive> for GatewayErrors {
        fn from(value: SubnetNotActive) -> Self {
            Self::SubnetNotActive(value)
        }
    }
    impl ::core::convert::From<ValidatorAlreadyVoted> for GatewayErrors {
        fn from(value: ValidatorAlreadyVoted) -> Self {
            Self::ValidatorAlreadyVoted(value)
        }
    }
    impl ::core::convert::From<ValidatorWeightIsZero> for GatewayErrors {
        fn from(value: ValidatorWeightIsZero) -> Self {
            Self::ValidatorWeightIsZero(value)
        }
    }
    impl ::core::convert::From<ValidatorsAndWeightsLengthMismatch> for GatewayErrors {
        fn from(value: ValidatorsAndWeightsLengthMismatch) -> Self {
            Self::ValidatorsAndWeightsLengthMismatch(value)
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
    ///Container type for all input parameters for the `addStake` function with signature `addStake()` and selector `0x5a627dbc`
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
    #[ethcall(name = "addStake", abi = "addStake()")]
    pub struct AddStakeCall;
    ///Container type for all input parameters for the `appliedTopDownNonce` function with signature `appliedTopDownNonce()` and selector `0x8789f83b`
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
    #[ethcall(name = "appliedTopDownNonce", abi = "appliedTopDownNonce()")]
    pub struct AppliedTopDownNonceCall;
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
    ///Container type for all input parameters for the `bottomUpCheckpoints` function with signature `bottomUpCheckpoints(uint64)` and selector `0x2cc14ea2`
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
    #[ethcall(name = "bottomUpCheckpoints", abi = "bottomUpCheckpoints(uint64)")]
    pub struct BottomUpCheckpointsCall(pub u64);
    ///Container type for all input parameters for the `bottomUpNonce` function with signature `bottomUpNonce()` and selector `0x41b6a2e8`
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
    #[ethcall(name = "bottomUpNonce", abi = "bottomUpNonce()")]
    pub struct BottomUpNonceCall;
    ///Container type for all input parameters for the `commitChildCheck` function with signature `commitChildCheck(((uint64,address[]),uint64,uint256,((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes),bool)[],((uint64,address[]),bytes32[])[],bytes32,bytes))` and selector `0xd4e149a8`
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
        name = "commitChildCheck",
        abi = "commitChildCheck(((uint64,address[]),uint64,uint256,((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes),bool)[],((uint64,address[]),bytes32[])[],bytes32,bytes))"
    )]
    pub struct CommitChildCheckCall {
        pub commit: BottomUpCheckpoint,
    }
    ///Container type for all input parameters for the `crossMsgFee` function with signature `crossMsgFee()` and selector `0x24729425`
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
    #[ethcall(name = "crossMsgFee", abi = "crossMsgFee()")]
    pub struct CrossMsgFeeCall;
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
    ///Container type for all input parameters for the `fund` function with signature `fund((uint64,address[]),(uint8,bytes))` and selector `0x18f44b70`
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
    #[ethcall(name = "fund", abi = "fund((uint64,address[]),(uint8,bytes))")]
    pub struct FundCall {
        pub subnet_id: SubnetID,
        pub to: FvmAddress,
    }
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
    ///Container type for all input parameters for the `getNetworkName` function with signature `getNetworkName()` and selector `0x94074b03`
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
    #[ethcall(name = "getNetworkName", abi = "getNetworkName()")]
    pub struct GetNetworkNameCall;
    ///Container type for all input parameters for the `getSubnet` function with signature `getSubnet((uint64,address[]))` and selector `0xc66c66a1`
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
    #[ethcall(name = "getSubnet", abi = "getSubnet((uint64,address[]))")]
    pub struct GetSubnetCall {
        pub subnet_id: SubnetID,
    }
    ///Container type for all input parameters for the `getSubnetTopDownMsg` function with signature `getSubnetTopDownMsg((uint64,address[]),uint256)` and selector `0x0ea746f2`
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
        name = "getSubnetTopDownMsg",
        abi = "getSubnetTopDownMsg((uint64,address[]),uint256)"
    )]
    pub struct GetSubnetTopDownMsgCall {
        pub subnet_id: SubnetID,
        pub index: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `getSubnetTopDownMsgsLength` function with signature `getSubnetTopDownMsgsLength((uint64,address[]))` and selector `0x9d3070b5`
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
        name = "getSubnetTopDownMsgsLength",
        abi = "getSubnetTopDownMsgsLength((uint64,address[]))"
    )]
    pub struct GetSubnetTopDownMsgsLengthCall {
        pub subnet_id: SubnetID,
    }
    ///Container type for all input parameters for the `getTopDownMsgs` function with signature `getTopDownMsgs((uint64,address[]),uint64)` and selector `0x13549315`
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
        name = "getTopDownMsgs",
        abi = "getTopDownMsgs((uint64,address[]),uint64)"
    )]
    pub struct GetTopDownMsgsCall {
        pub subnet_id: SubnetID,
        pub from_nonce: u64,
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
    ///Container type for all input parameters for the `initGenesisEpoch` function with signature `initGenesisEpoch(uint64)` and selector `0x13f35388`
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
    #[ethcall(name = "initGenesisEpoch", abi = "initGenesisEpoch(uint64)")]
    pub struct InitGenesisEpochCall {
        pub genesis_epoch: u64,
    }
    ///Container type for all input parameters for the `initialized` function with signature `initialized()` and selector `0x158ef93e`
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
    #[ethcall(name = "initialized", abi = "initialized()")]
    pub struct InitializedCall;
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
    ///Container type for all input parameters for the `listSubnets` function with signature `listSubnets()` and selector `0x5d029685`
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
    #[ethcall(name = "listSubnets", abi = "listSubnets()")]
    pub struct ListSubnetsCall;
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
    ///Container type for all input parameters for the `minStake` function with signature `minStake()` and selector `0x375b3c0a`
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
    #[ethcall(name = "minStake", abi = "minStake()")]
    pub struct MinStakeCall;
    ///Container type for all input parameters for the `postbox` function with signature `postbox(bytes32)` and selector `0x8cfd78e7`
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
    #[ethcall(name = "postbox", abi = "postbox(bytes32)")]
    pub struct PostboxCall(pub [u8; 32]);
    ///Container type for all input parameters for the `register` function with signature `register()` and selector `0x1aa3a008`
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
    #[ethcall(name = "register", abi = "register()")]
    pub struct RegisterCall;
    ///Container type for all input parameters for the `release` function with signature `release((uint8,bytes))` and selector `0x6b2c1eef`
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
    #[ethcall(name = "release", abi = "release((uint8,bytes))")]
    pub struct ReleaseCall {
        pub to: FvmAddress,
    }
    ///Container type for all input parameters for the `releaseRewards` function with signature `releaseRewards(uint256)` and selector `0xf8703bb8`
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
    #[ethcall(name = "releaseRewards", abi = "releaseRewards(uint256)")]
    pub struct ReleaseRewardsCall {
        pub amount: ::ethers::core::types::U256,
    }
    ///Container type for all input parameters for the `releaseStake` function with signature `releaseStake(uint256)` and selector `0x45f54485`
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
    #[ethcall(name = "releaseStake", abi = "releaseStake(uint256)")]
    pub struct ReleaseStakeCall {
        pub amount: ::ethers::core::types::U256,
    }
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
    ///Container type for all input parameters for the `submitTopDownCheckpoint` function with signature `submitTopDownCheckpoint((uint64,((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes),bool)[]))` and selector `0x986acf38`
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
        name = "submitTopDownCheckpoint",
        abi = "submitTopDownCheckpoint((uint64,((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes),bool)[]))"
    )]
    pub struct SubmitTopDownCheckpointCall {
        pub checkpoint: TopDownCheckpoint,
    }
    ///Container type for all input parameters for the `subnetKeys` function with signature `subnetKeys(uint256)` and selector `0x548b3b38`
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
    #[ethcall(name = "subnetKeys", abi = "subnetKeys(uint256)")]
    pub struct SubnetKeysCall(pub ::ethers::core::types::U256);
    ///Container type for all input parameters for the `subnets` function with signature `subnets(bytes32)` and selector `0x02e30f9a`
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
    #[ethcall(name = "subnets", abi = "subnets(bytes32)")]
    pub struct SubnetsCall(pub [u8; 32]);
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
    ///Container type for all input parameters for the `totalSubnets` function with signature `totalSubnets()` and selector `0xa2b67158`
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
    #[ethcall(name = "totalSubnets", abi = "totalSubnets()")]
    pub struct TotalSubnetsCall;
    ///Container type for all input parameters for the `totalWeight` function with signature `totalWeight()` and selector `0x96c82e57`
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
    #[ethcall(name = "totalWeight", abi = "totalWeight()")]
    pub struct TotalWeightCall;
    ///Container type for all input parameters for the `validatorNonce` function with signature `validatorNonce()` and selector `0xe17a684f`
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
    #[ethcall(name = "validatorNonce", abi = "validatorNonce()")]
    pub struct ValidatorNonceCall;
    ///Container type for all input parameters for the `validatorSet` function with signature `validatorSet(uint256,address)` and selector `0x223d9056`
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
    #[ethcall(name = "validatorSet", abi = "validatorSet(uint256,address)")]
    pub struct ValidatorSetCall(
        pub ::ethers::core::types::U256,
        pub ::ethers::core::types::Address,
    );
    ///Container type for all of the contract's call
    #[derive(Clone, ::ethers::contract::EthAbiType, Debug, PartialEq, Eq, Hash)]
    pub enum GatewayCalls {
        MinCheckpointPeriod(MinCheckpointPeriodCall),
        MinCollateralAmount(MinCollateralAmountCall),
        AddStake(AddStakeCall),
        AppliedTopDownNonce(AppliedTopDownNonceCall),
        BottomUpCheckPeriod(BottomUpCheckPeriodCall),
        BottomUpCheckpointAtEpoch(BottomUpCheckpointAtEpochCall),
        BottomUpCheckpointHashAtEpoch(BottomUpCheckpointHashAtEpochCall),
        BottomUpCheckpoints(BottomUpCheckpointsCall),
        BottomUpNonce(BottomUpNonceCall),
        CommitChildCheck(CommitChildCheckCall),
        CrossMsgFee(CrossMsgFeeCall),
        ExecutableQueue(ExecutableQueueCall),
        Fund(FundCall),
        GetGenesisEpoch(GetGenesisEpochCall),
        GetNetworkName(GetNetworkNameCall),
        GetSubnet(GetSubnetCall),
        GetSubnetTopDownMsg(GetSubnetTopDownMsgCall),
        GetSubnetTopDownMsgsLength(GetSubnetTopDownMsgsLengthCall),
        GetTopDownMsgs(GetTopDownMsgsCall),
        HasValidatorVotedForSubmission(HasValidatorVotedForSubmissionCall),
        InitGenesisEpoch(InitGenesisEpochCall),
        Initialized(InitializedCall),
        Kill(KillCall),
        LastVotingExecutedEpoch(LastVotingExecutedEpochCall),
        ListSubnets(ListSubnetsCall),
        MajorityPercentage(MajorityPercentageCall),
        MinStake(MinStakeCall),
        Postbox(PostboxCall),
        Register(RegisterCall),
        Release(ReleaseCall),
        ReleaseRewards(ReleaseRewardsCall),
        ReleaseStake(ReleaseStakeCall),
        SubmissionPeriod(SubmissionPeriodCall),
        SubmitTopDownCheckpoint(SubmitTopDownCheckpointCall),
        SubnetKeys(SubnetKeysCall),
        Subnets(SubnetsCall),
        TopDownCheckPeriod(TopDownCheckPeriodCall),
        TotalSubnets(TotalSubnetsCall),
        TotalWeight(TotalWeightCall),
        ValidatorNonce(ValidatorNonceCall),
        ValidatorSet(ValidatorSetCall),
    }
    impl ::ethers::core::abi::AbiDecode for GatewayCalls {
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
                = <AddStakeCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::AddStake(decoded));
            }
            if let Ok(decoded)
                = <AppliedTopDownNonceCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::AppliedTopDownNonce(decoded));
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
                = <BottomUpCheckpointsCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::BottomUpCheckpoints(decoded));
            }
            if let Ok(decoded)
                = <BottomUpNonceCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::BottomUpNonce(decoded));
            }
            if let Ok(decoded)
                = <CommitChildCheckCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::CommitChildCheck(decoded));
            }
            if let Ok(decoded)
                = <CrossMsgFeeCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::CrossMsgFee(decoded));
            }
            if let Ok(decoded)
                = <ExecutableQueueCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ExecutableQueue(decoded));
            }
            if let Ok(decoded)
                = <FundCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Fund(decoded));
            }
            if let Ok(decoded)
                = <GetGenesisEpochCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetGenesisEpoch(decoded));
            }
            if let Ok(decoded)
                = <GetNetworkNameCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetNetworkName(decoded));
            }
            if let Ok(decoded)
                = <GetSubnetCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetSubnet(decoded));
            }
            if let Ok(decoded)
                = <GetSubnetTopDownMsgCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::GetSubnetTopDownMsg(decoded));
            }
            if let Ok(decoded)
                = <GetSubnetTopDownMsgsLengthCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::GetSubnetTopDownMsgsLength(decoded));
            }
            if let Ok(decoded)
                = <GetTopDownMsgsCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::GetTopDownMsgs(decoded));
            }
            if let Ok(decoded)
                = <HasValidatorVotedForSubmissionCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::HasValidatorVotedForSubmission(decoded));
            }
            if let Ok(decoded)
                = <InitGenesisEpochCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::InitGenesisEpoch(decoded));
            }
            if let Ok(decoded)
                = <InitializedCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Initialized(decoded));
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
                = <ListSubnetsCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ListSubnets(decoded));
            }
            if let Ok(decoded)
                = <MajorityPercentageCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::MajorityPercentage(decoded));
            }
            if let Ok(decoded)
                = <MinStakeCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::MinStake(decoded));
            }
            if let Ok(decoded)
                = <PostboxCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Postbox(decoded));
            }
            if let Ok(decoded)
                = <RegisterCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Register(decoded));
            }
            if let Ok(decoded)
                = <ReleaseCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Release(decoded));
            }
            if let Ok(decoded)
                = <ReleaseRewardsCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ReleaseRewards(decoded));
            }
            if let Ok(decoded)
                = <ReleaseStakeCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ReleaseStake(decoded));
            }
            if let Ok(decoded)
                = <SubmissionPeriodCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::SubmissionPeriod(decoded));
            }
            if let Ok(decoded)
                = <SubmitTopDownCheckpointCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::SubmitTopDownCheckpoint(decoded));
            }
            if let Ok(decoded)
                = <SubnetKeysCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::SubnetKeys(decoded));
            }
            if let Ok(decoded)
                = <SubnetsCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::Subnets(decoded));
            }
            if let Ok(decoded)
                = <TopDownCheckPeriodCall as ::ethers::core::abi::AbiDecode>::decode(
                    data,
                ) {
                return Ok(Self::TopDownCheckPeriod(decoded));
            }
            if let Ok(decoded)
                = <TotalSubnetsCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::TotalSubnets(decoded));
            }
            if let Ok(decoded)
                = <TotalWeightCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::TotalWeight(decoded));
            }
            if let Ok(decoded)
                = <ValidatorNonceCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ValidatorNonce(decoded));
            }
            if let Ok(decoded)
                = <ValidatorSetCall as ::ethers::core::abi::AbiDecode>::decode(data) {
                return Ok(Self::ValidatorSet(decoded));
            }
            Err(::ethers::core::abi::Error::InvalidData.into())
        }
    }
    impl ::ethers::core::abi::AbiEncode for GatewayCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                Self::MinCheckpointPeriod(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MinCollateralAmount(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AddStake(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::AppliedTopDownNonce(element) => {
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
                Self::BottomUpCheckpoints(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::BottomUpNonce(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CommitChildCheck(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::CrossMsgFee(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ExecutableQueue(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Fund(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::GetGenesisEpoch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetNetworkName(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetSubnet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetSubnetTopDownMsg(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetSubnetTopDownMsgsLength(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::GetTopDownMsgs(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::HasValidatorVotedForSubmission(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::InitGenesisEpoch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Initialized(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Kill(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::LastVotingExecutedEpoch(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ListSubnets(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MajorityPercentage(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::MinStake(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Postbox(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::Register(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Release(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::ReleaseRewards(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ReleaseStake(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SubmissionPeriod(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SubmitTopDownCheckpoint(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::SubnetKeys(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::Subnets(element) => ::ethers::core::abi::AbiEncode::encode(element),
                Self::TopDownCheckPeriod(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TotalSubnets(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::TotalWeight(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ValidatorNonce(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
                Self::ValidatorSet(element) => {
                    ::ethers::core::abi::AbiEncode::encode(element)
                }
            }
        }
    }
    impl ::core::fmt::Display for GatewayCalls {
        fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
            match self {
                Self::MinCheckpointPeriod(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::MinCollateralAmount(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::AddStake(element) => ::core::fmt::Display::fmt(element, f),
                Self::AppliedTopDownNonce(element) => {
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
                Self::BottomUpCheckpoints(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::BottomUpNonce(element) => ::core::fmt::Display::fmt(element, f),
                Self::CommitChildCheck(element) => ::core::fmt::Display::fmt(element, f),
                Self::CrossMsgFee(element) => ::core::fmt::Display::fmt(element, f),
                Self::ExecutableQueue(element) => ::core::fmt::Display::fmt(element, f),
                Self::Fund(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetGenesisEpoch(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetNetworkName(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetSubnet(element) => ::core::fmt::Display::fmt(element, f),
                Self::GetSubnetTopDownMsg(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetSubnetTopDownMsgsLength(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::GetTopDownMsgs(element) => ::core::fmt::Display::fmt(element, f),
                Self::HasValidatorVotedForSubmission(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::InitGenesisEpoch(element) => ::core::fmt::Display::fmt(element, f),
                Self::Initialized(element) => ::core::fmt::Display::fmt(element, f),
                Self::Kill(element) => ::core::fmt::Display::fmt(element, f),
                Self::LastVotingExecutedEpoch(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::ListSubnets(element) => ::core::fmt::Display::fmt(element, f),
                Self::MajorityPercentage(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::MinStake(element) => ::core::fmt::Display::fmt(element, f),
                Self::Postbox(element) => ::core::fmt::Display::fmt(element, f),
                Self::Register(element) => ::core::fmt::Display::fmt(element, f),
                Self::Release(element) => ::core::fmt::Display::fmt(element, f),
                Self::ReleaseRewards(element) => ::core::fmt::Display::fmt(element, f),
                Self::ReleaseStake(element) => ::core::fmt::Display::fmt(element, f),
                Self::SubmissionPeriod(element) => ::core::fmt::Display::fmt(element, f),
                Self::SubmitTopDownCheckpoint(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::SubnetKeys(element) => ::core::fmt::Display::fmt(element, f),
                Self::Subnets(element) => ::core::fmt::Display::fmt(element, f),
                Self::TopDownCheckPeriod(element) => {
                    ::core::fmt::Display::fmt(element, f)
                }
                Self::TotalSubnets(element) => ::core::fmt::Display::fmt(element, f),
                Self::TotalWeight(element) => ::core::fmt::Display::fmt(element, f),
                Self::ValidatorNonce(element) => ::core::fmt::Display::fmt(element, f),
                Self::ValidatorSet(element) => ::core::fmt::Display::fmt(element, f),
            }
        }
    }
    impl ::core::convert::From<MinCheckpointPeriodCall> for GatewayCalls {
        fn from(value: MinCheckpointPeriodCall) -> Self {
            Self::MinCheckpointPeriod(value)
        }
    }
    impl ::core::convert::From<MinCollateralAmountCall> for GatewayCalls {
        fn from(value: MinCollateralAmountCall) -> Self {
            Self::MinCollateralAmount(value)
        }
    }
    impl ::core::convert::From<AddStakeCall> for GatewayCalls {
        fn from(value: AddStakeCall) -> Self {
            Self::AddStake(value)
        }
    }
    impl ::core::convert::From<AppliedTopDownNonceCall> for GatewayCalls {
        fn from(value: AppliedTopDownNonceCall) -> Self {
            Self::AppliedTopDownNonce(value)
        }
    }
    impl ::core::convert::From<BottomUpCheckPeriodCall> for GatewayCalls {
        fn from(value: BottomUpCheckPeriodCall) -> Self {
            Self::BottomUpCheckPeriod(value)
        }
    }
    impl ::core::convert::From<BottomUpCheckpointAtEpochCall> for GatewayCalls {
        fn from(value: BottomUpCheckpointAtEpochCall) -> Self {
            Self::BottomUpCheckpointAtEpoch(value)
        }
    }
    impl ::core::convert::From<BottomUpCheckpointHashAtEpochCall> for GatewayCalls {
        fn from(value: BottomUpCheckpointHashAtEpochCall) -> Self {
            Self::BottomUpCheckpointHashAtEpoch(value)
        }
    }
    impl ::core::convert::From<BottomUpCheckpointsCall> for GatewayCalls {
        fn from(value: BottomUpCheckpointsCall) -> Self {
            Self::BottomUpCheckpoints(value)
        }
    }
    impl ::core::convert::From<BottomUpNonceCall> for GatewayCalls {
        fn from(value: BottomUpNonceCall) -> Self {
            Self::BottomUpNonce(value)
        }
    }
    impl ::core::convert::From<CommitChildCheckCall> for GatewayCalls {
        fn from(value: CommitChildCheckCall) -> Self {
            Self::CommitChildCheck(value)
        }
    }
    impl ::core::convert::From<CrossMsgFeeCall> for GatewayCalls {
        fn from(value: CrossMsgFeeCall) -> Self {
            Self::CrossMsgFee(value)
        }
    }
    impl ::core::convert::From<ExecutableQueueCall> for GatewayCalls {
        fn from(value: ExecutableQueueCall) -> Self {
            Self::ExecutableQueue(value)
        }
    }
    impl ::core::convert::From<FundCall> for GatewayCalls {
        fn from(value: FundCall) -> Self {
            Self::Fund(value)
        }
    }
    impl ::core::convert::From<GetGenesisEpochCall> for GatewayCalls {
        fn from(value: GetGenesisEpochCall) -> Self {
            Self::GetGenesisEpoch(value)
        }
    }
    impl ::core::convert::From<GetNetworkNameCall> for GatewayCalls {
        fn from(value: GetNetworkNameCall) -> Self {
            Self::GetNetworkName(value)
        }
    }
    impl ::core::convert::From<GetSubnetCall> for GatewayCalls {
        fn from(value: GetSubnetCall) -> Self {
            Self::GetSubnet(value)
        }
    }
    impl ::core::convert::From<GetSubnetTopDownMsgCall> for GatewayCalls {
        fn from(value: GetSubnetTopDownMsgCall) -> Self {
            Self::GetSubnetTopDownMsg(value)
        }
    }
    impl ::core::convert::From<GetSubnetTopDownMsgsLengthCall> for GatewayCalls {
        fn from(value: GetSubnetTopDownMsgsLengthCall) -> Self {
            Self::GetSubnetTopDownMsgsLength(value)
        }
    }
    impl ::core::convert::From<GetTopDownMsgsCall> for GatewayCalls {
        fn from(value: GetTopDownMsgsCall) -> Self {
            Self::GetTopDownMsgs(value)
        }
    }
    impl ::core::convert::From<HasValidatorVotedForSubmissionCall> for GatewayCalls {
        fn from(value: HasValidatorVotedForSubmissionCall) -> Self {
            Self::HasValidatorVotedForSubmission(value)
        }
    }
    impl ::core::convert::From<InitGenesisEpochCall> for GatewayCalls {
        fn from(value: InitGenesisEpochCall) -> Self {
            Self::InitGenesisEpoch(value)
        }
    }
    impl ::core::convert::From<InitializedCall> for GatewayCalls {
        fn from(value: InitializedCall) -> Self {
            Self::Initialized(value)
        }
    }
    impl ::core::convert::From<KillCall> for GatewayCalls {
        fn from(value: KillCall) -> Self {
            Self::Kill(value)
        }
    }
    impl ::core::convert::From<LastVotingExecutedEpochCall> for GatewayCalls {
        fn from(value: LastVotingExecutedEpochCall) -> Self {
            Self::LastVotingExecutedEpoch(value)
        }
    }
    impl ::core::convert::From<ListSubnetsCall> for GatewayCalls {
        fn from(value: ListSubnetsCall) -> Self {
            Self::ListSubnets(value)
        }
    }
    impl ::core::convert::From<MajorityPercentageCall> for GatewayCalls {
        fn from(value: MajorityPercentageCall) -> Self {
            Self::MajorityPercentage(value)
        }
    }
    impl ::core::convert::From<MinStakeCall> for GatewayCalls {
        fn from(value: MinStakeCall) -> Self {
            Self::MinStake(value)
        }
    }
    impl ::core::convert::From<PostboxCall> for GatewayCalls {
        fn from(value: PostboxCall) -> Self {
            Self::Postbox(value)
        }
    }
    impl ::core::convert::From<RegisterCall> for GatewayCalls {
        fn from(value: RegisterCall) -> Self {
            Self::Register(value)
        }
    }
    impl ::core::convert::From<ReleaseCall> for GatewayCalls {
        fn from(value: ReleaseCall) -> Self {
            Self::Release(value)
        }
    }
    impl ::core::convert::From<ReleaseRewardsCall> for GatewayCalls {
        fn from(value: ReleaseRewardsCall) -> Self {
            Self::ReleaseRewards(value)
        }
    }
    impl ::core::convert::From<ReleaseStakeCall> for GatewayCalls {
        fn from(value: ReleaseStakeCall) -> Self {
            Self::ReleaseStake(value)
        }
    }
    impl ::core::convert::From<SubmissionPeriodCall> for GatewayCalls {
        fn from(value: SubmissionPeriodCall) -> Self {
            Self::SubmissionPeriod(value)
        }
    }
    impl ::core::convert::From<SubmitTopDownCheckpointCall> for GatewayCalls {
        fn from(value: SubmitTopDownCheckpointCall) -> Self {
            Self::SubmitTopDownCheckpoint(value)
        }
    }
    impl ::core::convert::From<SubnetKeysCall> for GatewayCalls {
        fn from(value: SubnetKeysCall) -> Self {
            Self::SubnetKeys(value)
        }
    }
    impl ::core::convert::From<SubnetsCall> for GatewayCalls {
        fn from(value: SubnetsCall) -> Self {
            Self::Subnets(value)
        }
    }
    impl ::core::convert::From<TopDownCheckPeriodCall> for GatewayCalls {
        fn from(value: TopDownCheckPeriodCall) -> Self {
            Self::TopDownCheckPeriod(value)
        }
    }
    impl ::core::convert::From<TotalSubnetsCall> for GatewayCalls {
        fn from(value: TotalSubnetsCall) -> Self {
            Self::TotalSubnets(value)
        }
    }
    impl ::core::convert::From<TotalWeightCall> for GatewayCalls {
        fn from(value: TotalWeightCall) -> Self {
            Self::TotalWeight(value)
        }
    }
    impl ::core::convert::From<ValidatorNonceCall> for GatewayCalls {
        fn from(value: ValidatorNonceCall) -> Self {
            Self::ValidatorNonce(value)
        }
    }
    impl ::core::convert::From<ValidatorSetCall> for GatewayCalls {
        fn from(value: ValidatorSetCall) -> Self {
            Self::ValidatorSet(value)
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
    ///Container type for all return fields from the `appliedTopDownNonce` function with signature `appliedTopDownNonce()` and selector `0x8789f83b`
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
    pub struct AppliedTopDownNonceReturn(pub u64);
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
    ///Container type for all return fields from the `bottomUpCheckpoints` function with signature `bottomUpCheckpoints(uint64)` and selector `0x2cc14ea2`
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
    pub struct BottomUpCheckpointsReturn {
        pub source: SubnetID,
        pub epoch: u64,
        pub fee: ::ethers::core::types::U256,
        pub prev_hash: [u8; 32],
        pub proof: ::ethers::core::types::Bytes,
    }
    ///Container type for all return fields from the `bottomUpNonce` function with signature `bottomUpNonce()` and selector `0x41b6a2e8`
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
    pub struct BottomUpNonceReturn(pub u64);
    ///Container type for all return fields from the `crossMsgFee` function with signature `crossMsgFee()` and selector `0x24729425`
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
    pub struct CrossMsgFeeReturn(pub ::ethers::core::types::U256);
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
    ///Container type for all return fields from the `getNetworkName` function with signature `getNetworkName()` and selector `0x94074b03`
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
    pub struct GetNetworkNameReturn(pub SubnetID);
    ///Container type for all return fields from the `getSubnet` function with signature `getSubnet((uint64,address[]))` and selector `0xc66c66a1`
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
    pub struct GetSubnetReturn(pub bool, pub Subnet);
    ///Container type for all return fields from the `getSubnetTopDownMsg` function with signature `getSubnetTopDownMsg((uint64,address[]),uint256)` and selector `0x0ea746f2`
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
    pub struct GetSubnetTopDownMsgReturn(pub CrossMsg);
    ///Container type for all return fields from the `getSubnetTopDownMsgsLength` function with signature `getSubnetTopDownMsgsLength((uint64,address[]))` and selector `0x9d3070b5`
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
    pub struct GetSubnetTopDownMsgsLengthReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `getTopDownMsgs` function with signature `getTopDownMsgs((uint64,address[]),uint64)` and selector `0x13549315`
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
    pub struct GetTopDownMsgsReturn(pub ::std::vec::Vec<CrossMsg>);
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
    ///Container type for all return fields from the `initialized` function with signature `initialized()` and selector `0x158ef93e`
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
    pub struct InitializedReturn(pub bool);
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
    ///Container type for all return fields from the `listSubnets` function with signature `listSubnets()` and selector `0x5d029685`
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
    pub struct ListSubnetsReturn(pub ::std::vec::Vec<Subnet>);
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
    ///Container type for all return fields from the `minStake` function with signature `minStake()` and selector `0x375b3c0a`
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
    pub struct MinStakeReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `postbox` function with signature `postbox(bytes32)` and selector `0x8cfd78e7`
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
    pub struct PostboxReturn {
        pub message: StorableMsg,
        pub wrapped: bool,
    }
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
    ///Container type for all return fields from the `subnetKeys` function with signature `subnetKeys(uint256)` and selector `0x548b3b38`
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
    pub struct SubnetKeysReturn(pub [u8; 32]);
    ///Container type for all return fields from the `subnets` function with signature `subnets(bytes32)` and selector `0x02e30f9a`
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
    pub struct SubnetsReturn {
        pub status: u8,
        pub top_down_nonce: u64,
        pub applied_bottom_up_nonce: u64,
        pub stake: ::ethers::core::types::U256,
        pub genesis_epoch: ::ethers::core::types::U256,
        pub circ_supply: ::ethers::core::types::U256,
        pub id: SubnetID,
        pub prev_checkpoint: BottomUpCheckpoint,
    }
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
    ///Container type for all return fields from the `totalSubnets` function with signature `totalSubnets()` and selector `0xa2b67158`
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
    pub struct TotalSubnetsReturn(pub u64);
    ///Container type for all return fields from the `totalWeight` function with signature `totalWeight()` and selector `0x96c82e57`
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
    pub struct TotalWeightReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `validatorNonce` function with signature `validatorNonce()` and selector `0xe17a684f`
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
    pub struct ValidatorNonceReturn(pub ::ethers::core::types::U256);
    ///Container type for all return fields from the `validatorSet` function with signature `validatorSet(uint256,address)` and selector `0x223d9056`
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
    pub struct ValidatorSetReturn(pub ::ethers::core::types::U256);
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
    ///`Subnet(uint8,uint64,uint64,uint256,uint256,uint256,(uint64,address[]),((uint64,address[]),uint64,uint256,((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes),bool)[],((uint64,address[]),bytes32[])[],bytes32,bytes),((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes),bool)[])`
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
    pub struct Subnet {
        pub status: u8,
        pub top_down_nonce: u64,
        pub applied_bottom_up_nonce: u64,
        pub stake: ::ethers::core::types::U256,
        pub genesis_epoch: ::ethers::core::types::U256,
        pub circ_supply: ::ethers::core::types::U256,
        pub id: SubnetID,
        pub prev_checkpoint: BottomUpCheckpoint,
        pub top_down_msgs: ::std::vec::Vec<CrossMsg>,
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
    ///`TopDownCheckpoint(uint64,((((uint64,address[]),(uint8,bytes)),((uint64,address[]),(uint8,bytes)),uint256,uint64,bytes4,bytes),bool)[])`
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
    pub struct TopDownCheckpoint {
        pub epoch: u64,
        pub top_down_msgs: ::std::vec::Vec<CrossMsg>,
    }
}
