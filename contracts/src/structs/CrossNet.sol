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

/// @notice A batch of bottom-up messages for execution.
struct BottomUpMsgBatch {
    /// @dev Child subnet ID, for replay protection from other subnets where the exact same validators operate.
    SubnetID subnetID;
    /// @dev The height of the child subnet at which the batch was cut.
    uint256 blockHeight;
    /// @dev Batch of envelopes to execute.
    IpcEnvelope[] msgs;
}

/// @notice Tracks information about the last batch executed.
struct BottomUpMsgBatchInfo {
    uint256 blockHeight;
    bytes32 hash;
}

/// @notice Tracks information about relayer rewards.
struct RelayerRewardsInfo {
    /// @dev user rewards
    mapping(address => uint256) rewards;
    /// @dev tracks the addresses rewarded for checkpoint submission on a specific epoch
    mapping(uint256 => EnumerableSet.AddressSet) checkpointRewarded;
    /// @dev tracks the addresses rewarded for batch submission on a specific epoch
    mapping(uint256 => EnumerableSet.AddressSet) batchRewarded;
}

/// @notice Type of cross-net messages currently supported
enum IpcMsgKind {
    /// @dev for cross-net messages that move native token, i.e. fund/release.
    /// and in the future multi-level token transactions.
    Transfer,
    /// @dev general-purpose cross-net transaction that call smart contracts.
    Call,
    /// @dev receipt from the execution of cross-net messages
    /// (currently limited to `Transfer` messages)
    Receipt
}

/// @notice Envelope used to propagate IPC cross-net messages
struct IpcEnvelope {
    /// @dev type of message being propagated.
    IpcMsgKind kind;
    /// @dev destination of the message
    /// It makes sense to extract from the encoded message
    /// all shared fields required by all message, so they
    /// can be inspected without having to decode the message.
    IPCAddress to;
    /// @dev address sending the message
    IPCAddress from;
    /// @dev value being sent in the message.
    /// If we want receipts to return value, and all messages to be able
    /// to handle different supply sources we can expose the value
    /// as a common field.
    uint256 value;
    /// @dev abi.encoded message
    bytes message;
    /// @dev outgoing nonce for the envelope.
    /// This nonce is set by the gateway when committing the message for propagation
    uint64 nonce;
    /// @dev fee is currently not used.
    uint256 fee;
    /// @dev the gas limit is currently not used.
    // FIXME: currently not used and no code uses it, so keeping it out.
    // uint256 gasLimit;
}

/// @notice Message format used for `Transfer` and `Call` messages.
struct IpcMsg {
    /// @dev abi.encoded method being called by the contract.
    bytes4 method;
    /// @dev arguments of the method being called.
    bytes params;
}

struct ReceiptMsg {
    /// @dev Flag to signal if the call succeeded or failed.
    bool success;
    /// @dev Id of the message the receipt belongs to.
    bytes32 id;
    /// @dev abi encoded return value, or the reason for the
    /// failure (if any).
    bytes ret;
    // TODO: In the future we may include here events and other
    // feedback information.
}
