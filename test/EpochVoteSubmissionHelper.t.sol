// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

import "forge-std/Test.sol";

import "../src/lib/EpochVoteSubmissionHelper.sol";
import "../src/structs/EpochVoteSubmission.sol";
import "../src/structs/Checkpoint.sol";

import "forge-std/console.sol";

contract EpochVoteSubmissionHelperTest is Test {
    using EpochVoteSubmissionHelper for EpochVoteSubmission;

    EpochVoteSubmission public voteSubmission;

    address constant FIRST_SUBMITTER = address(100);
    address constant SECOND_SUBMITTER = address(101);
    uint256 constant FIRST_SUBMITTER_WEIGHT = 100;
    uint256 constant SECOND_SUBMITTER_WEIGHT = 101;

    function test_Reset_Works() public {
        uint256 nonce = 0;
        bytes32 submissionHash = keccak256("most_voted_submission");

        voteSubmission.totalSubmissionWeight = FIRST_SUBMITTER_WEIGHT;
        voteSubmission.mostVotedSubmission = submissionHash;
        voteSubmission.submitters[nonce][FIRST_SUBMITTER] = true;
        voteSubmission.submissionWeights[nonce][submissionHash] = FIRST_SUBMITTER_WEIGHT;

        require(voteSubmission.nonce == nonce);
        require(voteSubmission.totalSubmissionWeight == FIRST_SUBMITTER_WEIGHT);
        require(voteSubmission.mostVotedSubmission == submissionHash);
        require(voteSubmission.submitters[nonce][FIRST_SUBMITTER] == true);
        require(voteSubmission.submissionWeights[nonce][submissionHash] == FIRST_SUBMITTER_WEIGHT);

        voteSubmission.reset();

        uint256 newNonce = 1;

        require(voteSubmission.nonce == newNonce);
        require(voteSubmission.totalSubmissionWeight == 0);
        require(voteSubmission.mostVotedSubmission == EMPTY_HASH);
        require(voteSubmission.submitters[newNonce][FIRST_SUBMITTER] == false);
        require(voteSubmission.submissionWeights[newNonce][submissionHash] == 0);
    }

    function test_GetMostVotedWeight_Works_SingleSubmitter() public {
        uint256 nonce = 0;
        bytes32 submissionHash = keccak256("most_voted_submission");

        voteSubmission.nonce = nonce;
        voteSubmission.totalSubmissionWeight = FIRST_SUBMITTER_WEIGHT;
        voteSubmission.mostVotedSubmission = submissionHash;
        voteSubmission.submitters[nonce][FIRST_SUBMITTER] = true;
        voteSubmission.submissionWeights[nonce][submissionHash] = FIRST_SUBMITTER_WEIGHT;

        require(voteSubmission.getMostVotedWeight() == FIRST_SUBMITTER_WEIGHT);
    }

    function test_GetMostVotedWeight_Works_Multipleubmitters() public {
        uint256 nonce = 0;
        bytes32 submissionHash1 = keccak256("most_voted_submission1");
        bytes32 submissionHash2 = keccak256("most_voted_submission2");

        voteSubmission.nonce = nonce;
        voteSubmission.totalSubmissionWeight = FIRST_SUBMITTER_WEIGHT;
        voteSubmission.submitters[nonce][FIRST_SUBMITTER] = true;
        voteSubmission.submissionWeights[nonce][submissionHash1] = FIRST_SUBMITTER_WEIGHT;

        voteSubmission.nonce = nonce;
        voteSubmission.totalSubmissionWeight += SECOND_SUBMITTER_WEIGHT;
        voteSubmission.submitters[nonce][SECOND_SUBMITTER] = true;
        voteSubmission.submissionWeights[nonce][submissionHash2] = SECOND_SUBMITTER_WEIGHT;

        voteSubmission.mostVotedSubmission = submissionHash1;
        if (
            voteSubmission.submissionWeights[nonce][submissionHash1]
                < voteSubmission.submissionWeights[nonce][submissionHash2]
        ) {
            voteSubmission.mostVotedSubmission = submissionHash2;
        }

        require(voteSubmission.getMostVotedWeight() == SECOND_SUBMITTER_WEIGHT);
    }

    function test_GetMostVotedWeight_Works_Empty() public view {
        require(voteSubmission.getMostVotedWeight() == 0);
    }
}
