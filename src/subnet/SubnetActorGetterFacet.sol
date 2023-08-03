// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.19;

import {FvmAddress} from "../structs/FvmAddress.sol";
import {BottomUpCheckpoint, CrossMsg, ChildCheck} from "../structs/Checkpoint.sol";
import {SubnetID} from "../structs/Subnet.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {SubnetActorStorage} from "../lib/LibSubnetActorStorage.sol";
import {CheckpointHelper} from "../lib/CheckpointHelper.sol";
import {EpochVoteSubmission} from "../structs/EpochVoteSubmission.sol";
import {ISubnetActor} from "../interfaces/ISubnetActor.sol";
import {IGateway} from "../interfaces/IGateway.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";
import {ExecutableQueue} from "../structs/ExecutableQueue.sol";
import {ExecutableQueueHelper} from "../lib/ExecutableQueueHelper.sol";
import {EpochVoteBottomUpSubmission} from "../structs/EpochVoteSubmission.sol";
import {ValidatorInfo, ValidatorSet} from "../structs/Validator.sol";
import {EpochVoteSubmissionHelper} from "../lib/EpochVoteSubmissionHelper.sol";
import {LibVoting} from "../lib/LibVoting.sol";
import {Status} from "../enums/Status.sol";
import {ConsensusType} from "../enums/ConsensusType.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";
import {FvmAddressHelper} from "../lib/FvmAddressHelper.sol";

contract SubnetActorGetterFacet {
    using EnumerableSet for EnumerableSet.AddressSet;
    using SubnetIDHelper for SubnetID;
    using CheckpointHelper for BottomUpCheckpoint;
    using FilAddress for address;
    using Address for address payable;
    using ExecutableQueueHelper for ExecutableQueue;
    using EpochVoteSubmissionHelper for EpochVoteSubmission;
    using CrossMsgHelper for CrossMsg;
    using FvmAddressHelper for FvmAddress;

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

    function lastVotingExecutedEpoch() external view returns (uint64) {
        return LibVoting.lastVotingExecutedEpoch();
    }

    function executableQueue() external view returns (uint64, uint64, uint64) {
        // slither-disable-next-line unused-return
        return LibVoting.executableQueue();
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

    function genesis() external view returns (bytes memory) {
        return s.genesis;
    }

    function majorityPercentage() external view returns (uint64) {
        return LibVoting.majorityPercentage();
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

    /// @notice get all the validators in the subnet.
    /// TODO: we can introduce pagination
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

    /// @notice get the full details of the validators, not just their addresses.
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

    /// @notice returns the list of registered subnets in IPC
    function listBottomUpCheckpoints(
        uint64 fromEpoch,
        uint64 toEpoch
    ) external view returns (BottomUpCheckpoint[] memory) {
        uint64 period = s.bottomUpCheckPeriod;

        // slither-disable-next-line divide-before-multiply
        uint64 from = (fromEpoch / period) * period;
        // slither-disable-next-line divide-before-multiply
        uint64 to = (toEpoch / period) * period;

        uint64 size = (to - from) / period;
        BottomUpCheckpoint[] memory out = new BottomUpCheckpoint[](size);

        uint64 nextEpoch = from;
        for (uint64 i = 0; i < size; ) {
            out[i] = s.committedCheckpoints[nextEpoch];
            unchecked {
                ++i;
                nextEpoch += period;
            }
        }

        return out;
    }
}
