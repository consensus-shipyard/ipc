// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.19;

enum VoteExecutionStatus {
    ThresholdNotReached,
    ReachingConsensus,
    RoundAbort,
    ConsensusReached
}
