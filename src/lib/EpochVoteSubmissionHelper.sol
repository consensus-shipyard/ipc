// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity 0.8.19;

import {EMPTY_HASH} from "../constants/Constants.sol";
import {EpochVoteSubmission} from "../structs/EpochVoteSubmission.sol";

library EpochVoteSubmissionHelper {
    function reset(EpochVoteSubmission storage voteSubmission) external {
        ++voteSubmission.nonce;
        voteSubmission.totalSubmissionWeight = 0;
        voteSubmission.mostVotedSubmission = EMPTY_HASH;
    }

    function getMostVotedWeight(EpochVoteSubmission storage voteSubmission) external view returns (uint256) {
        return voteSubmission.submissionWeights[voteSubmission.nonce][voteSubmission.mostVotedSubmission];
    }
}
