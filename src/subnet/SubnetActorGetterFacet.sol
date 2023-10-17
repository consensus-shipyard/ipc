// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {ConsensusType} from "../enums/ConsensusType.sol";
import {BottomUpCheckpoint} from "../structs/Checkpoint.sol";
import {SubnetID} from "../structs/Subnet.sol";
import {SubnetID, ValidatorInfo, Validator} from "../structs/Subnet.sol";
import {CheckpointHelper} from "../lib/CheckpointHelper.sol";
import {SubnetActorStorage} from "../lib/LibSubnetActorStorage.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";
import {LibStaking} from "../lib/LibStaking.sol";

contract SubnetActorGetterFacet {
    using EnumerableSet for EnumerableSet.AddressSet;
    using SubnetIDHelper for SubnetID;
    using CheckpointHelper for BottomUpCheckpoint;
    using Address for address payable;

    // slither-disable-next-line uninitialized-state
    SubnetActorStorage internal s;

    /// @notice get the parent subnet id
    function getParent() external view returns (SubnetID memory) {
        return s.parentId;
    }

    function ipcGatewayAddr() external view returns (address) {
        return s.ipcGatewayAddr;
    }

    function minValidators() external view returns (uint64) {
        return s.minValidators;
    }

    function majorityPercentage() external view returns (uint8) {
        return s.majorityPercentage;
    }

    function activeValidatorsLimit() external view returns (uint16) {
        return s.validatorSet.activeLimit;
    }

    function getConfigurationNumbers() external view returns (uint64, uint64) {
        return (s.changeSet.nextConfigurationNumber, s.changeSet.startConfigurationNumber);
    }

    function genesisValidators() external view returns (Validator[] memory) {
        return s.genesisValidators;
    }

    function bottomUpCheckPeriod() external view returns (uint64) {
        return s.bottomUpCheckPeriod;
    }

    function lastBottomUpCheckpointHeight() external view returns (uint64) {
        return s.lastBottomUpCheckpointHeight;
    }

    function consensus() external view returns (ConsensusType) {
        return s.consensus;
    }

    function bootstrapped() external view returns (bool) {
        return s.bootstrapped;
    }

    function killed() external view returns (bool) {
        return s.killed;
    }

    function minActivationCollateral() external view returns (uint256) {
        return s.minActivationCollateral;
    }

    function minCrossMsgFee() external view returns (uint256) {
        return s.minCrossMsgFee;
    }

    /// @notice Get the information of a validator
    function getValidator(address validatorAddress) external view returns (ValidatorInfo memory validator) {
        validator = s.validatorSet.validators[validatorAddress];
    }

    /// @notice Checks if the validator address is an active validator
    function isActiveValidator(address validator) external view returns (bool) {
        return LibStaking.isActiveValidator(validator);
    }

    /// @notice Checks if the validator is a waiting validator
    function isWaitingValidator(address validator) external view returns (bool) {
        return LibStaking.isWaitingValidator(validator);
    }

    function hasSubmittedInLastBottomUpCheckpointHeight(address validator) external view returns (bool) {
        uint64 height = s.lastBottomUpCheckpointHeight;
        return s.rewardedRelayers[height].contains(validator);
    }

    /// @notice returns the committed bottom-up checkpoint at specific epoch
    /// @param epoch - the epoch to check
    /// @return exists - whether the checkpoint exists
    /// @return checkpoint - the checkpoint struct
    function bottomUpCheckpointAtEpoch(
        uint64 epoch
    ) public view returns (bool exists, BottomUpCheckpoint memory checkpoint) {
        checkpoint = s.committedCheckpoints[epoch];
        exists = !checkpoint.subnetID.isEmpty();
        return (exists, checkpoint);
    }

    /// @notice returns the historical committed bottom-up checkpoint hash
    /// @param epoch - the epoch to check
    /// @return exists - whether the checkpoint exists
    /// @return hash - the hash of the checkpoint
    function bottomUpCheckpointHashAtEpoch(uint64 epoch) external view returns (bool, bytes32) {
        (bool exists, BottomUpCheckpoint memory checkpoint) = bottomUpCheckpointAtEpoch(epoch);
        return (exists, checkpoint.toHash());
    }

    function powerScale() external view returns (int8) {
        return s.powerScale;
    }

    /// @notice returns the bootstrap nodes addresses
    function getBootstrapNodes() external view returns (string[] memory) {
        uint256 n = s.bootstrapOwners.length();
        string[] memory nodes = new string[](n);
        if (n == 0) {
            return nodes;
        }
        address[] memory owners = s.bootstrapOwners.values();
        for (uint256 i = 0; i < n; ) {
            nodes[i] = s.bootstrapNodes[owners[i]];
            unchecked {
                ++i;
            }
        }
        return nodes;
    }
}
