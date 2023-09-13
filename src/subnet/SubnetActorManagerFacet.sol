// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {Status} from "../enums/Status.sol";
import {CollateralIsZero, EmptyAddress, MessagesNotSorted, NotEnoughBalanceForRewards, NoValidatorsInSubnet, NotValidator, NotAllValidatorsHaveLeft, SubnetNotActive, WrongCheckpointSource, NoRewardToWithdraw, InconsistentPrevCheckpoint} from "../errors/IPCErrors.sol";
import {IGateway} from "../interfaces/IGateway.sol";
import {ISubnetActor} from "../interfaces/ISubnetActor.sol";
import {BottomUpCheckpoint} from "../structs/Checkpoint.sol";
import {FvmAddress} from "../structs/FvmAddress.sol";
import {SubnetID} from "../structs/Subnet.sol";
import {CheckpointHelper} from "../lib/CheckpointHelper.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";
import {EpochVoteSubmissionHelper} from "../lib/EpochVoteSubmissionHelper.sol";
import {FvmAddressHelper} from "../lib/FvmAddressHelper.sol";
import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {SubnetActorModifiers} from "../lib/LibSubnetActorStorage.sol";
import {LibVoting} from "../lib/LibVoting.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";

contract SubnetActorManagerFacet is ISubnetActor, SubnetActorModifiers, ReentrancyGuard {
    using EnumerableSet for EnumerableSet.AddressSet;
    using SubnetIDHelper for SubnetID;
    using CheckpointHelper for BottomUpCheckpoint;
    using FilAddress for address;
    using Address for address payable;
    using FvmAddressHelper for FvmAddress;

    event BottomUpCheckpointSubmitted(BottomUpCheckpoint checkpoint, address submitter);
    event BottomUpCheckpointExecuted(uint64 epoch, address submitter);
    event NextBottomUpCheckpointExecuted(uint64 epoch, address submitter);

    /// @notice method that allows a validator to join the subnet
    /// @param netAddr - the network address of the validator
    function join(string calldata netAddr, FvmAddress calldata workerAddr) external payable notKilled {
        uint256 validatorStake = msg.value;
        address validator = msg.sender;
        if (validatorStake == 0) {
            revert CollateralIsZero();
        }

        s.stake[validator] += validatorStake;
        s.totalStake += validatorStake;

        if (s.stake[validator] >= s.minActivationCollateral) {
            if (!s.validators.contains(validator)) {
                // slither-disable-next-line unused-return
                s.validators.add(validator);
                s.validatorNetAddresses[validator] = netAddr;
                s.validatorWorkerAddresses[validator] = workerAddr;
            }
        }

        if (s.status == Status.Instantiated) {
            if (s.totalStake >= s.minActivationCollateral) {
                s.status = Status.Active;
                IGateway(s.ipcGatewayAddr).register{value: s.totalStake}();
            }
        } else {
            if (s.status == Status.Inactive) {
                if (s.totalStake >= s.minActivationCollateral) {
                    s.status = Status.Active;
                }
            }
            IGateway(s.ipcGatewayAddr).addStake{value: validatorStake}();
        }
    }

    /// @notice method that allows a validator to leave the subnet
    function leave() external nonReentrant notKilled {
        uint256 amount = s.stake[msg.sender];

        if (amount == 0) {
            revert NotValidator();
        }

        s.stake[msg.sender] = 0;
        s.totalStake -= amount;
        // slither-disable-next-line unused-return
        s.validators.remove(msg.sender);
        if (s.status == Status.Active) {
            if (s.totalStake < s.minActivationCollateral) {
                s.status = Status.Inactive;
            }
        }

        IGateway(s.ipcGatewayAddr).releaseStake(amount);

        payable(msg.sender).sendValue(amount);
    }

    /// @notice method that allows to kill the subnet when all validators left. It is not a privileged operation.
    function kill() external notKilled {
        if (s.validators.length() != 0 || s.totalStake != 0) {
            revert NotAllValidatorsHaveLeft();
        }

        s.status = Status.Killed;

        IGateway(s.ipcGatewayAddr).kill();
    }

    /// @notice method that distributes the rewards for the subnet to validators.
    function reward(uint256 amount) external onlyGateway {
        uint256 validatorsLength = s.validators.length();

        if (validatorsLength == 0) {
            revert NoValidatorsInSubnet();
        }
        if (amount < validatorsLength) {
            revert NotEnoughBalanceForRewards();
        }

        uint256 rewardAmount = amount / validatorsLength;

        for (uint256 i = 0; i < validatorsLength; ) {
            s.accumulatedRewards[s.validators.at(i)] += rewardAmount;
            unchecked {
                ++i;
            }
        }
    }

    /// @notice method that allows a validator to withdraw it's accumulated rewards using pull-based transfer
    function withdraw() external {
        uint256 amount = s.accumulatedRewards[msg.sender];

        if (amount == 0) {
            revert NoRewardToWithdraw();
        }

        s.accumulatedRewards[msg.sender] = 0;

        IGateway(s.ipcGatewayAddr).releaseRewards(amount);

        payable(msg.sender).sendValue(amount);
    }

    /// @notice get the total stake
    function committedCheckpoints(
        uint64 e
    ) external view returns (SubnetID memory source, uint64 epoch, uint256 fee, bytes32 prevHash, bytes memory proof) {
        source = s.committedCheckpoints[e].source;
        epoch = s.committedCheckpoints[e].epoch;
        fee = s.committedCheckpoints[e].fee;
        prevHash = s.committedCheckpoints[e].prevHash;
        proof = s.committedCheckpoints[e].proof;
    }

    function setValidatorNetAddr(string calldata newNetAddr) external {
        address validator = msg.sender;
        if (!s.validators.contains(validator)) {
            revert NotValidator();
        }
        if (bytes(newNetAddr).length == 0) {
            revert EmptyAddress();
        }
        s.validatorNetAddresses[validator] = newNetAddr;
    }

    function setValidatorWorkerAddr(FvmAddress calldata newWorkerAddr) external {
        address validator = msg.sender;
        if (!s.validators.contains(validator)) {
            revert NotValidator();
        }
        s.validatorWorkerAddresses[validator] = newWorkerAddr;
    }

    /// @notice methods that allows a validator to submit a checkpoint (batch of messages) and vote for it with it's own voting power.
    /// @param checkpoint - the batch messages data
    function submitCheckpoint(BottomUpCheckpoint calldata checkpoint) external {
        if (s.status != Status.Active) {
            revert SubnetNotActive();
        }
        if (!s.validators.contains(msg.sender)) {
            revert NotValidator();
        }
        if (checkpoint.source.toHash() != s.currentSubnetHash) {
            revert WrongCheckpointSource();
        }
        if (!CrossMsgHelper.isSorted(checkpoint.crossMsgs)) {
            revert MessagesNotSorted();
        }

        _commitCheckpoint(checkpoint);
    }

    /// @notice method that commits a checkpoint after reaching majority
    /// @param checkpoint - the batch messages data
    function _commitCheckpoint(BottomUpCheckpoint calldata checkpoint) internal {
        /// Ensures the checkpoints are chained. If not, should abort the current checkpoint.
        if (s.prevExecutedCheckpointHash != checkpoint.prevHash) {
            revert InconsistentPrevCheckpoint();
        }

        s.committedCheckpoints[checkpoint.epoch] = checkpoint;
        s.prevExecutedCheckpointHash = checkpoint.toHash();

        IGateway(s.ipcGatewayAddr).commitChildCheck(checkpoint);
    }
}
