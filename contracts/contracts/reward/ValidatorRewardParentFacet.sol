// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

import {Pausable} from "../lib/LibPausable.sol";
import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {NotValidator, SubnetNoTargetCommitment, CommitmentAlreadyInitialized, ValidatorAlreadyClaimed} from "../errors/IPCErrors.sol";
import {ValidatorSummary, ActivitySummary, LibActivitySummary} from "./ValidatorReward.sol";
import {IValidatorRewarder} from "./IValidatorRewarder.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {SubnetID} from "../structs/Subnet.sol";

/// The validator reward facet for the parent subnet, i.e. for validators in the child subnet
/// to claim their reward in the parent subnet, which should be the current subnet this facet
/// is deployed.
contract ValidatorRewardParentFacet is ReentrancyGuard, Pausable {
    using LibActivitySummary for ActivitySummary;

    /// Validators claim their reward for doing work in the child subnet
    function claim(SubnetID calldata subnetId, ActivitySummary calldata summary) external nonReentrant whenNotPaused {
        if (!summary.containsValidator(msg.sender)) {
            revert NotValidator(msg.sender);
        }

        // todo: check subnet is active

        ValidatorRewardParentStorage storage s = LibValidatorRewardParent.facetStorage();

        if (s.validatorRewarder == address(0)) {
            handleRelay();
        } else {
            handleDistribution(s, subnetId, summary);
        }
    }

    function handleRelay() internal pure {
        revert("not implemented yet");
    }

    function handleDistribution(ValidatorRewardParentStorage storage s, SubnetID calldata subnetId, ActivitySummary calldata summary) internal {
        bytes32 commitment = summary.commitment();

        LibValidatorRewardParent.ensureValidCommitment(s, commitment, subnetId);

        LibValidatorRewardParent.validatorTryClaim(s, commitment, msg.sender);
        IValidatorRewarder(s.validatorRewarder).disburseRewards(subnetId, msg.sender, summary);

        LibValidatorRewardParent.tryPurgeCommitment(s, subnetId, commitment, summary.numValidators());
    }
}

/// The activity summary commiment that is currently under reward distribution
struct RewardDistribution {
    /// The checkpoint height that this distribution
    uint64 checkpointHeight;
    /// The list of validators that have claimed the reward
    EnumerableSet.AddressSet claimed;
}

/// Used by the SubnetActor to track the rewards for each validator
struct ValidatorRewardParentStorage {
    /// @notice The contract address for validator rewarder
    address validatorRewarder;
    /// @notice Summaries pending to be processed.
    /// If the validator rewarder is non-zero, these denote summaries presentable at this level.
    /// If the validator rewarder is zero, these summaries must be relayed upwards in the next bottom-up checkpoint.
    /// Partitioned by subnet ID (hash), in the sequence they must be presented.
    mapping(bytes32 => EnumerableSet.Bytes32Set) commitmentsToDistribution;
    /// @notice Index over presentable summaries back to the subnet ID, so we can locate them quickly when they're presented.
    /// Only used if the validator rewarder is non-zero.
    /// TODO(rewarder): optimize this pair of data structures.
    mapping(bytes32 => RewardDistribution) distributions;
}

library LibValidatorRewardParent {
    bytes32 private constant NAMESPACE = keccak256("validator.reward.parent");

    using SubnetIDHelper for SubnetID;
    using EnumerableSet for EnumerableSet.Bytes32Set;
    using EnumerableSet for EnumerableSet.AddressSet;

    function facetStorage() internal pure returns (ValidatorRewardParentStorage storage ds) {
        bytes32 position = NAMESPACE;
        assembly {
            ds.slot := position
        }
        return ds;
    }

    function listCommitments(ValidatorRewardParentStorage storage ds, SubnetID calldata subnetId) internal view returns(bytes32[] memory) {
        bytes32 subnetKey = subnetId.toHash();
        return ds.commitmentsToDistribution[subnetKey].values();
    }

    function initNewDistribution(bytes32 commitment, SubnetID calldata subnetId) internal {
        ValidatorRewardParentStorage storage ds = facetStorage();

        bytes32 subnetKey = subnetId.toHash();

        if (ds.commitmentsToDistribution[subnetKey].contains(commitment)) {
            revert CommitmentAlreadyInitialized();
        }

        ds.commitmentsToDistribution[subnetKey].add(commitment);
        ds.distributions[commitment].checkpointHeight = uint64(block.number);
    }

    function ensureValidCommitment(ValidatorRewardParentStorage storage ds, bytes32 commitment, SubnetID calldata subnetId) internal view {
        bytes32 subnetKey = subnetId.toHash();

        if (!ds.commitmentsToDistribution[subnetKey].contains(commitment)) {
            revert SubnetNoTargetCommitment();
        }

        // Note: ideally we should check the commitment actually exists, but we dont have to as
        // Note: the code will ensure if commitmentsToDistribution contains the commitment,
        // Note: the commitment will have distribution
        // if (ds.distributions[commitment].checkpointHeight == 0) {
        //     revert CommitmentNotFound();
        // }
    }

    /// Validator tries to claim the reward. The validator can only claim the reward if the validator
    /// has not claimed before
    function validatorTryClaim(ValidatorRewardParentStorage storage ds, bytes32 commitment, address validator) internal {
        if(ds.distributions[commitment].claimed.contains(validator)) {
            revert ValidatorAlreadyClaimed();
        }

        ds.distributions[commitment].claimed.add(validator);
    }

    /// Try to remove the commiment in the target subnet when ALL VALIDATORS HAVE CLAIMED.
    function tryPurgeCommitment(ValidatorRewardParentStorage storage ds, SubnetID calldata subnetId, bytes32 commitment, uint64 totalValidators) internal {
        bytes32 subnetKey = subnetId.toHash();

        if (ds.distributions[commitment].claimed.length() < totalValidators) {
            return;
        }

        ds.commitmentsToDistribution[subnetKey].remove(commitment);
        delete ds.distributions[commitment];
    }
}