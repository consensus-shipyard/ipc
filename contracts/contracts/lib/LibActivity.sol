// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import "../activities/IValidatorRewarder.sol";
import {Consensus, CompressedActivityRollup} from "../activities/Activity.sol";
import {EnumerableMap} from "@openzeppelin/contracts/utils/structs/EnumerableMap.sol";
import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import {InvalidActivityProof} from "../../errors/IPCErrors.sol";
import {MerkleProof} from "@openzeppelin/contracts/utils/cryptography/MerkleProof.sol";
import {SubnetID} from "../structs/Subnet.sol";
import {MissingActivityCommitment, DuplicateCommitment, ValidatorAlreadyClaimed} from "../errors/IPCErrors.sol";

/// Library to handle activity rollups.
library LibActivity {
    bytes32 private constant NAMESPACE = keccak256("activity.consensus");

    using SubnetIDHelper for SubnetID;
    using EnumerableMap for EnumerableMap.Bytes32ToBytes32Map;
    using EnumerableSet for EnumerableSet.AddressSet;

    // Newtype for extra safety.
    type SubnetKey is bytes32;

    /// The activity summary commiment that is currently under reward distribution
    struct ConsensusProcessed {
        /// Total number of valdators to claim the distribution
        uint64 totalValidators;
        /// The list of validators that have claimed the reward
        EnumerableSet.AddressSet claimed;
    }

    /// Used by the SubnetActor to track the rewards for each validator
    struct ActivityConsensusStorage {
        /// @notice The contract address for the validator rewarder.
        address validatorRewarder;
        /// @notice Summaries pending to be processed.
        /// If the validator rewarder is non-zero, these denote summaries presentable at this level.
        /// If the validator rewarder is zero, these summaries must be relayed upwards in the next bottom-up checkpoint.
        /// Partitioned by subnet ID (hash) then by checkpoint block height in the child subnet to the commitment
        mapping(SubnetKey => EnumerableMap.Bytes32ToBytes32Map) pending;
        /// @notice Index over presentable summaries back to the subnet ID, so we can locate them quickly when they're presented.
        /// Only used if the validator rewarder is non-zero.
        /// Partitioned by subnet ID (hash) then by checkpoint block height in the child subnet to the commitment
        mapping(SubnetKey => mapping(uint64 => ConsensusProcessed)) processed;
    }

    /// The payload for list commitments query
    struct ListPendingResult {
        /// The child subnet checkpoint height
        uint64 checkpointHeight;
        /// The actual commiment of the activities
        bytes32 commitment;
    }

    function ensureValidProof(
        bytes32 commitment,
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
        bool valid = MerkleProof.verify({proof: proofBytes, root: commitment, leaf: leaf});
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
        ActivityConsensusStorage storage ds = consensusStorage();

        SubnetKey subnetKey = SubnetKey.wrap(subnet.toHash());

        if (ds.processed[subnetKey][checkpointHeight].totalValidators != 0) {
            revert DuplicateCommitment();
        }

        ds.pending[subnetKey].set(bytes32(uint256(checkpointHeight)), Consensus.MerkleHash.unwrap(activity.consensus.dataRootCommitment));
        ds.processed[subnetKey][checkpointHeight].totalValidators = activity.consensus.stats.totalActiveValidators;
    }

    function listPending(
        SubnetID calldata subnet
    ) internal view returns (ListPendingResult[] memory result) {
        ActivityConsensusStorage storage ds = consensusStorage();

        SubnetKey subnetKey = SubnetKey.wrap(subnet.toHash());

        uint256 size = ds.pending[subnetKey].length();
        result = new ListPendingResult[](size);

        for (uint256 i = 0; i < size; ) {
            (bytes32 heightBytes32, bytes32 commitment) = ds.pending[subnetKey].at(i);

            result[i] = ListPendingResult({
                checkpointHeight: uint64(uint256(heightBytes32)),
                commitment: commitment
            });

            unchecked {
                i++;
            }
        }

        return result;
    }

    function setRewarder(address rewarder) internal {
        ActivityConsensusStorage storage ds = consensusStorage();
        ds.validatorRewarder = rewarder;
    }

    // ============ Internal library functions ============

    function consensusStorage() internal pure returns (ActivityConsensusStorage storage ds) {
        bytes32 position = NAMESPACE;
        assembly {
            ds.slot := position
        }
        return ds;
    }

    function processConsensusClaim(
        SubnetID calldata subnet,
        uint64 checkpointHeight,
        Consensus.ValidatorData calldata data,
        Consensus.MerkleHash[] calldata proof
    ) internal {
        ActivityConsensusStorage storage s = consensusStorage();

        bytes32 subnetKey = subnet.toHash();

        (bool exists, bytes32 commitment) = s.pending[subnetKey].tryGet(bytes32(uint256(checkpointHeight)));
        if (!exists) {
            revert MissingActivityCommitment();
        }
        ensureValidProof(commitment, data, proof);

        EnumerableSet.AddressSet storage claimed = s.processed[subnetKey][checkpointHeight].claimed;
        if (claimed.contains(data.validator)) {
            revert ValidatorAlreadyClaimed();
        }
        claimed.add(data.validator);
        IValidatorRewarder(s.validatorRewarder).disburseRewards(subnet, data);

        // Prune state if all validators have claimed.
        if (claimed.length() == s.processed[subnetKey][checkpointHeight].totalValidators) {
            delete s.pending[subnetKey];
            delete s.processed[subnetKey][checkpointHeight];
        }
    }

}
