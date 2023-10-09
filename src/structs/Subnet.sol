// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {SubnetID} from "./Subnet.sol";
import {FvmAddress} from "./FvmAddress.sol";
import {BottomUpCheckpoint, CrossMsg} from "./Checkpoint.sol";
import {Status} from "../enums/Status.sol";
import {MaxPQ} from "../lib/priority/LibMaxPQ.sol";
import {MinPQ} from "../lib/priority/LibMinPQ.sol";

struct SubnetID {
    /// @notice chainID of the root subnet
    uint64 root;
    /// @notice parent path of the subnet
    address[] route;
}

struct Subnet {
    uint256 stake;
    uint256 genesisEpoch;
    uint256 circSupply;
    uint64 topDownNonce;
    uint64 appliedBottomUpNonce;
    Status status;
    SubnetID id;
    BottomUpCheckpoint prevCheckpoint;
}

// ======== Subnet Staking =======
enum StakingOperation {
    Deposit,
    Withdraw,
    SetMetadata
}

/// The change request to validator staking
struct StakingChange {
    StakingOperation op;
    bytes payload;
    address validator;
}

/// The change associated with its corresponding configuration number
struct StakingChangeRequest {
    StakingChange change;
    uint64 configurationNumber;
}

/// The collection of staking changes
struct StakingChangeLog {
    /// @notice The next configuration number to assign to new changes.
    uint64 nextConfigurationNumber;
    /// @notice The starting configuration number stored.
    uint64 startConfigurationNumber;
    /// The details of the changes, mapping of configuration number to changes.
    mapping(uint64 => StakingChange) changes;
}

/// Each staking release amount and time
struct StakingRelease {
    /// @notice The block number that this fund can be released
    uint256 releaseAt;
    /// @notice The amount that is locked in the release
    uint256 amount;
}

/// Tracks the staking relases of an address. Mimics the implementation of array in solidity, this
/// way is more aigned with our use case.
struct AddressStakingReleases {
    uint16 length;
    uint16 startIdx;
    mapping(uint16 => StakingRelease) releases;
}

/// Manages the staking release queue
struct StakingReleaseQueue {
    /// @notice The number of blocks that locks the collateral.
    uint256 lockingDuration;
    /// @notice Keeps track of the validators and their releases.
    mapping(address => AddressStakingReleases) releases;
}

/// Keeping track of the validator information. There are two types of collaterals:
///     - Confirmed: The amount of collateral actually confirmed in child subnet
///     - Total: Aside from Confirmed, there is also the collateral has been supplied, but not yet confirmed in child.
struct Validator {
    uint256 confirmedCollateral;
    uint256 totalCollateral;
    /// The metadata associated with the validator, i.e. offchain network address.
    /// This information is not important to the protocol, offchain should know how
    /// to parse or decode the bytes.
    bytes metadata;
}

/// Keeping track of the list of validators. There are two types of validators:
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
    /// The total number of active validators allowed.
    uint16 activeLimit;
    /// The total collateral confirmed.
    uint256 totalConfirmedCollateral;
    /// The mapping of each validator address to its information.
    mapping(address => Validator) validators;
    /// @notice The active validators tracked using min priority queue.
    MinPQ activeValidators;
    /// @notice The waiting validators trakced using max priority queue.
    MaxPQ waitingValidators;
}

/// Tracks the parent validator changes and apply them in the child
struct ParentValidatorsTracker {
    ValidatorSet validators;
    StakingChangeLog changes;
}

struct IPCAddress {
    SubnetID subnetId;
    FvmAddress rawAddress;
}
