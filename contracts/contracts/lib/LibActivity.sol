// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "../activities/IValidatorRewarder.sol";
import {Consensus, CompressedActivityRollup} from "../activities/Activity.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import {MerkleProof} from "@openzeppelin/contracts/utils/cryptography/MerkleProof.sol";
import {SubnetID} from "../structs/Subnet.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {InvalidActivityProof, MissingActivityCommitment, ValidatorAlreadyClaimed} from "../errors/IPCErrors.sol";

/// Library to handle activity rollups.
library LibActivity {
    bytes32 private constant CONSENSUS_NAMESPACE = keccak256("activity.consensus");

    using SubnetIDHelper for SubnetID;
    using EnumerableSet for EnumerableSet.AddressSet;
    using EnumerableSet for EnumerableSet.Bytes32Set;

    // Newtype for extra safety.
    type SubnetKey is bytes32;

    /// An object to track consensus-related activity submissions from subnets.
    struct ConsensusTracker {
        /// An enumeration of checkpoint heights that are pending.
        EnumerableSet.Bytes32Set pendingHeights;
        /// The pending summaries for each checkpoint height, including information about the validators that have already claimed rewards.
        mapping(uint64 => ConsensusPendingAtHeight) pending;
    }

    struct ConsensusPendingAtHeight {
        /// The original compressed summary submitted for this height.
        /// We store the summary in full, or we relay it upwards if there is no validator rewarder configured.
        Consensus.CompressedSummary summary;
        /// Tracks validators have already claimed rewards for this height.
        EnumerableSet.AddressSet claimed;
    }

    /// Used by the SubnetActor to track the rewards for each validator
    struct ConsensusStorage {
        /// @notice The contract address for the validator rewarder.
        address validatorRewarder;
        /// @notice Tracks pending summaries to be processed.
        /// If the validator rewarder is non-zero, these denote summaries presentable at this level.
        /// If the validator rewarder is zero, these summaries must be relayed upwards in the next bottom-up checkpoint.
        /// Partitioned by subnet ID (hash) then by checkpoint block height in the child subnet to the commitment
        mapping(SubnetKey => ConsensusTracker) tracker;
    }

    /// Return type for the list commitments view method.
    struct ListPendingReturnEntry {
        uint64 height;
        Consensus.CompressedSummary summary;
        address[] claimed;
    }

    function ensureValidProof(
        Consensus.MerkleHash commitment,
        Consensus.ValidatorData calldata detail,
        Consensus.MerkleHash[] calldata proof
    ) internal pure {
        // Constructing leaf: https://github.com/OpenZeppelin/merkle-tree#leaf-hash
        bytes32 leaf = keccak256(bytes.concat(keccak256(abi.encode(detail.validator, detail.blocksCommitted))));
        // converting proof to bytes32[]
        bytes32[] memory proofBytes = new bytes32[](proof.length);
        for (uint256 i = 0; i < proof.length; i++) {
            proofBytes[i] = Consensus.MerkleHash.unwrap(proof[i]);
        }
        bool valid = MerkleProof.verify({proof: proofBytes, root: Consensus.MerkleHash.unwrap(commitment), leaf: leaf});
        if (!valid) {
            revert InvalidActivityProof();
        }
    }

    // =========== External library functions =============

    function recordActivityRollup(
        SubnetID calldata subnet,
        uint64 checkpointHeight,
        CompressedActivityRollup calldata activity
    ) internal {
        ConsensusStorage storage s = consensusStorage();

        SubnetKey subnetKey = SubnetKey.wrap(subnet.toHash());

        ConsensusTracker storage tracker = s.tracker[subnetKey];
        bool added = tracker.pendingHeights.add(bytes32(uint256(checkpointHeight)));
        require(added, "duplicate checkpoint height");

        ConsensusPendingAtHeight storage pending = tracker.pending[checkpointHeight];
        pending.summary = activity.consensus;
    }

    /// A view accessor to query the pending consensus summaries for a given subnet.
    function listPendingConsensus(
        SubnetID calldata subnet
    ) internal view returns (ListPendingReturnEntry[] memory result) {
        ConsensusStorage storage s = consensusStorage();

        SubnetKey subnetKey = SubnetKey.wrap(subnet.toHash());

        uint256 size = s.tracker[subnetKey].pendingHeights.length();
        result = new ListPendingReturnEntry[](size);

        // Ok to not optimize with unchecked increments, since we expect this to be used off-chain only, for introspection.
        for (uint256 i = 0; i < size; i++) {
            ConsensusTracker storage tracker = s.tracker[subnetKey];
            bytes32[] memory heights = tracker.pendingHeights.values();

            for (uint256 j = 0; j < heights.length; j++) {
                uint64 height = uint64(uint256(heights[j]) << 192 >> 192);
                ConsensusPendingAtHeight storage pending = tracker.pending[height];
                result[i] = ListPendingReturnEntry({
                    height: height,
                    summary: pending.summary,
                    claimed: pending.claimed.values()
                });
            }
        }

        return result;
    }

    function processConsensusClaim(
        SubnetID calldata subnet,
        uint64 checkpointHeight,
        Consensus.ValidatorData calldata data,
        Consensus.MerkleHash[] calldata proof
    ) internal {
        ConsensusStorage storage s = consensusStorage();

        SubnetKey subnetKey = SubnetKey.wrap(subnet.toHash());

        // Check the validity of the proof.
        ConsensusPendingAtHeight storage pending = s.tracker[subnetKey].pending[checkpointHeight];
        Consensus.MerkleHash commitment = pending.summary.dataRootCommitment;
        if (Consensus.MerkleHash.unwrap(commitment) == bytes32(0)) {
            revert MissingActivityCommitment();
        }
        ensureValidProof(commitment, data, proof);

        // Mark the validator as claimed.
        bool added = pending.claimed.add(data.validator);
        if (!added) {
            revert ValidatorAlreadyClaimed();
        }

        // Notify the validator rewarder of a valid claim.
        IValidatorRewarder(s.validatorRewarder).notifyValidClaim(subnet, data);

        // Prune state for this height if all validators have claimed.
        if (pending.claimed.length() == pending.summary.stats.totalActiveValidators) {
            ConsensusTracker storage tracker = s.tracker[subnetKey];
            tracker.pendingHeights.remove(bytes32(uint256(checkpointHeight)));
            delete tracker.pending[checkpointHeight];
        }
    }

    function setRewarder(address rewarder) internal {
        ConsensusStorage storage ds = consensusStorage();
        ds.validatorRewarder = rewarder;
    }

    // ============ Internal library functions ============

    function consensusStorage() internal pure returns (ConsensusStorage storage ds) {
        bytes32 position = CONSENSUS_NAMESPACE;
        assembly {
            ds.slot := position
        }
        return ds;
    }
}
