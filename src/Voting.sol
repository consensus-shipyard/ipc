// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

import "./constants/Constants.sol";
import "./structs/ExecutableQueue.sol";
import "./structs/EpochVoteSubmission.sol";
import "./enums/VoteExecutionStatus.sol";
import "./lib/ExecutableQueueHelper.sol";
import "./lib/EpochVoteSubmissionHelper.sol";

abstract contract Voting {
    using ExecutableQueueHelper for ExecutableQueue;
    using EpochVoteSubmissionHelper for EpochVoteSubmission;

    /// @notice minimum checkpoint period. Values get clamped to this
    uint8 constant MIN_CHECKPOINT_PERIOD = 10;

    /// @notice percent approvals needed to reach consensus
    uint8 public immutable majorityPercentage;

    /// @notice number of blocks between two checkpoint submissions
    uint64 public immutable submissionPeriod;

    /// @notice last executed epoch after voting
    uint64 public lastVotingExecutedEpoch;

    /// @notice Initial epoch number
    uint64 public genesisEpoch;

    /// @notice Contains the executable epochs that are ready to be executed, but has yet to be executed.
    /// This usually happens when previous submission epoch has not executed, but the next submission
    /// epoch is ready to be executed. Most of the time this should be empty
    ExecutableQueue public executableQueue;

    error EpochAlreadyExecuted();
    error EpochNotVotable();
    error InvalidMajorityPercentage();
    error ValidatorAlreadyVoted();

    modifier validEpochOnly(uint64 epoch) {
        if (epoch <= lastVotingExecutedEpoch) revert EpochAlreadyExecuted();
        if (epoch > genesisEpoch) {
            if ((epoch - genesisEpoch) % submissionPeriod != 0) {
                revert EpochNotVotable();
            }
        }
        _;
    }

    constructor(uint8 _majorityPercentage, uint64 _submissionPeriod) {
        if (_majorityPercentage > 100) revert InvalidMajorityPercentage();

        majorityPercentage = _majorityPercentage;
        submissionPeriod = _submissionPeriod < MIN_CHECKPOINT_PERIOD ? MIN_CHECKPOINT_PERIOD : _submissionPeriod;

        executableQueue.period = submissionPeriod;
    }

    /// @notice returns the current checkpoint execution status based on the current vote
    /// @param vote - the vote submission data
    /// @param totalWeight - the total voting power of the validators
    function _deriveExecutionStatus(EpochVoteSubmission storage vote, uint256 totalWeight)
        internal
        view
        returns (VoteExecutionStatus)
    {
        uint256 threshold = (totalWeight * majorityPercentage) / 100;
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
    function _markSubmissionExecuted(uint64 epoch) internal {
        // epoch not the next executable epoch
        if (_isNextExecutableEpoch(epoch) == false) return;

        // epoch not the next executable epoch in the queue
        if (executableQueue.contains(epoch)) {
            if (executableQueue.first != epoch) {
                return;
            }
        }

        // remove from the queue if it exists
        executableQueue.remove(epoch);

        // update the last executed epoch
        lastVotingExecutedEpoch = epoch;
    }

    /// @notice method that checks if the given epoch is the next executable epoch
    /// @param epoch - the epoch to check
    /// @return whether the given epoch is the next executable epoch
    function _isNextExecutableEpoch(uint64 epoch) internal view returns (bool) {
        return epoch == lastVotingExecutedEpoch + submissionPeriod;
    }

    /// @notice method that returns the next executable epoch
    /// @return nextEpoch - the next executable epoch
    /// @return isExecutable - whether the next epoch is executable
    function _getNextExecutableEpoch() internal view returns (uint64 nextEpoch, bool isExecutable) {
        nextEpoch = executableQueue.first;
        isExecutable = _isNextExecutableEpoch(nextEpoch);
    }

    /// @notice method that submits a vote for a given epoch
    /// @param vote - the vote submission data
    /// @param submissionHash - the hash of the submission
    /// @param submitterAddress - the address of the submitter
    /// @param submitterWeight - the voting power of the submitter
    /// @param epoch - the epoch of the vote
    /// @param totalWeight - the total voting power of the validators
    /// @return shouldExecuteVote - whether the vote should be executed
    function _submitVote(
        EpochVoteSubmission storage vote,
        bytes32 submissionHash,
        address submitterAddress,
        uint256 submitterWeight,
        uint64 epoch,
        uint256 totalWeight
    ) internal returns (bool shouldExecuteVote) {
        uint256 nonce = vote.nonce;
        if (vote.submitters[nonce][submitterAddress]) revert ValidatorAlreadyVoted();

        vote.submitters[nonce][submitterAddress] = true;
        vote.totalSubmissionWeight += submitterWeight;
        vote.submissionWeights[nonce][submissionHash] += submitterWeight;

        uint256 mostVotedWeight = vote.submissionWeights[nonce][vote.mostVotedSubmission];
        uint256 currVotedWeight = vote.submissionWeights[nonce][submissionHash];

        if (mostVotedWeight < currVotedWeight) {
            vote.mostVotedSubmission = submissionHash;
        }

        VoteExecutionStatus status = _deriveExecutionStatus(vote, totalWeight);

        if (status == VoteExecutionStatus.ConsensusReached) {
            if (_isNextExecutableEpoch(epoch)) {
                shouldExecuteVote = true;
            } else {
                // there are pending epochs to be executed, just store the submission and skip execution
                executableQueue.push(epoch);
            }
        } else if (status == VoteExecutionStatus.RoundAbort) {
            // abort the current round and reset the submission data.
            vote.reset();
        }
    }

    /// @notice method that gives the epoch for a given block number and checkpoint period
    /// @return epoch - the epoch for the given block number and checkpoint period
    function _getEpoch(uint256 blockNumber, uint64 checkPeriod) internal pure returns (uint64) {
        return ((uint64(blockNumber) / checkPeriod) + 1) * checkPeriod;
    }
}
