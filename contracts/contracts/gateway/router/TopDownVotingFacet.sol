// SPDX-License-Identifier: MIT OR Apache-2.0
pragma solidity ^0.8.23;

import {EnumerableSet} from "@openzeppelin/contracts/utils/structs/EnumerableSet.sol";

import {SubnetIDHelper} from "../../lib/SubnetIDHelper.sol";
import {GatewayActorModifiers} from "../../lib/LibGatewayActorStorage.sol";
import {TopdownCheckpoint, TopdownVoting, SubnetID} from "../../structs/CrossNet.sol";
import {PowerChangeRequest, Membership, ParentValidatorsTracker, Validator, ValidatorInfo, ValidatorSet} from "../../structs/Subnet.sol";
import {LibValidatorTracking, LibValidatorSet} from "../../lib/LibPower.sol";
import {NotValidator, HasAlreadyVoted, ExpectingLivenessVote, InvalidLivenssSubmissionHeight, VoteNotProposed} from "../../errors/IPCErrors.sol";
import {LibGateway} from "../../lib/LibGateway.sol";

/// Performs topdown bridging voting on chain. This makes validator slashing possible and
/// avoid potential collusion issues.
contract TopDownVotingFacet is GatewayActorModifiers {
    using LibValidatorTracking for ParentValidatorsTracker;
    using LibValidatorSet for ValidatorSet;
    using EnumerableSet for EnumerableSet.Bytes32Set;
    using LibTopdownVoting for TopdownVoting;
    using SubnetIDHelper for SubnetID;

    event TopdownQuorumFormed(bytes32 vote, uint256 quorumThreshold, uint256 totalWeight);
    event VotingAborted(bytes32[] votedHashes);

    function latestCommitted() external view returns (uint64 blockHeight, bytes32 blockHash) {
        blockHeight = s.topdownVoting.committedParentHeight;
        blockHash = s.topdownVoting.committedBlockHash;
    }

    function propose(TopdownCheckpoint calldata checkpoint) external {
        bytes32 vote = keccak256(abi.encode(checkpoint));

        if (!s.topdownVoting.ongoingVoteHashes.contains(vote)) {
            s.topdownVoting.storeCheckpoint(vote, checkpoint);
            s.topdownVoting.ongoingVoteHashes.add(vote);
        }

        castVote(vote);
    }

    function castVote(bytes32 vote) internal {
        if (!s.topdownVoting.ongoingVoteHashes.contains(vote)) {
            revert VoteNotProposed(vote);
        }

        uint256 totalValidators = s.currentMembership.validators.length;
        uint256 totalWeight = 0;

        // can shift this query to off chain
        uint256 validatorIndex = type(uint256).max;
        for (uint256 i = 0; i < totalValidators; ) {
            if (s.currentMembership.validators[i].addr == msg.sender) {
                validatorIndex = i;
            }

            totalWeight += s.currentMembership.validators[i].weight;

            unchecked {
                i++;
            }
        }
        if (validatorIndex == type(uint256).max) revert NotValidator(msg.sender);

        if (s.topdownVoting.hasVoted(validatorIndex)) {
            revert HasAlreadyVoted(msg.sender);
        }
        s.topdownVoting.markVoted(validatorIndex);

        uint256 power = s.currentMembership.validators[validatorIndex].weight;
        (uint256 totalPowerVoted, uint256 voteTotalPower) = s.topdownVoting.increaseVotePower(vote, power);

        uint256 quorumThreshold = (totalWeight * 2) / 3;

        if (voteTotalPower > quorumThreshold) {
            emit TopdownQuorumFormed(vote, quorumThreshold, totalWeight);

            execute(vote);

            s.topdownVoting.clearVoting();
            return;
        }

        if (totalPowerVoted > quorumThreshold) {
            // this means more than quorum threshold of total weight has already
            // voted and no consensus reached
            emit VotingAborted(s.topdownVoting.ongoingVoteHashes.values());
            s.topdownVoting.clearVoting();
            return;
        }
    }

    function execute(bytes32 vote) internal {
        s.topdownVoting.voteCommitted(vote);
        s.validatorsTracker.batchStoreChangeMemory(s.topdownVoting.votes[vote].payload.powerChanges);
        require(s.networkName.route.length == 1, "route wrong");
        LibGateway.applyTopDownMessages(s.networkName.getParentSubnet(), s.topdownVoting.votes[vote].payload.xnetMsgs);

        // TODO: propagateAllPostboxMessages temporarily disabled due to contract size issue
        // LibGateway.propagateAllPostboxMessages();
    }
}

library LibTopdownVoting {
    using EnumerableSet for EnumerableSet.Bytes32Set;

    function voteCommitted(TopdownVoting storage self, bytes32 vote) internal {
        self.committedBlockHash = self.votes[vote].payload.blockHash;
        self.committedParentHeight = self.votes[vote].payload.height;
    }

    function hasVoted(TopdownVoting storage self, uint256 validatorIndex) internal view returns (bool) {
        return self.votedValidators & (1 << validatorIndex) != 0;
    }

    function markVoted(TopdownVoting storage self, uint256 validatorIndex) internal {
        self.votedValidators |= (1 << validatorIndex);
    }

    function storeCheckpoint(TopdownVoting storage self, bytes32 vote, TopdownCheckpoint calldata checkpoint) internal {
        self.votes[vote].payload = checkpoint;
        self.votes[vote].totalPower = 0;
    }

    function increaseVotePower(
        TopdownVoting storage self,
        bytes32 vote,
        uint256 powerIncrease
    ) internal returns (uint256 totalPowerVoted, uint256 voteTotalPower) {
        voteTotalPower = self.votes[vote].totalPower + powerIncrease;
        self.votes[vote].totalPower = voteTotalPower;

        totalPowerVoted = self.totalPowerVoted;
    }

    function clearVoting(TopdownVoting storage self) internal {
        uint256 totalNumVotes = self.ongoingVoteHashes.length();

        for (uint256 i = 0; i < totalNumVotes; ) {
            bytes32 voteHash = self.ongoingVoteHashes.at(i);

            self.ongoingVoteHashes.remove(voteHash);
            delete self.votes[voteHash];

            unchecked {
                i++;
            }
        }

        self.totalPowerVoted = 0;
        self.votedValidators = 0;
    }
}
