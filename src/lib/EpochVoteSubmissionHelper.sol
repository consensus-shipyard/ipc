// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

import "./CheckpointHelper.sol";
import "../structs/EpochVoteSubmission.sol";

library EpochVoteSubmissionHelper {
    function reset(EpochVoteSubmission storage voteSubmission) external {
        voteSubmission.nonce++;
        voteSubmission.totalSubmissionWeight = 0;
        voteSubmission.mostVotedSubmission = EMPTY_HASH;
    }

    function getMostVotedWeight(EpochVoteSubmission storage voteSubmission) external view returns (uint256) {
        return voteSubmission.submissionWeights[voteSubmission.nonce][voteSubmission.mostVotedSubmission];
    }
}
