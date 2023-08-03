// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {VoteExecutionStatus} from "../enums/VoteExecutionStatus.sol";
import {EpochAlreadyExecuted, EpochNotVotable, InvalidMajorityPercentage, EpochNotVotable, ValidatorAlreadyVoted} from "../errors/IPCErrors.sol";
import {ExecutableQueue} from "../structs/ExecutableQueue.sol";
import {EpochVoteSubmission} from "../structs/EpochVoteSubmission.sol";
import {EpochVoteSubmissionHelper} from "../lib/EpochVoteSubmissionHelper.sol";
import {ExecutableQueueHelper} from "../lib/ExecutableQueueHelper.sol";

struct VotingStorage {
    /// @notice last executed epoch after voting
    uint64 lastVotingExecutedEpoch;
    /// @notice Initial epoch number
    uint64 genesisEpoch;
    /// @notice Checkpoint submission period
    uint64 submissionPeriod;
    /// @notice Contains the executable epochs that are ready to be executed, but has yet to be executed.
    /// This usually happens when previous submission epoch has not executed, but the next submission
    /// epoch is ready to be executed. Most of the time this should be empty
    ExecutableQueue executableQueue;
    /// @notice Percentage of majority
    uint8 majorityPercentage;
}

library LibVoting {
    using ExecutableQueueHelper for ExecutableQueue;
    using EpochVoteSubmissionHelper for EpochVoteSubmission;

    /// @notice minimum checkpoint period. Values get clamped to this
    uint8 public constant MIN_CHECKPOINT_PERIOD = 10;

    bytes32 private constant VOTING_STORAGE_POSITION = keccak256("libvoting.lib.diamond.storage");

    function votingStorage() internal pure returns (VotingStorage storage s) {
        bytes32 position = VOTING_STORAGE_POSITION;
        assembly {
            s.slot := position
        }
    }

    modifier validEpochOnly(uint64 epoch) {
        _validEpochOnly(epoch);
        _;
    }

    function applyValidEpochOnly(uint64 epoch) internal view {
        _validEpochOnly(epoch);
    }

    function _validEpochOnly(uint64 epoch) private view {
        VotingStorage storage s = votingStorage();
        if (epoch <= s.lastVotingExecutedEpoch) {
            revert EpochAlreadyExecuted();
        }
        if (epoch < s.genesisEpoch) {
            revert EpochNotVotable();
        }
        if ((epoch - s.genesisEpoch) % s.submissionPeriod != 0) {
            revert EpochNotVotable();
        }
    }

    function initVoting(uint8 _majorityPercentage, uint64 _submissionPeriod) internal {
        VotingStorage storage s = votingStorage();
        if (_majorityPercentage > 100) {
            revert InvalidMajorityPercentage();
        }

        s.majorityPercentage = _majorityPercentage;
        s.submissionPeriod = _submissionPeriod < MIN_CHECKPOINT_PERIOD ? MIN_CHECKPOINT_PERIOD : _submissionPeriod;

        s.executableQueue.period = s.submissionPeriod;
    }

    function initGenesisEpoch(uint64 genesisEpoch) internal {
        VotingStorage storage s = votingStorage();
        s.genesisEpoch = genesisEpoch;
        s.executableQueue.genesisEpoch = genesisEpoch;
    }

    /// @notice method that gives the epoch for a given block number and checkpoint period
    /// @return epoch - the epoch for the given block number and checkpoint period
    function getNextEpoch(uint256 blockNumber, uint64 checkPeriod) internal pure returns (uint64) {
        return ((uint64(blockNumber) / checkPeriod) + 1) * checkPeriod;
    }

    /// @notice method that returns the genesis epoch
    /// @return epoch - the genesis epoch
    function getGenesisEpoch() internal view returns (uint64) {
        VotingStorage storage s = votingStorage();
        return s.genesisEpoch;
    }

    function getSubmissionPeriod() internal view returns (uint64) {
        VotingStorage storage s = votingStorage();
        return s.submissionPeriod;
    }

    /// @notice returns the current checkpoint execution status based on the current vote
    /// @param vote - the vote submission data
    /// @param totalWeight - the total voting power of the validators
    function deriveExecutionStatus(
        EpochVoteSubmission storage vote,
        uint256 totalWeight
    ) internal view returns (VoteExecutionStatus) {
        VotingStorage storage s = votingStorage();
        uint256 threshold = (totalWeight * s.majorityPercentage) / 100;
        uint256 mostVotedWeight = vote.getMostVotedWeight();

        // threshold not reached, require THRESHOLD to be surpassed, equality is not enough!
        if (vote.totalSubmissionWeight <= threshold) {
            return VoteExecutionStatus.ThresholdNotReached;
        }

        // consensus reached
        if (mostVotedWeight > threshold) {
            return VoteExecutionStatus.ConsensusReached;
        }

        // now the total submissions has reached the threshold, but the most submitted vote
        // has yet to reach the threshold, that means consensus has not reached.
        // we do a early termination check, to see if consensus will ever be reached.
        //
        // consider an example that consensus will never be reached:
        //
        // -------- | -------------------------|--------------- | ------------- |
        //     MOST_VOTED                 THRESHOLD     TOTAL_SUBMISSIONS  TOTAL_WEIGHT
        //
        // we see MOST_VOTED is smaller than THRESHOLD, TOTAL_SUBMISSIONS and TOTAL_WEIGHT, if
        // the potential extra votes any vote can obtain, i.e. TOTAL_WEIGHT - TOTAL_SUBMISSIONS,
        // is smaller than or equal to the potential extra vote the most voted can obtain, i.e.
        // THRESHOLD - MOST_VOTED, then consensus will never be reached, no point voting, just abort.
        if (threshold - mostVotedWeight >= totalWeight - vote.totalSubmissionWeight) {
            return VoteExecutionStatus.RoundAbort;
        }

        // TODO: we are never reaching here in tests
        return VoteExecutionStatus.ReachingConsensus;
    }

    /// @notice marks a checkpoint for a given epoch as executed
    /// @param epoch - the epoch to mark as executed
    function markSubmissionExecuted(uint64 epoch) internal {
        VotingStorage storage s = votingStorage();
        // epoch not the next executable epoch
        if (!isNextExecutableEpoch(epoch)) {
            return;
        }

        // epoch not the next executable epoch in the queue
        if (s.executableQueue.contains(epoch)) {
            if (s.executableQueue.first != epoch) {
                return;
            }
        }

        // remove from the queue if it exists
        s.executableQueue.remove(epoch);

        // update the last executed epoch
        s.lastVotingExecutedEpoch = epoch;
    }

    /// @notice method that checks if the given epoch is the next executable epoch
    /// @param epoch - the epoch to check
    /// @return whether the given epoch is the next executable epoch
    function isNextExecutableEpoch(uint64 epoch) internal view returns (bool) {
        VotingStorage storage s = votingStorage();
        return epoch == s.lastVotingExecutedEpoch + s.submissionPeriod;
    }

    /// @notice method that returns the next executable epoch
    /// @return nextEpoch - the next executable epoch
    /// @return isExecutable - whether the next epoch is executable
    function getNextExecutableEpoch() internal view returns (uint64 nextEpoch, bool isExecutable) {
        VotingStorage storage s = votingStorage();
        nextEpoch = s.executableQueue.first;
        isExecutable = isNextExecutableEpoch(nextEpoch);
    }

    /// @notice method that submits a vote for a given epoch
    /// @param vote - the vote submission data
    /// @param submissionHash - the hash of the submission
    /// @param submitterAddress - the address of the submitter
    /// @param submitterWeight - the voting power of the submitter
    /// @param epoch - the epoch of the vote
    /// @param totalWeight - the total voting power of the validators
    /// @return shouldExecuteVote - whether the vote should be executed
    function submitVote(
        EpochVoteSubmission storage vote,
        bytes32 submissionHash,
        address submitterAddress,
        uint256 submitterWeight,
        uint64 epoch,
        uint256 totalWeight
    ) internal returns (bool shouldExecuteVote) {
        VotingStorage storage s = votingStorage();
        uint256 nonce = vote.nonce;
        if (vote.submitters[nonce][submitterAddress]) {
            revert ValidatorAlreadyVoted();
        }

        vote.submitters[nonce][submitterAddress] = true;
        vote.totalSubmissionWeight += submitterWeight;
        vote.submissionWeights[nonce][submissionHash] += submitterWeight;

        uint256 mostVotedWeight = vote.submissionWeights[nonce][vote.mostVotedSubmission];
        uint256 currVotedWeight = vote.submissionWeights[nonce][submissionHash];

        if (mostVotedWeight < currVotedWeight) {
            vote.mostVotedSubmission = submissionHash;
        }

        VoteExecutionStatus status = deriveExecutionStatus(vote, totalWeight);

        if (status == VoteExecutionStatus.ConsensusReached) {
            if (isNextExecutableEpoch(epoch)) {
                shouldExecuteVote = true;
            } else {
                // there are pending epochs to be executed, just store the submission and skip execution
                s.executableQueue.push(epoch);
            }
        } else if (status == VoteExecutionStatus.RoundAbort) {
            // abort the current round and reset the submission data.
            vote.reset();
        }
    }

    function executableQueue() internal view returns (uint64, uint64, uint64) {
        VotingStorage storage s = votingStorage();
        return (s.executableQueue.period, s.executableQueue.first, s.executableQueue.last);
    }

    function lastVotingExecutedEpoch() internal view returns (uint64) {
        VotingStorage storage s = votingStorage();
        return s.lastVotingExecutedEpoch;
    }

    function majorityPercentage() internal view returns (uint64) {
        VotingStorage storage s = votingStorage();
        return s.majorityPercentage;
    }

    function removeFromExecutableQueue(uint64 e) internal {
        VotingStorage storage s = votingStorage();
        s.executableQueue.remove(e);
    }
}
