// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {SubnetID, IPCAddress} from "./Subnet.sol";

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
    uint64 blockHeight;
    /// @dev The hash of the block.
    bytes32 blockHash;
    /// @dev The number of the membership (validator set) which is going to sign the next checkpoint.
    /// This one expected to be signed by the validators from the membership reported in the previous checkpoint.
    /// 0 could mean "no change".
    uint64 nextConfigurationNumber;
    /// @dev Hash over the bottom-up messages.
    /// By not including cross messages here directly, we can be compatible with IPLD Resolver based
    /// approach where the messages are fetched with Bitswap and provided by Fendermint, or the full-fat
    /// approach we need with Lotus, where the messages are part of the relayed transaction.
    bytes32 crossMessagesHash;
}

struct CheckpointInfo {
    /// @dev The hash of the corresponding bottom-up checkpoint.
    bytes32 hash;
    /// @dev The root hash of the Merkle tree built from the validator public keys and their weight.
    bytes32 rootHash;
    /// @dev The target weight that must be reached to accept the checkpoint.
    uint256 threshold;
    /// @dev The current weight of the checkpoint.
    uint256 currentWeight;
    /// @dev Whether the quorum has already been reached.
    bool reached;
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
