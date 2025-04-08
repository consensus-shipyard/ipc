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
import {InvalidInclusionProof, BatchMsgAlreadyExecuted, MissingBatchCommitment} from "../errors/IPCErrors.sol";
import {BottomUpBatch} from "../structs/BottomUpBatch.sol";

/// Library to handle bottom up batch 2-phase execution.
library LibBottomUpBatch {
    bytes32 private constant NAMESPACE = keccak256("bottomupbatch");

    using SubnetIDHelper for SubnetID;
    using EnumerableSet for EnumerableSet.Bytes32Set;

    type SubnetKey is bytes32;

    /// @notice Tracks the execution status of committed bottom-up message batches for a specific subnet.
    struct Tracker {
        /// @notice Set of checkpoint heights with batches that are still pending execution.
        EnumerableSet.Bytes32Set pendingHeights;
        /// @notice Mapping of checkpoint height to its pending batch data.
        mapping(uint256 => BatchPending) pending;
    }

    /// @notice Represents a pending bottom-up batch commitment awaiting full execution at a specific checkpoint height.
    struct BatchPending {
      /// @notice The pending batch commitment.
      BottomUpBatch.Commitment commitment;
      /// @notice Set of message leaf hashes that have already been executed for this batch.
      EnumerableSet.Bytes32Set executed;
    }

    /// @notice Storage structure used by the SubnetActor to manage bottom-up message batches and their execution status.
    struct BottomUpBatchStorage {
        /// @notice Tracks pending batches and their execution progress per subnet key.
        mapping(SubnetKey => Tracker) tracker;
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
        SubnetID calldata subnet,
        uint64 checkpointHeight,
        BottomUpBatch.Commitment calldata commitment
    ) internal {
        BottomUpBatchStorage storage s = bottomUpBatchStorage();

        SubnetKey subnetKey = SubnetKey.wrap(subnet.toHash());

        Tracker storage tracker = s.tracker[subnetKey];
        bool added = tracker.pendingHeights.add(bytes32(uint256(checkpointHeight)));
        require(added, "duplicate checkpoint height");

        BatchPending storage pending = tracker.pending[checkpointHeight];
        pending.commitment = commitment;
    }

    function processBottomUpBatchMsg(
        SubnetID calldata subnet,
        uint256 checkpointHeight,
        IpcEnvelope calldata ipcMsg,
        BottomUpBatch.MerkleHash[] calldata proof
    ) internal {
        BottomUpBatchStorage storage s = bottomUpBatchStorage();

        SubnetKey subnetKey = SubnetKey.wrap(subnet.toHash());

        // Find the pending batch.
        BatchPending storage pending = s.tracker[subnetKey].pending[checkpointHeight];
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
            Tracker storage tracker = s.tracker[subnetKey];
            tracker.pendingHeights.remove(bytes32(uint256(checkpointHeight)));
            delete tracker.pending[checkpointHeight];
        }
    }

    /// Return type for the list pending commitments view method.
    struct ListPendingCommitmentsEntry {
        uint64 height;
        BottomUpBatch.Commitment commitment;
        bytes32[] executed;
    }

    /// A view accessor to query the pending commitments for a given subnet.
    function listPendingCommitments(
        SubnetID calldata subnet
    ) internal view returns (ListPendingCommitmentsEntry[] memory result) {
        BottomUpBatchStorage storage s = bottomUpBatchStorage();

        SubnetKey subnetKey = SubnetKey.wrap(subnet.toHash());

        uint256 size = s.tracker[subnetKey].pendingHeights.length();
        result = new ListPendingCommitmentsEntry[](size);

        // Ok to not optimize with unchecked increments, since we expect this to be used off-chain only, for introspection.
        for (uint256 i = 0; i < size; i++) {
            Tracker storage tracker = s.tracker[subnetKey];
            bytes32[] memory heights = tracker.pendingHeights.values();

            for (uint256 j = 0; j < heights.length; j++) {
                uint64 height = uint64(uint256(heights[j]) << 192 >> 192);
                BatchPending storage pending = tracker.pending[height];
                result[i] = ListPendingCommitmentsEntry({
                    height: height,
                    commitment: pending.commitment,
                    executed: pending.executed.values()
                });
            }
        }

        return result;
    }

    function makeLeaf(IpcEnvelope memory _msg) internal pure returns (BottomUpBatch.MerkleHash) {
        bytes32 leaf = keccak256(bytes.concat(keccak256(abi.encode(_msg))));
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
