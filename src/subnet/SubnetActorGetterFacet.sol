// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {ConsensusType} from "../enums/ConsensusType.sol";
import {NotEnoughValidatorsInSubnet} from "../errors/IPCErrors.sol";
import {BottomUpCheckpoint} from "../structs/Checkpoint.sol";
import {FvmAddress} from "../structs/FvmAddress.sol";
import {SubnetID} from "../structs/Subnet.sol";
import {SubnetID, GenesisValidator, Validator} from "../structs/Subnet.sol";
import {CheckpointHelper} from "../lib/CheckpointHelper.sol";
import {SubnetActorStorage} from "../lib/LibSubnetActorStorage.sol";
import {FvmAddressHelper} from "../lib/FvmAddressHelper.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
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

    function getConfigurationNumbers() external view returns (uint64, uint64) {
        return (s.changeSet.nextConfigurationNumber, s.changeSet.startConfigurationNumber);
    }

    function genesisValidators() external view returns (GenesisValidator[] memory) {
        return s.genesisValidators;
    }

    function bottomUpCheckPeriod() external view returns (uint64) {
        return s.bottomUpCheckPeriod;
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

    function name() external view returns (bytes32) {
        return s.name;
    }

    /// @notice Get the information of a validator
    function getValidator(address validatorAddress) external view returns (Validator memory validator) {
        validator = s.validatorSet.validators[validatorAddress];
    }

    /// @notice Checks if the validator address is an active validator
    function isActiveValidator(address validator) external view returns (bool) {
        return LibStaking.isActiveValidator(validator);
    }

    /// @notice Checks if the validator is a waiting validator
    function isWaitingValidator(address validator) internal view returns (bool) {
        return LibStaking.isWaitingValidator(validator);
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
}
