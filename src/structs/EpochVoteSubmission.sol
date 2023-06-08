// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

import "./Checkpoint.sol";

struct EpochVoteSubmission {
    uint256 nonce;
    uint256 totalSubmissionWeight;
    bytes32 mostVotedSubmission;
    // nonce => validator => hasSubmitted
    mapping(uint256 => mapping(address => bool)) submitters;
    // nonce => submissionHash => weight
    mapping(uint256 => mapping(bytes32 => uint256)) submissionWeights;
}

struct EpochVoteTopDownSubmission {
    EpochVoteSubmission vote;
    // submissionHash => submission
    mapping(bytes32 => TopDownCheckpoint) submissions;
}

struct EpochVoteBottomUpSubmission {
    EpochVoteSubmission vote;
    // submissionHash => submission
    mapping(bytes32 => BottomUpCheckpoint) submissions;
}
