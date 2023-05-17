// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.7;

enum VoteExecutionStatus {
    ThresholdNotReached,
    ReachingConsensus,
    RoundAbort,
    ConsensusReached
}
