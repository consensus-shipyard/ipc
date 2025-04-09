// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

import {SubnetIDHelper} from "../../lib/SubnetIDHelper.sol";
import {GatewayActorModifiers} from "../../lib/LibGatewayActorStorage.sol";
import {TopdownCheckpoint, TopdownVoting, SubnetID, Vote, IpcEnvelope} from "../../structs/CrossNet.sol";
import {LibTopdownVoting} from "./TopDownVotingFacet.sol";
import {PowerChangeRequest, Membership, ParentValidatorsTracker, Validator, ValidatorInfo, ValidatorSet} from "../../structs/Subnet.sol";
import {LibValidatorTracking, LibValidatorSet} from "../../lib/LibPower.sol";
import {NotValidator, HasAlreadyVoted, ExpectingLivenessVote, InvalidLivenssSubmissionHeight, InvalidTopdownCheckpointHeight, InvalidTopdownConfigNumber, NonSequential, VoteNotProposed, InvalidTopdownMessageNonce} from "../../errors/IPCErrors.sol";
import {LibGateway} from "../../lib/LibGateway.sol";

enum VoteOutcome {
    NotEnoughVote,
    QuorumFormed,
    QuorumAbandoned
}

/// Deals with the the topdown voting outcome. Calling pattern should be:
///
/// - Call getVoteOutcome to obtain the outcome
/// - If not enough votes submitted, then do nothing
/// - If enough votes but no quorum, clear everything and call clearVotes
/// - If enough votes with quorum, execute the vote by calling execute(vote)
///
/// It needs to be broken down into 3 methods due to IPC minting to gateway in the node.
/// That's why clearVotes and execute are only callable by systemActor.
contract TopDownVotingExecuteFacet is GatewayActorModifiers {
    using LibValidatorTracking for ParentValidatorsTracker;
    using LibValidatorSet for ValidatorSet;
    using EnumerableSet for EnumerableSet.Bytes32Set;
    using LibTopdownVoting for TopdownVoting;
    using SubnetIDHelper for SubnetID;

    event TopdownQuorumFormed(bytes32 vote, uint256 quorumThreshold, uint256 totalWeight);
    event VotingAborted();

    function getVoteOutcome() external view returns(VoteOutcome, bytes32 vote, uint256 tokensToMint) {
        (, uint256 quorumThreshold) = getQuorumThreshold();

        uint256 totalNumVotes = s.topdownVoting.ongoingVoteHashes.length();
        for (uint256 i = 0; i < totalNumVotes; ) {
            vote = s.topdownVoting.ongoingVoteHashes.at(i);
            if (s.topdownVoting.votes[vote].totalPower <= quorumThreshold) {
                unchecked {
                    i++;
                }
                continue;
            }

            uint256 numXnetMsgs = s.topdownVoting.votes[vote].payload.xnetMsgs.length;
            for (uint256 j = 0; j < numXnetMsgs; ) {
                tokensToMint += s.topdownVoting.votes[vote].payload.xnetMsgs[j].value;
                unchecked {
                    j++;
                }
            }

            return (VoteOutcome.QuorumFormed, vote, tokensToMint);
        }

        if (s.topdownVoting.totalPowerVoted > quorumThreshold) {
            // this means more than threshold of power has voted, but no winning vote
            return (VoteOutcome.QuorumAbandoned, bytes32(0), 0);
        }
        return (VoteOutcome.NotEnoughVote, bytes32(0), 0);
    }

    function clearVotes() external systemActorOnly() {
        (, uint256 quorumThreshold) = getQuorumThreshold();

        uint256 totalNumVotes = s.topdownVoting.ongoingVoteHashes.length();
        for (uint256 i = 0; i < totalNumVotes; ) {
            bytes32 vote = s.topdownVoting.ongoingVoteHashes.at(i);
            require(s.topdownVoting.votes[vote].totalPower <= quorumThreshold, "not minority");

            unchecked {
                i++;
            }
        }

        require(s.topdownVoting.totalPowerVoted > quorumThreshold, "not enough votes");
        s.topdownVoting.clearVoting();
        emit VotingAborted();
    }

    function execute(bytes32 vote) external systemActorOnly() {
        (, uint256 quorumThreshold) = getQuorumThreshold();
        require(s.topdownVoting.votes[vote].totalPower > quorumThreshold, "minority vote");

        s.validatorsTracker.batchStoreChangeMemory(s.topdownVoting.votes[vote].payload.powerChanges);
        LibGateway.applyTopDownMessages(s.networkName.getParentSubnet(), s.topdownVoting.votes[vote].payload.xnetMsgs);

        // TODO: propagateAllPostboxMessages temporarily disabled due to contract size issue
        LibGateway.propagateAllPostboxMessages();

        emit TopdownQuorumFormed(vote, quorumThreshold, s.topdownVoting.votes[vote].totalPower);
        s.topdownVoting.voteCommitted(vote);
        s.topdownVoting.clearVoting();
        
    }

    function getQuorumThreshold() internal view returns(uint256 totalWeight, uint256 threshold) {
        uint256 totalValidators = s.currentMembership.validators.length;
        for (uint256 i = 0; i < totalValidators; ) {
            totalWeight += s.currentMembership.validators[i].weight;
            unchecked {
                i++;
            }
        }
        threshold = totalWeight * 2 / 3;
    }
}