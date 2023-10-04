// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {ConsensusType} from "../enums/ConsensusType.sol";
import {Status} from "../enums/Status.sol";
import {NotEnoughValidatorsInSubnet} from "../errors/IPCErrors.sol";
import {BottomUpCheckpoint} from "../structs/Checkpoint.sol";
import {FvmAddress} from "../structs/FvmAddress.sol";
import {SubnetID} from "../structs/Subnet.sol";
import {ValidatorInfo, ValidatorSet} from "../structs/Validator.sol";
import {CheckpointHelper} from "../lib/CheckpointHelper.sol";
import {SubnetActorStorage} from "../lib/LibSubnetActorStorage.sol";
import {FvmAddressHelper} from "../lib/FvmAddressHelper.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";

contract SubnetActorGetterFacet {
    using EnumerableSet for EnumerableSet.AddressSet;
    using SubnetIDHelper for SubnetID;
    using CheckpointHelper for BottomUpCheckpoint;
    using FilAddress for address;
    using Address for address payable;

    // slither-disable-next-line uninitialized-state-variables
    SubnetActorStorage internal s;

    /// @notice get the parent subnet id
    function getParent() external view returns (SubnetID memory) {
        return s.parentId;
    }

    /// @notice get the current status
    function status() external view returns (Status) {
        return s.status;
    }

    /// @notice get the total stake
    function totalStake() external view returns (uint256) {
        return s.totalStake;
    }

    function prevExecutedCheckpointHash() external view returns (bytes32) {
        return s.prevExecutedCheckpointHash;
    }

    function accumulatedRewards(address a) external view returns (uint256) {
        return s.accumulatedRewards[a];
    }

    function stake(address a) external view returns (uint256) {
        return s.stake[a];
    }

    function ipcGatewayAddr() external view returns (address) {
        return s.ipcGatewayAddr;
    }

    function minValidators() external view returns (uint64) {
        return s.minValidators;
    }

    function topDownCheckPeriod() external view returns (uint64) {
        return s.topDownCheckPeriod;
    }

    function bottomUpCheckPeriod() external view returns (uint64) {
        return s.bottomUpCheckPeriod;
    }

    function consensus() external view returns (ConsensusType) {
        return s.consensus;
    }

    function minActivationCollateral() external view returns (uint256) {
        return s.minActivationCollateral;
    }

    function name() external view returns (bytes32) {
        return s.name;
    }

    /// @notice get validator count
    function validatorCount() external view returns (uint256) {
        return s.validators.length();
    }

    /// @notice get validator at index
    /// @param index - the index of the validator set
    function validatorAt(uint256 index) external view returns (address) {
        return s.validators.at(index);
    }

    /// @notice get validator network address
    /// @param addr - validator address
    function validatorNetAddr(address addr) external view returns (string memory) {
        return s.validatorNetAddresses[addr];
    }

    /// @notice get validator worker address
    /// @param addr - validator address
    function validatorWorkerAddr(address addr) external view returns (FvmAddress memory) {
        return s.validatorWorkerAddresses[addr];
    }

    /// @notice get all the validators in the subnet.
    function getValidators() external view returns (address[] memory) {
        uint256 length = s.validators.length();
        address[] memory result = new address[](length);

        for (uint256 i = 0; i < length; ) {
            result[i] = s.validators.at(i);
            unchecked {
                ++i;
            }
        }

        return result;
    }

    /// @notice get no more than `limit` number of validators starting from the validator with index `offset`
    /// @dev It returns an empty array[] and 0 if there are no validators to return according to the input parameters
    /// @param offset The first index of the first validator to return
    /// @param limit The maximum number of validators to return
    /// @return the array of validators, the size of that array is no more than `limit`
    /// @return the next `offset` that needs to query the next range of validators
    function getRangeOfValidators(uint256 offset, uint256 limit) external view returns (address[] memory, uint256) {
        uint256 n = s.validators.length();
        address[] memory empty = new address[](0);
        if (limit == 0) {
            return (empty, 0);
        }
        if (n <= offset) {
            return (empty, 0);
        }

        if (limit > n - offset) {
            limit = n - offset;
        }
        address[] memory result = new address[](limit);

        for (uint256 i = 0; i < limit; ) {
            result[i] = s.validators.at(i + offset);
            unchecked {
                ++i;
            }
        }

        return (result, offset + limit);
    }

    /// @notice returns the configuration number.
    function getConfigurationNumber() external view returns (uint256) {
        return s.configurationNumber;
    }

    // TODO: is this relevant? should it be updated or deleted?
    /// @notice get the full details of the validators, not just their addresses
    function getValidatorSet() external view returns (ValidatorSet memory) {
        uint256 length = s.validators.length();

        ValidatorInfo[] memory details = new ValidatorInfo[](length);
        address a;

        for (uint256 i = 0; i < length; ) {
            a = s.validators.at(i);
            details[i] = ValidatorInfo({
                addr: a,
                weight: s.stake[a],
                workerAddr: s.validatorWorkerAddresses[a],
                netAddresses: s.validatorNetAddresses[a]
            });
            unchecked {
                ++i;
            }
        }

        return ValidatorSet({validators: details, configurationNumber: s.configurationNumber});
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
