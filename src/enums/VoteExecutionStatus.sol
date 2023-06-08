// SPDX-License-Identifier: UNLICENSED
pragma solidity 0.8.18;

enum VoteExecutionStatus {
    ThresholdNotReached,
    ReachingConsensus,
    RoundAbort,
    ConsensusReached
}
