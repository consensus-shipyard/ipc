// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {IValidatorRewarder} from "../interfaces/IValidatorRewarder.sol";
import {Consensus, CompressedActivityRollup} from "../structs/Activity.sol";
import {BottomUpBatch} from "../structs/BottomUpBatch.sol";
import {IpcEnvelope} from "../structs/CrossNet.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import {MerkleProof} from "@openzeppelin/contracts/utils/cryptography/MerkleProof.sol";
import {SubnetID} from "../structs/Subnet.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {InvalidInclusionProof, BatchMsgAlreadyExecuted, MissingBatchCommitment, DuplicatedCheckpointHeight} from "../errors/IPCErrors.sol";
import {BottomUpBatch} from "../structs/BottomUpBatch.sol";

/// Library to handle bottom up batch 2-phase execution.
library LibBottomUpBatch {
    bytes32 private constant NAMESPACE = keccak256("bottomupbatch");

    using EnumerableSet for EnumerableSet.Bytes32Set;

    /// @notice Represents a pending bottom-up batch commitment awaiting full execution at a specific checkpoint height.
    struct PendingBatch {
      /// @notice The pending batch commitment.
      BottomUpBatch.Commitment commitment;
      /// @notice Set of message leaf hashes that have already been executed for this batch.
      EnumerableSet.Bytes32Set executed;
    }

    /// @notice Storage structure used by the SubnetActor to manage bottom-up message batches and their execution status.
    struct BottomUpBatchStorage {
        /// @notice Set of checkpoint heights with batches that are still pending execution.
        EnumerableSet.Bytes32Set pendingHeights;
        /// @notice Mapping of checkpoint height to its pending batch data.
        mapping(uint256 => PendingBatch) pending;
    }

    function ensureValidProof(
        BottomUpBatch.MerkleHash[] memory proof,
        BottomUpBatch.MerkleHash root,
        BottomUpBatch.MerkleHash leaf
    ) internal pure {
        bytes32[] memory proofBytes = new bytes32[](proof.length);
        for (uint256 i = 0; i < proof.length; i++) {
            proofBytes[i] = BottomUpBatch.MerkleHash.unwrap(proof[i]);
        }
        bool valid = MerkleProof.verify({
            proof: proofBytes,
            root: BottomUpBatch.MerkleHash.unwrap(root),
            leaf: BottomUpBatch.MerkleHash.unwrap(leaf)
        });
        if (!valid) {
            revert InvalidInclusionProof();
        }
    }

    function recordBottomUpBatchCommitment(
        uint64 checkpointHeight,
        BottomUpBatch.Commitment calldata commitment
    ) internal {
        BottomUpBatchStorage storage s = bottomUpBatchStorage();

        bool added = s.pendingHeights.add(bytes32(uint256(checkpointHeight)));
        if (!added) {
            revert DuplicatedCheckpointHeight(checkpointHeight);
        }

        PendingBatch storage pending = s.pending[checkpointHeight];
        pending.commitment = commitment;
    }

    function processBottomUpBatchMsg(
        uint256 checkpointHeight,
        IpcEnvelope calldata ipcMsg,
        BottomUpBatch.MerkleHash[] calldata proof
    ) internal {
        BottomUpBatchStorage storage s = bottomUpBatchStorage();

        // Find the pending batch.
        PendingBatch storage pending = s.pending[checkpointHeight];
        BottomUpBatch.MerkleHash root = pending.commitment.msgsRoot;
        if (BottomUpBatch.MerkleHash.unwrap(root) == bytes32(0)) {
            revert MissingBatchCommitment();
        }

        // Check the validity of the proof.
        BottomUpBatch.MerkleHash leaf = makeLeaf(ipcMsg);
        ensureValidProof(
            proof,
            root,
            leaf
        );

        bool added = pending.executed.add(BottomUpBatch.MerkleHash.unwrap(leaf));
        if (!added) {
            revert BatchMsgAlreadyExecuted();
        }

        // Prune state for this height if all msgs were executed.
        if (pending.executed.length() == pending.commitment.totalNumMsgs) {
            s.pendingHeights.remove(bytes32(uint256(checkpointHeight)));

            // Clear nested set before deleting the struct.
            PendingBatch storage pending = s.pending[checkpointHeight];
            uint256 len = pending.executed.length();
            for (uint256 i = 0; i < len; i++) {
                bytes32 leaf = pending.executed.at(0);
                pending.executed.remove(leaf);
            }
            delete s.pending[checkpointHeight];
        }
    }

    /// Return type for the list pending commitments view method.
    struct ListPendingCommitmentsEntry {
        uint64 height;
        BottomUpBatch.Commitment commitment;
        bytes32[] executed;
    }

    /// A view accessor to query the pending commitments for a given subnet.
    function listPendingCommitments() internal view returns (ListPendingCommitmentsEntry[] memory result) {
        BottomUpBatchStorage storage s = bottomUpBatchStorage();

        bytes32[] memory heights = s.pendingHeights.values();
        uint256 size = heights.length;

        result = new ListPendingCommitmentsEntry[](size);

        for (uint256 i = 0; i < size; i++) {
            uint64 height = uint64(uint256(heights[i]));
            PendingBatch storage pending = s.pending[height];
            result[i] = ListPendingCommitmentsEntry({
                height: height,
                commitment: pending.commitment,
                executed: pending.executed.values()
            });
        }
    }


    function makeLeaf(IpcEnvelope memory _msg) public pure returns (BottomUpBatch.MerkleHash) {
        // solhint-disable-next-line func-named-parameters
        bytes32 leaf = keccak256(bytes.concat(keccak256(abi.encode(
            _msg.kind,
            _msg.localNonce,
            _msg.originalNonce,
            _msg.value,
            _msg.to.subnetId.root,
            _msg.to.subnetId.route,
            _msg.to.rawAddress.addrType,
            _msg.to.rawAddress.payload,
            _msg.from.subnetId.root,
            _msg.from.subnetId.route,
            _msg.from.rawAddress.addrType,
            _msg.from.rawAddress.payload,
            _msg.message
        ))));
        return BottomUpBatch.MerkleHash.wrap(leaf);
    }

    function bottomUpBatchStorage() internal pure returns (BottomUpBatchStorage storage ds) {
        bytes32 position = NAMESPACE;
        assembly {
            ds.slot := position
        }
        return ds;
    }
}
