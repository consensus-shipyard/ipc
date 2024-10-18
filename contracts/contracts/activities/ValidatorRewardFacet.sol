// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";
import {EnumerableMap} from "@openzeppelin/contracts/utils/structs/EnumerableMap.sol";

import {Pausable} from "../lib/LibPausable.sol";
import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {NotValidator, SubnetNoTargetCommitment, CommitmentAlreadyInitialized, ValidatorAlreadyClaimed, NotGateway, NotOwner} from "../errors/IPCErrors.sol";
import {ValidatorSummary, BatchClaimProofs} from "./Activity.sol";
import {IValidatorRewarder, IValidatorRewardSetup} from "./IValidatorRewarder.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {SubnetID} from "../structs/Subnet.sol";
import {LibActivityMerkleVerifier} from "./LibActivityMerkleVerifier.sol";
import {LibDiamond} from "../lib/LibDiamond.sol";

/// The validator reward facet for the parent subnet, i.e. for validators in the child subnet
/// to claim their reward in the parent subnet, which should be the current subnet this facet
/// is deployed.
contract ValidatorRewardFacet is ReentrancyGuard, Pausable {
    function batchClaim(
        BatchClaimProofs calldata payload
    ) external nonReentrant whenNotPaused {
        uint256 len = payload.proofs.length;
        for (uint256 i = 0; i < len; ) {
            _claim(payload.subnetId, payload.proofs[i].summary, payload.proofs[i].proof);
            unchecked {
                i++;
            }
        }
    }

    /// Validators claim their reward for doing work in the child subnet
    function claim(
        SubnetID calldata subnetId,
        ValidatorSummary calldata summary,
        bytes32[] calldata proof
    ) external nonReentrant whenNotPaused {
        _claim(subnetId, summary, proof);
    }

    function handleRelay() internal pure {
        // no opt for now
        return;
    }

    function _claim(SubnetID calldata subnetId, ValidatorSummary calldata summary, bytes32[] calldata proof) internal {
        // note: No need to check if the subnet is active. If the subnet is not active, the checkpointHeight
        // note: will never exist.

        if (msg.sender != summary.validator) {
            revert NotValidator(msg.sender);
        }

        ValidatorRewardStorage storage s = LibValidatorReward.facetStorage();

        if (s.validatorRewarder == address(0)) {
            return handleRelay();
        }

        LibValidatorReward.handleDistribution(s, subnetId, summary, proof);
    }
}

/// The activity summary commiment that is currently under reward distribution
struct RewardDistribution {
    /// Total number of valdators to claim the distribution
    uint64 totalValidators;
    /// The list of validators that have claimed the reward
    EnumerableSet.AddressSet claimed;
}

/// Used by the SubnetActor to track the rewards for each validator
struct ValidatorRewardStorage {
    /// @notice The contract address for validator rewarder
    address validatorRewarder;
    /// @notice Summaries look up pending to be processed.
    /// If the validator rewarder is non-zero, these denote summaries presentable at this level.
    /// If the validator rewarder is zero, these summaries must be relayed upwards in the next bottom-up checkpoint.
    /// Partitioned by subnet ID (hash) then by checkpoint block height in the child subnet to the commitment
    mapping(bytes32 => EnumerableMap.Bytes32ToBytes32Map) commitments;
    /// @notice Index over presentable summaries back to the subnet ID, so we can locate them quickly when they're presented.
    /// Only used if the validator rewarder is non-zero.
    /// Partitioned by subnet ID (hash) then by checkpoint block height in the child subnet to the commitment
    mapping(bytes32 => mapping(uint64 => RewardDistribution)) distributions;
}

/// The payload for list commitments query
struct ListCommimentDetail {
    /// The child subnet checkpoint height
    uint64 checkpointHeight;
    /// The actual commiment of the activities
    bytes32 commitment;
}

library LibValidatorReward {
    bytes32 private constant NAMESPACE = keccak256("validator.reward.parent");

    using SubnetIDHelper for SubnetID;
    using EnumerableMap for EnumerableMap.Bytes32ToBytes32Map;
    using EnumerableSet for EnumerableSet.AddressSet;

    // =========== External library functions =============

    function initNewDistribution(
        SubnetID calldata subnetId,
        uint64 checkpointHeight,
        bytes32 commitment,
        uint64 totalActiveValidators
    ) internal {
        ValidatorRewardStorage storage ds = facetStorage();

        bytes32 subnetKey = subnetId.toHash();

        if (ds.distributions[subnetKey][checkpointHeight].totalValidators != 0) {
            revert CommitmentAlreadyInitialized();
        }

        ds.commitments[subnetKey].set(bytes32(uint256(checkpointHeight)), commitment);
        ds.distributions[subnetKey][checkpointHeight].totalValidators = totalActiveValidators;
    }

    function listCommitments(
        SubnetID calldata subnetId
    ) internal view returns (ListCommimentDetail[] memory listDetails) {
        ValidatorRewardStorage storage ds = facetStorage();

        bytes32 subnetKey = subnetId.toHash();

        uint256 size = ds.commitments[subnetKey].length();
        listDetails = new ListCommimentDetail[](size);

        for (uint256 i = 0; i < size; ) {
            (bytes32 heightBytes32, bytes32 commitment) = ds.commitments[subnetKey].at(i);

            listDetails[i] = ListCommimentDetail({
                checkpointHeight: uint64(uint256(heightBytes32)),
                commitment: commitment
            });

            unchecked {
                i++;
            }
        }

        return listDetails;
    }

    function updateRewarder(address rewarder) internal {
        ValidatorRewardStorage storage ds = facetStorage();
        ds.validatorRewarder = rewarder;
    }

    // ============ Internal library functions ============

    function facetStorage() internal pure returns (ValidatorRewardStorage storage ds) {
        bytes32 position = NAMESPACE;
        assembly {
            ds.slot := position
        }
        return ds;
    }

    function handleDistribution(
        ValidatorRewardStorage storage s,
        SubnetID calldata subnetId,
        ValidatorSummary calldata summary,
        bytes32[] calldata proof
    ) internal {
        bytes32 subnetKey = subnetId.toHash();

        bytes32 commitment = ensureValidCommitment(s, subnetKey, summary.checkpointHeight);
        LibActivityMerkleVerifier.ensureValidProof(commitment, summary, proof);

        validatorTryClaim(s, subnetKey, summary.checkpointHeight, summary.validator);
        IValidatorRewarder(s.validatorRewarder).disburseRewards(subnetId, summary);
    }

    function ensureValidCommitment(
        ValidatorRewardStorage storage ds,
        bytes32 subnetKey,
        uint64 checkpointHeight
    ) internal view returns (bytes32) {
        (bool exists, bytes32 commitment) = ds.commitments[subnetKey].tryGet(bytes32(uint256(checkpointHeight)));
        if (!exists) {
            revert SubnetNoTargetCommitment();
        }

        // Note: ideally we should check the commitment actually exists, but we dont have to as
        // Note: the code will ensure if commitments contains the commitment,
        // Note: the commitment will have distribution
        // if (ds.distributions[commitment].checkpointHeight == 0) {
        //     revert CommitmentNotFound();
        // }

        return commitment;
    }

    /// Validator tries to claim the reward. The validator can only claim the reward if the validator
    /// has not claimed before
    function validatorTryClaim(
        ValidatorRewardStorage storage ds,
        bytes32 subnetKey,
        uint64 checkpointHeight,
        address validator
    ) internal {
        if (ds.distributions[subnetKey][checkpointHeight].claimed.contains(validator)) {
            revert ValidatorAlreadyClaimed();
        }

        ds.distributions[subnetKey][checkpointHeight].claimed.add(validator);
    }

    /// Try to remove the distribution in the target subnet when ALL VALIDATORS HAVE CLAIMED.
    function tryPurgeDistribution(
        ValidatorRewardStorage storage ds,
        SubnetID calldata subnetId,
        uint64 checkpointHeight
    ) internal {
        bytes32 subnetKey = subnetId.toHash();

        uint256 total = uint256(ds.distributions[subnetKey][checkpointHeight].totalValidators);
        uint256 claimed = ds.distributions[subnetKey][checkpointHeight].claimed.length();

        if (claimed < total) {
            return;
        }

        delete ds.commitments[subnetKey];
        delete ds.distributions[subnetKey][checkpointHeight];
    }
}
