// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {SubnetActorModifiers} from "../lib/LibSubnetActorStorage.sol";
import {ReentrancyGuard} from "../lib/LibReentrancyGuard.sol";
import {FvmAddress} from "../structs/FvmAddress.sol";
import {BottomUpCheckpoint, CrossMsg, ChildCheck} from "../structs/Checkpoint.sol";
import {SubnetID} from "../structs/Subnet.sol";
import {CollateralIsZero, MessagesNotSorted, NotEnoughBalanceForRewards, NoValidatorsInSubnet, NotValidator, NotAllValidatorsHaveLeft, SubnetNotActive, WrongCheckpointSource, MessageNotSorted, NoRewardToWithdraw} from "../errors/IPCErrors.sol";
import {SubnetIDHelper} from "../lib/SubnetIDHelper.sol";
import {CheckpointHelper} from "../lib/CheckpointHelper.sol";
import {EpochVoteSubmission} from "../structs/EpochVoteSubmission.sol";
import {ISubnetActor} from "../interfaces/ISubnetActor.sol";
import {IGateway} from "../interfaces/IGateway.sol";
import {CrossMsgHelper} from "../lib/CrossMsgHelper.sol";
import {ExecutableQueue} from "../structs/ExecutableQueue.sol";
import {ExecutableQueueHelper} from "../lib/ExecutableQueueHelper.sol";
import {EpochVoteBottomUpSubmission} from "../structs/EpochVoteSubmission.sol";
import {EpochVoteSubmissionHelper} from "../lib/EpochVoteSubmissionHelper.sol";
import {LibVoting} from "../lib/LibVoting.sol";
import {Status} from "../enums/Status.sol";
import {ConsensusType} from "../enums/ConsensusType.sol";
import {EnumerableSet} from "openzeppelin-contracts/utils/structs/EnumerableSet.sol";
import {FilAddress} from "fevmate/utils/FilAddress.sol";
import {Address} from "openzeppelin-contracts/utils/Address.sol";
import {FvmAddressHelper} from "../lib/FvmAddressHelper.sol";

contract SubnetActorManagerFacet is ISubnetActor, SubnetActorModifiers, ReentrancyGuard {
    using EnumerableSet for EnumerableSet.AddressSet;
    using SubnetIDHelper for SubnetID;
    using CheckpointHelper for BottomUpCheckpoint;
    using FilAddress for address;
    using Address for address payable;
    using ExecutableQueueHelper for ExecutableQueue;
    using EpochVoteSubmissionHelper for EpochVoteSubmission;
    using CrossMsgHelper for CrossMsg;
    using FvmAddressHelper for FvmAddress;

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

    /// @notice methods that allows a validator to submit a checkpoint (batch of messages) and vote for it with it's own voting power.
    /// @param checkpoint - the batch messages data
    function submitCheckpoint(BottomUpCheckpoint calldata checkpoint) external {
        LibVoting.applyValidEpochOnly(checkpoint.epoch);

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

        EpochVoteBottomUpSubmission storage voteSubmission = s.epochVoteSubmissions[checkpoint.epoch];

        // submit the vote
        bool shouldExecuteVote = _submitBottomUpVote(voteSubmission, checkpoint, msg.sender, s.stake[msg.sender]);

        if (shouldExecuteVote) {
            _commitCheckpoint(voteSubmission);
        } else {
            // try to get the next executable epoch from the queue
            (uint64 nextExecutableEpoch, bool isExecutableEpoch) = LibVoting.getNextExecutableEpoch();

            if (isExecutableEpoch) {
                EpochVoteBottomUpSubmission storage nextVoteSubmission = s.epochVoteSubmissions[nextExecutableEpoch];

                _commitCheckpoint(nextVoteSubmission);
            }
        }
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

    /// @notice whether a validator has voted for a checkpoint submission during an epoch
    /// @param epoch - the epoch to check
    /// @param submitter - the validator to check
    function hasValidatorVotedForSubmission(uint64 epoch, address submitter) external view returns (bool) {
        EpochVoteBottomUpSubmission storage voteSubmission = s.epochVoteSubmissions[epoch];

        return voteSubmission.vote.submitters[voteSubmission.vote.nonce][submitter];
    }

    /// @notice submits a vote for a checkpoint
    /// @param voteSubmission - the vote submission data
    /// @param submitterAddress - the validator that submits the vote
    /// @param submitterWeight - the weight of the validator
    function _submitBottomUpVote(
        EpochVoteBottomUpSubmission storage voteSubmission,
        BottomUpCheckpoint calldata submission,
        address submitterAddress,
        uint256 submitterWeight
    ) internal returns (bool shouldExecuteVote) {
        bytes32 submissionHash = submission.toHash();

        shouldExecuteVote = LibVoting.submitVote(
            voteSubmission.vote,
            submissionHash,
            submitterAddress,
            submitterWeight,
            submission.epoch,
            s.totalStake
        );

        // store the submission only the first time
        if (voteSubmission.submissions[submissionHash].isEmpty()) {
            voteSubmission.submissions[submissionHash] = submission;
        }
    }

    /// @notice method that commits a checkpoint after reaching majority
    /// @param voteSubmission - the last vote submission that reached majority for commit
    function _commitCheckpoint(EpochVoteBottomUpSubmission storage voteSubmission) internal {
        BottomUpCheckpoint storage checkpoint = voteSubmission.submissions[voteSubmission.vote.mostVotedSubmission];
        /// Ensures the checkpoints are chained. If not, should abort the current checkpoint.

        if (s.prevExecutedCheckpointHash != checkpoint.prevHash) {
            voteSubmission.vote.reset();
            LibVoting.removeFromExecutableQueue(checkpoint.epoch);
            return;
        }

        LibVoting.markSubmissionExecuted(checkpoint.epoch);

        s.committedCheckpoints[checkpoint.epoch] = checkpoint;
        s.prevExecutedCheckpointHash = checkpoint.toHash();

        IGateway(s.ipcGatewayAddr).commitChildCheck(checkpoint);
    }
}
