// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {LibUtil} from "../lib/LibUtil.sol";
import {FvmAddress} from "../structs/FvmAddress.sol";

import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";

import {InvalidXnetMessage, InvalidXnetMessageReason, CheckpointAlreadyExists, CheckpointNotCreated} from "../errors/IPCErrors.sol";

import {LibGatewayChildQuery} from "./GatewayChildFacet.sol";
import {BURNT_FUNDS_ACTOR} from "../constants/Constants.sol";
import {IpcEnvelope, BottomUpMsgBatch, BottomUpCheckpoint} from "../structs/CrossNet.sol";
import {LibQuorum} from "../lib/LibQuorum.sol";
import {QuorumMap} from "../structs/Quorum.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";

/// @notice Handles the requests to the parent subnet in the child subnet.
contract GatewayBottomUpFacet {
    using FilAddress for address payable;

    /// @notice release burns the received value locally in subnet and commits a bottom-up message to release the assets in the parent.
    ///         The local supply of a subnet is always the native coin, so this method doesn't have to deal with tokens.
    function release(FvmAddress calldata to, uint256 amount) external payable {
        if (amount == 0) {
            // prevent spamming if there's no value to release.
            revert InvalidXnetMessage(InvalidXnetMessageReason.Value);
        }
        IpcEnvelope memory crossMsg = CrossMsgHelper.createReleaseMsg({
            subnet: LibGatewayChildQuery.id(),
            signer: msg.sender,
            to: to,
            value: amount
        });

        LibBottomUp.commitBottomUpMsg(crossMsg);

        // burn funds that are being released
        // TODO: should only burn once the operation is successful
        payable(BURNT_FUNDS_ACTOR).sendValue(msg.value);
    }

    /// @notice creates a new bottom-up checkpoint
    /// @param checkpoint - a bottom-up checkpoint
    /// @param membershipRootHash - a root hash of the Merkle tree built from the validator public keys and their weight
    /// @param membershipWeight - the total weight of the membership
    function createBottomUpCheckpoint(
        BottomUpCheckpoint calldata checkpoint,
        bytes32 membershipRootHash,
        uint256 membershipWeight
    ) external {
        LibUtil.enforceSystemActorOnly();
        LibBottomUp.createBottomUpCheckpoint(checkpoint, membershipRootHash, membershipWeight);
    }

    /// @notice Set a new checkpoint retention height and garbage collect all checkpoints in range [`retentionHeight`, `newRetentionHeight`)
    /// @dev `retentionHeight` is the height of the first incomplete checkpointswe must keep to implement checkpointing.
    /// All checkpoints with a height less than `retentionHeight` are removed from the history, assuming they are committed to the parent.
    /// @param newRetentionHeight - the height of the oldest checkpoint to keep
    function pruneBottomUpCheckpoints(uint256 newRetentionHeight) external {
        LibUtil.enforceSystemActorOnly();
        LibBottomUp.pruneBottomUpCheckpoints(newRetentionHeight);
    }

    /// @notice checks whether the provided checkpoint signature for the block at height `height` is valid and accumulates that it
    /// @dev If adding the signature leads to reaching the threshold, then the checkpoint is removed from `incompleteCheckpoints`
    /// @param height - the height of the block in the checkpoint
    /// @param membershipProof - a Merkle proof that the validator was in the membership at height `height` with weight `weight`
    /// @param weight - the weight of the validator
    /// @param signature - the signature of the checkpoint
    function addCheckpointSignature(
        uint256 height,
        bytes32[] memory membershipProof,
        uint256 weight,
        bytes memory signature
    ) external {
        LibUtil.enforceSystemActorOnly();
        LibBottomUp.addCheckpointSignature(height, membershipProof, weight, signature);
    }
}

// ============ Internal Usage Only ============
library LibBottomUp {
    /// @notice checks if the bottom-up checkpoint already exists at the target epoch
    function bottomUpCheckpointExists(uint256 epoch) internal view returns (bool) {
        BottomUpStorage storage s = LibBottomUpStorage.diamondStorage();
        return s.bottomUpCheckpoints[epoch].blockHeight != 0;
    }

    function createBottomUpCheckpoint(
        BottomUpCheckpoint calldata checkpoint,
        bytes32 membershipRootHash,
        uint256 membershipWeight
    ) internal {
        BottomUpStorage storage s = LibBottomUpStorage.diamondStorage();

        if (LibBottomUp.bottomUpCheckpointExists(checkpoint.blockHeight)) {
            revert CheckpointAlreadyExists();
        }

        LibQuorum.createQuorumInfo({
            self: s.checkpointQuorumMap,
            objHeight: checkpoint.blockHeight,
            objHash: keccak256(abi.encode(checkpoint)),
            membershipRootHash: membershipRootHash,
            membershipWeight: membershipWeight,
            majorityPercentage: s.majorityPercentage
        });

        storeBottomUpCheckpoint(checkpoint);
    }

    function pruneBottomUpCheckpoints(uint256 newRetentionHeight) internal {
        BottomUpStorage storage s = LibBottomUpStorage.diamondStorage();

        // we need to clean manually the checkpoints because Solidity does not support passing
        // a storage variable as an interface (so we can iterate and remove directly inside pruneQuorums)
        for (uint256 h = s.checkpointQuorumMap.retentionHeight; h < newRetentionHeight; ) {
            delete s.bottomUpCheckpoints[h];
            delete s.bottomUpMsgBatches[h];
            unchecked {
                ++h;
            }
        }

        LibQuorum.pruneQuorums(s.checkpointQuorumMap, newRetentionHeight);
    }

    /// @notice checks whether the provided checkpoint signature for the block at height `height` is valid and accumulates that it
    /// @dev If adding the signature leads to reaching the threshold, then the checkpoint is removed from `incompleteCheckpoints`
    /// @param height - the height of the block in the checkpoint
    /// @param membershipProof - a Merkle proof that the validator was in the membership at height `height` with weight `weight`
    /// @param weight - the weight of the validator
    /// @param signature - the signature of the checkpoint
    function addCheckpointSignature(
        uint256 height,
        bytes32[] memory membershipProof,
        uint256 weight,
        bytes memory signature
    ) external {
        BottomUpStorage storage s = LibBottomUpStorage.diamondStorage();

        // check if the checkpoint was already pruned before getting checkpoint
        // and triggering the signature
        LibQuorum.isHeightAlreadyProcessed(s.checkpointQuorumMap, height);

        if (!bottomUpCheckpointExists(height)) {
            revert CheckpointNotCreated();
        }
        LibQuorum.addQuorumSignature({
            self: s.checkpointQuorumMap,
            height: height,
            membershipProof: membershipProof,
            weight: weight,
            signature: signature
        });
    }

    /// @notice Commits a new cross-net message to a message batch for execution
    /// @param crossMessage - the cross message to be committed
    function commitBottomUpMsg(IpcEnvelope memory crossMessage) internal {
        BottomUpStorage storage s = LibBottomUpStorage.diamondStorage();

        uint256 epoch = LibUtil.nextBottomUpCheckpointEpoch(block.number, s.bottomUpCheckPeriod);

        // assign nonce to the message.
        crossMessage.nonce = s.bottomUpNonce;
        s.bottomUpNonce += 1;

        // populate the batch for that epoch
        (bool exists, BottomUpMsgBatch storage batch) = getBottomUpMsgBatch(epoch);
        if (!exists) {
            batch.subnetID = LibGatewayChildQuery.id();
            batch.blockHeight = epoch;
        }

        batch.msgs.push(crossMessage);

        // TODO: the message batch logic should be removed and favor message batch proofs
    }

    /// @notice returns the bottom-up batch
    function getBottomUpMsgBatch(uint256 epoch) internal view returns (bool exists, BottomUpMsgBatch storage batch) {
        BottomUpStorage storage s = LibBottomUpStorage.diamondStorage();

        batch = s.bottomUpMsgBatches[epoch];
        exists = batch.blockHeight != 0;
    }

    /// @notice stores checkpoint
    function storeBottomUpCheckpoint(BottomUpCheckpoint memory checkpoint) internal {
        BottomUpStorage storage s = LibBottomUpStorage.diamondStorage();

        BottomUpCheckpoint storage b = s.bottomUpCheckpoints[checkpoint.blockHeight];
        b.blockHash = checkpoint.blockHash;
        b.subnetID = checkpoint.subnetID;
        b.nextConfigurationNumber = checkpoint.nextConfigurationNumber;
        b.blockHeight = checkpoint.blockHeight;

        uint256 msgLength = checkpoint.msgs.length;
        for (uint256 i; i < msgLength; ) {
            // We need to push because initializing an array with a static
            // length will cause a copy from memory to storage, making
            // the compiler unhappy.
            b.msgs.push(checkpoint.msgs[i]);
            unchecked {
                ++i;
            }
        }
    }
}

// ============ Private Usage Only ============

struct BottomUpStorage {
    /// @notice bottom-up period in number of epochs for the subnet
    uint256 bottomUpCheckPeriod;
    /// @notice nonce for bottom-up messages
    uint64 bottomUpNonce;
    /// @notice Maximum number of messages per bottom up checkpoint
    uint64 maxMsgsPerBottomUpBatch;
    /// @notice majority percentage value (must be greater than or equal to 51)
    uint8 majorityPercentage;
    /// @notice A mapping of block numbers to bottom-up cross-messages
    // slither-disable-next-line uninitialized-state
    mapping(uint256 => BottomUpMsgBatch) bottomUpMsgBatches;
    /// @notice A mapping of block numbers to bottom-up checkpoints
    // slither-disable-next-line uninitialized-state
    mapping(uint256 => BottomUpCheckpoint) bottomUpCheckpoints;
    /// @notice Quorum information for checkpoints
    QuorumMap checkpointQuorumMap;
}

library LibBottomUpStorage {
    function diamondStorage() internal pure returns (BottomUpStorage storage ds) {
        bytes32 position = keccak256("ipc.gateway.bottomup.storage");
        assembly {
            ds.slot := position
        }
    }
}