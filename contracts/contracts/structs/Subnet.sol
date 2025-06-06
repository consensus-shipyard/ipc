// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {FvmAddress} from "./FvmAddress.sol";
import {MaxPQ} from "../lib/priority/LibMaxPQ.sol";
import {MinPQ} from "../lib/priority/LibMinPQ.sol";

/// @notice A subnet identity type.
struct SubnetID {
    /// @notice chainID of the root subnet
    uint64 root;
    /// @notice parent path of the subnet
    address[] route;
}

/// @notice A Subnet type.
struct Subnet {
    uint256 stake;
    uint256 genesisEpoch;
    uint256 circSupply;
    uint64 topDownNonce;
    uint64 appliedBottomUpNonce;
    SubnetID id;
}

/// @notice Subnet validator power update operations types.
enum PowerOperation {
    /// @dev Update the power to the new specified value
    SetPower,
    /// @dev Update the metadata associated with the power
    SetMetadata
}

/// @notice The power change operation detail.
struct PowerChange {
    PowerOperation op;
    bytes payload;
    address validator;
}

/// @notice The change associated with its corresponding configuration number.
struct PowerChangeRequest {
    PowerChange change;
    uint64 configurationNumber;
}

/// @notice The collection of staking changes.
struct PowerChangeLog {
    /// @notice The next configuration number to assign to new changes.
    uint64 nextConfigurationNumber;
    /// @notice The starting configuration number stored.
    uint64 startConfigurationNumber;
    /// The details of the changes, mapping of configuration number to changes.
    mapping(uint64 => PowerChange) changes;
}

/// @notice Each staking release amount and time.
struct StakingRelease {
    /// @notice The block number that this fund can be released
    uint256 releaseAt;
    /// @notice The amount that is locked in the release
    uint256 amount;
}

/// @notice Tracks the staking releases of an address.
/// @dev    Mimics the implementation of array in solidity,
///         this way is more aligned with our use case.
struct AddressStakingReleases {
    /// @dev The total number of releases, the tail of the release queue
    uint32 tail;
    /// @dev The next release that one can collect
    uint32 head;
    mapping(uint32 => StakingRelease) releases;
}

/// @notice Manages the staking release queue.
struct StakingReleaseQueue {
    /// @notice The number of blocks that locks the collateral.
    uint256 lockingDuration;
    /// @notice Keeps track of the validators and their releases.
    mapping(address => AddressStakingReleases) releases;
}

/// @notice Keeping track of the validator information.
/// @notice When a power change request is triggered, the effects are applied to next power and
/// @notice after a complete topdown and bottom up cycle, it will be updated to current power which means
/// @notice the child subnet as acknowledged this change.
struct ValidatorInfo {
    /// The current voting power the validator has in the child subnet.
    uint256 currentPower;
    /// The power of a validator that is awaiting child subnet confirmation.
    /// TODO: this should be deprecated
    uint256 nextPower;
    /// The metadata associated with the validator, i.e. off-chain network address.
    /// This information is not important to the protocol, off-chain should know how
    /// to parse or decode the bytes.
    bytes metadata;
}

/// @notice Determines the permission mode for validators.
enum PermissionMode {
    /// Validator power is determined by the collateral staked
    Collateral,
    /// Validator power is assigned by the owner of the subnet
    Federated,
    /// Validator power is determined by the initial collateral staked and does not change anymore
    Static
}

/// @notice Determines the permission mode for who can create subet
enum SubnetCreationPrivileges {
    /// No permission check at all, any address can create
    Unrestricted,
    /// Only the owner can create subnet
    Owner
}

/// @notice Keeping track of the list of validators.
/// @dev There are two types of validators:
///     - Active
///     - Waiting
/// Active validators are those that are producing blocks in the child subnet.
/// Waiting validators are those that do no have as high collateral as Active validators.
///
/// The max number of active validators is limited by `activeLimit` and the size of waiting
/// validators is not bounded.
///
/// With each validator staking change, waiting validators can be promoted to active validators
/// and active validators can be knocked off.
struct ValidatorSet {
    /// The permission mode for validators
    PermissionMode permissionMode;
    /// The total number of active validators allowed.
    uint16 activeLimit;
    /// The total collateral confirmed.
    uint256 currentTotalPower;
    /// The mapping of each validator address to their details.
    mapping(address => ValidatorInfo) validators;
    /// @notice The active validators tracked using min priority queue.
    MinPQ activeValidators;
    /// @notice The waiting validators tracked using max priority queue.
    MaxPQ waitingValidators;
}

/// @notice Tracks the parent validator changes and apply them in the child.
struct ParentValidatorsTracker {
    ValidatorSet validators;
    PowerChangeLog changes;
}

/// @notice An IPC address type.
struct IPCAddress {
    SubnetID subnetId;
    FvmAddress rawAddress;
}

/// @notice Validator struct stored in the gateway.
struct Validator {
    uint256 weight;
    address addr;
    /// The metadata associated with the validator, i.e. off-chain network address.
    /// This information is not important to the protocol, off-chain should know how
    /// to parse or decode the bytes.
    bytes metadata;
}

/// @notice Membership information stored in the gateway.
struct Membership {
    Validator[] validators;
    uint64 configurationNumber;
}

/// @notice Defines a more general representation of a token in subnet. Using the same set of interfaces
/// @notice for handling token operations.
struct Asset {
    /// @notice The kind of token.
    AssetKind kind;
    /// @notice The address of the ERC20 token if that supply kind is selected.
    address tokenAddress;
}

/// @notice Determines the type of the token used in the subnet.
enum AssetKind {
    Native,
    ERC20
}
