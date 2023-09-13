// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {SubnetID, IPCAddress} from "./Subnet.sol";

/// @notice The parent finality for IPC parent at certain height.
struct ParentFinality {
    uint256 height;
    bytes32 blockHash;
}

/// @title BottomUpCheckpoint struct
/// @author LimeChain team
struct BottomUpCheckpoint {
    SubnetID source;
    uint64 epoch;
    uint256 fee;
    CrossMsg[] crossMsgs;
    ChildCheck[] children;
    bytes32 prevHash;
    bytes proof;
}

struct TopDownCheckpoint {
    uint64 epoch;
    CrossMsg[] topDownMsgs;
}

struct ChildCheck {
    SubnetID source;
    bytes32[] checks;
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
}
