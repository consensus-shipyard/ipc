// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {ConsensusType} from "../enums/ConsensusType.sol";
import {BottomUpCheckpoint, CrossMsg} from "../structs/CrossNet.sol";
import {SubnetID} from "../structs/Subnet.sol";
import {SubnetID, ValidatorInfo, Validator, PermissionMode} from "../structs/Subnet.sol";
import {SubnetActorStorage} from "../lib/LibSubnetActorStorage.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";
import {LibStaking} from "../lib/LibStaking.sol";

contract SubnetActorGetterFacet {
    using EnumerableSet for EnumerableSet.AddressSet;
    using SubnetIDHelper for SubnetID;
    using Address for address payable;

    // slither-disable-next-line uninitialized-state
    SubnetActorStorage internal s;

    /// @notice get the parent subnet id
    function getParent() external view returns (SubnetID memory) {
        return s.parentId;
    }

    function permissionMode() external view returns (PermissionMode) {
        return s.validatorSet.permissionMode;
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

    function genesisCircSupply() external view returns (uint256) {
        return s.genesisCircSupply;
    }

    function genesisBalances() external view returns (address[] memory, uint256[] memory) {
        uint256 numAddresses = s.genesisBalanceKeys.length;
        address[] memory addresses = new address[](numAddresses);
        uint256[] memory balances = new uint256[](numAddresses);

        for (uint256 i; i < numAddresses; ) {
            address addr = s.genesisBalanceKeys[i];
            addresses[i] = addr;
            balances[i] = s.genesisBalance[addr];

            unchecked {
                ++i;
            }
        }
        return (addresses, balances);
    }

    function bottomUpCheckPeriod() external view returns (uint256) {
        return s.bottomUpCheckPeriod;
    }

    function bottomUpMsgBatchPeriod() external view returns (uint256) {
        return s.bottomUpMsgBatchPeriod;
    }

    function lastBottomUpCheckpointHeight() external view returns (uint256) {
        return s.lastBottomUpCheckpointHeight;
    }

    function lastBottomUpMsgBatchHeight() external view returns (uint256) {
        return s.lastBottomUpBatch.blockHeight;
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

    /// @notice Get the total number of validators (active and waiting).
    function getTotalValidatorsNumber() external view returns (uint16) {
        return LibStaking.totalValidators();
    }

    /// @notice Get the number of active validators.
    function getActiveValidatorsNumber() external view returns (uint16) {
        return LibStaking.totalActiveValidators();
    }

    function getTotalConfirmedCollateral() external view returns (uint256) {
        return LibStaking.getTotalConfirmedCollateral();
    }

    function getTotalCollateral() external view returns (uint256) {
        return LibStaking.getTotalCollateral();
    }

    function getTotalValidatorCollateral(address validator) external view returns (uint256) {
        return LibStaking.totalValidatorCollateral(validator);
    }

    /// @notice Checks if the validator address is an active validator
    function getPower(address validator) external view returns (uint256) {
        return LibStaking.getPower(validator);
    }

    /// @notice Checks if the validator address is an active validator
    function isActiveValidator(address validator) external view returns (bool) {
        return LibStaking.isActiveValidator(validator);
    }

    /// @notice Checks if the validator is a waiting validator
    function isWaitingValidator(address validator) external view returns (bool) {
        return LibStaking.isWaitingValidator(validator);
    }

    function hasSubmittedInLastBottomUpMsgBatchHeight(address validator) external view returns (bool) {
        uint256 height = s.lastBottomUpBatch.blockHeight;
        return s.relayerRewards.batchRewarded[height].contains(validator);
    }

    function hasSubmittedInLastBottomUpCheckpointHeight(address validator) external view returns (bool) {
        uint256 height = s.lastBottomUpCheckpointHeight;
        return s.relayerRewards.checkpointRewarded[height].contains(validator);
    }

    /// @notice returns the committed bottom-up checkpoint at specific epoch
    /// @param epoch - the epoch to check
    /// @return exists - whether the checkpoint exists
    /// @return checkpoint - the checkpoint struct
    function bottomUpCheckpointAtEpoch(
        uint256 epoch
    ) public view returns (bool exists, BottomUpCheckpoint memory checkpoint) {
        checkpoint = s.committedCheckpoints[epoch];
        exists = !checkpoint.subnetID.isEmpty();
        return (exists, checkpoint);
    }

    /// @notice returns the historical committed bottom-up checkpoint hash
    /// @param epoch - the epoch to check
    /// @return exists - whether the checkpoint exists
    /// @return hash - the hash of the checkpoint
    function bottomUpCheckpointHashAtEpoch(uint256 epoch) external view returns (bool, bytes32) {
        (bool exists, BottomUpCheckpoint memory checkpoint) = bottomUpCheckpointAtEpoch(epoch);
        return (exists, keccak256(abi.encode(checkpoint)));
    }

    /// @notice returns the power scale in number of decimals from whole FIL
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
        for (uint256 i; i < n; ) {
            nodes[i] = s.bootstrapNodes[owners[i]];
            unchecked {
                ++i;
            }
        }
        return nodes;
    }

    /// @notice This exists for testing purposes.
    /// @param messages - cross-messages to hash
    function crossMsgsHash(CrossMsg[] calldata messages) external pure returns (bytes32) {
        return keccak256(abi.encode(messages));
    }

    /// @notice Returns the current reward for the relayer
    /// @param relayer - relayer address
    function getRelayerReward(address relayer) external view returns (uint256) {
        return s.relayerRewards.rewards[relayer];
    }
}
