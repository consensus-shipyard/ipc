// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {SubnetID, IPCAddress} from "./Subnet.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";

uint64 constant MAX_MSGS_PER_BATCH = 10;
uint256 constant BATCH_PERIOD = 100;

/// @notice The parent finality for IPC parent at certain height.
struct ParentFinality {
    uint256 height;
    bytes32 blockHash;
}

/// @notice A bottom-up checkpoint type.
struct BottomUpCheckpoint {
    /// @dev Child subnet ID, for replay protection from other subnets where the exact same validators operate.
    /// Alternatively it can be appended to the hash before signing, similar to how we use the chain ID.
    SubnetID subnetID;
    /// @dev The height of the child subnet at which this checkpoint was cut.
    /// Has to follow the previous checkpoint by checkpoint period.
    uint256 blockHeight;
    /// @dev The hash of the block.
    bytes32 blockHash;
    /// @dev The number of the membership (validator set) which is going to sign the next checkpoint.
    /// This one expected to be signed by the validators from the membership reported in the previous checkpoint.
    /// 0 could mean "no change".
    uint64 nextConfigurationNumber;
}

/// @notice A batch of bottom-up messages for execution
struct BottomUpMsgBatch {
    /// @dev Child subnet ID, for replay protection from other subnets where the exact same validators operate.
    SubnetID subnetID;
    /// @dev The height of the child subnet at which the batch was cut.
    uint256 blockHeight;
    /// @dev Batch of messages to execute.
    CrossMsg[] msgs;
}

/// @notice Tracks information about the last batch executed
struct BottomUpMsgBatchInfo {
    uint256 blockHeight;
    bytes32 hash;
}

/// @notice Tracks information about relayer rewards
struct RelayerRewardsInfo {
    /// @dev user rewards
    mapping(address => uint256) rewards;
    /// @dev tracks the addresses rewarded for checkpoint submission on a specific epoch
    mapping(uint256 => EnumerableSet.AddressSet) checkpointRewarded;
    /// @dev tracks the addresses rewarded for batch submission on a specific epoch
    mapping(uint256 => EnumerableSet.AddressSet) batchRewarded;
}

/**
 * @dev The goal of `wrapped` flag is to signal that a cross-net message should be sent as-is without changes to the destination.
 *
 * IMPORTANT: This is not currently used but it is a basic primitive required for atomic execution.
 */
struct CrossMsg {
    StorableMsg message;
    bool wrapped;
}

struct StorableMsg {
    IPCAddress from;
    IPCAddress to;
    uint256 value;
    uint64 nonce;
    bytes4 method;
    bytes params;
    uint256 fee;
}
